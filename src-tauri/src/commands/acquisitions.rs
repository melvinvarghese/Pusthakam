use crate::error::AppResult;
use crate::services::acquisitions::{
    self, AcquisitionBudget, AcquisitionRequest, BudgetInput, CreateOrderInput, CreateRequestInput,
    PurchaseOrderDetail, PurchaseOrderSummary, ReceiveItemInput, ReceiveResult, Vendor,
    VendorInput,
};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn acquisitions_list_vendors(
    state: State<'_, AppState>,
    include_archived: bool,
) -> AppResult<Vec<Vendor>> {
    state.require("acquisitions.read")?;
    state.with_conn(|conn| acquisitions::list_vendors(conn, include_archived))
}

#[tauri::command]
pub fn acquisitions_save_vendor(state: State<'_, AppState>, input: VendorInput) -> AppResult<i64> {
    let session = state.require("acquisitions.write")?;
    acquisitions::save_vendor(&mut *state.conn()?, input, &session)
}

#[tauri::command]
pub fn acquisitions_archive_vendor(
    state: State<'_, AppState>,
    id: i64,
    archive: bool,
) -> AppResult<()> {
    let session = state.require("acquisitions.write")?;
    acquisitions::archive_vendor(&mut *state.conn()?, id, archive, &session)
}

#[tauri::command]
pub fn acquisitions_list_budgets(
    state: State<'_, AppState>,
    include_archived: bool,
) -> AppResult<Vec<AcquisitionBudget>> {
    state.require("acquisitions.read")?;
    state.with_conn(|conn| acquisitions::list_budgets(conn, include_archived))
}

#[tauri::command]
pub fn acquisitions_save_budget(state: State<'_, AppState>, input: BudgetInput) -> AppResult<i64> {
    let session = state.require("acquisitions.write")?;
    acquisitions::save_budget(&mut *state.conn()?, input, &session)
}

#[tauri::command]
pub fn acquisitions_archive_budget(
    state: State<'_, AppState>,
    id: i64,
    archive: bool,
) -> AppResult<()> {
    let session = state.require("acquisitions.write")?;
    acquisitions::archive_budget(&mut *state.conn()?, id, archive, &session)
}

#[tauri::command]
pub fn acquisitions_list_requests(
    state: State<'_, AppState>,
    status: Option<String>,
) -> AppResult<Vec<AcquisitionRequest>> {
    state.require("acquisitions.read")?;
    state.with_conn(|conn| acquisitions::list_requests(conn, status))
}

#[tauri::command]
pub fn acquisitions_create_request(
    state: State<'_, AppState>,
    input: CreateRequestInput,
) -> AppResult<i64> {
    let session = state.require("acquisitions.write")?;
    acquisitions::create_request(&mut *state.conn()?, input, &session)
}

#[tauri::command]
pub fn acquisitions_approve_request(
    state: State<'_, AppState>,
    id: i64,
    budget_id: Option<i64>,
) -> AppResult<()> {
    let session = state.require("acquisitions.write")?;
    acquisitions::approve_request(&mut *state.conn()?, id, budget_id, &session)
}

#[tauri::command]
pub fn acquisitions_reject_request(
    state: State<'_, AppState>,
    id: i64,
    reason: Option<String>,
) -> AppResult<()> {
    let session = state.require("acquisitions.write")?;
    acquisitions::reject_request(&mut *state.conn()?, id, reason, &session)
}

#[tauri::command]
pub fn acquisitions_list_orders(
    state: State<'_, AppState>,
    status: Option<String>,
) -> AppResult<Vec<PurchaseOrderSummary>> {
    state.require("acquisitions.read")?;
    state.with_conn(|conn| acquisitions::list_orders(conn, status))
}

#[tauri::command]
pub fn acquisitions_get_order(
    state: State<'_, AppState>,
    id: i64,
) -> AppResult<PurchaseOrderDetail> {
    state.require("acquisitions.read")?;
    state.with_conn(|conn| acquisitions::get_order(conn, id))
}

#[tauri::command]
pub fn acquisitions_create_order(
    state: State<'_, AppState>,
    input: CreateOrderInput,
) -> AppResult<i64> {
    let session = state.require("acquisitions.write")?;
    acquisitions::create_order(&mut *state.conn()?, input, &session)
}

#[tauri::command]
pub fn acquisitions_update_order_status(
    state: State<'_, AppState>,
    id: i64,
    status: String,
    invoice_ref: Option<String>,
) -> AppResult<()> {
    let session = state.require("acquisitions.write")?;
    acquisitions::update_order_status(&mut *state.conn()?, id, status, invoice_ref, &session)
}

#[tauri::command]
pub fn acquisitions_receive_items(
    state: State<'_, AppState>,
    input: ReceiveItemInput,
) -> AppResult<ReceiveResult> {
    let session = state.require("acquisitions.write")?;
    acquisitions::receive_order_item(&mut *state.conn()?, input, &session)
}
