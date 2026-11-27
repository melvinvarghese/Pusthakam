use crate::domain::validate;
use crate::error::{AppError, AppResult};
use crate::services::{audit, auth::Session};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};

const PAGE_SIZE: i64 = 25;
const FILTER: &str = "FROM members m JOIN member_categories c ON c.id = m.category_id
    WHERE (m.archived_at IS NOT NULL) = ?1
      AND (m.member_number LIKE ?2 ESCAPE '!' OR m.name LIKE ?2 ESCAPE '!')";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberInput {
    pub(super) id: Option<i64>,
    pub(super) member_number: String,
    pub(super) name: String,
    pub(super) category_id: i64,
    pub(super) email: Option<String>,
    pub(super) phone: Option<String>,
    pub(super) expires_on: Option<String>,
    #[serde(default)]
    pub(super) address: Option<String>,
    #[serde(default)]
    pub(super) department: Option<String>,
    #[serde(default)]
    pub(super) joined_on: Option<String>,
    #[serde(default)]
    pub(super) notes: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Member {
    id: i64,
    member_number: String,
    name: String,
    category_id: i64,
    category_name: String,
    email: Option<String>,
    phone: Option<String>,
    expires_on: String,
    address: Option<String>,
    department: Option<String>,
    joined_on: Option<String>,
    notes: Option<String>,
    opac_username: Option<String>,
    has_opac_password: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberPage {
    rows: Vec<Member>,
    total: i64,
    page: i64,
    page_size: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MemberLoan {
    id: i64,
    title: String,
    barcode: String,
    issued_on: String,
    due_on: String,
    returned_on: Option<String>,
    overdue: bool,
    return_condition: Option<String>,
    import_source: Option<String>,
    source_staff: Option<String>,
    renewal_count: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberLoanPage {
    rows: Vec<MemberLoan>,
    total: i64,
    page: i64,
    page_size: i64,
}

pub fn loans(conn: &Connection, member_id: i64, page: i64) -> AppResult<MemberLoanPage> {
    let total: i64 = conn.query_row(
        "SELECT (SELECT count(*) FROM loans WHERE member_id = m.id) FROM members m WHERE m.id = ?1",
        [member_id], |r| r.get(0),
    ).optional()?.ok_or_else(|| AppError::NotFound("Member not found.".into()))?;
    let page = page.clamp(1, (total + PAGE_SIZE - 1).div_euclid(PAGE_SIZE).max(1));
    let mut stmt = conn.prepare(
        "SELECT l.id, r.title, i.barcode, date(l.issued_at,'localtime'), l.due_on,
                date(l.returned_at,'localtime'), l.returned_at IS NULL AND l.due_on < date('now','localtime'), l.return_condition, s.source_system, s.source_staff,
                (SELECT count(*) FROM loan_renewals n WHERE n.loan_id=l.id)
         FROM loans l JOIN physical_items i ON i.id = l.item_id
         JOIN bibliographic_records r ON r.id = i.record_id
         LEFT JOIN loan_import_sources s ON s.loan_id = l.id
         WHERE l.member_id = ?1
         ORDER BY l.returned_at IS NOT NULL, l.issued_at DESC, l.id DESC
         LIMIT ?2 OFFSET ?3",
    )?;
    let rows = stmt
        .query_map(params![member_id, PAGE_SIZE, (page - 1) * PAGE_SIZE], |r| {
            Ok(MemberLoan {
                id: r.get(0)?,
                title: r.get(1)?,
                barcode: r.get(2)?,
                issued_on: r.get(3)?,
                due_on: r.get(4)?,
                returned_on: r.get(5)?,
                overdue: r.get(6)?,
                return_condition: r.get(7)?,
                import_source: r.get(8)?,
                source_staff: r.get(9)?,
                renewal_count: r.get(10)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(MemberLoanPage {
        rows,
        total,
        page,
        page_size: PAGE_SIZE,
    })
}

pub fn list(conn: &Connection, search: &str, page: i64, archived: bool) -> AppResult<MemberPage> {
    let escaped = search
        .trim()
        .replace('!', "!!")
        .replace('%', "!%")
        .replace('_', "!_");
    let pattern = format!("%{escaped}%");
    let total: i64 = conn.query_row(
        &format!("SELECT count(*) {FILTER}"),
        params![archived, pattern],
        |r| r.get(0),
    )?;
    let page = page.clamp(1, (total + PAGE_SIZE - 1).div_euclid(PAGE_SIZE).max(1));
    let mut stmt = conn.prepare(&format!(
        "SELECT m.id, m.member_number, m.name, c.id, c.name, m.email, m.phone, m.expires_on,
                m.address, m.department, m.joined_on, m.notes,
                m.opac_username, m.opac_password_hash IS NOT NULL
         {FILTER}
         ORDER BY m.member_number = ?3 COLLATE NOCASE DESC, m.name COLLATE NOCASE, m.id
         LIMIT ?4 OFFSET ?5"
    ))?;
    let rows = stmt
        .query_map(
            params![
                archived,
                pattern,
                search.trim(),
                PAGE_SIZE,
                (page - 1) * PAGE_SIZE
            ],
            |r| {
                Ok(Member {
                    id: r.get(0)?,
                    member_number: r.get(1)?,
                    name: r.get(2)?,
                    category_id: r.get(3)?,
                    category_name: r.get(4)?,
                    email: r.get(5)?,
                    phone: r.get(6)?,
                    expires_on: r.get(7)?,
                    address: r.get(8)?,
                    department: r.get(9)?,
                    joined_on: r.get(10)?,
                    notes: r.get(11)?,
                    opac_username: r.get(12)?,
                    has_opac_password: r.get(13)?,
                })
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(MemberPage {
        rows,
        total,
        page,
        page_size: PAGE_SIZE,
    })
}

pub fn save(conn: &mut Connection, actor: &Session, input: MemberInput) -> AppResult<i64> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let id = save_in_transaction(&tx, actor, input)?;
    tx.commit()?;
    Ok(id)
}

pub(super) fn save_in_transaction(
    tx: &rusqlite::Transaction<'_>,
    actor: &Session,
    input: MemberInput,
) -> AppResult<i64> {
    let number = validate::required("Member number", &input.member_number, 64)?;
    let name = validate::required("Name", &input.name, 160)?;
    let email = validate::optional("Email", input.email.as_deref(), 254)?;
    let phone = validate::optional("Phone", input.phone.as_deref(), 40)?;
    let address = validate::optional("Address", input.address.as_deref(), 500)?;
    let department = validate::optional("Department", input.department.as_deref(), 120)?;
    let notes = validate::optional("Notes", input.notes.as_deref(), 2000)?;
    let joined_on_input = match input.joined_on.as_deref() {
        Some(date) if !date.trim().is_empty() => Some(validate::date("Start date", date)?),
        _ => None,
    };
    let previous: Option<(i64, String, Option<String>)> = match input.id {
        Some(id) => Some(
            tx.query_row(
                "SELECT category_id, expires_on, joined_on FROM members WHERE id = ?1 AND archived_at IS NULL",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?
            .ok_or_else(|| AppError::NotFound("Active member not found.".into()))?,
        ),
        None => None,
    };
    let default_expiry: String = tx
        .query_row(
            "SELECT date('now','localtime','+' || membership_days || ' days')
         FROM member_categories WHERE id = ?1 AND (archived_at IS NULL OR id = ?2)",
            params![
                input.category_id,
                previous.as_ref().map(|(category, _, _)| *category)
            ],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| AppError::Validation("Choose an active member category.".into()))?;
    let default_joined: String =
        tx.query_row("SELECT date('now','localtime')", [], |r| r.get(0))?;
    let (expires_on, joined_on) = match previous {
        Some((_, prev_expiry, prev_joined)) => (
            match input.expires_on.as_deref() {
                Some(date) => validate::date("Expiry date", date)?,
                None => prev_expiry,
            },
            joined_on_input.or(prev_joined),
        ),
        None => (
            match input.expires_on.as_deref() {
                Some(date) => validate::date("Expiry date", date)?,
                None => default_expiry,
            },
            Some(joined_on_input.unwrap_or(default_joined)),
        ),
    };
    let duplicate: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM members WHERE member_number = ?1 AND id IS NOT ?2)",
        params![number, input.id],
        |r| r.get(0),
    )?;
    if duplicate {
        return Err(AppError::Conflict(
            "That member number is already in use.".into(),
        ));
    }
    let (id, action) = if let Some(id) = input.id {
        tx.execute(
            "UPDATE members SET member_number = ?1, name = ?2, category_id = ?3, email = ?4,
             phone = ?5, expires_on = ?6, address = ?7, department = ?8, joined_on = ?9, notes = ?10
             WHERE id = ?11",
            params![
                number,
                name,
                input.category_id,
                email,
                phone,
                expires_on,
                address,
                department,
                joined_on,
                notes,
                id
            ],
        )?;
        (id, "member.update")
    } else {
        tx.execute(
            "INSERT INTO members (member_number, name, category_id, email, phone, expires_on, address, department, joined_on, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![number, name, input.category_id, email, phone, expires_on, address, department, joined_on, notes],
        )?;
        (tx.last_insert_rowid(), "member.create")
    };
    audit::record(
        tx,
        Some(actor),
        action,
        "member",
        Some(id),
        "Saved member profile",
    )?;
    Ok(id)
}

pub fn archive(conn: &mut Connection, actor: &Session, id: i64, archived: bool) -> AppResult<()> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let category_archived: bool = tx.query_row(
        "SELECT c.archived_at IS NOT NULL FROM members m JOIN member_categories c ON c.id = m.category_id WHERE m.id = ?1",
        [id], |r| r.get(0),
    ).optional()?.ok_or_else(|| AppError::NotFound("Member not found.".into()))?;
    if !archived && category_archived {
        return Err(AppError::Validation(
            "Restore the member category in Library Settings first.".into(),
        ));
    }
    if archived
        && tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM loans WHERE member_id = ?1 AND returned_at IS NULL)",
            [id],
            |r| r.get::<_, bool>(0),
        )?
    {
        return Err(AppError::Conflict(
            "Return this member's open loans before archiving.".into(),
        ));
    }
    let changed = tx.execute(
        "UPDATE members SET archived_at = CASE WHEN ?2 THEN strftime('%Y-%m-%dT%H:%M:%SZ','now') END
         WHERE id = ?1 AND (archived_at IS NULL) = ?2",
        params![id, archived],
    )?;
    if changed > 0 {
        let action = if archived {
            "member.archive"
        } else {
            "member.restore"
        };
        audit::record(
            &tx,
            Some(actor),
            action,
            "member",
            Some(id),
            "Changed member status",
        )?;
    }
    tx.commit()?;
    Ok(())
}

pub fn set_opac_credentials(
    conn: &mut Connection,
    actor: &Session,
    id: i64,
    username: Option<&str>,
    password: Option<&str>,
    clear_password: bool,
) -> AppResult<()> {
    let clean_user = username
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|u| {
            if u.len() < 3
                || u.len() > 40
                || !u
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
            {
                return Err(AppError::Validation(
                    "OPAC username must be 3 to 40 letters, numbers, dots, underscores, or hyphens."
                        .into(),
                ));
            }
            Ok(u.to_string())
        })
        .transpose()?;

    let clean_pass = password
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|p| {
            if p.len() < 4 || p.len() > 128 {
                return Err(AppError::Validation(
                    "OPAC password or PIN must be at least 4 characters.".into(),
                ));
            }
            crate::services::auth::hash_password(p)
        })
        .transpose()?;

    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let exists: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM members WHERE id = ?1)",
        [id],
        |r| r.get(0),
    )?;
    if !exists {
        return Err(AppError::NotFound("Member not found.".into()));
    }

    let update_res = if clear_password {
        tx.execute(
            "UPDATE members SET opac_username = ?1, opac_password_hash = NULL WHERE id = ?2",
            params![clean_user, id],
        )
    } else if let Some(hash) = clean_pass {
        tx.execute(
            "UPDATE members SET opac_username = ?1, opac_password_hash = ?2 WHERE id = ?3",
            params![clean_user, hash, id],
        )
    } else {
        tx.execute(
            "UPDATE members SET opac_username = ?1 WHERE id = ?2",
            params![clean_user, id],
        )
    };

    if let Err(err) = update_res {
        if let rusqlite::Error::SqliteFailure(inner, _) = &err {
            if inner.extended_code == 2067 || inner.extended_code == 1555 {
                return Err(AppError::Conflict(
                    "That OPAC username is already assigned to another member.".into(),
                ));
            }
        }
        return Err(err.into());
    }

    audit::record(
        &tx,
        Some(actor),
        "member.opac_credentials",
        "member",
        Some(id),
        "Updated member OPAC login credentials",
    )?;
    tx.commit()?;
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberFineBalance {
    pub currency: String,
    pub outstanding_minor: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberSummary {
    pub active_loans: i64,
    pub overdue_loans: i64,
    pub total_loans: i64,
    pub active_holds: i64,
    pub max_loans_limit: Option<i64>,
    pub membership_days: i64,
    pub eligible_to_borrow: bool,
    pub unpaid_fines: Vec<MemberFineBalance>,
}

pub fn summary(conn: &Connection, id: i64) -> AppResult<MemberSummary> {
    let (category_id, membership_days, eligible_to_borrow): (i64, i64, bool) = conn
        .query_row(
            "SELECT m.category_id, c.membership_days,
                    (m.archived_at IS NULL AND m.expires_on >= date('now','localtime'))
             FROM members m
             JOIN member_categories c ON c.id = m.category_id
             WHERE m.id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound("Member not found.".into()))?;

    let (active_loans, overdue_loans, total_loans): (i64, i64, i64) = conn.query_row(
        "SELECT COALESCE(SUM(CASE WHEN returned_at IS NULL THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN returned_at IS NULL AND due_on < date('now','localtime') THEN 1 ELSE 0 END), 0),
                count(*)
         FROM loans WHERE member_id = ?1",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;

    let active_holds: i64 = conn.query_row(
        "SELECT count(*) FROM reservations WHERE member_id = ?1 AND closed_at IS NULL",
        [id],
        |r| r.get(0),
    )?;

    let max_loans_limit: Option<i64> = conn.query_row(
        "SELECT MAX(max_loans) FROM circulation_policies WHERE member_category_id = ?1",
        [category_id],
        |r| r.get(0),
    )?;

    let mut stmt = conn.prepare(
        "SELECT f.currency,
                COALESCE(SUM(MAX(0, f.amount_minor - COALESCE(s.paid_minor, 0))), 0) AS outstanding
         FROM fines f
         JOIN loans l ON l.id = f.loan_id
         LEFT JOIN (
             SELECT fine_id, SUM(amount_minor) AS paid_minor FROM fine_settlements GROUP BY fine_id
         ) s ON s.fine_id = f.id
         WHERE l.member_id = ?1
         GROUP BY f.currency
         HAVING outstanding > 0",
    )?;
    let unpaid_fines = stmt
        .query_map([id], |r| {
            Ok(MemberFineBalance {
                currency: r.get(0)?,
                outstanding_minor: r.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(MemberSummary {
        active_loans,
        overdue_loans,
        total_loans,
        active_holds,
        max_loans_limit,
        membership_days,
        eligible_to_borrow,
        unpaid_fines,
    })
}

pub fn renew_membership(conn: &mut Connection, actor: &Session, id: i64) -> AppResult<String> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let new_expiry: String = tx
        .query_row(
            "SELECT date(MAX(date('now','localtime'), m.expires_on), '+' || c.membership_days || ' days')
             FROM members m
             JOIN member_categories c ON c.id = m.category_id
             WHERE m.id = ?1 AND m.archived_at IS NULL",
            [id],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound("Active member not found.".into()))?;
    tx.execute(
        "UPDATE members SET expires_on = ?1 WHERE id = ?2",
        params![new_expiry, id],
    )?;
    audit::record(
        &tx,
        Some(actor),
        "member.renew",
        "member",
        Some(id),
        &format!("Renewed membership through {new_expiry}"),
    )?;
    tx.commit()?;
    Ok(new_expiry)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberHold {
    pub id: i64,
    pub title: String,
    pub barcode: String,
    pub status: String,
    pub placed_at: String,
    pub expires_at: Option<String>,
    pub closed_at: Option<String>,
    pub queue_position: Option<i64>,
}

pub fn holds(conn: &mut Connection, member_id: i64) -> AppResult<Vec<MemberHold>> {
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM members WHERE id = ?1)",
        [member_id],
        |r| r.get(0),
    )?;
    if !exists {
        return Err(AppError::NotFound("Member not found.".into()));
    }
    let _ = crate::services::reservations::refresh(conn);
    let mut stmt = conn.prepare(
        "SELECT r.id, br.title, i.barcode, r.status, r.placed_at, r.expires_at, r.closed_at,
                CASE WHEN r.closed_at IS NULL THEN (
                    SELECT count(*) FROM reservations q
                    WHERE q.item_id = r.item_id AND q.closed_at IS NULL AND q.id <= r.id
                ) END
         FROM reservations r
         JOIN physical_items i ON i.id = r.item_id
         JOIN bibliographic_records br ON br.id = i.record_id
         WHERE r.member_id = ?1
         ORDER BY r.closed_at IS NOT NULL, r.status = 'ready' DESC, r.id DESC
         LIMIT 50",
    )?;
    let rows = stmt
        .query_map([member_id], |r| {
            Ok(MemberHold {
                id: r.get(0)?,
                title: r.get(1)?,
                barcode: r.get(2)?,
                status: r.get(3)?,
                placed_at: r.get(4)?,
                expires_at: r.get(5)?,
                closed_at: r.get(6)?,
                queue_position: r.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberActivityItem {
    pub id: String,
    pub kind: String,
    pub occurred_at: String,
    pub title: String,
    pub detail: String,
    pub staff_name: Option<String>,
}

pub fn activity(conn: &Connection, member_id: i64) -> AppResult<Vec<MemberActivityItem>> {
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM members WHERE id = ?1)",
        [member_id],
        |r| r.get(0),
    )?;
    if !exists {
        return Err(AppError::NotFound("Member not found.".into()));
    }

    let mut items: Vec<MemberActivityItem> = Vec::new();

    // Loans & returns
    {
        let mut stmt = conn.prepare(
            "SELECT l.id, br.title, i.barcode, l.issued_at, l.due_on, u1.display_name,
                    l.returned_at, l.return_condition, u2.display_name
             FROM loans l
             JOIN physical_items i ON i.id = l.item_id
             JOIN bibliographic_records br ON br.id = i.record_id
             LEFT JOIN staff_users u1 ON u1.id = l.issued_by
             LEFT JOIN staff_users u2 ON u2.id = l.returned_by
             WHERE l.member_id = ?1
             ORDER BY l.issued_at DESC LIMIT 30",
        )?;
        let rows = stmt.query_map([member_id], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, Option<String>>(5)?,
                r.get::<_, Option<String>>(6)?,
                r.get::<_, Option<String>>(7)?,
                r.get::<_, Option<String>>(8)?,
            ))
        })?;
        for row in rows {
            let (id, title, barcode, issued_at, due_on, issued_by, returned_at, cond, returned_by) =
                row?;
            items.push(MemberActivityItem {
                id: format!("loan-{id}"),
                kind: "loan".into(),
                occurred_at: issued_at,
                title: format!("Borrowed: {title}"),
                detail: format!("Copy {barcode} · Due {due_on}"),
                staff_name: issued_by,
            });
            if let Some(ret_at) = returned_at {
                items.push(MemberActivityItem {
                    id: format!("return-{id}"),
                    kind: "return".into(),
                    occurred_at: ret_at,
                    title: format!("Returned: {title}"),
                    detail: format!(
                        "Copy {barcode} · Condition: {}",
                        cond.as_deref().unwrap_or("available")
                    ),
                    staff_name: returned_by,
                });
            }
        }
    }

    // Renewals
    {
        let mut stmt = conn.prepare(
            "SELECT n.id, br.title, i.barcode, n.renewed_at, n.previous_due_on, n.due_on, u.display_name
             FROM loan_renewals n
             JOIN loans l ON l.id = n.loan_id
             JOIN physical_items i ON i.id = l.item_id
             JOIN bibliographic_records br ON br.id = i.record_id
             LEFT JOIN staff_users u ON u.id = n.renewed_by
             WHERE l.member_id = ?1
             ORDER BY n.renewed_at DESC LIMIT 20",
        )?;
        let rows = stmt.query_map([member_id], |r| {
            Ok(MemberActivityItem {
                id: format!("renew-{}", r.get::<_, i64>(0)?),
                kind: "renewal".into(),
                occurred_at: r.get(3)?,
                title: format!("Renewed: {}", r.get::<_, String>(1)?),
                detail: format!(
                    "Copy {} · {} → {}",
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, String>(5)?
                ),
                staff_name: r.get(6)?,
            })
        })?;
        for row in rows {
            items.push(row?);
        }
    }

    // Fine settlements
    {
        let mut stmt = conn.prepare(
            "SELECT s.id, s.kind, s.amount_minor, f.currency, s.method, s.recorded_at, br.title, u.display_name
             FROM fine_settlements s
             JOIN fines f ON f.id = s.fine_id
             JOIN loans l ON l.id = f.loan_id
             JOIN physical_items i ON i.id = l.item_id
             JOIN bibliographic_records br ON br.id = i.record_id
             LEFT JOIN staff_users u ON u.id = s.staff_user_id
             WHERE l.member_id = ?1
             ORDER BY s.recorded_at DESC LIMIT 20",
        )?;
        let rows = stmt.query_map([member_id], |r| {
            let kind: String = r.get(1)?;
            let minor: i64 = r.get(2)?;
            let currency: String = r.get(3)?;
            let method: Option<String> = r.get(4)?;
            let book: String = r.get(6)?;
            Ok(MemberActivityItem {
                id: format!("fine-{}", r.get::<_, i64>(0)?),
                kind: "fine".into(),
                occurred_at: r.get(5)?,
                title: format!(
                    "Fine {}: {} {:.2}",
                    kind,
                    currency,
                    (minor as f64) / 100.0
                ),
                detail: format!(
                    "{book}{}",
                    method.map(|m| format!(" · {m}")).unwrap_or_default()
                ),
                staff_name: r.get(7)?,
            })
        })?;
        for row in rows {
            items.push(row?);
        }
    }

    // Reservations
    {
        let mut stmt = conn.prepare(
            "SELECT r.id, br.title, i.barcode, r.status, r.placed_at, u.display_name
             FROM reservations r
             JOIN physical_items i ON i.id = r.item_id
             JOIN bibliographic_records br ON br.id = i.record_id
             LEFT JOIN staff_users u ON u.id = r.placed_by
             WHERE r.member_id = ?1
             ORDER BY r.placed_at DESC LIMIT 20",
        )?;
        let rows = stmt.query_map([member_id], |r| {
            Ok(MemberActivityItem {
                id: format!("hold-{}", r.get::<_, i64>(0)?),
                kind: "hold".into(),
                occurred_at: r.get(4)?,
                title: format!("Hold ({}): {}", r.get::<_, String>(3)?, r.get::<_, String>(1)?),
                detail: format!("Copy {}", r.get::<_, String>(2)?),
                staff_name: r.get(5)?,
            })
        })?;
        for row in rows {
            items.push(row?);
        }
    }

    // Administrative / membership audit events
    {
        let mut stmt = conn.prepare(
            "SELECT a.id, a.action, a.summary, a.occurred_at, a.actor_name
             FROM audit_events a
             WHERE a.entity_type = 'member' AND a.entity_id = ?1
             ORDER BY a.occurred_at DESC, a.id DESC LIMIT 20",
        )?;
        let rows = stmt.query_map([member_id], |r| {
            Ok(MemberActivityItem {
                id: format!("audit-{}", r.get::<_, i64>(0)?),
                kind: "admin".into(),
                occurred_at: r.get(3)?,
                title: r.get(2)?,
                detail: r.get(1)?,
                staff_name: r.get(4)?,
            })
        })?;
        for row in rows {
            items.push(row?);
        }
    }

    items.sort_by(|a, b| b.occurred_at.cmp(&a.occurred_at));
    items.truncate(50);
    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::services::auth;

    #[test]
    fn summary_and_renew_membership_work() {
        let dir = tempfile::tempdir().unwrap();
        let mut conn = db::open(&dir.path().join("test.sqlite3")).unwrap();
        auth::create_user(&conn, "admin", "Admin", "admin_pass_123", "administrator").unwrap();
        let actor = Session {
            user_id: 1,
            username: "admin".into(),
            display_name: "Admin".into(),
            role: "administrator".into(),
            permissions: Default::default(),
            revision: 1,
        };
        conn.execute(
            "INSERT INTO member_categories (id, name, membership_days) VALUES (1, 'Student', 180)",
            [],
        )
        .unwrap();
        conn.execute("INSERT INTO item_categories (id, name) VALUES (1, 'General')", [])
            .unwrap();
        conn.execute(
            "INSERT INTO circulation_policies (member_category_id, item_category_id, loan_days, max_loans)
             VALUES (1, 1, 14, 4)",
            [],
        )
        .unwrap();
        let id = save(
            &mut conn,
            &actor,
            MemberInput {
                id: None,
                member_number: "S-101".into(),
                name: "Meera Nair".into(),
                category_id: 1,
                email: None,
                phone: None,
                expires_on: Some("2020-01-01".into()),
                address: Some("Kottayam".into()),
                department: Some("Physics".into()),
                joined_on: Some("2020-01-01".into()),
                notes: Some("Honors student".into()),
            },
        )
        .unwrap();

        let s1 = summary(&conn, id).unwrap();
        assert!(!s1.eligible_to_borrow);
        assert_eq!(s1.max_loans_limit, Some(4));
        assert_eq!(s1.membership_days, 180);

        let new_expiry = renew_membership(&mut conn, &actor, id).unwrap();
        let s2 = summary(&conn, id).unwrap();
        assert!(s2.eligible_to_borrow);
        assert!(new_expiry >= "2026-01-01".to_string());

        let act = activity(&conn, id).unwrap();
        assert!(act.iter().any(|item| item.kind == "admin" && item.detail == "member.renew"));
        let h = holds(&mut conn, id).unwrap();
        assert!(h.is_empty());
    }
}
