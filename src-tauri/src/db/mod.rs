//! SQLite access. All SQL lives under `db/` and `services/`; commands never write SQL.

pub mod migrations;

use crate::error::AppResult;
use rusqlite::Connection;
use std::path::Path;

/// Opens (creating if needed) the database, applies PRAGMAs and runs pending migrations.
pub fn open(path: &Path) -> AppResult<Connection> {
    let mut conn = Connection::open(path)?;
    configure(&conn)?;
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
