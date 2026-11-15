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
    fn tampered_migration_is_rejected() {
        let mut c = mem();
        run(&mut c, MIGRATIONS).unwrap();
        let tampered = [
            Migration { version: 1, name: "core", sql: "CREATE TABLE evil (id INTEGER);" },
            Migration { version: 2, name: "config", sql: MIGRATIONS[1].sql },
            Migration { version: 3, name: "auth", sql: MIGRATIONS[2].sql },
            Migration { version: 4, name: "catalogue", sql: MIGRATIONS[3].sql },
        ];
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
