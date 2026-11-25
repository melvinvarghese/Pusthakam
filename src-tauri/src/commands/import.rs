//! Import commands: preview a file (nothing is saved), then commit it after the librarian confirms.

use crate::error::{AppError, AppResult};
use crate::services::import::{self, CommitOptions, Preview, PreviewRequest, Report};
use crate::state::AppState;
use tauri::State;

crate::shared_command! {

pub fn import_preview(state: State<'_, AppState>, request: PreviewRequest) -> AppResult<Preview> {
    state.require("catalogue.write")?;
    let (pending, preview) = state.with_conn(|c| import::prepare(c, &request))?;
    *state
        .pending_import
        .lock()
        .map_err(|_| AppError::Internal("import lock poisoned".into()))? = Some(pending);
    Ok(preview)
}
}

crate::shared_command! {

pub fn import_commit(
    state: State<'_, AppState>,
    token: String,
    options: CommitOptions,
) -> AppResult<Report> {
    let who = state.require("catalogue.write")?;
    let pending = {
        let mut slot = state
            .pending_import
            .lock()
            .map_err(|_| AppError::Internal("import lock poisoned".into()))?;
        match slot.as_ref() {
            Some(p) if p.token == token => slot.take(),
            _ => None,
        }
    }
    .ok_or_else(|| {
        AppError::NotFound("This import is no longer available. Choose the file again.".into())
    })?;
    let mut conn = state.conn()?;
    match import::commit(
        &mut conn,
        Some(&who),
        &pending.records,
        &options,
        &pending.source,
    ) {
        Ok(report) => Ok(report),
        Err(e) => {
            // Keep the preview so the librarian can fix the option (for example the default category) and retry.
            drop(conn);
            *state
                .pending_import
                .lock()
                .map_err(|_| AppError::Internal("import lock poisoned".into()))? = Some(pending);
            Err(e)
        }
    }
}
}

crate::shared_command! {

pub fn import_cancel(state: State<'_, AppState>) -> AppResult<()> {
    state.require("catalogue.write")?;
    *state
        .pending_import
        .lock()
        .map_err(|_| AppError::Internal("import lock poisoned".into()))? = None;
    Ok(())
}
}
