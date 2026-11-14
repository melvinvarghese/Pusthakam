//! Application settings stored in `application_settings` (key -> JSON text).

use crate::error::{AppError, AppResult};
use rusqlite::{Connection, OptionalExtension};

const MAX_VALUE_LEN: usize = 10_000;

fn validate_key(key: &str) -> AppResult<()> {
    let ok = !key.is_empty()
        && key.len() <= 64
        && key.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '.');
    if ok {
        Ok(())
    } else {
        Err(AppError::Validation(
            "Setting names may only contain lowercase letters, digits, '_' and '.'.".into(),
        ))
    }
}

pub fn get(conn: &Connection, key: &str) -> AppResult<Option<String>> {
    validate_key(key)?;
    Ok(conn
        .query_row("SELECT value FROM application_settings WHERE key = ?1", [key], |r| r.get(0))
        .optional()?)
}

pub fn set(conn: &Connection, key: &str, value: &str) -> AppResult<()> {
    validate_key(key)?;
    if value.len() > MAX_VALUE_LEN {
        return Err(AppError::Validation("Setting value is too long.".into()));
    }
    conn.execute(
        "INSERT INTO application_settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value,
                                        updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now')",
        (key, value),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn conn() -> (tempfile::TempDir, Connection) {
        let tmp = tempfile::tempdir().unwrap();
        let c = db::open(&tmp.path().join("s.db")).unwrap();
        (tmp, c)
    }

    #[test]
    fn missing_key_returns_none() {
        let (_t, c) = conn();
        assert_eq!(get(&c, "ui.theme").unwrap(), None);
    }

    #[test]
    fn set_then_get_and_overwrite() {
        let (_t, c) = conn();
        set(&c, "ui.theme", "\"dark\"").unwrap();
        assert_eq!(get(&c, "ui.theme").unwrap().as_deref(), Some("\"dark\""));
        set(&c, "ui.theme", "\"light\"").unwrap();
        assert_eq!(get(&c, "ui.theme").unwrap().as_deref(), Some("\"light\""));
        let n: i64 = c.query_row("SELECT count(*) FROM application_settings", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 1);
    }

    #[test]
    fn rejects_bad_keys_and_oversized_values() {
        let (_t, c) = conn();
        for bad in ["", "Has Space", "UPPER", "a'; DROP TABLE x;--", &"k".repeat(65)] {
            assert!(matches!(set(&c, bad, "1"), Err(AppError::Validation(_))), "key {bad:?}");
        }
        assert!(matches!(set(&c, "ok", &"x".repeat(MAX_VALUE_LEN + 1)), Err(AppError::Validation(_))));
    }
}
