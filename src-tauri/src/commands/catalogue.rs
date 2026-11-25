//! Catalogue commands. Reading needs `catalogue.read`; changes need `catalogue.write`;
//! the accession number format is a library setting (`admin.settings`).

use crate::error::AppResult;
use crate::services::catalogue::{
    self, AccessionFormat, Copy, CopyBatchInput, CopyPage, CopyQuery, CopyUpdateInput,
    DuplicateHit, DuplicateProbe, RecordDetail, RecordInput, SearchQuery, SearchResult,
};
use crate::services::reservations;
use crate::state::AppState;
use tauri::State;

crate::shared_command! {

pub fn catalogue_get_cover(state: State<'_, AppState>, id: i64) -> AppResult<Option<String>> {
    state.require("catalogue.read")?;
    state.with_conn(|conn| crate::services::book_cover::get(conn, id))
}
}

crate::shared_command! {

pub fn catalogue_set_cover(
    state: State<'_, AppState>,
    id: i64,
    png_base64: Option<String>,
) -> AppResult<()> {
    let actor = state.require("catalogue.write")?;
    crate::services::book_cover::set(&mut *state.conn()?, &actor, id, png_base64.as_deref())
}
}

crate::shared_command! {

pub fn catalogue_search(state: State<'_, AppState>, query: SearchQuery) -> AppResult<SearchResult> {
    state.require("catalogue.read")?;
    let mut conn = state.conn()?;
    reservations::refresh(&mut conn)?;
    catalogue::search(&conn, &query)
}
}

crate::shared_command! {

pub fn catalogue_get_record(state: State<'_, AppState>, id: i64) -> AppResult<RecordDetail> {
    state.require("catalogue.read")?;
    let mut conn = state.conn()?;
    reservations::refresh(&mut conn)?;
    catalogue::get_record(&conn, id)
}
}

crate::shared_command! {

pub fn catalogue_save_record(state: State<'_, AppState>, input: RecordInput) -> AppResult<i64> {
    let who = state.require("catalogue.write")?;
    // One transaction so the record, its people, subjects and search entry are saved together or not at all.
    let mut conn = state.conn()?;
    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let id = catalogue::save_record(&tx, Some(&who), &input)?;
    tx.commit()?;
    Ok(id)
}
}

crate::shared_command! {

pub fn catalogue_find_duplicates(
    state: State<'_, AppState>,
    probe: DuplicateProbe,
) -> AppResult<Vec<DuplicateHit>> {
    state.require("catalogue.read")?;
    state.with_conn(|c| catalogue::find_duplicates(c, &probe))
}
}

crate::shared_command! {

pub fn catalogue_archive_record(
    state: State<'_, AppState>,
    id: i64,
    archived: bool,
) -> AppResult<()> {
    archive_records(state, vec![id], archived)
}
}

crate::shared_command! {

pub fn catalogue_archive_records(state: State<'_, AppState>, ids: Vec<i64>) -> AppResult<()> {
    archive_records(state, ids, true)
}
}

fn archive_records(state: &AppState, mut ids: Vec<i64>, archived: bool) -> AppResult<()> {
    let who = state.require("catalogue.write")?;
    if ids.is_empty() || ids.len() > 100 {
        return Err(crate::error::AppError::Validation(
            "Select between 1 and 100 records.".into(),
        ));
    }
    ids.sort_unstable();
    ids.dedup();
    let mut conn = state.conn()?;
    reservations::refresh(&mut conn)?;
    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    for id in ids {
        catalogue::set_record_archived(&tx, Some(&who), id, archived)?;
    }
    tx.commit()?;
    Ok(())
}

crate::shared_command! {

pub fn catalogue_add_copies(
    state: State<'_, AppState>,
    record_id: i64,
    input: CopyBatchInput,
) -> AppResult<Vec<Copy>> {
    let who = state.require("catalogue.write")?;
    // All copies of a batch are added together or not at all.
    let mut conn = state.conn()?;
    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let copies = catalogue::add_copies(&tx, Some(&who), record_id, &input)?;
    tx.commit()?;
    Ok(copies)
}
}

crate::shared_command! {

pub fn catalogue_update_copy(
    state: State<'_, AppState>,
    id: i64,
    input: CopyUpdateInput,
) -> AppResult<()> {
    let who = state.require("catalogue.write")?;
    state.with_conn(|c| catalogue::update_copy(c, Some(&who), id, &input))
}
}

crate::shared_command! {

pub fn catalogue_set_copy_status(
    state: State<'_, AppState>,
    id: i64,
    status: String,
    note: Option<String>,
) -> AppResult<()> {
    let who = state.require("catalogue.write")?;
    let mut conn = state.conn()?;
    reservations::refresh(&mut conn)?;
    catalogue::set_copy_status(&conn, Some(&who), id, &status, note.as_deref())
}
}

crate::shared_command! {

pub fn catalogue_list_copies(state: State<'_, AppState>, query: CopyQuery) -> AppResult<CopyPage> {
    state.require("catalogue.read")?;
    let mut conn = state.conn()?;
    reservations::refresh(&mut conn)?;
    catalogue::list_copies(&conn, &query)
}
}

crate::shared_command! {

pub fn catalogue_export_copies(state: State<'_, AppState>, query: CopyQuery) -> AppResult<String> {
    state.require("catalogue.read")?;
    let mut conn = state.conn()?;
    reservations::refresh(&mut conn)?;
    catalogue::export_copies_csv(&conn, &query)
}
}

crate::shared_command! {

pub fn accession_format_get(state: State<'_, AppState>) -> AppResult<AccessionFormat> {
    state.current()?;
    state.with_conn(catalogue::accession_format)
}
}

crate::shared_command! {

pub fn accession_format_set(state: State<'_, AppState>, format: AccessionFormat) -> AppResult<()> {
    let who = state.require("admin.settings")?;
    state.with_conn(|c| catalogue::set_accession_format(c, Some(&who), &format))
}
}

crate::shared_command! {

pub fn catalogue_export_records(
    state: State<'_, AppState>,
    ids: Vec<i64>,
    format: String,
) -> AppResult<crate::services::catalogue_export::ExportFile> {
    state.require("catalogue.read")?;
    crate::services::catalogue_export::export(&mut *state.conn()?, &ids, &format)
}
}

#[tauri::command]
pub async fn catalogue_save_export<R: tauri::Runtime>(
    window: tauri::WebviewWindow<R>,
    state: State<'_, AppState>,
    file_name: String,
    data_base64: String,
) -> AppResult<crate::services::export_save::SaveResult> {
    use crate::services::export_save;
    state.require("catalogue.read")?;
    let payload = export_save::prepare(&file_name, &data_base64)?;
    export_save::save(window, state, payload, "catalogue.read").await
}
