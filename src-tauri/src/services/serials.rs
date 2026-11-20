use crate::domain::validate;
use crate::error::{AppError, AppResult};
use crate::services::audit;
use crate::services::auth::Session;
use crate::services::catalogue;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Data Transfer Objects
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerialSubscription {
    pub id: i64,
    pub title: String,
    pub issn: Option<String>,
    pub publisher: Option<String>,
    pub frequency: String,
    pub vendor_id: Option<i64>,
    pub vendor_name: Option<String>,
    pub budget_id: Option<i64>,
    pub budget_name: Option<String>,
    pub cost_minor: i64,
    pub currency: String,
    pub start_date: String,
    pub end_date: String,
    pub status: String,
    pub notes: Option<String>,
    pub archived: bool,
    pub total_issues_count: i64,
    pub received_issues_count: i64,
    pub missing_issues_count: i64,
    pub expected_issues_count: i64,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSubscriptionInput {
    pub id: Option<i64>,
    pub title: String,
    pub issn: Option<String>,
    pub publisher: Option<String>,
    pub frequency: String,
    pub vendor_id: Option<i64>,
    pub budget_id: Option<i64>,
    pub cost_minor: Option<i64>,
    pub currency: Option<String>,
    pub start_date: String,
    pub end_date: String,
    pub notes: Option<String>,
    pub auto_generate_issues: Option<bool>,
    pub issue_pattern_prefix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerialIssue {
    pub id: i64,
    pub subscription_id: i64,
    pub subscription_title: String,
    pub issue_number: String,
    pub expected_date: String,
    pub received_date: Option<String>,
    pub status: String,
    pub barcode: Option<String>,
    pub accession_no: Option<String>,
    pub physical_item_id: Option<i64>,
    pub claim_count: i64,
    pub claim_date: Option<String>,
    pub notes: Option<String>,
    pub received_by: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddIssueInput {
    pub subscription_id: i64,
    pub issue_number: String,
    pub expected_date: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckInIssueInput {
    pub issue_id: i64,
    pub received_date: Option<String>,
    pub accession_copy: Option<bool>,
    pub item_category_id: Option<i64>,
    pub location_id: Option<i64>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckInResult {
    pub issue_id: i64,
    pub status: String,
    pub received_date: String,
    pub accession_no: Option<String>,
    pub barcode: Option<String>,
    pub copy_id: Option<i64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionFilter {
    pub status: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueFilter {
    pub subscription_id: Option<i64>,
    pub status: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerialRenewal {
    pub id: i64,
    pub subscription_id: i64,
    pub start_date: String,
    pub end_date: String,
    pub cost_minor: i64,
    pub currency: String,
    pub budget_id: Option<i64>,
    pub budget_name: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenewSubscriptionInput {
    pub subscription_id: i64,
    pub start_date: String,
    pub end_date: String,
    pub cost_minor: i64,
    pub currency: Option<String>,
    pub budget_id: Option<i64>,
    pub notes: Option<String>,
    pub auto_generate_issues: Option<bool>,
    pub issue_pattern_prefix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerialsExpenditureItem {
    pub id: i64,
    pub subscription_id: i64,
    pub title: String,
    pub issn: Option<String>,
    pub item_type: String, // "initial" or "renewal"
    pub date: String,
    pub start_date: String,
    pub end_date: String,
    pub cost_minor: i64,
    pub currency: String,
    pub budget_id: Option<i64>,
    pub budget_name: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerialsExpenditureSummary {
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub total_spend_minor: i64,
    pub currency: String,
    pub item_count: usize,
    pub items: Vec<SerialsExpenditureItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpiringSubscription {
    pub subscription: SerialSubscription,
    pub days_remaining: i32,
    pub is_overdue: bool,
}

// ---------------------------------------------------------------------------
// Date & Schedule Calculation Helpers
// ---------------------------------------------------------------------------

fn parse_date(s: &str) -> Option<(i32, u32, u32)> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let y = parts[0].parse::<i32>().ok()?;
    let m = parts[1].parse::<u32>().ok()?;
    let d = parts[2].parse::<u32>().ok()?;
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    Some((y, m, d))
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

fn add_days(mut y: i32, mut m: u32, mut d: u32, mut days: u32) -> (i32, u32, u32) {
    while days > 0 {
        let dim = days_in_month(y, m);
        if d + days <= dim {
            d += days;
            days = 0;
        } else {
            days -= dim - d + 1;
            d = 1;
            if m == 12 {
                m = 1;
                y += 1;
            } else {
                m += 1;
            }
        }
    }
    (y, m, d)
}

fn add_months(mut y: i32, mut m: u32, orig_d: u32, months: u32) -> (i32, u32, u32) {
    let total_m = (m - 1) + months;
    y += (total_m / 12) as i32;
    m = (total_m % 12) + 1;
    let max_d = days_in_month(y, m);
    let d = orig_d.min(max_d);
    (y, m, d)
}

pub fn calculate_expected_dates(start_date: &str, end_date: &str, frequency: &str) -> Vec<String> {
    if frequency == "irregular" {
        return vec![];
    }
    let (mut y, mut m, mut d) = match parse_date(start_date) {
        Some(dt) => dt,
        None => return vec![],
    };
    let orig_d = d;
    let end_tuple = match parse_date(end_date) {
        Some(dt) => dt,
        None => return vec![],
    };

    let mut dates = Vec::new();

    loop {
        if (y, m, d) > end_tuple {
            break;
        }
        dates.push(format!("{:04}-{:02}-{:02}", y, m, d));
        if dates.len() >= 366 {
            // Safety cap: max 366 issues per subscription batch
            break;
        }

        match frequency {
            "daily" => {
                let next = add_days(y, m, d, 1);
                y = next.0;
                m = next.1;
                d = next.2;
            }
            "weekly" => {
                let next = add_days(y, m, d, 7);
                y = next.0;
                m = next.1;
                d = next.2;
            }
            "biweekly" => {
                let next = add_days(y, m, d, 14);
                y = next.0;
                m = next.1;
                d = next.2;
            }
            "monthly" => {
                let next = add_months(y, m, orig_d, 1);
                y = next.0;
                m = next.1;
                d = next.2;
            }
            "bimonthly" => {
                let next = add_months(y, m, orig_d, 2);
                y = next.0;
                m = next.1;
                d = next.2;
            }
            "quarterly" => {
                let next = add_months(y, m, orig_d, 3);
                y = next.0;
                m = next.1;
                d = next.2;
            }
            "semiannual" => {
                let next = add_months(y, m, orig_d, 6);
                y = next.0;
                m = next.1;
                d = next.2;
            }
            "annual" => {
                let next = add_months(y, m, orig_d, 12);
                y = next.0;
                m = next.1;
                d = next.2;
            }
            _ => {
                // Irregular: no auto schedule
                break;
            }
        }
    }

    dates
}

// ---------------------------------------------------------------------------
// Subscriptions Service
// ---------------------------------------------------------------------------

pub fn list_subscriptions(
    conn: &Connection,
    filter: SubscriptionFilter,
) -> AppResult<Vec<SerialSubscription>> {
    let mut sql = String::from(
        "SELECT s.id, s.title, s.issn, s.publisher, s.frequency,
                s.vendor_id, v.name AS vendor_name,
                s.budget_id, b.name AS budget_name,
                (s.cost_minor + (SELECT COALESCE(SUM(cost_minor), 0) FROM serial_renewals WHERE subscription_id = s.id)) AS cost_minor,
                s.currency, s.start_date, s.end_date,
                s.status, s.notes, s.archived_at IS NOT NULL AS archived,
                s.created_at,
                (SELECT COUNT(*) FROM serial_issues WHERE subscription_id = s.id) AS total_count,
                (SELECT COUNT(*) FROM serial_issues WHERE subscription_id = s.id AND status = 'received') AS rec_count,
                (SELECT COUNT(*) FROM serial_issues WHERE subscription_id = s.id AND status IN ('missing', 'claimed')) AS miss_count,
                (SELECT COUNT(*) FROM serial_issues WHERE subscription_id = s.id AND status = 'expected') AS exp_count
         FROM serial_subscriptions s
         LEFT JOIN vendors v ON s.vendor_id = v.id
         LEFT JOIN acquisition_budgets b ON s.budget_id = b.id
         WHERE 1=1 ",
    );

    let mut param_values: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(status) = filter.status {
        if !status.is_empty() && status != "all" {
            param_values.push(Box::new(status));
            sql.push_str(&format!("AND s.status = ?{} ", param_values.len()));
        }
    }

    if let Some(search) = filter.search {
        let trimmed = search.trim();
        if !trimmed.is_empty() {
            let pattern = format!("%{trimmed}%");
            param_values.push(Box::new(pattern.clone()));
            let p1 = param_values.len();
            param_values.push(Box::new(pattern));
            let p2 = param_values.len();
            sql.push_str(&format!("AND (s.title LIKE ?{p1} OR s.issn LIKE ?{p2}) "));
        }
    }

    sql.push_str("ORDER BY s.title COLLATE NOCASE ASC");

    let params_refs: Vec<&dyn rusqlite::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(&params_refs[..], |r| {
            Ok(SerialSubscription {
                id: r.get(0)?,
                title: r.get(1)?,
                issn: r.get(2)?,
                publisher: r.get(3)?,
                frequency: r.get(4)?,
                vendor_id: r.get(5)?,
                vendor_name: r.get(6)?,
                budget_id: r.get(7)?,
                budget_name: r.get(8)?,
                cost_minor: r.get(9)?,
                currency: r.get(10)?,
                start_date: r.get(11)?,
                end_date: r.get(12)?,
                status: r.get(13)?,
                notes: r.get(14)?,
                archived: r.get(15)?,
                created_at: r.get(16)?,
                total_issues_count: r.get(17)?,
                received_issues_count: r.get(18)?,
                missing_issues_count: r.get(19)?,
                expected_issues_count: r.get(20)?,
            })
        })?
        .collect::<Result<_, _>>()?;

    Ok(rows)
}

pub fn get_subscription(conn: &Connection, id: i64) -> AppResult<SerialSubscription> {
    conn.query_row(
        "SELECT s.id, s.title, s.issn, s.publisher, s.frequency,
                s.vendor_id, v.name AS vendor_name,
                s.budget_id, b.name AS budget_name,
                (s.cost_minor + (SELECT COALESCE(SUM(cost_minor), 0) FROM serial_renewals WHERE subscription_id = s.id)) AS cost_minor,
                s.currency, s.start_date, s.end_date,
                s.status, s.notes, s.archived_at IS NOT NULL AS archived,
                s.created_at,
                (SELECT COUNT(*) FROM serial_issues WHERE subscription_id = s.id) AS total_count,
                (SELECT COUNT(*) FROM serial_issues WHERE subscription_id = s.id AND status = 'received') AS rec_count,
                (SELECT COUNT(*) FROM serial_issues WHERE subscription_id = s.id AND status IN ('missing', 'claimed')) AS miss_count,
                (SELECT COUNT(*) FROM serial_issues WHERE subscription_id = s.id AND status = 'expected') AS exp_count
         FROM serial_subscriptions s
         LEFT JOIN vendors v ON s.vendor_id = v.id
         LEFT JOIN acquisition_budgets b ON s.budget_id = b.id
         WHERE s.id = ?1",
        params![id],
        |r| {
            Ok(SerialSubscription {
                id: r.get(0)?,
                title: r.get(1)?,
                issn: r.get(2)?,
                publisher: r.get(3)?,
                frequency: r.get(4)?,
                vendor_id: r.get(5)?,
                vendor_name: r.get(6)?,
                budget_id: r.get(7)?,
                budget_name: r.get(8)?,
                cost_minor: r.get(9)?,
                currency: r.get(10)?,
                start_date: r.get(11)?,
                end_date: r.get(12)?,
                status: r.get(13)?,
                notes: r.get(14)?,
                archived: r.get(15)?,
                created_at: r.get(16)?,
                total_issues_count: r.get(17)?,
                received_issues_count: r.get(18)?,
                missing_issues_count: r.get(19)?,
                expected_issues_count: r.get(20)?,
            })
        },
    )
    .optional()?
    .ok_or_else(|| AppError::NotFound(format!("Serial subscription {id} not found")))
}

pub fn save_subscription(
    conn: &mut Connection,
    actor: &Session,
    input: SaveSubscriptionInput,
) -> AppResult<i64> {
    let title = validate::required("Title", &input.title, 200)?;
    let issn = validate::optional("ISSN", input.issn.as_deref(), 30)?;
    let publisher = validate::optional("Publisher", input.publisher.as_deref(), 150)?;

    let valid_freqs = [
        "daily",
        "weekly",
        "biweekly",
        "monthly",
        "bimonthly",
        "quarterly",
        "semiannual",
        "annual",
        "irregular",
    ];
    let freq = input.frequency.to_lowercase();
    if !valid_freqs.contains(&freq.as_str()) {
        return Err(AppError::Validation(format!(
            "Invalid frequency '{freq}'. Allowed: {}",
            valid_freqs.join(", ")
        )));
    }

    if parse_date(&input.start_date).is_none() {
        return Err(AppError::Validation(
            "Start date must be in YYYY-MM-DD format".into(),
        ));
    }
    if parse_date(&input.end_date).is_none() {
        return Err(AppError::Validation(
            "End date must be in YYYY-MM-DD format".into(),
        ));
    }
    if input.end_date < input.start_date {
        return Err(AppError::Validation(
            "End date cannot be earlier than start date".into(),
        ));
    }

    let cost_minor = input.cost_minor.unwrap_or(0);
    if cost_minor < 0 {
        return Err(AppError::Validation("Cost cannot be negative".into()));
    }
    let currency = input.currency.unwrap_or_else(|| "INR".into());
    let notes = validate::optional("Notes", input.notes.as_deref(), 500)?;

    let tx = conn.transaction()?;

    let sub_id = if let Some(id) = input.id {
        tx.execute(
            "UPDATE serial_subscriptions
             SET title = ?1, issn = ?2, publisher = ?3, frequency = ?4,
                 vendor_id = ?5, budget_id = ?6, cost_minor = ?7, currency = ?8,
                 start_date = ?9, end_date = ?10, notes = ?11, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now')
             WHERE id = ?12",
            params![
                title, issn, publisher, freq, input.vendor_id, input.budget_id,
                cost_minor, currency, input.start_date, input.end_date, notes, id
            ],
        )?;
        audit::record(
            &tx,
            Some(actor),
            "serial.update",
            "serial_subscription",
            Some(id),
            &format!("Updated serial subscription {title}"),
        )?;
        id
    } else {
        tx.execute(
            "INSERT INTO serial_subscriptions (
                title, issn, publisher, frequency, vendor_id, budget_id,
                cost_minor, currency, start_date, end_date, notes, created_by
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                title,
                issn,
                publisher,
                freq,
                input.vendor_id,
                input.budget_id,
                cost_minor,
                currency,
                input.start_date,
                input.end_date,
                notes,
                actor.user_id
            ],
        )?;
        let new_id = tx.last_insert_rowid();

        // Auto-generate expected issues if requested or if non-irregular frequency
        let should_gen = input.auto_generate_issues.unwrap_or(freq != "irregular");
        if should_gen {
            let dates = calculate_expected_dates(&input.start_date, &input.end_date, &freq);
            let prefix = input.issue_pattern_prefix.as_deref().unwrap_or("Issue ");
            for (idx, d) in dates.into_iter().enumerate() {
                let issue_num = format!("{}{}", prefix, idx + 1);
                tx.execute(
                    "INSERT INTO serial_issues (subscription_id, issue_number, expected_date)
                     VALUES (?1, ?2, ?3)",
                    params![new_id, issue_num, d],
                )?;
            }
        }

        audit::record(
            &tx,
            Some(actor),
            "serial.create",
            "serial_subscription",
            Some(new_id),
            &format!("Created serial subscription {title}"),
        )?;
        new_id
    };

    tx.commit()?;
    Ok(sub_id)
}

pub fn cancel_subscription(
    conn: &mut Connection,
    actor: &Session,
    subscription_id: i64,
) -> AppResult<()> {
    let title: String = conn
        .query_row(
            "SELECT title FROM serial_subscriptions WHERE id = ?1",
            params![subscription_id],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| {
            AppError::NotFound(format!("Serial subscription {subscription_id} not found"))
        })?;

    let tx = conn.transaction()?;
    tx.execute(
        "UPDATE serial_subscriptions
         SET status = 'cancelled', updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now')
         WHERE id = ?1",
        params![subscription_id],
    )?;

    // Cancel all remaining expected issues
    tx.execute(
        "UPDATE serial_issues
         SET status = 'cancelled', updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now')
         WHERE subscription_id = ?1 AND status = 'expected'",
        params![subscription_id],
    )?;

    audit::record(
        &tx,
        Some(actor),
        "serial.cancel",
        "serial_subscription",
        Some(subscription_id),
        &format!("Cancelled serial subscription {title}"),
    )?;
    tx.commit()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Issues Service & Check-In
// ---------------------------------------------------------------------------

pub fn list_issues(conn: &Connection, filter: IssueFilter) -> AppResult<Vec<SerialIssue>> {
    let mut sql = String::from(
        "SELECT i.id, i.subscription_id, s.title AS subscription_title,
                i.issue_number, i.expected_date, i.received_date, i.status,
                i.barcode, i.accession_no, i.physical_item_id,
                i.claim_count, i.claim_date, i.notes, i.created_at, i.received_by
         FROM serial_issues i
         JOIN serial_subscriptions s ON i.subscription_id = s.id
         WHERE 1=1 ",
    );

    let mut param_values: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(sub_id) = filter.subscription_id {
        param_values.push(Box::new(sub_id));
        sql.push_str(&format!("AND i.subscription_id = ?{} ", param_values.len()));
    }

    if let Some(status) = filter.status {
        if !status.is_empty() && status != "all" {
            param_values.push(Box::new(status));
            sql.push_str(&format!("AND i.status = ?{} ", param_values.len()));
        }
    }

    if let Some(start) = filter.start_date {
        if !start.is_empty() {
            param_values.push(Box::new(start));
            sql.push_str(&format!("AND i.expected_date >= ?{} ", param_values.len()));
        }
    }

    if let Some(end) = filter.end_date {
        if !end.is_empty() {
            param_values.push(Box::new(end));
            sql.push_str(&format!("AND i.expected_date <= ?{} ", param_values.len()));
        }
    }

    sql.push_str("ORDER BY i.expected_date ASC, i.id ASC");

    let params_refs: Vec<&dyn rusqlite::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(&params_refs[..], |r| {
            Ok(SerialIssue {
                id: r.get(0)?,
                subscription_id: r.get(1)?,
                subscription_title: r.get(2)?,
                issue_number: r.get(3)?,
                expected_date: r.get(4)?,
                received_date: r.get(5)?,
                status: r.get(6)?,
                barcode: r.get(7)?,
                accession_no: r.get(8)?,
                physical_item_id: r.get(9)?,
                claim_count: r.get(10)?,
                claim_date: r.get(11)?,
                notes: r.get(12)?,
                created_at: r.get(13)?,
                received_by: r.get(14)?,
            })
        })?
        .collect::<Result<_, _>>()?;

    Ok(rows)
}

pub fn list_missing_issues(conn: &Connection) -> AppResult<Vec<SerialIssue>> {
    let mut stmt = conn.prepare(
        "SELECT i.id, i.subscription_id, s.title AS subscription_title,
                i.issue_number, i.expected_date, i.received_date, i.status,
                i.barcode, i.accession_no, i.physical_item_id,
                i.claim_count, i.claim_date, i.notes, i.created_at, i.received_by
         FROM serial_issues i
         JOIN serial_subscriptions s ON i.subscription_id = s.id
         WHERE i.status IN ('missing', 'claimed')
            OR (i.status = 'expected' AND i.expected_date < strftime('%Y-%m-%d', 'now'))
         ORDER BY i.expected_date ASC",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok(SerialIssue {
                id: r.get(0)?,
                subscription_id: r.get(1)?,
                subscription_title: r.get(2)?,
                issue_number: r.get(3)?,
                expected_date: r.get(4)?,
                received_date: r.get(5)?,
                status: r.get(6)?,
                barcode: r.get(7)?,
                accession_no: r.get(8)?,
                physical_item_id: r.get(9)?,
                claim_count: r.get(10)?,
                claim_date: r.get(11)?,
                notes: r.get(12)?,
                created_at: r.get(13)?,
                received_by: r.get(14)?,
            })
        })?
        .collect::<Result<_, _>>()?;

    Ok(rows)
}

pub fn add_issue(conn: &mut Connection, actor: &Session, input: AddIssueInput) -> AppResult<i64> {
    let issue_number = validate::required("Issue number", &input.issue_number, 50)?;
    if parse_date(&input.expected_date).is_none() {
        return Err(AppError::Validation(
            "Expected date must be in YYYY-MM-DD format".into(),
        ));
    }
    let notes = validate::optional("Notes", input.notes.as_deref(), 500)?;

    let (sub_exists, title): (bool, String) = conn
        .query_row(
            "SELECT 1, title FROM serial_subscriptions WHERE id = ?1",
            params![input.subscription_id],
            |r| Ok((true, r.get(1)?)),
        )
        .optional()?
        .unwrap_or((false, String::new()));

    if !sub_exists {
        return Err(AppError::NotFound(format!(
            "Serial subscription {} not found",
            input.subscription_id
        )));
    }

    let existing: Option<i64> = conn
        .query_row(
            "SELECT id FROM serial_issues WHERE subscription_id = ?1 AND issue_number = ?2",
            params![input.subscription_id, issue_number],
            |r| r.get(0),
        )
        .optional()?;

    if existing.is_some() {
        return Err(AppError::Conflict(format!(
            "Issue '{issue_number}' already exists for subscription '{title}'"
        )));
    }

    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO serial_issues (subscription_id, issue_number, expected_date, notes)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            input.subscription_id,
            issue_number,
            input.expected_date,
            notes
        ],
    )?;
    let issue_id = tx.last_insert_rowid();

    audit::record(
        &tx,
        Some(actor),
        "serial.add_issue",
        "serial_issue",
        Some(issue_id),
        &format!("Added issue {issue_number} to {title}"),
    )?;
    tx.commit()?;
    Ok(issue_id)
}

pub fn check_in_issue(
    conn: &mut Connection,
    actor: &Session,
    input: CheckInIssueInput,
) -> AppResult<CheckInResult> {
    let (sub_id, sub_title, sub_issn, sub_publisher, issue_number, current_status): (
        i64,
        String,
        Option<String>,
        Option<String>,
        String,
        String,
    ) = conn
        .query_row(
            "SELECT i.subscription_id, s.title, s.issn, s.publisher, i.issue_number, i.status
             FROM serial_issues i
             JOIN serial_subscriptions s ON i.subscription_id = s.id
             WHERE i.id = ?1",
            params![input.issue_id],
            |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound(format!("Issue {} not found", input.issue_id)))?;

    if current_status == "received" {
        return Err(AppError::Conflict(format!(
            "Issue '{issue_number}' of '{sub_title}' has already been received"
        )));
    }

    let received_date = if let Some(d) = input.received_date {
        if parse_date(&d).is_none() {
            return Err(AppError::Validation(
                "Received date must be in YYYY-MM-DD format".into(),
            ));
        }
        d
    } else {
        conn.query_row("SELECT strftime('%Y-%m-%d', 'now')", [], |r| r.get(0))?
    };

    let tx = conn.transaction()?;

    let (accession_no, barcode, copy_id) = if input.accession_copy.unwrap_or(false) {
        // Resolve or create bibliographic record for periodical title
        let record_id: i64 = if let Some(ref issn) = sub_issn {
            let existing: Option<i64> = tx
                .query_row(
                    "SELECT id FROM bibliographic_records WHERE isbn13 = ?1 OR isbn10 = ?1 LIMIT 1",
                    params![issn],
                    |r| r.get(0),
                )
                .optional()?;
            match existing {
                Some(id) => id,
                None => {
                    tx.execute(
                        "INSERT INTO bibliographic_records (title, publisher, isbn13)
                         VALUES (?1, ?2, ?3)",
                        params![sub_title, sub_publisher, issn],
                    )?;
                    let new_rec = tx.last_insert_rowid();
                    catalogue::reindex(&tx, new_rec)?;
                    new_rec
                }
            }
        } else {
            let existing: Option<i64> = tx
                .query_row(
                    "SELECT id FROM bibliographic_records WHERE title = ?1 COLLATE NOCASE LIMIT 1",
                    params![sub_title],
                    |r| r.get(0),
                )
                .optional()?;
            match existing {
                Some(id) => id,
                None => {
                    tx.execute(
                        "INSERT INTO bibliographic_records (title, publisher)
                         VALUES (?1, ?2)",
                        params![sub_title, sub_publisher],
                    )?;
                    let new_rec = tx.last_insert_rowid();
                    catalogue::reindex(&tx, new_rec)?;
                    new_rec
                }
            }
        };

        // Determine category and location
        let cat_id: i64 = if let Some(cid) = input.item_category_id {
            cid
        } else {
            tx.query_row(
                "SELECT id FROM item_categories WHERE archived_at IS NULL ORDER BY id LIMIT 1",
                [],
                |r| r.get(0),
            )
            .optional()?
            .unwrap_or(1)
        };

        let loc_id: i64 = if let Some(lid) = input.location_id {
            lid
        } else {
            tx.query_row(
                "SELECT id FROM locations WHERE archived_at IS NULL ORDER BY id LIMIT 1",
                [],
                |r| r.get(0),
            )
            .optional()?
            .unwrap_or(1)
        };

        let acc_no = catalogue::next_accession(&tx)?;
        let bcode = acc_no.clone();

        tx.execute(
            "INSERT INTO physical_items (
                record_id, item_category_id, location_id, accession_no, barcode,
                status, acquisition_source
             ) VALUES (?1, ?2, ?3, ?4, ?5, 'available', 'purchase')",
            params![record_id, cat_id, loc_id, acc_no, bcode],
        )?;
        let cid = tx.last_insert_rowid();

        tx.execute(
            "INSERT INTO item_events (item_id, event_type, to_status, note, staff_user_id)
             VALUES (?1, 'created', 'available', ?2, ?3)",
            params![
                cid,
                format!("Checked in periodical issue {issue_number} of {sub_title}"),
                actor.user_id,
            ],
        )?;

        (Some(acc_no), Some(bcode), Some(cid))
    } else {
        (None, None, None)
    };

    tx.execute(
        "UPDATE serial_issues
         SET status = 'received', received_date = ?1, barcode = ?2, accession_no = ?3,
             physical_item_id = ?4, received_by = ?5,
             notes = COALESCE(?6, notes), updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now')
         WHERE id = ?7",
        params![
            received_date,
            barcode,
            accession_no,
            copy_id,
            actor.user_id,
            input.notes,
            input.issue_id
        ],
    )?;

    audit::record(
        &tx,
        Some(actor),
        "serial.check_in",
        "serial_issue",
        Some(input.issue_id),
        &format!("Checked in issue {issue_number} of {sub_title} (subscription {sub_id})"),
    )?;

    tx.commit()?;

    Ok(CheckInResult {
        issue_id: input.issue_id,
        status: "received".to_string(),
        received_date,
        accession_no,
        barcode,
        copy_id,
    })
}

pub fn update_issue_status(
    conn: &mut Connection,
    actor: &Session,
    issue_id: i64,
    status: &str,
) -> AppResult<()> {
    let valid_statuses = ["expected", "received", "missing", "claimed", "cancelled"];
    if !valid_statuses.contains(&status) {
        return Err(AppError::Validation(format!(
            "Invalid status '{status}'. Allowed: {}",
            valid_statuses.join(", ")
        )));
    }

    let (issue_number, sub_title, old_status): (String, String, String) = conn
        .query_row(
            "SELECT i.issue_number, s.title, i.status
             FROM serial_issues i
             JOIN serial_subscriptions s ON i.subscription_id = s.id
             WHERE i.id = ?1",
            params![issue_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound(format!("Issue {issue_id} not found")))?;

    let tx = conn.transaction()?;

    if status == "claimed" {
        tx.execute(
            "UPDATE serial_issues
             SET status = 'claimed',
                 claim_count = claim_count + 1,
                 claim_date = strftime('%Y-%m-%d', 'now'),
                 updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now')
             WHERE id = ?1",
            params![issue_id],
        )?;
    } else {
        tx.execute(
            "UPDATE serial_issues
             SET status = ?1,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now')
             WHERE id = ?2",
            params![status, issue_id],
        )?;
    }

    audit::record(
        &tx,
        Some(actor),
        "serial.issue_status",
        "serial_issue",
        Some(issue_id),
        &format!(
            "Updated issue {issue_number} of {sub_title} status from {old_status} to {status}"
        ),
    )?;

    tx.commit()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Renewals Service (supports SER-RENEW)
// ---------------------------------------------------------------------------

pub fn renew_subscription(
    conn: &mut Connection,
    actor: &Session,
    input: RenewSubscriptionInput,
) -> AppResult<i64> {
    let (title, freq, current_end): (String, String, String) = conn
        .query_row(
            "SELECT title, frequency, end_date FROM serial_subscriptions WHERE id = ?1",
            params![input.subscription_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "Serial subscription {} not found",
                input.subscription_id
            ))
        })?;

    if parse_date(&input.start_date).is_none() {
        return Err(AppError::Validation(
            "Start date must be in YYYY-MM-DD format".into(),
        ));
    }
    if parse_date(&input.end_date).is_none() {
        return Err(AppError::Validation(
            "End date must be in YYYY-MM-DD format".into(),
        ));
    }
    if input.end_date < input.start_date {
        return Err(AppError::Validation(
            "End date cannot be earlier than start date".into(),
        ));
    }
    if input.cost_minor < 0 {
        return Err(AppError::Validation("Cost cannot be negative".into()));
    }
    let currency = input.currency.unwrap_or_else(|| "INR".into());

    let tx = conn.transaction()?;

    tx.execute(
        "INSERT INTO serial_renewals (subscription_id, start_date, end_date, cost_minor, currency, budget_id, notes, created_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            input.subscription_id,
            input.start_date,
            input.end_date,
            input.cost_minor,
            currency,
            input.budget_id,
            input.notes,
            actor.user_id
        ],
    )?;
    let renewal_id = tx.last_insert_rowid();

    // Extend subscription end date and set active
    tx.execute(
        "UPDATE serial_subscriptions
         SET end_date = ?1, status = 'active',
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now')
         WHERE id = ?2",
        params![input.end_date, input.subscription_id],
    )?;

    // Auto-generate issues for extended period if requested
    let should_gen = input.auto_generate_issues.unwrap_or(freq != "irregular");
    if should_gen {
        let existing_issues_count: i64 = tx.query_row(
            "SELECT COUNT(*) FROM serial_issues WHERE subscription_id = ?1",
            params![input.subscription_id],
            |r| r.get(0),
        )?;
        let dates = calculate_expected_dates(&input.start_date, &input.end_date, &freq);
        let prefix = input.issue_pattern_prefix.as_deref().unwrap_or("Issue ");
        for (idx, d) in dates.into_iter().enumerate() {
            let issue_num = format!("{}{}", prefix, existing_issues_count as usize + idx + 1);
            let _ = tx.execute(
                "INSERT INTO serial_issues (subscription_id, issue_number, expected_date)
                 VALUES (?1, ?2, ?3)",
                params![input.subscription_id, issue_num, d],
            );
        }
    }

    audit::record(
        &tx,
        Some(actor),
        "serial.renew",
        "serial_subscription",
        Some(input.subscription_id),
        &format!(
            "Renewed subscription {title} from {current_end} to {}",
            input.end_date
        ),
    )?;

    tx.commit()?;
    Ok(renewal_id)
}

pub fn list_renewals(conn: &Connection, subscription_id: i64) -> AppResult<Vec<SerialRenewal>> {
    let mut stmt = conn.prepare(
        "SELECT r.id, r.subscription_id, r.start_date, r.end_date,
                r.cost_minor, r.currency, r.budget_id, b.name AS budget_name,
                r.notes, r.created_at
         FROM serial_renewals r
         LEFT JOIN acquisition_budgets b ON r.budget_id = b.id
         WHERE r.subscription_id = ?1
         ORDER BY r.start_date DESC",
    )?;
    let rows = stmt
        .query_map(params![subscription_id], |r| {
            Ok(SerialRenewal {
                id: r.get(0)?,
                subscription_id: r.get(1)?,
                start_date: r.get(2)?,
                end_date: r.get(3)?,
                cost_minor: r.get(4)?,
                currency: r.get(5)?,
                budget_id: r.get(6)?,
                budget_name: r.get(7)?,
                notes: r.get(8)?,
                created_at: r.get(9)?,
            })
        })?
        .collect::<Result<_, _>>()?;

    Ok(rows)
}

pub fn get_serials_expenditure(
    conn: &Connection,
    from_date: Option<&str>,
    to_date: Option<&str>,
) -> AppResult<SerialsExpenditureSummary> {
    let mut items = Vec::new();

    // Query initial subscriptions
    let mut stmt1 = conn.prepare(
        "SELECT s.id, s.id, s.title, s.issn, 'initial', s.start_date, s.start_date, s.end_date,
                s.cost_minor, s.currency, s.budget_id, b.name AS budget_name, s.notes
         FROM serial_subscriptions s
         LEFT JOIN acquisition_budgets b ON s.budget_id = b.id
         WHERE s.archived_at IS NULL
           AND (?1 IS NULL OR s.start_date >= ?1)
           AND (?2 IS NULL OR s.start_date <= ?2)
           AND s.cost_minor > 0
         ORDER BY s.start_date DESC",
    )?;
    let rows1 = stmt1.query_map(params![from_date, to_date], |r| {
        Ok(SerialsExpenditureItem {
            id: r.get(0)?,
            subscription_id: r.get(1)?,
            title: r.get(2)?,
            issn: r.get(3)?,
            item_type: r.get(4)?,
            date: r.get(5)?,
            start_date: r.get(6)?,
            end_date: r.get(7)?,
            cost_minor: r.get(8)?,
            currency: r.get(9)?,
            budget_id: r.get(10)?,
            budget_name: r.get(11)?,
            notes: r.get(12)?,
        })
    })?;
    for item in rows1 {
        items.push(item?);
    }

    // Query renewals
    let mut stmt2 = conn.prepare(
        "SELECT r.id, r.subscription_id, s.title, s.issn, 'renewal', r.start_date, r.start_date, r.end_date,
                r.cost_minor, r.currency, r.budget_id, b.name AS budget_name, r.notes
         FROM serial_renewals r
         JOIN serial_subscriptions s ON r.subscription_id = s.id
         LEFT JOIN acquisition_budgets b ON r.budget_id = b.id
         WHERE (?1 IS NULL OR r.start_date >= ?1)
           AND (?2 IS NULL OR r.start_date <= ?2)
           AND r.cost_minor > 0
         ORDER BY r.start_date DESC",
    )?;
    let rows2 = stmt2.query_map(params![from_date, to_date], |r| {
        Ok(SerialsExpenditureItem {
            id: r.get(0)?,
            subscription_id: r.get(1)?,
            title: r.get(2)?,
            issn: r.get(3)?,
            item_type: r.get(4)?,
            date: r.get(5)?,
            start_date: r.get(6)?,
            end_date: r.get(7)?,
            cost_minor: r.get(8)?,
            currency: r.get(9)?,
            budget_id: r.get(10)?,
            budget_name: r.get(11)?,
            notes: r.get(12)?,
        })
    })?;
    for item in rows2 {
        items.push(item?);
    }

    items.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| a.title.cmp(&b.title)));

    let total_spend_minor: i64 = items.iter().map(|i| i.cost_minor).sum();
    let currency: String = conn
        .query_row("SELECT currency FROM library WHERE id = 1", [], |r| {
            r.get(0)
        })
        .unwrap_or_else(|_| "INR".to_string());

    let item_count = items.len();

    Ok(SerialsExpenditureSummary {
        from_date: from_date.map(str::to_string),
        to_date: to_date.map(str::to_string),
        total_spend_minor,
        currency,
        item_count,
        items,
    })
}

pub fn list_expiring_subscriptions(
    conn: &Connection,
    days_ahead: Option<u32>,
) -> AppResult<Vec<ExpiringSubscription>> {
    let window_days = days_ahead.unwrap_or(30);
    let mut stmt = conn.prepare(
        "SELECT s.id, s.title, s.issn, s.publisher, s.frequency,
                s.vendor_id, v.name AS vendor_name,
                s.budget_id, b.name AS budget_name,
                (s.cost_minor + (SELECT COALESCE(SUM(cost_minor), 0) FROM serial_renewals WHERE subscription_id = s.id)) AS cost_minor,
                s.currency, s.start_date, s.end_date,
                s.status, s.notes, s.archived_at IS NOT NULL AS archived,
                s.created_at,
                (SELECT count(*) FROM serial_issues WHERE subscription_id = s.id) AS total_issues,
                (SELECT count(*) FROM serial_issues WHERE subscription_id = s.id AND status = 'received') AS received_issues,
                (SELECT count(*) FROM serial_issues WHERE subscription_id = s.id AND status = 'missing') AS missing_issues,
                (SELECT count(*) FROM serial_issues WHERE subscription_id = s.id AND status = 'expected') AS expected_issues,
                CAST(julianday(s.end_date) - julianday(date('now','localtime')) AS INTEGER) AS days_rem
         FROM serial_subscriptions s
         LEFT JOIN vendors v ON s.vendor_id = v.id
         LEFT JOIN acquisition_budgets b ON s.budget_id = b.id
         WHERE s.archived_at IS NULL
           AND s.status = 'active'
           AND s.end_date <= date('now','localtime','+' || ?1 || ' days')
         ORDER BY s.end_date ASC",
    )?;

    let rows = stmt
        .query_map(params![window_days], |r| {
            let days_rem: i32 = r.get(21)?;
            let sub = SerialSubscription {
                id: r.get(0)?,
                title: r.get(1)?,
                issn: r.get(2)?,
                publisher: r.get(3)?,
                frequency: r.get(4)?,
                vendor_id: r.get(5)?,
                vendor_name: r.get(6)?,
                budget_id: r.get(7)?,
                budget_name: r.get(8)?,
                cost_minor: r.get(9)?,
                currency: r.get(10)?,
                start_date: r.get(11)?,
                end_date: r.get(12)?,
                status: r.get(13)?,
                notes: r.get(14)?,
                archived: r.get(15)?,
                created_at: r.get(16)?,
                total_issues_count: r.get(17)?,
                received_issues_count: r.get(18)?,
                missing_issues_count: r.get(19)?,
                expected_issues_count: r.get(20)?,
            };
            Ok(ExpiringSubscription {
                subscription: sub,
                days_remaining: days_rem,
                is_overdue: days_rem < 0,
            })
        })?
        .collect::<Result<_, _>>()?;

    Ok(rows)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;

    fn setup_test_db() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        migrations::run(&mut conn, migrations::MIGRATIONS).unwrap();

        // Seed item category and location
        conn.execute(
            "INSERT INTO item_categories (name) VALUES ('Periodical')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO locations (name) VALUES ('Periodical Rack')",
            [],
        )
        .unwrap();

        // Seed a staff user
        conn.execute(
            "INSERT INTO staff_users (id, username, display_name, password_hash, role_id)
             VALUES (1, 'admin', 'Administrator', 'hash', 1)",
            [],
        )
        .unwrap();

        conn
    }

    fn test_actor() -> Session {
        Session {
            user_id: 1,
            username: "admin".into(),
            display_name: "Administrator".into(),
            role: "administrator".into(),
            permissions: vec!["serials.read".into(), "serials.write".into()],
            revision: 1,
        }
    }

    #[test]
    fn schedule_generation_and_frequency_calculations() {
        let daily = calculate_expected_dates("2026-01-01", "2026-01-05", "daily");
        assert_eq!(
            daily,
            vec![
                "2026-01-01",
                "2026-01-02",
                "2026-01-03",
                "2026-01-04",
                "2026-01-05"
            ]
        );

        let weekly = calculate_expected_dates("2026-01-01", "2026-01-22", "weekly");
        assert_eq!(
            weekly,
            vec!["2026-01-01", "2026-01-08", "2026-01-15", "2026-01-22"]
        );

        let monthly = calculate_expected_dates("2026-01-15", "2026-04-15", "monthly");
        assert_eq!(
            monthly,
            vec!["2026-01-15", "2026-02-15", "2026-03-15", "2026-04-15"]
        );

        let irregular = calculate_expected_dates("2026-01-01", "2026-12-31", "irregular");
        assert!(irregular.is_empty());
    }

    #[test]
    fn serial_subscription_lifecycle_and_issue_generation() {
        let mut conn = setup_test_db();
        let actor = test_actor();

        let sub_id = save_subscription(
            &mut conn,
            &actor,
            SaveSubscriptionInput {
                id: None,
                title: "Journal of Library Science".into(),
                issn: Some("1234-5679".into()),
                publisher: Some("Academic Press".into()),
                frequency: "quarterly".into(),
                vendor_id: None,
                budget_id: None,
                cost_minor: Some(150000),
                currency: Some("INR".into()),
                start_date: "2026-01-01".into(),
                end_date: "2026-12-31".into(),
                notes: Some("Quarterly print subscription".into()),
                auto_generate_issues: Some(true),
                issue_pattern_prefix: Some("Vol 14 No ".into()),
            },
        )
        .unwrap();

        let sub = get_subscription(&conn, sub_id).unwrap();
        assert_eq!(sub.title, "Journal of Library Science");
        assert_eq!(sub.frequency, "quarterly");
        assert_eq!(sub.total_issues_count, 4);
        assert_eq!(sub.expected_issues_count, 4);
        assert_eq!(sub.received_issues_count, 0);

        let issues = list_issues(
            &conn,
            IssueFilter {
                subscription_id: Some(sub_id),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(issues.len(), 4);
        assert_eq!(issues[0].issue_number, "Vol 14 No 1");
        assert_eq!(issues[0].expected_date, "2026-01-01");
        assert_eq!(issues[3].issue_number, "Vol 14 No 4");
        assert_eq!(issues[3].expected_date, "2026-10-01");
    }

    #[test]
    fn issue_check_in_with_accessioning_and_duplicate_rejection() {
        let mut conn = setup_test_db();
        let actor = test_actor();

        let sub_id = save_subscription(
            &mut conn,
            &actor,
            SaveSubscriptionInput {
                id: None,
                title: "Nature".into(),
                issn: Some("0028-0836".into()),
                publisher: Some("Springer Nature".into()),
                frequency: "monthly".into(),
                vendor_id: None,
                budget_id: None,
                cost_minor: Some(240000),
                currency: Some("INR".into()),
                start_date: "2026-01-01".into(),
                end_date: "2026-03-31".into(),
                notes: None,
                auto_generate_issues: Some(true),
                issue_pattern_prefix: Some("Issue ".into()),
            },
        )
        .unwrap();

        let issues = list_issues(
            &conn,
            IssueFilter {
                subscription_id: Some(sub_id),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(issues.len(), 3);
        let first_issue_id = issues[0].id;

        // Check in first issue with physical copy accessioning
        let res = check_in_issue(
            &mut conn,
            &actor,
            CheckInIssueInput {
                issue_id: first_issue_id,
                received_date: Some("2026-01-05".into()),
                accession_copy: Some(true),
                item_category_id: None,
                location_id: None,
                notes: Some("Delivered in good condition".into()),
            },
        )
        .unwrap();

        assert_eq!(res.status, "received");
        assert_eq!(res.received_date, "2026-01-05");
        assert!(res.accession_no.is_some());
        assert!(res.barcode.is_some());
        assert!(res.copy_id.is_some());

        // Verify copy exists in physical_items
        let copy_status: String = conn
            .query_row(
                "SELECT status FROM physical_items WHERE id = ?1",
                params![res.copy_id.unwrap()],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(copy_status, "available");

        // Duplicate check-in must fail
        let dup_err = check_in_issue(
            &mut conn,
            &actor,
            CheckInIssueInput {
                issue_id: first_issue_id,
                received_date: None,
                accession_copy: None,
                item_category_id: None,
                location_id: None,
                notes: None,
            },
        );
        assert!(dup_err.is_err(), "Duplicate check-in should be rejected");

        // Verify sub counts
        let sub = get_subscription(&conn, sub_id).unwrap();
        assert_eq!(sub.received_issues_count, 1);
        assert_eq!(sub.expected_issues_count, 2);
    }

    #[test]
    fn missing_and_claimed_issue_tracking() {
        let mut conn = setup_test_db();
        let actor = test_actor();

        let sub_id = save_subscription(
            &mut conn,
            &actor,
            SaveSubscriptionInput {
                id: None,
                title: "The Economist".into(),
                issn: Some("0013-0613".into()),
                publisher: Some("Economist Group".into()),
                frequency: "weekly".into(),
                vendor_id: None,
                budget_id: None,
                cost_minor: Some(180000),
                currency: Some("INR".into()),
                start_date: "2026-01-01".into(),
                end_date: "2026-01-15".into(),
                notes: None,
                auto_generate_issues: Some(true),
                issue_pattern_prefix: None,
            },
        )
        .unwrap();

        let issues = list_issues(
            &conn,
            IssueFilter {
                subscription_id: Some(sub_id),
                ..Default::default()
            },
        )
        .unwrap();
        let issue_id = issues[0].id;

        // Mark as missing
        update_issue_status(&mut conn, &actor, issue_id, "missing").unwrap();
        let updated = list_issues(
            &conn,
            IssueFilter {
                subscription_id: Some(sub_id),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(updated[0].status, "missing");

        // Claim issue from vendor
        update_issue_status(&mut conn, &actor, issue_id, "claimed").unwrap();
        let claimed = list_issues(
            &conn,
            IssueFilter {
                subscription_id: Some(sub_id),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(claimed[0].status, "claimed");
        assert_eq!(claimed[0].claim_count, 1);
        assert!(claimed[0].claim_date.is_some());

        // Check list_missing_issues
        let missing_list = list_missing_issues(&conn).unwrap();
        assert!(!missing_list.is_empty());
    }

    #[test]
    fn irregular_frequency_manual_issue_and_renewal() {
        let mut conn = setup_test_db();
        let actor = test_actor();

        let sub_id = save_subscription(
            &mut conn,
            &actor,
            SaveSubscriptionInput {
                id: None,
                title: "Special Memoirs Series".into(),
                issn: None,
                publisher: Some("Research Society".into()),
                frequency: "irregular".into(),
                vendor_id: None,
                budget_id: None,
                cost_minor: Some(50000),
                currency: Some("INR".into()),
                start_date: "2026-01-01".into(),
                end_date: "2026-12-31".into(),
                notes: None,
                auto_generate_issues: Some(false),
                issue_pattern_prefix: None,
            },
        )
        .unwrap();

        // Irregular has 0 auto issues
        let issues = list_issues(
            &conn,
            IssueFilter {
                subscription_id: Some(sub_id),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(issues.len(), 0);

        // Manually add issue
        let issue_id = add_issue(
            &mut conn,
            &actor,
            AddIssueInput {
                subscription_id: sub_id,
                issue_number: "Special Edition No. 1".into(),
                expected_date: "2026-06-15".into(),
                notes: Some("Monograph issue".into()),
            },
        )
        .unwrap();
        assert!(issue_id > 0);

        // Duplicate issue number is rejected
        let dup = add_issue(
            &mut conn,
            &actor,
            AddIssueInput {
                subscription_id: sub_id,
                issue_number: "Special Edition No. 1".into(),
                expected_date: "2026-07-15".into(),
                notes: None,
            },
        );
        assert!(dup.is_err());

        // Renew subscription
        let renewal_id = renew_subscription(
            &mut conn,
            &actor,
            RenewSubscriptionInput {
                subscription_id: sub_id,
                start_date: "2027-01-01".into(),
                end_date: "2027-12-31".into(),
                cost_minor: 60000,
                currency: Some("INR".into()),
                budget_id: None,
                notes: Some("Renewed for 2027".into()),
                auto_generate_issues: Some(false),
                issue_pattern_prefix: None,
            },
        )
        .unwrap();
        assert!(renewal_id > 0);

        let renewals = list_renewals(&conn, sub_id).unwrap();
        assert_eq!(renewals.len(), 1);
        assert_eq!(renewals[0].cost_minor, 60000);

        let sub = get_subscription(&conn, sub_id).unwrap();
        assert_eq!(sub.end_date, "2027-12-31");
    }

    #[test]
    fn serials_expenditure_and_due_reminders() {
        let mut conn = setup_test_db();
        let actor = test_actor();

        // 1. Subscription A: starts 2026-01-01, cost 50,000
        let _sub_a = save_subscription(
            &mut conn,
            &actor,
            SaveSubscriptionInput {
                id: None,
                title: "ACM Computing Surveys".into(),
                issn: Some("0360-0300".into()),
                publisher: Some("ACM".into()),
                frequency: "quarterly".into(),
                vendor_id: None,
                budget_id: None,
                cost_minor: Some(50000),
                currency: Some("INR".into()),
                start_date: "2026-01-01".into(),
                end_date: "2026-12-31".into(),
                notes: None,
                auto_generate_issues: Some(false),
                issue_pattern_prefix: None,
            },
        )
        .unwrap();

        // 2. Subscription B: starts 2025-01-01, ends in the past, cost 30,000
        let sub_b = save_subscription(
            &mut conn,
            &actor,
            SaveSubscriptionInput {
                id: None,
                title: "IEEE Software".into(),
                issn: Some("0740-7459".into()),
                publisher: Some("IEEE".into()),
                frequency: "bimonthly".into(),
                vendor_id: None,
                budget_id: None,
                cost_minor: Some(30000),
                currency: Some("INR".into()),
                start_date: "2025-01-01".into(),
                end_date: "2025-12-31".into(),
                notes: None,
                auto_generate_issues: Some(false),
                issue_pattern_prefix: None,
            },
        )
        .unwrap();

        // 3. Renew Subscription B for 2026, cost 35,000
        let ren_b = renew_subscription(
            &mut conn,
            &actor,
            RenewSubscriptionInput {
                subscription_id: sub_b,
                start_date: "2026-01-01".into(),
                end_date: "2026-12-31".into(),
                cost_minor: 35000,
                currency: Some("INR".into()),
                budget_id: None,
                notes: Some("2026 renewal".into()),
                auto_generate_issues: Some(false),
                issue_pattern_prefix: None,
            },
        )
        .unwrap();
        assert!(ren_b > 0);

        // 4. Test date range expenditure for 2026: should include Sub A initial (50,000) and Sub B renewal (35,000) = 85,000
        let exp_2026 =
            get_serials_expenditure(&conn, Some("2026-01-01"), Some("2026-12-31")).unwrap();
        assert_eq!(exp_2026.total_spend_minor, 85000);
        assert_eq!(exp_2026.item_count, 2);
        assert_eq!(exp_2026.items.len(), 2);
        assert!(exp_2026
            .items
            .iter()
            .any(|i| i.title == "ACM Computing Surveys"
                && i.item_type == "initial"
                && i.cost_minor == 50000));
        assert!(exp_2026.items.iter().any(|i| i.title == "IEEE Software"
            && i.item_type == "renewal"
            && i.cost_minor == 35000));

        // 5. Test date range expenditure for 2025: should include only Sub B initial (30,000)
        let exp_2025 =
            get_serials_expenditure(&conn, Some("2025-01-01"), Some("2025-12-31")).unwrap();
        assert_eq!(exp_2025.total_spend_minor, 30000);
        assert_eq!(exp_2025.item_count, 1);
        assert_eq!(exp_2025.items[0].title, "IEEE Software");
        assert_eq!(exp_2025.items[0].item_type, "initial");

        // 6. Test all-time expenditure (no date filters)
        let exp_all = get_serials_expenditure(&conn, None, None).unwrap();
        assert_eq!(exp_all.total_spend_minor, 115000);
        assert_eq!(exp_all.item_count, 3);

        // 7. Test expiring subscriptions / due reminders
        // Sub B was extended to 2026-12-31, but let's insert a subscription expiring in 5 days
        let exp_soon_end: String = conn
            .query_row("SELECT date('now','localtime','+5 days')", [], |r| r.get(0))
            .unwrap();
        let _sub_c = save_subscription(
            &mut conn,
            &actor,
            SaveSubscriptionInput {
                id: None,
                title: "Expiring Today Gazette".into(),
                issn: None,
                publisher: None,
                frequency: "monthly".into(),
                vendor_id: None,
                budget_id: None,
                cost_minor: Some(12000),
                currency: Some("INR".into()),
                start_date: "2026-01-01".into(),
                end_date: exp_soon_end,
                notes: None,
                auto_generate_issues: Some(false),
                issue_pattern_prefix: None,
            },
        )
        .unwrap();

        let expiring = list_expiring_subscriptions(&conn, Some(10)).unwrap();
        assert!(!expiring.is_empty());
        let found = expiring
            .iter()
            .find(|e| e.subscription.title == "Expiring Today Gazette")
            .unwrap();
        assert_eq!(found.days_remaining, 5);
        assert!(!found.is_overdue);
    }
}
