use crate::domain::{accession, validate};
use crate::error::{AppError, AppResult};
use crate::services::{audit, auth::Session, circulation};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::Serialize;

const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%SZ','now')";
const ELIGIBLE: &str = "m.archived_at IS NULL AND m.expires_on >= date('now','localtime')
    AND mc.archived_at IS NULL AND ic.archived_at IS NULL AND br.archived_at IS NULL";
const COPY_POLICY: &str = "JOIN physical_items i ON i.id = r.item_id
    JOIN bibliographic_records br ON br.id = i.record_id
    JOIN item_categories ic ON ic.id = i.item_category_id
    JOIN members m ON m.id = r.member_id JOIN member_categories mc ON mc.id = m.category_id
    JOIN circulation_policies p ON p.member_category_id = m.category_id AND p.item_category_id = i.item_category_id";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Reservation {
    id: i64,
    title: String,
    barcode: String,
    member_name: String,
    member_number: String,
    status: String,
    expires_at: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReservationPage {
    rows: Vec<Reservation>,
    page: i64,
    page_size: i64,
    total: i64,
}

pub(super) fn copy_id(conn: &Connection, code: &str) -> AppResult<i64> {
    let code = accession::validate_code("Book barcode or accession number", code)?;
    let mut stmt = conn.prepare(
        "SELECT id FROM physical_items WHERE barcode = ?1 OR accession_no = ?1 LIMIT 2",
    )?;
    let mut rows = stmt.query([&code])?;
    let id = rows.next()?.map(|row| row.get(0)).transpose()?
        .ok_or_else(|| AppError::NotFound("No copy has that barcode or accession number.".into()))?;
    if rows.next()?.is_some() {
        return Err(AppError::Conflict("That code matches two different copies. Enter a unique barcode or accession number.".into()));
    }
    Ok(id)
}

/// Apply expiry and promote the next eligible request within the caller's transaction.
pub(super) fn refresh_copy(conn: &Connection, item_id: i64) -> AppResult<()> {
    conn.execute(&format!(
        "UPDATE reservations SET status = 'expired', closed_at = {NOW}
         WHERE item_id = ?1 AND status = 'ready' AND expires_at <= {NOW}"
    ), [item_id])?;
    conn.execute(&format!(
        "UPDATE reservations SET status = 'cancelled', closed_at = {NOW}
         WHERE item_id = ?1 AND closed_at IS NULL AND id NOT IN (
            SELECT r.id FROM reservations r {COPY_POLICY} WHERE r.item_id = ?1 AND r.closed_at IS NULL AND {ELIGIBLE}
         )"
    ), [item_id])?;
    let status: String = conn.query_row("SELECT status FROM physical_items WHERE id = ?1", [item_id], |r| r.get(0))?;
    if !matches!(status.as_str(), "available" | "reserved_hold") {
        return Ok(());
    }
    if ready(conn, item_id)?.is_some() {
        return Ok(());
    }
    let promoted = conn.execute(&format!(
        "UPDATE reservations SET status = 'ready', expires_at = (
            SELECT strftime('%Y-%m-%dT%H:%M:%SZ','now','+' || p.reservation_hold_days || ' days')
            FROM reservations r {COPY_POLICY} WHERE r.id = reservations.id
         ) WHERE id = (
            SELECT id FROM reservations WHERE item_id = ?1 AND status = 'pending' ORDER BY id LIMIT 1
         )"
    ), [item_id])?;
    let next_status = if promoted == 1 { "reserved_hold" } else { "available" };
    if status != next_status {
        let event = if promoted == 1 { "reservation_ready" } else { "reservation_released" };
        circulation::change_copy_status(conn, None, item_id, &status, next_status, event)?;
    }
    Ok(())
}

pub(super) fn ready(conn: &Connection, item_id: i64) -> AppResult<Option<(i64, i64)>> {
    Ok(conn.query_row(
        "SELECT id, member_id FROM reservations WHERE item_id = ?1 AND status = 'ready'",
        [item_id], |r| Ok((r.get(0)?, r.get(1)?)),
    ).optional()?)
}

pub(super) fn has_waiting(conn: &Connection, item_id: i64, borrower_id: i64) -> AppResult<bool> {
    Ok(conn.query_row(&format!(
        "SELECT EXISTS(SELECT 1 FROM reservations r {COPY_POLICY}
         WHERE r.item_id = ?1 AND r.member_id <> ?2 AND r.status = 'pending' AND {ELIGIBLE})"
    ), params![item_id, borrower_id], |r| r.get(0))?)
}

pub(super) fn fulfill(conn: &Connection, actor: &Session, id: i64) -> AppResult<()> {
    conn.execute(&format!("UPDATE reservations SET status = 'fulfilled', closed_at = {NOW} WHERE id = ?1"), [id])?;
    audit::record(conn, Some(actor), "reservation.fulfill", "reservations", Some(id), "Fulfilled reservation")
}

pub fn refresh(conn: &mut Connection) -> AppResult<()> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let ids = {
        let mut stmt = tx.prepare(&format!(
            "SELECT DISTINCT candidate.item_id
             FROM reservations candidate JOIN physical_items copy ON copy.id = candidate.item_id
             WHERE candidate.closed_at IS NULL AND (
                copy.status = 'available' OR (candidate.status = 'ready' AND candidate.expires_at <= {NOW})
                OR NOT EXISTS (SELECT 1 FROM reservations r {COPY_POLICY} WHERE r.id = candidate.id AND {ELIGIBLE})
             )"
        ))?;
        let rows = stmt.query_map([], |r| r.get::<_, i64>(0))?;
        rows.collect::<Result<Vec<_>, _>>()?
    };
    for item_id in ids {
        refresh_copy(&tx, item_id)?;
    }
    tx.commit()?;
    Ok(())
}

pub fn place(conn: &mut Connection, actor: &Session, member_number: &str, barcode: &str) -> AppResult<()> {
    let number = validate::required("Member number", member_number, 64)?;
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let item_id = copy_id(&tx, barcode)?;
    refresh_copy(&tx, item_id)?;
    let member_id: i64 = tx.query_row(
        &format!(
            "SELECT m.id FROM members m JOIN member_categories mc ON mc.id = m.category_id
             JOIN physical_items i ON i.id = ?2 JOIN item_categories ic ON ic.id = i.item_category_id
             JOIN bibliographic_records br ON br.id = i.record_id
             JOIN circulation_policies p ON p.member_category_id = m.category_id AND p.item_category_id = i.item_category_id
             WHERE m.member_number = ?1 AND {ELIGIBLE} AND i.status IN ('available','on_loan','reserved_hold')"
        ),
        params![number, item_id], |r| r.get(0),
    ).optional()?.ok_or_else(|| AppError::Validation("This member and copy are not eligible for a reservation. Check the member number, copy status, and loan rules.".into()))?;
    let duplicate: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM reservations WHERE item_id = ?1 AND member_id = ?2 AND closed_at IS NULL)
             OR EXISTS(SELECT 1 FROM loans WHERE item_id = ?1 AND member_id = ?2 AND returned_at IS NULL)",
        params![item_id, member_id], |r| r.get(0),
    )?;
    if duplicate {
        return Err(AppError::Conflict("This member already has a reservation or open loan for this copy.".into()));
    }
    tx.execute("INSERT INTO reservations (item_id, member_id, placed_by) VALUES (?1, ?2, ?3)",
        params![item_id, member_id, actor.user_id])?;
    let id = tx.last_insert_rowid();
    refresh_copy(&tx, item_id)?;
    audit::record(&tx, Some(actor), "reservation.place", "reservations", Some(id), "Placed reservation")?;
    tx.commit()?;
    Ok(())
}

pub fn cancel(conn: &mut Connection, actor: &Session, id: i64) -> AppResult<()> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let item_id: i64 = tx.query_row(&format!(
        "UPDATE reservations SET status = 'cancelled', closed_at = {NOW}
         WHERE id = ?1 AND closed_at IS NULL RETURNING item_id"
    ), [id], |r| r.get(0)).optional()?.ok_or_else(|| AppError::Conflict("This reservation is no longer active.".into()))?;
    refresh_copy(&tx, item_id)?;
    audit::record(&tx, Some(actor), "reservation.cancel", "reservations", Some(id), "Cancelled reservation")?;
    tx.commit()?;
    Ok(())
}

pub fn list(conn: &mut Connection, page: i64) -> AppResult<ReservationPage> {
    refresh(conn)?;
    const SIZE: i64 = 25;
    let total: i64 = conn.query_row("SELECT count(*) FROM reservations WHERE closed_at IS NULL", [], |r| r.get(0))?;
    let page = page.clamp(1, ((total + SIZE - 1) / SIZE).max(1));
    let mut stmt = conn.prepare(
        "SELECT r.id, br.title, i.barcode, m.name, m.member_number, r.status, r.expires_at
         FROM reservations r JOIN physical_items i ON i.id = r.item_id
         JOIN bibliographic_records br ON br.id = i.record_id JOIN members m ON m.id = r.member_id
         WHERE r.closed_at IS NULL
         ORDER BY r.status = 'ready' DESC, r.id LIMIT ?1 OFFSET ?2",
    )?;
    let rows = stmt.query_map(params![SIZE, (page - 1) * SIZE], |r| {
        Ok(Reservation {
            id: r.get(0)?, title: r.get(1)?, barcode: r.get(2)?, member_name: r.get(3)?,
            member_number: r.get(4)?, status: r.get(5)?, expires_at: r.get(6)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;
    Ok(ReservationPage { rows, page, page_size: SIZE, total })
}
