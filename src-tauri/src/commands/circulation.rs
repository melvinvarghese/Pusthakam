use crate::error::AppResult;
use crate::services::circulation::{self, IssuePreview, OverduePage, RenewalConfirmation, RenewalPreview, ReturnCondition, ReturnPreview};
use crate::services::fines::{self, FineAmount, FinePage, Settlement, SettlementPage};
use crate::services::reservations::{self, ReservationPage};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn circulation_preview_renewal_import(state: State<'_, AppState>, data_base64: String, source_system: String) -> AppResult<crate::services::renewal_import::Preview> {
    state.require("admin.settings")?;
    state.with_conn(|conn| crate::services::renewal_import::preview(conn, &data_base64, &source_system))
}

#[tauri::command]
pub fn circulation_commit_renewal_import(state: State<'_, AppState>, data_base64: String, source_system: String, expected_fingerprint: String) -> AppResult<usize> {
    let actor=state.require("admin.settings")?;
    crate::services::renewal_import::commit(&mut *state.conn()?, &actor, &data_base64, &source_system, &expected_fingerprint)
}

#[tauri::command]
pub fn circulation_preview_import(state: State<'_, AppState>, data_base64: String) -> AppResult<crate::services::loan_import::Preview> {
    state.require("admin.settings")?;
    state.with_conn(|conn| crate::services::loan_import::preview_base64(conn, &data_base64))
}

#[tauri::command]
pub fn circulation_commit_import(state: State<'_, AppState>, data_base64: String, source_system: String, expected_fingerprint: String, use_current_terms: bool) -> AppResult<usize> {
    let actor = state.require("admin.settings")?;
    crate::services::loan_import::commit(&mut *state.conn()?, &actor, &data_base64, &source_system, &expected_fingerprint, use_current_terms)
}

#[tauri::command]
pub fn circulation_list_overdue(state: State<'_, AppState>, page: i64) -> AppResult<OverduePage> {
    state.require("circulation.issue")?;
    state.with_conn(|conn| circulation::list_overdue(conn, page))
}

#[tauri::command]
pub fn circulation_prepare_issue(state: State<'_, AppState>, member_number: String, barcode: String) -> AppResult<IssuePreview> {
    state.require("circulation.issue")?;
    circulation::prepare_issue(&mut *state.conn()?, &member_number, &barcode)
}

#[tauri::command]
pub fn circulation_prepare_return(state: State<'_, AppState>, barcode: String) -> AppResult<ReturnPreview> {
    state.require("circulation.issue")?;
    state.with_conn(|conn| circulation::prepare_return(conn, &barcode))
}

#[tauri::command]
pub fn circulation_issue(
    state: State<'_, AppState>, member_number: String, barcode: String, member_id: i64, item_id: i64, due_on: String,
) -> AppResult<()> {
    let actor = state.require("circulation.issue")?;
    circulation::issue(&mut *state.conn()?, &actor, &member_number, &barcode, (member_id, item_id, &due_on))
}

#[tauri::command]
pub fn circulation_return(state: State<'_, AppState>, loan_id: i64, condition: Option<ReturnCondition>) -> AppResult<Option<FineAmount>> {
    let actor = state.require("circulation.issue")?;
    circulation::return_copy(&mut *state.conn()?, &actor, loan_id, condition.unwrap_or_default())
}

#[tauri::command]
pub fn circulation_prepare_renewal(state: State<'_, AppState>, barcode: String) -> AppResult<RenewalPreview> {
    state.require("circulation.issue")?;
    state.with_conn(|conn| circulation::prepare_renewal(conn, &barcode))
}

#[tauri::command]
pub fn circulation_renew(state: State<'_, AppState>, confirmation: RenewalConfirmation) -> AppResult<()> {
    let actor = state.require("circulation.issue")?;
    circulation::renew(&mut *state.conn()?, &actor, &confirmation)
}

#[tauri::command]
pub fn circulation_list_reservations(state: State<'_, AppState>, page: i64) -> AppResult<ReservationPage> {
    state.require("circulation.issue")?;
    reservations::list(&mut *state.conn()?, page)
}

#[tauri::command]
pub fn circulation_place_reservation(state: State<'_, AppState>, member_number: String, barcode: String) -> AppResult<()> {
    let actor = state.require("circulation.issue")?;
    reservations::place(&mut *state.conn()?, &actor, &member_number, &barcode)
}

#[tauri::command]
pub fn circulation_cancel_reservation(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    let actor = state.require("circulation.issue")?;
    reservations::cancel(&mut *state.conn()?, &actor, id)
}

#[tauri::command]
pub fn circulation_list_fines(state: State<'_, AppState>, page: i64, include_settled: bool) -> AppResult<FinePage> {
    state.require("circulation.issue")?;
    state.with_conn(|conn| fines::list(conn, page, include_settled))
}

#[tauri::command]
pub fn circulation_settle_fine(
    state: State<'_, AppState>, fine_id: i64, amount_minor: i64, expected_balance: i64, entry: Settlement,
) -> AppResult<()> {
    let actor = state.require("circulation.issue")?;
    fines::settle(&mut *state.conn()?, &actor, fine_id, amount_minor, expected_balance, entry)
}

#[tauri::command]
pub fn circulation_fine_history(state: State<'_, AppState>, fine_id: i64, page: i64) -> AppResult<SettlementPage> {
    let actor = state.require("circulation.issue")?;
    state.with_conn(|conn| fines::history(conn, fine_id, page, actor.can("circulation.manage")))
}
