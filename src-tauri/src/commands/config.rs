//! Library configuration commands. Reading needs any signed-in user; changing needs `admin.settings`.

use crate::error::AppResult;
use crate::services::config::{
    self, ArchiveKind, LibraryInput, MemberCategoryInput, NamedInput, NamedKind, Overview,
    PolicyInput, ShelfInput,
};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn config_overview(state: State<'_, AppState>) -> AppResult<Overview> {
    state.current()?;
    state.with_conn(config::overview)
}

#[tauri::command]
pub fn config_update_library(state: State<'_, AppState>, input: LibraryInput) -> AppResult<()> {
    let who = state.require("admin.settings")?;
    state.with_conn(|c| config::update_library(c, Some(&who), &input))
}

#[tauri::command]
pub fn config_save_named(
    state: State<'_, AppState>,
    kind: NamedKind,
    input: NamedInput,
) -> AppResult<i64> {
    let who = state.require("admin.settings")?;
    state.with_conn(|c| config::save_named(c, Some(&who), kind, &input))
}

#[tauri::command]
pub fn config_save_member_category(
    state: State<'_, AppState>,
    input: MemberCategoryInput,
) -> AppResult<i64> {
    let who = state.require("admin.settings")?;
    state.with_conn(|c| config::save_member_category(c, Some(&who), &input))
}

#[tauri::command]
pub fn config_save_shelf(state: State<'_, AppState>, input: ShelfInput) -> AppResult<i64> {
    let who = state.require("admin.settings")?;
    state.with_conn(|c| config::save_shelf(c, Some(&who), &input))
}

#[tauri::command]
pub fn config_save_policy(state: State<'_, AppState>, input: PolicyInput) -> AppResult<i64> {
    let who = state.require("admin.settings")?;
    state.with_conn(|c| config::save_policy(c, Some(&who), &input))
}

#[tauri::command]
pub fn config_delete_policy(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    let who = state.require("admin.settings")?;
    state.with_conn(|c| config::delete_policy(c, Some(&who), id))
}

#[tauri::command]
pub fn config_set_archived(
    state: State<'_, AppState>,
    kind: ArchiveKind,
    id: i64,
    archived: bool,
) -> AppResult<()> {
    let who = state.require("admin.settings")?;
    state.with_conn(|c| config::set_archived(c, Some(&who), kind, id, archived))
}

#[tauri::command]
pub fn config_calendar(
    state: State<'_, AppState>,
) -> AppResult<crate::services::calendar::Calendar> {
    state.current()?;
    state.with_conn(crate::services::calendar::load)
}
#[tauri::command]
pub fn config_save_closed_weekdays(state: State<'_, AppState>, days: Vec<i64>) -> AppResult<()> {
    let who = state.require("admin.settings")?;
    crate::services::calendar::save_weekdays(&mut *state.conn()?, &who, &days)
}
#[tauri::command]
pub fn config_save_holiday(
    state: State<'_, AppState>,
    input: crate::services::calendar::HolidayInput,
) -> AppResult<i64> {
    let who = state.require("admin.settings")?;
    crate::services::calendar::save_holiday(&mut *state.conn()?, &who, &input)
}
#[tauri::command]
pub fn config_remove_holiday(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    let who = state.require("admin.settings")?;
    crate::services::calendar::remove_holiday(&mut *state.conn()?, &who, id)
}
