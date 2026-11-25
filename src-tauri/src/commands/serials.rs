use crate::error::AppResult;
use crate::services::serials::{
    self, AddIssueInput, CheckInIssueInput, CheckInResult, ExpiringSubscription, IssueFilter,
    RenewSubscriptionInput, SaveSubscriptionInput, SerialIssue, SerialRenewal, SerialSubscription,
    SerialsExpenditureSummary, SubscriptionFilter,
};
use crate::state::AppState;
use tauri::State;

crate::shared_command! {

pub fn serials_list_subscriptions(
    state: State<'_, AppState>,
    filter: SubscriptionFilter,
) -> AppResult<Vec<SerialSubscription>> {
    state.require("serials.read")?;
    state.with_conn(|conn| serials::list_subscriptions(conn, filter))
}
}

crate::shared_command! {

pub fn serials_get_subscription(
    state: State<'_, AppState>,
    id: i64,
) -> AppResult<SerialSubscription> {
    state.require("serials.read")?;
    state.with_conn(|conn| serials::get_subscription(conn, id))
}
}

crate::shared_command! {

pub fn serials_save_subscription(
    state: State<'_, AppState>,
    input: SaveSubscriptionInput,
) -> AppResult<i64> {
    let session = state.require("serials.write")?;
    serials::save_subscription(&mut *state.conn()?, &session, input)
}
}

crate::shared_command! {

pub fn serials_cancel_subscription(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    let session = state.require("serials.write")?;
    serials::cancel_subscription(&mut *state.conn()?, &session, id)
}
}

crate::shared_command! {

pub fn serials_list_issues(
    state: State<'_, AppState>,
    filter: IssueFilter,
) -> AppResult<Vec<SerialIssue>> {
    state.require("serials.read")?;
    state.with_conn(|conn| serials::list_issues(conn, filter))
}
}

crate::shared_command! {

pub fn serials_list_missing_issues(state: State<'_, AppState>) -> AppResult<Vec<SerialIssue>> {
    state.require("serials.read")?;
    state.with_conn(serials::list_missing_issues)
}
}

crate::shared_command! {

pub fn serials_add_issue(state: State<'_, AppState>, input: AddIssueInput) -> AppResult<i64> {
    let session = state.require("serials.write")?;
    serials::add_issue(&mut *state.conn()?, &session, input)
}
}

crate::shared_command! {

pub fn serials_check_in_issue(
    state: State<'_, AppState>,
    input: CheckInIssueInput,
) -> AppResult<CheckInResult> {
    let session = state.require("serials.write")?;
    serials::check_in_issue(&mut *state.conn()?, &session, input)
}
}

crate::shared_command! {

pub fn serials_update_issue_status(
    state: State<'_, AppState>,
    issue_id: i64,
    status: String,
) -> AppResult<()> {
    let session = state.require("serials.write")?;
    serials::update_issue_status(&mut *state.conn()?, &session, issue_id, &status)
}
}

crate::shared_command! {

pub fn serials_renew_subscription(
    state: State<'_, AppState>,
    input: RenewSubscriptionInput,
) -> AppResult<i64> {
    let session = state.require("serials.write")?;
    serials::renew_subscription(&mut *state.conn()?, &session, input)
}
}

crate::shared_command! {

pub fn serials_list_renewals(
    state: State<'_, AppState>,
    subscription_id: i64,
) -> AppResult<Vec<SerialRenewal>> {
    state.require("serials.read")?;
    state.with_conn(|conn| serials::list_renewals(conn, subscription_id))
}
}

crate::shared_command! {

pub fn serials_get_expenditure(
    state: State<'_, AppState>,
    from_date: Option<String>,
    to_date: Option<String>,
) -> AppResult<SerialsExpenditureSummary> {
    state.require("serials.read")?;
    state.with_conn(|conn| {
        serials::get_serials_expenditure(conn, from_date.as_deref(), to_date.as_deref())
    })
}
}

crate::shared_command! {

pub fn serials_list_expiring(
    state: State<'_, AppState>,
    days_ahead: Option<u32>,
) -> AppResult<Vec<ExpiringSubscription>> {
    state.require("serials.read")?;
    state.with_conn(|conn| serials::list_expiring_subscriptions(conn, days_ahead))
}
}
