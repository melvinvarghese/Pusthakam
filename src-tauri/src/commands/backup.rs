use crate::error::AppResult;
use crate::services::backup::{self, BackupFile, BackupList};
use crate::state::AppState;
use tauri::State;

crate::shared_command! {

pub fn backup_list(state: State<'_, AppState>) -> AppResult<BackupList> {
    state.require("admin.backup")?;
    state.with_conn(|conn| backup::list(conn, &state.paths))
}
}

crate::shared_command! {

pub fn backup_set_daily(state: State<'_, AppState>, enabled: bool) -> AppResult<()> {
    state.require("admin.backup")?;
    state.with_conn(|conn| backup::set_daily(conn, enabled))
}
}

crate::shared_command! {

pub async fn backup_verify(state: State<'_, AppState>, name: String) -> AppResult<i64> {
    state.require("admin.backup")?;
    backup::verify(&state.paths, &name)
}
}

crate::shared_command! {

pub async fn backup_request_restore(state: State<'_, AppState>, name: String) -> AppResult<()> {
    state.require("admin.backup")?;
    state.with_conn(|conn| backup::request_restore(conn, &state.paths, &name))
}
}

crate::shared_command! {

pub fn backup_cancel_restore(state: State<'_, AppState>) -> AppResult<()> {
    state.require("admin.backup")?;
    backup::cancel_restore(&state.paths)
}
}

crate::shared_command! {

pub async fn backup_create(state: State<'_, AppState>) -> AppResult<BackupFile> {
    state.require("admin.backup")?;
    state.with_conn(|conn| backup::create(conn, &state.paths.backups))
}
}

crate::shared_command! {

pub async fn backup_copy_to_folder(
    state: State<'_, AppState>,
    name: String,
    folder: String,
) -> AppResult<BackupFile> {
    state.require("admin.backup")?;
    backup::copy_to_folder(&state.paths, &name, &folder)
}
}

/// Opens the Windows folder dialog so staff can point at a USB or second drive instead of typing
/// a path. Allowed in the setup wizard (before any account exists) and to backup administrators.
/// Desktop only: a browser user must not open dialogs on the host computer.
#[tauri::command]
pub async fn backup_pick_folder<R: tauri::Runtime>(
    window: tauri::WebviewWindow<R>,
    state: State<'_, AppState>,
) -> AppResult<Option<String>> {
    let configured =
        state.with_conn(|conn| Ok(crate::services::setup::status(conn)?.configured))?;
    if configured {
        state.require("admin.backup")?;
    }
    #[cfg(windows)]
    let owner = window
        .hwnd()
        .map_err(|_| {
            crate::error::AppError::Validation("Could not open the folder dialog.".into())
        })?
        .0 as usize;
    #[cfg(not(windows))]
    let owner = {
        let _ = window;
        0
    };
    let chosen = tauri::async_runtime::spawn_blocking(move || {
        crate::services::folder_picker::choose(
            owner,
            "Choose the folder for Pusthakam's daily backups",
        )
    })
    .await
    .map_err(|_| crate::error::AppError::Internal("Folder dialog task failed.".into()))??;
    Ok(chosen.map(|path| path.to_string_lossy().into_owned()))
}

crate::shared_command! {

/// Folder for dated daily backups on this computer. An empty value means the Pusthakam data folder.
pub fn backup_set_daily_folder(state: State<'_, AppState>, folder: String) -> AppResult<Option<String>> {
    state.require("admin.backup")?;
    backup::set_daily_folder(&state.paths, &folder)
}
}
