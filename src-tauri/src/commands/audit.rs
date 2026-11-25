use crate::error::AppResult;
use crate::services::audit::{self, AuditPage, AuditQuery};
use crate::state::AppState;
use tauri::State;

crate::shared_command! {

pub fn audit_search(state: State<'_, AppState>, query: AuditQuery) -> AppResult<AuditPage> {
    state.require("audit.read")?;
    state.with_conn(|conn| audit::search(conn, &query))
}
}
