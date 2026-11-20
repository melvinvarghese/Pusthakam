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
        "SELECT m.id, m.member_number, m.name, c.id, c.name, m.email, m.phone, m.expires_on
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
    let previous: Option<(i64, String)> = match input.id {
        Some(id) => Some(
            tx.query_row(
                "SELECT category_id, expires_on FROM members WHERE id = ?1 AND archived_at IS NULL",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?)),
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
                previous.as_ref().map(|(category, _)| *category)
            ],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| AppError::Validation("Choose an active member category.".into()))?;
    let expires_on = match input.expires_on.as_deref() {
        Some(date) => validate::date("Expiry date", date)?,
        None => previous.map_or(default_expiry, |(_, expiry)| expiry),
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
             phone = ?5, expires_on = ?6 WHERE id = ?7",
            params![
                number,
                name,
                input.category_id,
                email,
                phone,
                expires_on,
                id
            ],
        )?;
        (id, "member.update")
    } else {
        tx.execute(
            "INSERT INTO members (member_number, name, category_id, email, phone, expires_on)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![number, name, input.category_id, email, phone, expires_on],
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
