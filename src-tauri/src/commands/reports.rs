use crate::error::AppResult;
use crate::services::report_export::{self, ReportExportFile};
use crate::services::reports::{self, AccessionRow, CatalogueGrowthDay, CirculationDay, ExpenditureRow, FineSummary, LossDamageRow, MemberSummary, OverdueRow, PopularTitle, StockRow};
use crate::state::AppState;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
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
    state.with_conn(reports::fine_summary)
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

#[tauri::command]
pub fn reports_overdues_list(state: State<'_, AppState>, from: String, to: String) -> AppResult<Vec<OverdueRow>> {
    state.require("reports.read")?;
    state.with_conn(|conn| reports::overdues_list(conn, &from, &to))
}

#[tauri::command]
pub fn reports_accession_list(state: State<'_, AppState>, from: String, to: String) -> AppResult<Vec<AccessionRow>> {
    state.require("reports.read")?;
    state.with_conn(|conn| reports::accession_list(conn, &from, &to))
}

#[tauri::command]
pub fn reports_expenditure_summary(state: State<'_, AppState>, from: String, to: String) -> AppResult<Vec<ExpenditureRow>> {
    state.require("reports.read")?;
    state.with_conn(|conn| reports::expenditure_summary(conn, &from, &to))
}

#[tauri::command]
pub fn reports_loss_damage_list(state: State<'_, AppState>, from: String, to: String) -> AppResult<Vec<LossDamageRow>> {
    state.require("reports.read")?;
    state.with_conn(|conn| reports::loss_damage_list(conn, &from, &to))
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn reports_export_file(
    state: State<'_, AppState>,
    format: String,
    file_name: String,
    title: String,
    subtitle: String,
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    alignments: Vec<String>,
) -> AppResult<ReportExportFile> {
    state.require("reports.read")?;
    match format.as_str() {
        "xlsx" => {
            let bytes = report_export::generate_xlsx(&title, &headers, &rows)?;
            Ok(ReportExportFile {
                file_name,
                mime_type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".into(),
                data_base64: BASE64.encode(bytes),
            })
        }
        "pdf" => {
            let bytes = report_export::generate_pdf(&title, &subtitle, &headers, &rows, &alignments)?;
            Ok(ReportExportFile {
                file_name,
                mime_type: "application/pdf".into(),
                data_base64: BASE64.encode(bytes),
            })
        }
        _ => Err(crate::error::AppError::Validation("Choose xlsx or pdf.".into())),
    }
}
