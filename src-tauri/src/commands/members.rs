use crate::error::AppResult;
use crate::services::members::{self, MemberInput, MemberPage};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn members_list(state: State<'_, AppState>, search: String, page: i64, archived: bool) -> AppResult<MemberPage> {
    state.require("members.read")?;
    state.with_conn(|conn| members::list(conn, &search, page, archived))
}

#[tauri::command]
pub fn members_save(state: State<'_, AppState>, input: MemberInput) -> AppResult<i64> {
    let actor = state.require("members.write")?;
    members::save(&mut *state.conn()?, &actor, input)
}

#[tauri::command]
pub fn members_archive(state: State<'_, AppState>, id: i64, archived: bool) -> AppResult<()> {
    let actor = state.require("members.write")?;
    members::archive(&mut *state.conn()?, &actor, id, archived)
}
