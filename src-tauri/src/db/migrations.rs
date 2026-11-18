//! Version-controlled migrations. Each migration is an immutable SQL file embedded in the binary.
//! Applied migrations are recorded with a SHA-256 checksum; if an applied migration's text changes,
//! startup aborts rather than running against a schema that no longer matches the code.

use crate::error::{AppError, AppResult};
use rusqlite::{Connection, OptionalExtension};
use sha2::{Digest, Sha256};

pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub sql: &'static str,
}

pub const MIGRATIONS: &[Migration] = &[
    Migration { version: 1, name: "core", sql: include_str!("../../migrations/0001_core.sql") },
    Migration { version: 2, name: "config", sql: include_str!("../../migrations/0002_config.sql") },
    Migration { version: 3, name: "auth", sql: include_str!("../../migrations/0003_auth.sql") },
    Migration { version: 4, name: "catalogue", sql: include_str!("../../migrations/0004_catalogue.sql") },
    Migration { version: 5, name: "members", sql: include_str!("../../migrations/0005_members.sql") },
    Migration { version: 6, name: "loans", sql: include_str!("../../migrations/0006_loans.sql") },
    Migration { version: 7, name: "loan_renewals", sql: include_str!("../../migrations/0007_loan_renewals.sql") },
    Migration { version: 8, name: "reservations", sql: include_str!("../../migrations/0008_reservations.sql") },
    Migration { version: 9, name: "fines", sql: include_str!("../../migrations/0009_fines.sql") },
    Migration { version: 10, name: "loan_return_condition", sql: include_str!("../../migrations/0010_loan_return_condition.sql") },
    Migration { version: 11, name: "member_photos", sql: include_str!("../../migrations/0011_member_photos.sql") },
    Migration { version: 12, name: "staff_session_revision", sql: include_str!("../../migrations/0012_staff_session_revision.sql") },
    Migration { version: 13, name: "administrator_recovery", sql: include_str!("../../migrations/0013_administrator_recovery.sql") },
    Migration { version: 14, name: "stock_checks", sql: include_str!("../../migrations/0014_stock_checks.sql") },
    Migration { version: 15, name: "stock_transfers", sql: include_str!("../../migrations/0015_stock_transfers.sql") },
    Migration { version: 16, name: "book_covers", sql: include_str!("../../migrations/0016_book_covers.sql") },
    Migration { version: 17, name: "calendar_closed_weekdays", sql: include_str!("../../migrations/0017_calendar_closed_weekdays.sql") },
    Migration { version: 18, name: "loan_import_provenance", sql: include_str!("../../migrations/0018_loan_import_provenance.sql") },
    Migration { version: 19, name: "renewal_import_provenance", sql: include_str!("../../migrations/0019_renewal_import_provenance.sql") },
];

pub fn checksum(sql: &str) -> String {
    Sha256::digest(sql.as_bytes()).iter().map(|b| format!("{b:02x}")).collect()
}

/// Latest version defined in this build.
pub fn latest_version() -> i64 {
    MIGRATIONS.last().map(|m| m.version).unwrap_or(0)
}

/// Applies pending migrations in order, each inside its own transaction.
/// Returns the resulting schema version.
pub fn run(conn: &mut Connection, migrations: &[Migration]) -> AppResult<i64> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version    INTEGER PRIMARY KEY,
            name       TEXT NOT NULL,
            checksum   TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
         );",
    )?;

    // Refuse to open a database created by a newer version of the app.
    let current: i64 = conn.query_row("SELECT COALESCE(MAX(version),0) FROM schema_migrations", [], |r| r.get(0))?;
    let newest_known = migrations.last().map(|m| m.version).unwrap_or(0);
    if current > newest_known {
        return Err(AppError::Conflict(format!(
            "This library database (schema v{current}) was created by a newer version of Pusthakam. \
             Please update the application."
        )));
    }

    for m in migrations {
        let existing: Option<String> = conn
            .query_row("SELECT checksum FROM schema_migrations WHERE version = ?1", [m.version], |r| r.get(0))
            .optional()?;
        let sum = checksum(m.sql);
        match existing {
            Some(stored) if stored == sum => continue,
            Some(_) => {
                return Err(AppError::Internal(format!(
                    "migration {:04}_{} was modified after it was applied",
                    m.version, m.name
                )))
            }
            None => {
                let tx = conn.transaction()?;
                tx.execute_batch(m.sql)?;
                tx.execute(
                    "INSERT INTO schema_migrations (version, name, checksum) VALUES (?1, ?2, ?3)",
                    (m.version, m.name, &sum),
                )?;
                tx.commit()?;
                tracing::info!(version = m.version, name = m.name, "applied migration");
            }
        }
    }
    Ok(conn.query_row("SELECT COALESCE(MAX(version),0) FROM schema_migrations", [], |r| r.get(0))?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mem() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        c.pragma_update(None, "foreign_keys", "ON").unwrap();
        c
    }

    #[test]
    fn fresh_database_gets_all_migrations() {
        let mut c = mem();
        assert_eq!(run(&mut c, MIGRATIONS).unwrap(), latest_version());
        let n: i64 = c.query_row("SELECT count(*) FROM schema_migrations", [], |r| r.get(0)).unwrap();
        assert_eq!(n as usize, MIGRATIONS.len());
    }

    #[test]
    fn running_twice_is_a_no_op() {
        let mut c = mem();
        run(&mut c, MIGRATIONS).unwrap();
        run(&mut c, MIGRATIONS).unwrap();
        let n: i64 = c.query_row("SELECT count(*) FROM schema_migrations", [], |r| r.get(0)).unwrap();
        assert_eq!(n as usize, MIGRATIONS.len());
    }

    #[test]
    fn upgrade_applies_only_new_migrations() {
        let mut c = mem();
        run(&mut c, MIGRATIONS).unwrap();
        let next = latest_version() + 1;
        let mut all: Vec<Migration> =
            MIGRATIONS.iter().map(|m| Migration { version: m.version, name: m.name, sql: m.sql }).collect();
        all.push(Migration { version: next, name: "extra", sql: "CREATE TABLE extra (id INTEGER PRIMARY KEY);" });
        assert_eq!(run(&mut c, &all).unwrap(), next);
        c.execute("INSERT INTO extra DEFAULT VALUES", []).unwrap();
    }

    #[test]
    fn upgrade_preserves_open_and_closed_loans_and_snapshots_only_open_fine_terms() {
        let mut c = mem();
        let previous = MIGRATIONS.iter().position(|m| m.version == 9).unwrap();
        assert_eq!(run(&mut c, &MIGRATIONS[..previous]).unwrap(), 8);
        c.execute_batch(
            "INSERT INTO libraries (id, name, profile, currency, locale) VALUES (1, 'Test Library', 'school', 'INR', 'en');
             INSERT INTO staff_users (id, username, display_name, password_hash, role_id)
                SELECT 1, 'test', 'Test Librarian', 'unused', id FROM roles WHERE code = 'administrator';
             INSERT INTO member_categories (id, name) VALUES (1, 'Students');
             INSERT INTO item_categories (id, name) VALUES (1, 'Books');
             INSERT INTO members (id, member_number, name, category_id, expires_on)
                VALUES (1, 'M-1', 'Asha Nair', 1, '2030-12-31');
             INSERT INTO circulation_policies (member_category_id, item_category_id, loan_days, max_loans,
                fine_per_day_minor, fine_grace_days, fine_cap_minor) VALUES (1, 1, 14, 4, 150, 2, 800);
             INSERT INTO bibliographic_records (id, title) VALUES (1, 'Test Book');
             INSERT INTO physical_items (id, record_id, accession_no, barcode, item_category_id, status)
                VALUES (1, 1, 'COPY-1', 'COPY-1', 1, 'on_loan'), (2, 1, 'COPY-2', 'COPY-2', 1, 'available');
             INSERT INTO loans (id, item_id, member_id, item_category_id, issued_by, due_on, returned_at, returned_by)
                VALUES (1, 1, 1, 1, 1, '2026-10-10', NULL, NULL), (2, 2, 1, 1, 1, '2026-10-10', '2026-10-12T00:00:00Z', 1);",
        ).unwrap();
        assert_eq!(run(&mut c, MIGRATIONS).unwrap(), latest_version());
        let terms: (i64, i64, i64, String) = c.query_row(
            "SELECT rate_minor, grace_days, cap_minor, currency FROM loan_fine_terms WHERE loan_id = 1",
            [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        ).unwrap();
        assert_eq!(terms, (150, 2, 800, "INR".into()));
        let count: i64 = c.query_row("SELECT count(*) FROM loan_fine_terms", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 1);
        let closed: (String, String) = c.query_row(
            "SELECT returned_at, return_condition FROM loans WHERE id = 2", [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        ).unwrap();
        assert_eq!(closed, ("2026-10-12T00:00:00Z".into(), "available".into()));
        assert_eq!(c.query_row("SELECT count(*) FROM fines", [], |r| r.get::<_, i64>(0)).unwrap(), 0);
    }

    #[test]
    fn tampered_migration_is_rejected() {
        let mut c = mem();
        run(&mut c, MIGRATIONS).unwrap();
        let mut tampered: Vec<Migration> =
            MIGRATIONS.iter().map(|m| Migration { version: m.version, name: m.name, sql: m.sql }).collect();
        tampered[0].sql = "CREATE TABLE evil (id INTEGER);";
        let err = run(&mut c, &tampered).unwrap_err();
        assert!(matches!(err, AppError::Internal(_)));
    }

    #[test]
    fn failed_migration_rolls_back_completely() {
        let mut c = mem();
        let bad = [Migration {
            version: 1,
            name: "bad",
            sql: "CREATE TABLE half (id INTEGER); INSERT INTO nonexistent VALUES (1);",
        }];
        assert!(run(&mut c, &bad).is_err());
        let exists: Option<String> = c
            .query_row("SELECT name FROM sqlite_master WHERE name='half'", [], |r| r.get(0))
            .optional()
            .unwrap();
        assert!(exists.is_none(), "partial migration must not persist");
        let n: i64 = c.query_row("SELECT count(*) FROM schema_migrations", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 0);
    }

    #[test]
    fn database_from_newer_app_is_refused() {
        let mut c = mem();
        run(&mut c, MIGRATIONS).unwrap();
        c.execute("INSERT INTO schema_migrations (version, name, checksum) VALUES (999,'future','x')", []).unwrap();
        assert!(matches!(run(&mut c, MIGRATIONS).unwrap_err(), AppError::Conflict(_)));
    }

    #[test]
    fn library_row_is_limited_to_one() {
        let mut c = mem();
        run(&mut c, MIGRATIONS).unwrap();
        let ins = "INSERT INTO libraries (id,name,profile,currency,locale) VALUES (?1,'Test','school','INR','en')";
        c.execute(ins, [1]).unwrap();
        assert!(c.execute(ins, [2]).is_err());
    }
}
