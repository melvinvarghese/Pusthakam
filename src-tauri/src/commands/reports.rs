use crate::error::AppResult;
use crate::services::reports::{self, CirculationDay, PopularTitle, StockRow};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn reports_circulation_activity(state: State<'_, AppState>, from: String, to: String) -> AppResult<Vec<CirculationDay>> {
    state.require("reports.read")?;
    state.with_conn(|conn| reports::circulation_activity(conn, &from, &to))
}

#[tauri::command]
pub fn reports_popular_titles(state: State<'_, AppState>, from: String, to: String) -> AppResult<Vec<PopularTitle>> {
    state.require("reports.read")?;
    state.with_conn(|conn| reports::popular_titles(conn, &from, &to))
}

#[tauri::command]
pub fn reports_stock_by_location(state: State<'_, AppState>) -> AppResult<Vec<StockRow>> {
    state.require("reports.read")?;
    reports::stock_by_location(&mut *state.conn()?)
}
