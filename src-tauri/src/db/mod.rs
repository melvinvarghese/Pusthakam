//! SQLite access. All SQL lives under `db/` and `services/`; commands never write SQL.

pub mod migrations;

use crate::error::AppResult;
use rusqlite::{Connection, OptionalExtension};
use std::path::Path;

/// Opens (creating if needed) the database, applies PRAGMAs and runs pending migrations.
pub fn open(path: &Path) -> AppResult<Connection> {
    open_with_pre_upgrade(path, |_| Ok(()))
}

/// Takes a snapshot of an existing versioned library before applying newer migrations.
pub fn open_with_pre_upgrade(
    path: &Path,
    snapshot: impl FnOnce(&Connection) -> AppResult<()>,
) -> AppResult<Connection> {
    let mut conn = Connection::open(path)?;
    configure(&conn)?;
    let has_history = conn.query_row(
        "SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name = 'schema_migrations'",
        [], |_| Ok(()),
    ).optional()?.is_some();
    if has_history {
        let version: i64 = conn.query_row(
            "SELECT coalesce(max(version), 0) FROM schema_migrations", [], |row| row.get(0),
        )?;
        if version > 0 && version < migrations::latest_version() {
            snapshot(&conn)?;
        }
    }
    migrations::run(&mut conn, migrations::MIGRATIONS)?;
    Ok(conn)
}

/// Connection settings required for correctness. `foreign_keys` is per-connection in SQLite.
pub fn configure(conn: &Connection) -> AppResult<()> {
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.busy_timeout(std::time::Duration::from_millis(5000))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_preserves_a_pre_upgrade_snapshot_and_stops_if_it_cannot_save_one() {
        let tmp = tempfile::tempdir().unwrap();
        let paths = crate::paths::AppPaths::ensure(tmp.path()).unwrap();
        let previous = &migrations::MIGRATIONS[..migrations::MIGRATIONS.len() - 1];
        let previous_version = previous.last().unwrap().version;
        {
            let mut conn = Connection::open(&paths.database).unwrap();
            configure(&conn).unwrap();
            migrations::run(&mut conn, previous).unwrap();
            conn.execute("INSERT INTO application_settings (key, value) VALUES ('test', '17')", []).unwrap();
        }
        let result = open_with_pre_upgrade(&paths.database, |_| {
            Err(crate::error::AppError::Validation("Backup location unavailable.".into()))
        });
        assert!(result.is_err());
        {
            let conn = Connection::open(&paths.database).unwrap();
            let version: i64 = conn.query_row("SELECT max(version) FROM schema_migrations", [], |r| r.get(0)).unwrap();
            assert_eq!(version, previous_version);
        }
        let upgraded = open_with_pre_upgrade(&paths.database, |conn| {
            crate::services::backup::create(conn, &paths.backups)?;
            Ok(())
        }).unwrap();
        let version: i64 = upgraded.query_row("SELECT max(version) FROM schema_migrations", [], |r| r.get(0)).unwrap();
        assert_eq!(version, migrations::latest_version());
        let saved = std::fs::read_dir(&paths.backups).unwrap().next().unwrap().unwrap().path();
        let snapshot = Connection::open_with_flags(saved, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
        let (version, value): (i64, String) = snapshot.query_row(
            "SELECT (SELECT max(version) FROM schema_migrations), value FROM application_settings WHERE key = 'test'",
            [], |r| Ok((r.get(0)?, r.get(1)?)),
        ).unwrap();
        assert_eq!((version, value.as_str()), (previous_version, "17"));
    }

    #[test]
    fn open_sets_required_pragmas() {
        let tmp = tempfile::tempdir().unwrap();
        let conn = open(&tmp.path().join("t.db")).unwrap();
        let fk: i64 = conn.query_row("PRAGMA foreign_keys", [], |r| r.get(0)).unwrap();
        let jm: String = conn.query_row("PRAGMA journal_mode", [], |r| r.get(0)).unwrap();
        assert_eq!(fk, 1);
        assert_eq!(jm.to_lowercase(), "wal");
    }

    #[test]
    fn data_persists_after_close_and_reopen() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("persist.db");
        {
            let conn = open(&path).unwrap();
            conn.execute(
                "INSERT INTO application_settings (key, value) VALUES ('theme', '\"dark\"')",
                [],
            )
            .unwrap();
        } // connection dropped = app closed
        let conn = open(&path).unwrap();
        let v: String = conn
            .query_row("SELECT value FROM application_settings WHERE key='theme'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(v, "\"dark\"");
    }

    #[test]
    fn fts5_is_available() {
        let tmp = tempfile::tempdir().unwrap();
        let conn = open(&tmp.path().join("fts.db")).unwrap();
        conn.execute_batch("CREATE VIRTUAL TABLE t USING fts5(title); INSERT INTO t VALUES ('Malayalam grammar');")
            .unwrap();
        let n: i64 = conn
            .query_row("SELECT count(*) FROM t WHERE t MATCH 'gram*'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1);
    }

    #[test]
    fn audit_events_are_append_only() {
        let tmp = tempfile::tempdir().unwrap();
        let conn = open(&tmp.path().join("audit.db")).unwrap();
        conn.execute(
            "INSERT INTO audit_events (action, entity_type, summary) VALUES ('test','x','created')",
            [],
        )
        .unwrap();
        assert!(conn.execute("UPDATE audit_events SET summary='tampered'", []).is_err());
        assert!(conn.execute("DELETE FROM audit_events", []).is_err());
    }
}
