use crate::error::AppResult;
use crate::services::{
    inventory::{self, Page, Review, ScanOutcome, StockCheck, View},
    reservations,
    transfers::{self, Transfer},
};
use crate::state::AppState;
use tauri::State;

crate::shared_command! {

pub fn stock_check_list(state: State<'_, AppState>, page: i64) -> AppResult<Page<StockCheck>> {
    state.require("catalogue.read")?;
    state.with_conn(|conn| inventory::list(conn, page))
}
}

crate::shared_command! {

pub fn stock_check_review(
    state: State<'_, AppState>,
    id: i64,
    view: View,
    page: i64,
) -> AppResult<Review> {
    state.require("catalogue.read")?;
    state.with_conn(|conn| inventory::review(conn, id, view, page))
}
}

crate::shared_command! {

pub fn stock_check_start(
    state: State<'_, AppState>,
    location_id: i64,
    shelf_id: Option<i64>,
) -> AppResult<i64> {
    let actor = state.require("catalogue.write")?;
    let mut conn = state.conn()?;
    reservations::refresh(&mut conn)?;
    inventory::start(&mut conn, &actor, location_id, shelf_id)
}
}

crate::shared_command! {

pub fn stock_check_scan(
    state: State<'_, AppState>,
    id: i64,
    barcode: String,
) -> AppResult<ScanOutcome> {
    let actor = state.require("catalogue.write")?;
    inventory::scan(&mut *state.conn()?, &actor, id, &barcode)
}
}

crate::shared_command! {

pub fn stock_check_close(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    let actor = state.require("catalogue.write")?;
    inventory::close(&mut *state.conn()?, &actor, id)
}
}

crate::shared_command! {

pub fn stock_transfer_list(state: State<'_, AppState>, page: i64) -> AppResult<Page<Transfer>> {
    state.require("catalogue.read")?;
    state.with_conn(|conn| transfers::list(conn, page))
}
}

crate::shared_command! {

pub fn stock_transfer_send(
    state: State<'_, AppState>,
    barcode: String,
    location_id: i64,
    shelf_id: Option<i64>,
) -> AppResult<i64> {
    let actor = state.require("catalogue.write")?;
    transfers::send(&mut *state.conn()?, &actor, &barcode, location_id, shelf_id)
}
}

crate::shared_command! {

pub fn stock_transfer_finish(state: State<'_, AppState>, id: i64, receive: bool) -> AppResult<()> {
    let actor = state.require("catalogue.write")?;
    transfers::finish(&mut *state.conn()?, &actor, id, receive)
}
}
