use crate::error::AppResult;
use crate::services::classification::{
    self, AuthorAuthority, AuthorAuthorityInput, ClassificationEntry, ClassificationEntryInput,
    ClassificationScheme, ClassificationSchemeInput, MarcMapping, MarcMappingInput, MarcTestInput,
    MarcTestResult, PublisherAuthority, PublisherAuthorityInput, SubjectAuthority,
    SubjectAuthorityInput,
};
use crate::state::AppState;
use tauri::State;

// ============================================================================
// Classification Schemes & Entries
// ============================================================================

#[tauri::command]
pub fn classification_list_schemes(
    state: State<'_, AppState>,
) -> AppResult<Vec<ClassificationScheme>> {
    state.require("classification.read")?;
    state.with_conn(classification::list_schemes)
}

#[tauri::command]
pub fn classification_save_scheme(
    state: State<'_, AppState>,
    input: ClassificationSchemeInput,
) -> AppResult<i64> {
    let session = state.require("classification.write")?;
    classification::save_scheme(&*state.conn()?, &session, &input)
}

#[tauri::command]
pub fn classification_delete_scheme(
    state: State<'_, AppState>,
    id: i64,
) -> AppResult<()> {
    let session = state.require("classification.write")?;
    classification::delete_custom_scheme(&*state.conn()?, &session, id)
}

#[tauri::command]
pub fn classification_list_entries(
    state: State<'_, AppState>,
    scheme_id: i64,
    parent_code: Option<String>,
    search_query: Option<String>,
) -> AppResult<Vec<ClassificationEntry>> {
    state.require("classification.read")?;
    state.with_conn(|conn| {
        classification::list_entries(
            conn,
            scheme_id,
            parent_code.as_deref(),
            search_query.as_deref(),
        )
    })
}

#[tauri::command]
pub fn classification_save_entry(
    state: State<'_, AppState>,
    input: ClassificationEntryInput,
) -> AppResult<i64> {
    let session = state.require("classification.write")?;
    classification::save_entry(&*state.conn()?, &session, &input)
}

#[tauri::command]
pub fn classification_delete_entry(
    state: State<'_, AppState>,
    id: i64,
) -> AppResult<()> {
    let session = state.require("classification.write")?;
    classification::delete_custom_entry(&*state.conn()?, &session, id)
}

// ============================================================================
// Author Authorities
// ============================================================================

#[tauri::command]
pub fn classification_list_authors(
    state: State<'_, AppState>,
    search_query: Option<String>,
    kind_filter: Option<String>,
) -> AppResult<Vec<AuthorAuthority>> {
    state.require("classification.read")?;
    state.with_conn(|conn| {
        classification::list_authors(conn, search_query.as_deref(), kind_filter.as_deref())
    })
}

#[tauri::command]
pub fn classification_save_author(
    state: State<'_, AppState>,
    input: AuthorAuthorityInput,
) -> AppResult<i64> {
    let session = state.require("classification.write")?;
    classification::save_author(&*state.conn()?, &session, &input)
}

#[tauri::command]
pub fn classification_merge_authors(
    state: State<'_, AppState>,
    from_id: i64,
    to_id: i64,
) -> AppResult<()> {
    let session = state.require("classification.write")?;
    classification::merge_authors(&*state.conn()?, &session, from_id, to_id)
}

// ============================================================================
// Publisher Authorities
// ============================================================================

#[tauri::command]
pub fn classification_list_publishers(
    state: State<'_, AppState>,
    search_query: Option<String>,
) -> AppResult<Vec<PublisherAuthority>> {
    state.require("classification.read")?;
    state.with_conn(|conn| classification::list_publishers(conn, search_query.as_deref()))
}

#[tauri::command]
pub fn classification_save_publisher(
    state: State<'_, AppState>,
    input: PublisherAuthorityInput,
) -> AppResult<i64> {
    let session = state.require("classification.write")?;
    classification::save_publisher(&*state.conn()?, &session, &input)
}

#[tauri::command]
pub fn classification_merge_publishers(
    state: State<'_, AppState>,
    from_id: i64,
    to_id: i64,
) -> AppResult<()> {
    let session = state.require("classification.write")?;
    classification::merge_publishers(&*state.conn()?, &session, from_id, to_id)
}

// ============================================================================
// Subject Authorities
// ============================================================================

#[tauri::command]
pub fn classification_list_subjects(
    state: State<'_, AppState>,
    search_query: Option<String>,
    scheme_filter: Option<String>,
) -> AppResult<Vec<SubjectAuthority>> {
    state.require("classification.read")?;
    state.with_conn(|conn| {
        classification::list_subjects(conn, search_query.as_deref(), scheme_filter.as_deref())
    })
}

#[tauri::command]
pub fn classification_save_subject(
    state: State<'_, AppState>,
    input: SubjectAuthorityInput,
) -> AppResult<i64> {
    let session = state.require("classification.write")?;
    classification::save_subject(&*state.conn()?, &session, &input)
}

#[tauri::command]
pub fn classification_merge_subjects(
    state: State<'_, AppState>,
    from_id: i64,
    to_id: i64,
) -> AppResult<()> {
    let session = state.require("classification.write")?;
    classification::merge_subjects(&*state.conn()?, &session, from_id, to_id)
}

// ============================================================================
// Controlled MARC Mappings & Testing
// ============================================================================

#[tauri::command]
pub fn classification_list_marc_mappings(
    state: State<'_, AppState>,
) -> AppResult<Vec<MarcMapping>> {
    state.require("classification.read")?;
    state.with_conn(classification::list_marc_mappings)
}

#[tauri::command]
pub fn classification_save_marc_mapping(
    state: State<'_, AppState>,
    input: MarcMappingInput,
) -> AppResult<i64> {
    let session = state.require("classification.write")?;
    classification::save_marc_mapping(&*state.conn()?, &session, &input)
}

#[tauri::command]
pub fn classification_test_marc_mapping(
    state: State<'_, AppState>,
    input: MarcTestInput,
) -> AppResult<MarcTestResult> {
    state.require("classification.read")?;
    state.with_conn(|conn| classification::test_marc_mapping(conn, &input))
}
