use crate::domain::validate;
use crate::error::{AppError, AppResult};
use crate::services::{audit, auth::Session};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};

const MAX_SAFE_MINOR: i64 = (1i64 << 53) - 1;
const BALANCES: &str = "WITH balances AS (
    SELECT f.id, f.loan_id, f.amount_minor, f.currency,
           COALESCE(SUM(s.amount_minor) FILTER (WHERE s.kind = 'payment'), 0) AS paid_minor,
           COALESCE(SUM(s.amount_minor) FILTER (WHERE s.kind = 'waiver'), 0) AS waived_minor
    FROM fines f LEFT JOIN fine_settlements s ON s.fine_id = f.id GROUP BY f.id
)";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FineAmount {
    amount_minor: i64,
    currency: String,
}

pub(super) fn snapshot(
    conn: &Connection,
    loan_id: i64,
    member_category_id: i64,
    item_category_id: i64,
) -> AppResult<()> {
    conn.execute(
        "INSERT INTO loan_fine_terms (loan_id, rate_minor, grace_days, cap_minor, currency)
         SELECT ?1, p.fine_per_day_minor, p.fine_grace_days, p.fine_cap_minor, lib.currency
         FROM circulation_policies p JOIN libraries lib ON lib.id = 1
         WHERE p.member_category_id = ?2 AND p.item_category_id = ?3",
        params![loan_id, member_category_id, item_category_id],
    )?;
    Ok(())
}

pub(super) fn estimate_return(conn: &Connection, loan_id: i64) -> AppResult<Option<FineAmount>> {
    let terms: Option<(i64, i64, Option<i64>, String)> = conn.query_row(
        "SELECT CAST(julianday(COALESCE(date(l.returned_at,'localtime'), date('now','localtime'))) - julianday(l.due_on) AS INTEGER) - t.grace_days,
                t.rate_minor, t.cap_minor, t.currency
         FROM loans l JOIN loan_fine_terms t ON t.loan_id = l.id WHERE l.id = ?1",
        [loan_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
    ).optional()?;
    let Some((chargeable_days, rate, cap, currency)) = terms else {
        return Ok(None);
    };
    let amount_minor = chargeable_days
        .max(0)
        .saturating_mul(rate)
        .min(cap.unwrap_or(MAX_SAFE_MINOR))
        .min(MAX_SAFE_MINOR);
    Ok((amount_minor > 0).then_some(FineAmount {
        amount_minor,
        currency,
    }))
}

pub(super) fn assess_return(
    conn: &Connection,
    actor: &Session,
    loan_id: i64,
) -> AppResult<Option<FineAmount>> {
    let Some(fine) = estimate_return(conn, loan_id)? else {
        return Ok(None);
    };
    conn.execute(
        "INSERT INTO fines (loan_id, amount_minor, currency) VALUES (?1, ?2, ?3)",
        params![loan_id, fine.amount_minor, fine.currency],
    )?;
    audit::record(
        conn,
        Some(actor),
        "fine.assess",
        "fines",
        Some(conn.last_insert_rowid()),
        "Assessed overdue fine",
    )?;
    Ok(Some(fine))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Fine {
    id: i64,
    member_name: String,
    member_number: String,
    title: String,
    amount_minor: i64,
    paid_minor: i64,
    waived_minor: i64,
    outstanding_minor: i64,
    currency: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinePage {
    rows: Vec<Fine>,
    page: i64,
    page_size: i64,
    total: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MemberFineRow {
    #[serde(flatten)]
    fine: Fine,
    barcode: String,
    assessed_at: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MemberBalance {
    currency: String,
    assessed_minor: i64,
    paid_minor: i64,
    waived_minor: i64,
    outstanding_minor: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberAccount {
    rows: Vec<MemberFineRow>,
    balances: Vec<MemberBalance>,
    page: i64,
    page_size: i64,
    total: i64,
}

/// Uses the same balance calculation as the circulation ledger. Summary spans all pages.
pub fn member_account(
    conn: &mut Connection,
    member_id: i64,
    page: i64,
) -> AppResult<MemberAccount> {
    const SIZE: i64 = 25;
    let tx = conn.transaction()?;
    let exists: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM members WHERE id=?1)",
        [member_id],
        |r| r.get(0),
    )?;
    if !exists {
        return Err(AppError::NotFound("Member not found.".into()));
    }
    let total: i64 = tx.query_row(
        "SELECT count(*) FROM fines f JOIN loans l ON l.id=f.loan_id WHERE l.member_id=?1",
        [member_id],
        |r| r.get(0),
    )?;
    let page = page.clamp(1, ((total + SIZE - 1) / SIZE).max(1));
    let balances = {
        let mut stmt = tx.prepare(&format!("{BALANCES}
            SELECT b.currency, SUM(b.amount_minor), SUM(b.paid_minor), SUM(b.waived_minor), SUM(b.amount_minor-b.paid_minor-b.waived_minor)
            FROM balances b JOIN loans l ON l.id=b.loan_id WHERE l.member_id=?1 GROUP BY b.currency ORDER BY b.currency"))?;
        let rows = stmt
            .query_map([member_id], |r| {
                Ok(MemberBalance {
                    currency: r.get(0)?,
                    assessed_minor: r.get(1)?,
                    paid_minor: r.get(2)?,
                    waived_minor: r.get(3)?,
                    outstanding_minor: r.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        for b in &rows {
            if [
                b.assessed_minor,
                b.paid_minor,
                b.waived_minor,
                b.outstanding_minor,
            ]
            .iter()
            .any(|v| !(0..=MAX_SAFE_MINOR).contains(v))
            {
                return Err(AppError::Validation(
                    "The member account totals cannot be displayed safely. Review the fine ledger."
                        .into(),
                ));
            }
        }
        rows
    };
    let rows = {
        let mut stmt = tx.prepare(&format!("{BALANCES}
            SELECT b.id,m.name,m.member_number,r.title,b.amount_minor,b.paid_minor,b.waived_minor,b.currency,i.barcode,f.assessed_at
            FROM balances b JOIN loans l ON l.id=b.loan_id JOIN members m ON m.id=l.member_id
            JOIN physical_items i ON i.id=l.item_id JOIN bibliographic_records r ON r.id=i.record_id JOIN fines f ON f.id=b.id
            WHERE l.member_id=?1 ORDER BY b.id DESC LIMIT ?2 OFFSET ?3"))?;
        let rows = stmt
            .query_map(params![member_id, SIZE, (page - 1) * SIZE], |r| {
                let amount: i64 = r.get(4)?;
                let paid: i64 = r.get(5)?;
                let waived: i64 = r.get(6)?;
                Ok(MemberFineRow {
                    fine: Fine {
                        id: r.get(0)?,
                        member_name: r.get(1)?,
                        member_number: r.get(2)?,
                        title: r.get(3)?,
                        amount_minor: amount,
                        paid_minor: paid,
                        waived_minor: waived,
                        outstanding_minor: amount - paid - waived,
                        currency: r.get(7)?,
                    },
                    barcode: r.get(8)?,
                    assessed_at: r.get(9)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        rows
    };
    tx.commit()?;
    Ok(MemberAccount {
        rows,
        balances,
        page,
        page_size: SIZE,
        total,
    })
}

pub fn member_history(
    conn: &Connection,
    member_id: i64,
    fine_id: i64,
    page: i64,
    show_waiver_reason: bool,
) -> AppResult<SettlementPage> {
    let owned: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM fines f JOIN loans l ON l.id=f.loan_id WHERE f.id=?1 AND l.member_id=?2)", params![fine_id,member_id], |r| r.get(0))?;
    if !owned {
        return Err(AppError::NotFound("Fine not found for this member.".into()));
    }
    history(conn, fine_id, page, show_waiver_reason)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SettlementEntry {
    id: i64,
    kind: String,
    amount_minor: i64,
    method: Option<String>,
    reason: Option<String>,
    recorded_at: String,
    staff_name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettlementPage {
    rows: Vec<SettlementEntry>,
    page: i64,
    page_size: i64,
    total: i64,
}

pub fn history(
    conn: &Connection,
    fine_id: i64,
    page: i64,
    show_waiver_reason: bool,
) -> AppResult<SettlementPage> {
    const SIZE: i64 = 25;
    let total: i64 = conn.query_row(
        "SELECT (SELECT count(*) FROM fine_settlements WHERE fine_id = f.id) FROM fines f WHERE f.id = ?1",
        [fine_id], |row| row.get(0),
    ).optional()?.ok_or_else(|| AppError::NotFound("Fine not found.".into()))?;
    let page = page.clamp(1, ((total + SIZE - 1) / SIZE).max(1));
    let mut stmt = conn.prepare(
        "SELECT s.id, s.kind, s.amount_minor, s.method, CASE WHEN ?2 THEN s.reason END, s.recorded_at,
                COALESCE(p.source_staff || ' (imported source)', u.display_name)
         FROM fine_settlements s JOIN staff_users u ON u.id = s.staff_user_id
         LEFT JOIN history_import_sources p ON p.kind='settlement' AND p.local_id=s.id
           WHERE s.fine_id = ?1 ORDER BY s.recorded_at DESC, s.id DESC LIMIT ?3 OFFSET ?4",
    )?;
    let rows = stmt
        .query_map(
            params![fine_id, show_waiver_reason, SIZE, (page - 1) * SIZE],
            |r| {
                Ok(SettlementEntry {
                    id: r.get(0)?,
                    kind: r.get(1)?,
                    amount_minor: r.get(2)?,
                    method: r.get(3)?,
                    reason: r.get(4)?,
                    recorded_at: r.get(5)?,
                    staff_name: r.get(6)?,
                })
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(SettlementPage {
        rows,
        page,
        page_size: SIZE,
        total,
    })
}

pub fn list(conn: &Connection, page: i64, include_settled: bool) -> AppResult<FinePage> {
    const SIZE: i64 = 25;
    const FILTER: &str = "WHERE (?1 OR b.amount_minor > b.paid_minor + b.waived_minor)";
    let total: i64 = conn.query_row(
        &format!("{BALANCES} SELECT count(*) FROM balances b {FILTER}"),
        [include_settled],
        |r| r.get(0),
    )?;
    let page = page.clamp(1, ((total + SIZE - 1) / SIZE).max(1));
    let mut stmt = conn.prepare(&format!(
        "{BALANCES} SELECT b.id, m.name, m.member_number, r.title, b.amount_minor, b.paid_minor, b.waived_minor, b.currency
         FROM balances b JOIN loans l ON l.id = b.loan_id JOIN members m ON m.id = l.member_id
         JOIN physical_items i ON i.id = l.item_id JOIN bibliographic_records r ON r.id = i.record_id
         {FILTER} ORDER BY b.id DESC LIMIT ?2 OFFSET ?3"
    ))?;
    let rows = stmt
        .query_map(params![include_settled, SIZE, (page - 1) * SIZE], |r| {
            let (amount_minor, paid_minor, waived_minor): (i64, i64, i64) =
                (r.get(4)?, r.get(5)?, r.get(6)?);
            Ok(Fine {
                id: r.get(0)?,
                member_name: r.get(1)?,
                member_number: r.get(2)?,
                title: r.get(3)?,
                amount_minor,
                paid_minor,
                waived_minor,
                outstanding_minor: amount_minor - paid_minor - waived_minor,
                currency: r.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(FinePage {
        rows,
        page,
        page_size: SIZE,
        total,
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum Settlement {
    Payment { method: String },
    Waiver { reason: String },
}

pub fn settle(
    conn: &mut Connection,
    actor: &Session,
    fine_id: i64,
    amount_minor: i64,
    expected_balance: i64,
    entry: Settlement,
) -> AppResult<()> {
    let reason;
    let (kind, method, note) = match entry {
        Settlement::Payment { ref method } => {
            actor.require("circulation.issue")?;
            if !matches!(method.as_str(), "cash" | "card" | "bank_transfer" | "other") {
                return Err(AppError::Validation(
                    "Choose a valid payment method.".into(),
                ));
            }
            ("payment", Some(method.as_str()), None)
        }
        Settlement::Waiver { reason: ref value } => {
            actor.require("circulation.manage")?;
            reason = validate::required("Waiver reason", value, 500)?;
            ("waiver", None, Some(reason.as_str()))
        }
    };
    validate::in_range("Amount", amount_minor, 1, MAX_SAFE_MINOR)?;
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let balance: i64 = tx
        .query_row(
            &format!(
        "{BALANCES} SELECT amount_minor - paid_minor - waived_minor FROM balances WHERE id = ?1"
    ),
            [fine_id],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound("Fine not found.".into()))?;
    if balance != expected_balance {
        return Err(AppError::Conflict(
            "The outstanding balance changed. Review the fine again.".into(),
        ));
    }
    if amount_minor > balance {
        return Err(AppError::Validation(
            "The amount cannot exceed the outstanding balance.".into(),
        ));
    }
    tx.execute(
        "INSERT INTO fine_settlements (fine_id, staff_user_id, kind, amount_minor, method, reason)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![fine_id, actor.user_id, kind, amount_minor, method, note],
    )?;
    audit::record(
        &tx,
        Some(actor),
        &format!("fine.{kind}"),
        "fines",
        Some(fine_id),
        "Recorded fine settlement",
    )?;
    tx.commit()?;
    Ok(())
}
