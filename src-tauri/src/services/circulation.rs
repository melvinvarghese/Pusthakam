use crate::domain::validate;
use crate::error::{AppError, AppResult};
use crate::services::{audit, auth::Session, catalogue, fines, reservations};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssuePreview {
    pub member_id: i64,
    pub item_id: i64,
    pub member_name: String,
    pub title: String,
    pub due_on: String,
    #[serde(skip)]
    pub item_category_id: i64,
    #[serde(skip)]
    pub reservation_id: Option<i64>,
    #[serde(skip)]
    pub member_category_id: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReturnPreview {
    pub loan_id: i64,
    pub member_name: String,
    pub title: String,
    pub due_on: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_fine: Option<fines::FineAmount>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ReturnCondition {
    #[default]
    Available,
    Damaged,
    Lost,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OverdueLoan {
    #[serde(flatten)]
    loan: ReturnPreview,
    barcode: String,
    member_number: String,
    days_overdue: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverduePage {
    rows: Vec<OverdueLoan>,
    page: i64,
    page_size: i64,
    total: i64,
}

pub fn list_overdue(conn: &Connection, page: i64) -> AppResult<OverduePage> {
    const PAGE_SIZE: i64 = 25;
    let today: String = conn.query_row("SELECT date('now','localtime')", [], |r| r.get(0))?;
    let total: i64 = conn.query_row(
        "SELECT count(*) FROM loans WHERE returned_at IS NULL AND due_on < ?1",
        [&today],
        |r| r.get(0),
    )?;
    let page = page.clamp(1, ((total + PAGE_SIZE - 1) / PAGE_SIZE).max(1));
    let mut stmt = conn.prepare(
        "SELECT l.id, m.name, r.title, l.due_on, i.barcode, m.member_number,
                CAST(julianday(?1) - julianday(l.due_on) AS INTEGER)
         FROM loans l JOIN members m ON m.id = l.member_id
         JOIN physical_items i ON i.id = l.item_id JOIN bibliographic_records r ON r.id = i.record_id
         WHERE l.returned_at IS NULL AND l.due_on < ?1
         ORDER BY l.due_on, l.id LIMIT ?2 OFFSET ?3",
    )?;
    let rows = stmt
        .query_map(params![today, PAGE_SIZE, (page - 1) * PAGE_SIZE], |r| {
            Ok(OverdueLoan {
                loan: ReturnPreview {
                    loan_id: r.get(0)?,
                    member_name: r.get(1)?,
                    title: r.get(2)?,
                    due_on: r.get(3)?,
                    estimated_fine: None,
                },
                barcode: r.get(4)?,
                member_number: r.get(5)?,
                days_overdue: r.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(OverduePage {
        rows,
        page,
        page_size: PAGE_SIZE,
        total,
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenewalPreview {
    loan_id: i64,
    member_name: String,
    title: String,
    previous_due_on: String,
    due_on: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenewalConfirmation {
    pub loan_id: i64,
    pub previous_due_on: String,
    pub due_on: String,
}

pub fn prepare_issue(
    conn: &mut Connection,
    member_number: &str,
    barcode: &str,
) -> AppResult<IssuePreview> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let item_id = reservations::copy_id(&tx, barcode)?;
    reservations::refresh_copy(&tx, item_id)?;
    let preview = issue_terms(&tx, member_number, item_id);
    tx.commit()?;
    preview
}

fn issue_terms(conn: &Connection, member_number: &str, item_id: i64) -> AppResult<IssuePreview> {
    let number = validate::required("Member number", member_number, 64)?;
    let (member_id, member_name, category_id, eligible): (i64, String, i64, bool) = conn.query_row(
        "SELECT m.id, m.name, m.category_id,
                m.archived_at IS NULL AND c.archived_at IS NULL AND m.expires_on >= date('now','localtime')
         FROM members m JOIN member_categories c ON c.id = m.category_id WHERE m.member_number = ?1",
        [&number], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
    ).optional()?.ok_or_else(|| AppError::NotFound("No member has that member number.".into()))?;
    if !eligible {
        return Err(AppError::Conflict(
            "This membership or its category is archived or expired.".into(),
        ));
    }
    let (title, item_category_id, status, active): (String, i64, String, bool) = conn.query_row(
        "SELECT r.title, i.item_category_id, i.status, r.archived_at IS NULL AND c.archived_at IS NULL
         FROM physical_items i JOIN bibliographic_records r ON r.id = i.record_id
         JOIN item_categories c ON c.id = i.item_category_id WHERE i.id = ?1",
        [item_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
    )?;
    if !active || !matches!(status.as_str(), "available" | "reserved_hold") {
        return Err(AppError::Conflict(
            "This copy is unavailable for issue.".into(),
        ));
    }
    let reservation_id = if status == "reserved_hold" {
        let (id, _) = reservations::ready(conn, item_id)?
            .filter(|(_, owner)| *owner == member_id)
            .ok_or_else(|| {
                AppError::Conflict("This copy is reserved for another member.".into())
            })?;
        Some(id)
    } else {
        None
    };
    let (due_on, max_loans): (String, i64) = conn.query_row(
        "SELECT date('now','localtime','+' || loan_days || ' days'), max_loans FROM circulation_policies
         WHERE member_category_id = ?1 AND item_category_id = ?2",
        params![category_id, item_category_id], |r| Ok((r.get(0)?, r.get(1)?)),
    ).optional()?.ok_or_else(|| AppError::Validation("No loan rule allows this member category to borrow this type of copy.".into()))?;
    let open: i64 = conn.query_row(
        "SELECT count(*) FROM loans WHERE member_id = ?1 AND item_category_id = ?2 AND returned_at IS NULL",
        params![member_id, item_category_id], |r| r.get(0),
    )?;
    if open >= max_loans {
        return Err(AppError::Conflict(
            "This member has reached the loan limit for this type of copy.".into(),
        ));
    }
    let due_on = crate::services::calendar::next_open_date(conn, &due_on)?;
    Ok(IssuePreview {
        member_id,
        item_id,
        member_name,
        title,
        due_on,
        item_category_id,
        reservation_id,
        member_category_id: category_id,
    })
}

pub fn prepare_return(conn: &Connection, barcode: &str) -> AppResult<ReturnPreview> {
    let item_id = reservations::copy_id(conn, barcode)?;
    let mut preview = conn
        .query_row(
            "SELECT l.id, m.name, r.title, l.due_on FROM loans l
         JOIN members m ON m.id = l.member_id JOIN physical_items i ON i.id = l.item_id
         JOIN bibliographic_records r ON r.id = i.record_id
         WHERE l.item_id = ?1 AND l.returned_at IS NULL",
            [item_id],
            |r| {
                Ok(ReturnPreview {
                    loan_id: r.get(0)?,
                    member_name: r.get(1)?,
                    title: r.get(2)?,
                    due_on: r.get(3)?,
                    estimated_fine: None,
                })
            },
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound("There is no open loan for that copy.".into()))?;
    preview.estimated_fine = fines::estimate_return(conn, preview.loan_id)?;
    Ok(preview)
}

pub fn prepare_renewal(conn: &Connection, barcode: &str) -> AppResult<RenewalPreview> {
    let loan = prepare_return(conn, barcode)?;
    renewal_terms(conn, loan.loan_id)
}

fn renewal_terms(conn: &Connection, loan_id: i64) -> AppResult<RenewalPreview> {
    let (member_name, title, previous_due_on, due_on, eligible, overdue, allowed, item_id, member_id):
        (String, String, String, Option<String>, bool, bool, bool, i64, i64) = conn.query_row(
        "SELECT m.name, r.title, l.due_on, date(l.due_on, '+' || p.renewal_days || ' days'),
                m.archived_at IS NULL AND m.expires_on >= date('now','localtime') AND mc.archived_at IS NULL
                    AND ic.archived_at IS NULL AND r.archived_at IS NULL AND i.status = 'on_loan',
                l.due_on < date('now','localtime'),
                COALESCE(p.max_renewals > (SELECT count(*) FROM loan_renewals WHERE loan_id = l.id), 0),
                l.item_id, l.member_id
         FROM loans l JOIN members m ON m.id = l.member_id
         JOIN physical_items i ON i.id = l.item_id JOIN bibliographic_records r ON r.id = i.record_id
         JOIN member_categories mc ON mc.id = m.category_id JOIN item_categories ic ON ic.id = l.item_category_id
         LEFT JOIN circulation_policies p ON p.member_category_id = m.category_id AND p.item_category_id = l.item_category_id
         WHERE l.id = ?1 AND l.returned_at IS NULL",
        [loan_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?, r.get(8)?)),
    ).optional()?.ok_or_else(|| AppError::Conflict("This loan is no longer open.".into()))?;
    if !eligible {
        return Err(AppError::Conflict(
            "This membership or copy is no longer eligible for renewal.".into(),
        ));
    }
    if overdue {
        return Err(AppError::Conflict(
            "This loan is overdue. Return the copy before issuing it again.".into(),
        ));
    }
    if !allowed {
        return Err(AppError::Conflict(
            "The current loan rule allows no further renewals.".into(),
        ));
    }
    if reservations::has_waiting(conn, item_id, member_id)? {
        return Err(AppError::Conflict(
            "Another member is waiting for this copy.".into(),
        ));
    }
    let due_on = due_on.ok_or_else(|| {
        AppError::Validation("The renewal due date could not be calculated.".into())
    })?;
    let due_on = crate::services::calendar::next_open_date(conn, &due_on)?;
    Ok(RenewalPreview {
        loan_id,
        member_name,
        title,
        previous_due_on,
        due_on,
    })
}

pub fn renew(
    conn: &mut Connection,
    actor: &Session,
    confirmation: &RenewalConfirmation,
) -> AppResult<()> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let preview = renewal_terms(&tx, confirmation.loan_id)?;
    if preview.previous_due_on != confirmation.previous_due_on
        || preview.due_on != confirmation.due_on
    {
        return Err(AppError::Conflict(
            "Renewal details changed. Review the loan again.".into(),
        ));
    }
    tx.execute(
        "UPDATE loans SET due_on = ?2 WHERE id = ?1",
        params![preview.loan_id, preview.due_on],
    )?;
    tx.execute(
        "INSERT INTO loan_renewals (loan_id, renewed_by, previous_due_on, due_on) VALUES (?1, ?2, ?3, ?4)",
        params![preview.loan_id, actor.user_id, preview.previous_due_on, preview.due_on],
    )?;
    audit::record(
        &tx,
        Some(actor),
        "circulation.renew",
        "loans",
        Some(preview.loan_id),
        "Renewed loan",
    )?;
    tx.commit()?;
    Ok(())
}

pub fn issue(
    conn: &mut Connection,
    actor: &Session,
    member_number: &str,
    barcode: &str,
    expected: (i64, i64, &str),
) -> AppResult<()> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let item_id = reservations::copy_id(&tx, barcode)?;
    reservations::refresh_copy(&tx, item_id)?;
    let preview = issue_terms(&tx, member_number, item_id)?;
    if (preview.member_id, preview.item_id, preview.due_on.as_str()) != expected {
        return Err(AppError::Conflict(
            "Issue details changed. Review the loan again.".into(),
        ));
    }
    tx.execute(
        "INSERT INTO loans (item_id, member_id, item_category_id, issued_by, due_on) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![preview.item_id, preview.member_id, preview.item_category_id, actor.user_id, preview.due_on],
    )?;
    let loan_id = tx.last_insert_rowid();
    fines::snapshot(
        &tx,
        loan_id,
        preview.member_category_id,
        preview.item_category_id,
    )?;
    let from = if let Some(id) = preview.reservation_id {
        reservations::fulfill(&tx, actor, id)?;
        "reserved_hold"
    } else {
        "available"
    };
    change_copy_status(&tx, Some(actor), preview.item_id, from, "on_loan", "issued")?;
    audit::record(
        &tx,
        Some(actor),
        "circulation.issue",
        "loans",
        Some(loan_id),
        "Issued copy",
    )?;
    tx.commit()?;
    Ok(())
}

pub fn return_copy(
    conn: &mut Connection,
    actor: &Session,
    loan_id: i64,
    condition: ReturnCondition,
) -> AppResult<Option<fines::FineAmount>> {
    let (status, event, summary) = match condition {
        ReturnCondition::Available => ("available", "returned", "Returned copy"),
        ReturnCondition::Damaged => ("damaged", "returned_damaged", "Returned damaged copy"),
        ReturnCondition::Lost => ("lost", "reported_lost", "Closed loan for lost copy"),
    };
    if !matches!(condition, ReturnCondition::Available) {
        actor.require("circulation.manage")?;
    }
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let item_id: i64 = tx.query_row(
        "UPDATE loans SET returned_at = strftime('%Y-%m-%dT%H:%M:%SZ','now'), returned_by = ?2, return_condition = ?3
         WHERE id = ?1 AND returned_at IS NULL RETURNING item_id",
        params![loan_id, actor.user_id, status], |r| r.get(0),
    ).optional()?.ok_or_else(|| AppError::Conflict("This loan has already been closed or no longer exists.".into()))?;
    change_copy_status(&tx, Some(actor), item_id, "on_loan", status, event)?;
    reservations::refresh_copy(&tx, item_id)?;
    let fine = fines::assess_return(&tx, actor, loan_id)?;
    audit::record(
        &tx,
        Some(actor),
        "circulation.return",
        "loans",
        Some(loan_id),
        summary,
    )?;
    tx.commit()?;
    Ok(fine)
}

pub(super) fn change_copy_status(
    conn: &Connection,
    actor: Option<&Session>,
    item_id: i64,
    from: &str,
    to: &str,
    event: &str,
) -> AppResult<()> {
    let changed = conn.execute(
        "UPDATE physical_items SET status = ?3, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now')
         WHERE id = ?1 AND status = ?2",
        params![item_id, from, to],
    )?;
    if changed != 1 {
        return Err(AppError::Conflict(
            "The copy's status has changed. Review it before proceeding.".into(),
        ));
    }
    catalogue::event(conn, actor, item_id, event, Some(from), Some(to), None)
}
