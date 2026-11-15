//! Audit trail helper. Rows are append-only (enforced by triggers). Never put passwords or
//! other secrets in `summary`.

use crate::error::AppResult;
use crate::services::auth::Session;
use rusqlite::Connection;

pub fn record(
    conn: &Connection,
    actor: Option<&Session>,
    action: &str,
    entity_type: &str,
    entity_id: Option<i64>,
    summary: &str,
) -> AppResult<()> {
    conn.execute(
        "INSERT INTO audit_events (staff_user_id, actor_name, action, entity_type, entity_id, summary)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            actor.map(|a| a.user_id),
            actor.map(|a| a.display_name.as_str()),
            action,
            entity_type,
            entity_id,
            summary
        ],
    )?;
    Ok(())
}
