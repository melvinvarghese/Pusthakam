use crate::error::AppResult;
use crate::services::reports::{self, CatalogueGrowthDay, CirculationDay, FineSummary, MemberSummary, PopularTitle, StockRow};
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

#[tauri::command]
pub fn reports_fine_summary(state: State<'_, AppState>) -> AppResult<Vec<FineSummary>> {
    state.require("reports.read")?;
    state.with_conn(|conn| reports::fine_summary(conn))
}

#[tauri::command]
pub fn reports_member_summary(state: State<'_, AppState>) -> AppResult<Vec<MemberSummary>> {
    state.require("reports.read")?;
    state.with_conn(reports::member_summary)
}

#[tauri::command]
pub fn reports_catalogue_growth(state: State<'_, AppState>, from: String, to: String) -> AppResult<Vec<CatalogueGrowthDay>> {
    state.require("reports.read")?;
    state.with_conn(|conn| reports::catalogue_growth(conn, &from, &to))
}
