//! Application settings stored in `application_settings` (key -> JSON text).

use crate::error::{AppError, AppResult};
use rusqlite::{Connection, OptionalExtension};

const MAX_VALUE_LEN: usize = 10_000;
const CUSTOM_FIELDS_KEY: &str = "library.custom_fields";

#[derive(serde::Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
enum LibraryField {
    Text { label: String, value: String },
    Number { label: String, value: f64 },
    Checkbox { label: String, value: bool },
}

fn validate_custom_fields(json: &str) -> AppResult<()> {
    let invalid = || AppError::Validation("Enter unique field labels and values that match each field type.".into());
    let fields: Vec<LibraryField> = serde_json::from_str(json).map_err(|_| invalid())?;
    let mut seen = std::collections::HashSet::new();
    for field in fields {
        let (label, valid) = match field {
            LibraryField::Text { label, value } => (label, value.chars().count() <= 500),
            LibraryField::Number { label, value } => (label, value.is_finite()),
            LibraryField::Checkbox { label, value: _value } => (label, true),
        };
        let label = label.trim();
        if !valid || label.is_empty() || label.chars().count() > 80 || !seen.insert(label.to_lowercase()) {
            return Err(invalid());
        }
    }
    Ok(())
}

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
    if key == CUSTOM_FIELDS_KEY {
        validate_custom_fields(value)?;
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

    #[test]
    fn custom_library_fields_validate_and_survive_reopen() {
        let (tmp, conn) = conn();
        let valid = r#"[{"label":"District","kind":"text","value":"Ernakulam"},{"label":"Floors","kind":"number","value":2},{"label":"Public access","kind":"checkbox","value":true}]"#;
        set(&conn, CUSTOM_FIELDS_KEY, valid).unwrap();
        for invalid in [
            r#"[{"label":"Floors","kind":"number","value":"2"}]"#,
            r#"[{"label":"District","kind":"text","value":"A"},{"label":" district ","kind":"text","value":"B"}]"#,
            r#"[{"label":" ","kind":"checkbox","value":true}]"#,
        ] {
            assert!(matches!(set(&conn, CUSTOM_FIELDS_KEY, invalid), Err(AppError::Validation(_))));
        }
        drop(conn);
        let reopened = db::open(&tmp.path().join("s.db")).unwrap();
        assert_eq!(get(&reopened, CUSTOM_FIELDS_KEY).unwrap().as_deref(), Some(valid));
    }

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
