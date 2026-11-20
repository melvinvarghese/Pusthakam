//! LAN server, pairing, and OPAC commands.

use crate::error::AppResult;
use crate::services::lan::{
    self, LanConfig, LanLoginRequest, LanLoginResponse, LanPairingCode, PairRequest, PairResponse,
    PairedClient, PublicOpacItem, PublicOpacSearchResult, UpdateLanConfigInput,
};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn lan_get_config(state: State<'_, AppState>) -> AppResult<LanConfig> {
    state.require("lan.read")?;
    state.with_conn(lan::get_or_init_config)
}

#[tauri::command]
pub fn lan_update_config(
    state: State<'_, AppState>,
    input: UpdateLanConfigInput,
) -> AppResult<LanConfig> {
    let session = state.require("lan.manage")?;
    state.with_conn(|conn| lan::update_config(conn, &session, &input))
}

#[tauri::command]
pub fn lan_generate_pairing_code(state: State<'_, AppState>) -> AppResult<LanPairingCode> {
    let session = state.require("lan.manage")?;
    state.with_conn(|conn| lan::generate_pairing_code(conn, &session))
}

#[tauri::command]
pub fn lan_list_pairing_codes(state: State<'_, AppState>) -> AppResult<Vec<LanPairingCode>> {
    let session = state.require("lan.read")?;
    state.with_conn(|conn| lan::list_pairing_codes(conn, &session))
}

#[tauri::command]
pub fn lan_list_paired_clients(state: State<'_, AppState>) -> AppResult<Vec<PairedClient>> {
    let session = state.require("lan.read")?;
    state.with_conn(|conn| lan::list_paired_clients(conn, &session))
}

#[tauri::command]
pub fn lan_revoke_client(state: State<'_, AppState>, client_id: String) -> AppResult<()> {
    let session = state.require("lan.manage")?;
    state.with_conn(|conn| lan::revoke_client(conn, &session, &client_id))
}

#[tauri::command]
pub fn lan_pair_client(
    state: State<'_, AppState>,
    request: PairRequest,
) -> AppResult<PairResponse> {
    state.with_conn(|conn| lan::pair_client(conn, &request))
}

#[tauri::command]
pub fn lan_login(
    state: State<'_, AppState>,
    request: LanLoginRequest,
) -> AppResult<LanLoginResponse> {
    state.with_conn(|conn| lan::lan_login(conn, &request))
}

#[tauri::command]
pub fn lan_opac_search(
    state: State<'_, AppState>,
    query: String,
    page: i64,
    page_size: i64,
) -> AppResult<PublicOpacSearchResult> {
    state.with_conn(|conn| lan::opac_search(conn, &query, page, page_size))
}

#[tauri::command]
pub fn lan_opac_get_record(
    state: State<'_, AppState>,
    id: i64,
) -> AppResult<Option<PublicOpacItem>> {
    state.with_conn(|conn| lan::opac_get_record(conn, id))
}
