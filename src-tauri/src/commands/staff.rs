use crate::error::AppResult;
use crate::services::staff::{self, Directory, NewStaff, StaffChanges};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn staff_list(state: State<'_, AppState>) -> AppResult<Directory> {
    state.require("admin.staff")?;
    state.with_conn(staff::list)
}

#[tauri::command]
pub fn staff_create(state: State<'_, AppState>, input: NewStaff) -> AppResult<i64> {
    let actor = state.require("admin.staff")?;
    staff::create(&mut *state.conn()?, &actor, &input)
}

#[tauri::command]
pub fn staff_update(state: State<'_, AppState>, id: i64, input: StaffChanges) -> AppResult<()> {
    let actor = state.require("admin.staff")?;
    staff::update(&mut *state.conn()?, &actor, id, &input)
}

#[tauri::command]
pub fn staff_reset_password(state: State<'_, AppState>, id: i64, password: String) -> AppResult<()> {
    let actor = state.require("admin.staff")?;
    staff::reset_password(&mut *state.conn()?, &actor, id, &password)
}

#[tauri::command]
pub fn staff_unlock(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    let actor = state.require("admin.staff")?;
    staff::unlock(&mut *state.conn()?, &actor, id)
}

#[tauri::command]
pub fn staff_change_password(state: State<'_, AppState>, current_password: String, new_password: String) -> AppResult<()> {
    let actor = state.current()?;
    staff::change_password(&mut *state.conn()?, &actor, &current_password, &new_password)
}

#[tauri::command]
pub fn staff_generate_recovery_code(state: State<'_, AppState>, current_password: String) -> AppResult<String> {
    let actor = state.require("admin.staff")?;
    staff::generate_recovery_code(&mut *state.conn()?, &actor, &current_password)
}

#[tauri::command]
pub fn staff_recover(state: State<'_, AppState>, username: String, code: String, new_password: String) -> AppResult<()> {
    staff::recover(&mut *state.conn()?, &username, &code, &new_password)
}
