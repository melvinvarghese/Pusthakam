//! Audit trail helper. Rows are append-only (enforced by triggers). Never put passwords or
//! other secrets in `summary`.

use crate::domain::validate;
use crate::error::{AppError, AppResult};
use crate::services::auth::Session;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditQuery {
    from: Option<String>,
    to: Option<String>,
    actor: Option<String>,
    action: Option<String>,
    page: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AuditEntry {
    id: i64,
    occurred_at: String,
    actor: Option<String>,
    action: String,
    entity_type: String,
    entity_id: Option<i64>,
    summary: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditPage {
    rows: Vec<AuditEntry>,
    total: i64,
    page: i64,
    page_size: i64,
}

pub fn search(conn: &Connection, query: &AuditQuery) -> AppResult<AuditPage> {
    const PAGE_SIZE: i64 = 25;
    const FILTER: &str = "FROM audit_events
        WHERE (?1 IS NULL OR date(occurred_at, 'localtime') >= ?1)
          AND (?2 IS NULL OR date(occurred_at, 'localtime') <= ?2)
          AND (?3 IS NULL OR instr(lower(coalesce(actor_name, '')), lower(?3)) > 0)
          AND (?4 IS NULL OR instr(lower(action), lower(?4)) > 0)";
    let from = query.from.as_deref().filter(|value| !value.trim().is_empty()).map(|value| validate::date("Start date", value)).transpose()?;
    let to = query.to.as_deref().filter(|value| !value.trim().is_empty()).map(|value| validate::date("End date", value)).transpose()?;
    if matches!((&from, &to), (Some(start), Some(end)) if start > end) {
        return Err(AppError::Validation("The end date must be on or after the start date.".into()));
    }
    let actor = validate::optional("Staff name", query.actor.as_deref(), 120)?;
    let action = validate::optional("Action code", query.action.as_deref(), 120)?;
    let total: i64 = conn.query_row(&format!("SELECT count(*) {FILTER}"), params![from, to, actor, action], |row| row.get(0))?;
    let page = query.page.clamp(1, ((total + PAGE_SIZE - 1) / PAGE_SIZE).max(1));
    let mut statement = conn.prepare(&format!(
        "SELECT id, occurred_at, actor_name, action, entity_type, entity_id, summary {FILTER}
         ORDER BY occurred_at DESC, id DESC LIMIT ?5 OFFSET ?6"
    ))?;
    let rows = statement.query_map(params![from, to, actor, action, PAGE_SIZE, (page - 1) * PAGE_SIZE], |row| Ok(AuditEntry {
        id: row.get(0)?, occurred_at: row.get(1)?, actor: row.get(2)?, action: row.get(3)?,
        entity_type: row.get(4)?, entity_id: row.get(5)?, summary: row.get(6)?,
    }))?.collect::<Result<_, _>>()?;
    Ok(AuditPage { rows, total, page, page_size: PAGE_SIZE })
}
