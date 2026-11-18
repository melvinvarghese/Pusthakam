use crate::error::AppResult;
use crate::services::{member_import, member_photo};
use crate::services::members::{self, MemberInput, MemberLoanPage, MemberPage};
use crate::state::AppState;
use std::collections::BTreeMap;
use tauri::State;

#[tauri::command]
pub fn members_list(state: State<'_, AppState>, search: String, page: i64, archived: bool) -> AppResult<MemberPage> {
    state.require("members.read")?;
    state.with_conn(|conn| members::list(conn, &search, page, archived))
}

#[tauri::command]
pub fn members_list_loans(state: State<'_, AppState>, member_id: i64, page: i64) -> AppResult<MemberLoanPage> {
    state.require("members.read")?;
    state.with_conn(|conn| members::loans(conn, member_id, page))
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

#[tauri::command]
pub async fn members_preview_koha(state: State<'_, AppState>, data_base64: String) -> AppResult<member_import::Preview> {
    state.require("members.write")?;
    state.with_conn(|conn| member_import::preview(conn, &data_base64))
}

#[tauri::command]
pub async fn members_import_koha(
    state: State<'_, AppState>, data_base64: String, mappings: BTreeMap<String, i64>, expected_ready: usize,
) -> AppResult<usize> {
    let actor = state.require("members.write")?;
    member_import::commit(&mut *state.conn()?, &actor, &data_base64, &mappings, expected_ready)
}

#[tauri::command]
pub fn members_get_photo(state: State<'_, AppState>, id: i64) -> AppResult<Option<String>> {
    state.require("members.read")?;
    state.with_conn(|conn| member_photo::get(conn, id))
}

#[tauri::command]
pub fn members_set_photo(state: State<'_, AppState>, id: i64, png_base64: Option<String>) -> AppResult<()> {
    let actor = state.require("members.write")?;
    member_photo::set(&mut *state.conn()?, &actor, id, png_base64.as_deref())
}

#[tauri::command]
pub fn members_fine_account(state: State<'_, AppState>, member_id: i64, page: i64) -> AppResult<crate::services::fines::MemberAccount> {
    state.require("members.read")?;
    crate::services::fines::member_account(&mut *state.conn()?, member_id, page)
}
#[tauri::command]
pub fn members_fine_history(state: State<'_, AppState>, member_id: i64, fine_id: i64, page: i64) -> AppResult<crate::services::fines::SettlementPage> {
    let actor = state.require("members.read")?;
    state.with_conn(|conn| crate::services::fines::member_history(conn, member_id, fine_id, page, actor.can("circulation.manage")))
}
