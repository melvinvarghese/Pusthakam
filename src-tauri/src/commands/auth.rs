//! Setup and sign-in commands. These are the only commands usable before signing in.

use crate::error::AppResult;
use crate::services::auth::{self, Session};
use crate::services::setup::{self, SetupInput, SetupStatus};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn setup_status(state: State<'_, AppState>) -> AppResult<SetupStatus> {
    state.with_conn(setup::status)
}

/// Creates the library and first administrator (only once), then signs the administrator in.
#[tauri::command]
pub fn complete_setup(state: State<'_, AppState>, input: SetupInput) -> AppResult<Session> {
    let session = {
        let mut conn = state.conn()?;
        let admin_id = setup::complete(&mut conn, &input)?;
        auth::load_session(&conn, admin_id)?
    };
    state.set_session(Some(session.clone()))?;
    Ok(session)
}

#[tauri::command]
pub fn login(state: State<'_, AppState>, username: String, password: String) -> AppResult<Session> {
    let session = state.with_conn(|c| auth::login(c, &username, &password))?;
    state.set_session(Some(session.clone()))?;
    Ok(session)
}

#[tauri::command]
pub fn logout(state: State<'_, AppState>) -> AppResult<()> {
    state.set_session(None)
}

#[tauri::command]
pub fn current_session(state: State<'_, AppState>) -> AppResult<Option<Session>> {
    Ok(state.current().ok())
}
