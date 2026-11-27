use crate::error::AppResult;
use crate::services::members::{
    self, MemberActivityItem, MemberHold, MemberInput, MemberLoanPage, MemberPage, MemberSummary,
};
use crate::services::{member_import, member_photo};
use crate::state::AppState;
use std::collections::BTreeMap;
use tauri::State;

crate::shared_command! {

pub fn members_list(
    state: State<'_, AppState>,
    search: String,
    page: i64,
    archived: bool,
) -> AppResult<MemberPage> {
    state.require("members.read")?;
    state.with_conn(|conn| members::list(conn, &search, page, archived))
}
}

crate::shared_command! {

pub fn members_list_loans(
    state: State<'_, AppState>,
    member_id: i64,
    page: i64,
) -> AppResult<MemberLoanPage> {
    state.require("members.read")?;
    state.with_conn(|conn| members::loans(conn, member_id, page))
}
}

crate::shared_command! {

pub fn members_save(state: State<'_, AppState>, input: MemberInput) -> AppResult<i64> {
    let actor = state.require("members.write")?;
    members::save(&mut *state.conn()?, &actor, input)
}
}

crate::shared_command! {

pub fn members_archive(state: State<'_, AppState>, id: i64, archived: bool) -> AppResult<()> {
    let actor = state.require("members.write")?;
    members::archive(&mut *state.conn()?, &actor, id, archived)
}
}

crate::shared_command! {

pub async fn members_preview_koha(
    state: State<'_, AppState>,
    data_base64: String,
) -> AppResult<member_import::Preview> {
    state.require("members.write")?;
    state.with_conn(|conn| member_import::preview(conn, &data_base64))
}
}

crate::shared_command! {

pub async fn members_import_koha(
    state: State<'_, AppState>,
    data_base64: String,
    mappings: BTreeMap<String, i64>,
    expected_ready: usize,
) -> AppResult<usize> {
    let actor = state.require("members.write")?;
    member_import::commit(
        &mut *state.conn()?,
        &actor,
        &data_base64,
        &mappings,
        expected_ready,
    )
}
}

crate::shared_command! {

pub fn members_get_photo(state: State<'_, AppState>, id: i64) -> AppResult<Option<String>> {
    state.require("members.read")?;
    state.with_conn(|conn| member_photo::get(conn, id))
}
}

crate::shared_command! {

pub fn members_set_photo(
    state: State<'_, AppState>,
    id: i64,
    png_base64: Option<String>,
) -> AppResult<()> {
    let actor = state.require("members.write")?;
    member_photo::set(&mut *state.conn()?, &actor, id, png_base64.as_deref())
}
}

crate::shared_command! {

pub fn members_set_opac_credentials(
    state: State<'_, AppState>,
    id: i64,
    username: Option<String>,
    password: Option<String>,
    clear_password: bool,
) -> AppResult<()> {
    let actor = state.require("members.write")?;
    members::set_opac_credentials(
        &mut *state.conn()?,
        &actor,
        id,
        username.as_deref(),
        password.as_deref(),
        clear_password,
    )
}
}

crate::shared_command! {

pub fn members_summary(state: State<'_, AppState>, id: i64) -> AppResult<MemberSummary> {
    state.require("members.read")?;
    state.with_conn(|conn| members::summary(conn, id))
}
}

crate::shared_command! {

pub fn members_renew(state: State<'_, AppState>, id: i64) -> AppResult<String> {
    let actor = state.require("members.write")?;
    members::renew_membership(&mut *state.conn()?, &actor, id)
}
}

crate::shared_command! {

pub fn members_holds(state: State<'_, AppState>, id: i64) -> AppResult<Vec<MemberHold>> {
    state.require("members.read")?;
    members::holds(&mut *state.conn()?, id)
}
}

crate::shared_command! {

pub fn members_activity(state: State<'_, AppState>, id: i64) -> AppResult<Vec<MemberActivityItem>> {
    state.require("members.read")?;
    state.with_conn(|conn| members::activity(conn, id))
}
}

crate::shared_command! {

pub fn members_fine_account(
    state: State<'_, AppState>,
    member_id: i64,
    page: i64,
) -> AppResult<crate::services::fines::MemberAccount> {
    state.require("members.read")?;
    crate::services::fines::member_account(&mut *state.conn()?, member_id, page)
}
}
crate::shared_command! {

pub fn members_fine_history(
    state: State<'_, AppState>,
    member_id: i64,
    fine_id: i64,
    page: i64,
) -> AppResult<crate::services::fines::SettlementPage> {
    let actor = state.require("members.read")?;
    state.with_conn(|conn| {
        crate::services::fines::member_history(
            conn,
            member_id,
            fine_id,
            page,
            actor.can("circulation.manage"),
        )
    })
}
}
