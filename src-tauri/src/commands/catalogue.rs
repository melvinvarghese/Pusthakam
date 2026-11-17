//! Catalogue commands. Reading needs `catalogue.read`; changes need `catalogue.write`;
//! the accession number format is a library setting (`admin.settings`).

use crate::error::AppResult;
use crate::services::catalogue::{
    self, AccessionFormat, Copy, CopyBatchInput, CopyPage, CopyQuery, CopyUpdateInput, DuplicateHit, DuplicateProbe,
    RecordDetail, RecordInput, SearchQuery, SearchResult,
};
use crate::services::reservations;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn catalogue_search(state: State<'_, AppState>, query: SearchQuery) -> AppResult<SearchResult> {
    state.require("catalogue.read")?;
    let mut conn = state.conn()?;
    reservations::refresh(&mut conn)?;
    catalogue::search(&conn, &query)
}

#[tauri::command]
pub fn catalogue_get_record(state: State<'_, AppState>, id: i64) -> AppResult<RecordDetail> {
    state.require("catalogue.read")?;
    let mut conn = state.conn()?;
    reservations::refresh(&mut conn)?;
    catalogue::get_record(&conn, id)
}

#[tauri::command]
pub fn catalogue_save_record(state: State<'_, AppState>, input: RecordInput) -> AppResult<i64> {
    let who = state.require("catalogue.write")?;
    // One transaction so the record, its people, subjects and search entry are saved together or not at all.
    let mut conn = state.conn()?;
    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let id = catalogue::save_record(&tx, Some(&who), &input)?;
    tx.commit()?;
    Ok(id)
}

#[tauri::command]
pub fn catalogue_find_duplicates(state: State<'_, AppState>, probe: DuplicateProbe) -> AppResult<Vec<DuplicateHit>> {
    state.require("catalogue.read")?;
    state.with_conn(|c| catalogue::find_duplicates(c, &probe))
}

#[tauri::command]
pub fn catalogue_archive_record(state: State<'_, AppState>, id: i64, archived: bool) -> AppResult<()> {
    let who = state.require("catalogue.write")?;
    let mut conn = state.conn()?;
    reservations::refresh(&mut conn)?;
    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    catalogue::set_record_archived(&tx, Some(&who), id, archived)?;
    tx.commit()?;
    Ok(())
}

#[tauri::command]
pub fn catalogue_add_copies(state: State<'_, AppState>, record_id: i64, input: CopyBatchInput) -> AppResult<Vec<Copy>> {
    let who = state.require("catalogue.write")?;
    // All copies of a batch are added together or not at all.
    let mut conn = state.conn()?;
    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let copies = catalogue::add_copies(&tx, Some(&who), record_id, &input)?;
    tx.commit()?;
    Ok(copies)
}

#[tauri::command]
pub fn catalogue_update_copy(state: State<'_, AppState>, id: i64, input: CopyUpdateInput) -> AppResult<()> {
    let who = state.require("catalogue.write")?;
    state.with_conn(|c| catalogue::update_copy(c, Some(&who), id, &input))
}

#[tauri::command]
pub fn catalogue_set_copy_status(state: State<'_, AppState>, id: i64, status: String, note: Option<String>) -> AppResult<()> {
    let who = state.require("catalogue.write")?;
    let mut conn = state.conn()?;
    reservations::refresh(&mut conn)?;
    catalogue::set_copy_status(&conn, Some(&who), id, &status, note.as_deref())
}

#[tauri::command]
pub fn catalogue_list_copies(state: State<'_, AppState>, query: CopyQuery) -> AppResult<CopyPage> {
    state.require("catalogue.read")?;
    let mut conn = state.conn()?;
    reservations::refresh(&mut conn)?;
    catalogue::list_copies(&conn, &query)
}

#[tauri::command]
pub fn catalogue_export_copies(state: State<'_, AppState>, query: CopyQuery) -> AppResult<String> {
    state.require("catalogue.read")?;
    let mut conn = state.conn()?;
    reservations::refresh(&mut conn)?;
    catalogue::export_copies_csv(&conn, &query)
}

#[tauri::command]
pub fn accession_format_get(state: State<'_, AppState>) -> AppResult<AccessionFormat> {
    state.current()?;
    state.with_conn(catalogue::accession_format)
}

#[tauri::command]
pub fn accession_format_set(state: State<'_, AppState>, format: AccessionFormat) -> AppResult<()> {
    let who = state.require("admin.settings")?;
    state.with_conn(|c| catalogue::set_accession_format(c, Some(&who), &format))
}
