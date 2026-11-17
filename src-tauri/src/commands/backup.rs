use crate::error::AppResult;
use crate::services::backup::{self, BackupFile, BackupList};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn backup_list(state: State<'_, AppState>) -> AppResult<BackupList> {
    state.require("admin.backup")?;
    state.with_conn(|conn| backup::list(conn, &state.paths))
}

#[tauri::command]
pub fn backup_set_daily(state: State<'_, AppState>, enabled: bool) -> AppResult<()> {
    state.require("admin.backup")?;
    state.with_conn(|conn| backup::set_daily(conn, enabled))
}

#[tauri::command]
pub async fn backup_verify(state: State<'_, AppState>, name: String) -> AppResult<i64> {
    state.require("admin.backup")?;
    backup::verify_saved(&state.paths.backups, &name)
}

#[tauri::command]
pub async fn backup_request_restore(state: State<'_, AppState>, name: String) -> AppResult<()> {
    state.require("admin.backup")?;
    backup::request_restore(&state.paths, &name)
}

#[tauri::command]
pub fn backup_cancel_restore(state: State<'_, AppState>) -> AppResult<()> {
    state.require("admin.backup")?;
    backup::cancel_restore(&state.paths)
}

#[tauri::command]
pub async fn backup_create(state: State<'_, AppState>) -> AppResult<BackupFile> {
    state.require("admin.backup")?;
    state.with_conn(|conn| backup::create(conn, &state.paths.backups))
}
