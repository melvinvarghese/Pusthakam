//! Application-level commands. Thin: validate, call a service, return a DTO.
//! NOTE: authentication/permission checks arrive with staff accounts (Phase 2); see KNOWN_ISSUES KI-004.

use crate::db::migrations::latest_version;
use crate::error::{AppError, AppResult};
use crate::services::settings;
use crate::state::AppState;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: String,
    pub schema_version: i64,
    pub data_dir: String,
    pub database_path: String,
}

#[tauri::command]
pub fn app_info(state: State<'_, AppState>) -> AppResult<AppInfo> {
    let conn = state.db.lock().map_err(|_| AppError::Internal("database lock poisoned".into()))?;
    let schema_version: i64 =
        conn.query_row("SELECT COALESCE(MAX(version),0) FROM schema_migrations", [], |r| r.get(0))?;
    debug_assert!(schema_version <= latest_version());
    Ok(AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        schema_version,
        data_dir: state.paths.data_dir.display().to_string(),
        database_path: state.paths.database.display().to_string(),
    })
}

#[tauri::command]
pub fn get_setting(state: State<'_, AppState>, key: String) -> AppResult<Option<String>> {
    let conn = state.db.lock().map_err(|_| AppError::Internal("database lock poisoned".into()))?;
    settings::get(&conn, &key)
}

#[tauri::command]
pub fn set_setting(state: State<'_, AppState>, key: String, value: String) -> AppResult<()> {
    let conn = state.db.lock().map_err(|_| AppError::Internal("database lock poisoned".into()))?;
    settings::set(&conn, &key, &value)
}
