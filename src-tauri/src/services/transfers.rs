use crate::domain::validate;
use crate::error::{AppError, AppResult};
use crate::services::{audit, auth::Session, inventory, reservations};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Transfer {
    id: i64,
    barcode: String,
    title: String,
    source: String,
    destination: String,
    status: String,
    sent_at: String,
    finished_at: Option<String>,
}

pub fn list(conn: &Connection, page: i64) -> AppResult<inventory::Page<Transfer>> {
    let total: i64 =
        conn.query_row("SELECT count(*) FROM stock_transfers", [], |row| row.get(0))?;
    let page = inventory::page_number(page, total);
    let mut stmt = conn.prepare(
        "SELECT id, barcode, title, source_label, destination_label, status, sent_at, finished_at
         FROM stock_transfers ORDER BY id DESC LIMIT ?1 OFFSET ?2",
    )?;
    let rows = stmt
        .query_map(
            params![inventory::PAGE_SIZE, (page - 1) * inventory::PAGE_SIZE],
            |row| {
                Ok(Transfer {
                    id: row.get(0)?,
                    barcode: row.get(1)?,
                    title: row.get(2)?,
                    source: row.get(3)?,
                    destination: row.get(4)?,
                    status: row.get(5)?,
                    sent_at: row.get(6)?,
                    finished_at: row.get(7)?,
                })
            },
        )?
        .collect::<Result<_, _>>()?;
    Ok(inventory::Page {
        rows,
        total,
        page,
        page_size: inventory::PAGE_SIZE,
    })
}

pub fn ensure_idle(conn: &Connection, item_id: i64) -> AppResult<()> {
    let active: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM stock_transfers WHERE item_id = ?1 AND status = 'in_transit')",
        [item_id],
        |row| row.get(0),
    )?;
    if active {
        return Err(AppError::Conflict(
            "Receive or cancel the open transfer in Inventory before editing this copy.".into(),
        ));
    }
    Ok(())
}

fn label(location: String, shelf: Option<String>) -> String {
    match shelf {
        Some(shelf) => format!("{location} / {shelf}"),
        None => location,
    }
}

fn history(
    conn: &Connection,
    actor: &Session,
    item_id: i64,
    action: &str,
    from: &str,
    to: &str,
) -> AppResult<()> {
    conn.execute(
        "INSERT INTO item_events (item_id, event_type, from_status, to_status, staff_user_id)
        VALUES (?1, ?2, ?3, ?4, ?5)",
        params![item_id, action, from, to, actor.user_id],
    )?;
    Ok(())
}

pub fn send(
    conn: &mut Connection,
    actor: &Session,
    barcode: &str,
    location: i64,
    shelf: Option<i64>,
) -> AppResult<i64> {
    let barcode = validate::required("Barcode", barcode, 120)?;
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let item_id = reservations::copy_id(&tx, &barcode)?;
    reservations::refresh_copy(&tx, item_id)?;
    let (status, archived, source, source_shelf, source_name, source_code): (String, bool, Option<i64>, Option<i64>, Option<String>, Option<String>) = tx.query_row(
        "SELECT i.status, r.archived_at IS NOT NULL, i.location_id, i.shelf_id, l.name, s.code
         FROM physical_items i JOIN bibliographic_records r ON r.id = i.record_id
         LEFT JOIN locations l ON l.id = i.location_id LEFT JOIN shelves s ON s.id = i.shelf_id WHERE i.id = ?1",
        [item_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
    )?;
    if status != "available" || archived {
        return Err(AppError::Conflict(
            "Only an available copy of an active record can be transferred.".into(),
        ));
    }
    let (source, source_name) = source.zip(source_name).ok_or_else(|| {
        AppError::Validation("Assign the copy a sending location before transferring it.".into())
    })?;
    if (source, source_shelf) == (location, shelf) {
        return Err(AppError::Validation(
            "Choose a different destination.".into(),
        ));
    }
    let (destination, destination_shelf) = inventory::active_place(&tx, location, shelf)?;
    tx.execute("UPDATE physical_items SET status = 'in_transit', updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now')
        WHERE id = ?1", [item_id])?;
    tx.execute("INSERT INTO stock_transfers (item_id, source_location_id, source_shelf_id, destination_location_id,
        destination_shelf_id, source_label, destination_label, sent_by, barcode, title)
        SELECT i.id, ?2, ?3, ?4, ?5, ?6, ?7, ?8, i.barcode, r.title
        FROM physical_items i JOIN bibliographic_records r ON r.id = i.record_id WHERE i.id = ?1",
        params![item_id, source, source_shelf, location, shelf, label(source_name, source_code),
            label(destination, destination_shelf), actor.user_id])?;
    let id = tx.last_insert_rowid();
    history(
        &tx,
        actor,
        item_id,
        "transfer_sent",
        "available",
        "in_transit",
    )?;
    audit::record(
        &tx,
        Some(actor),
        "inventory.transfer_sent",
        "stock_transfer",
        Some(id),
        "Sent a copy transfer",
    )?;
    tx.commit()?;
    Ok(id)
}

pub fn finish(conn: &mut Connection, actor: &Session, id: i64, receive: bool) -> AppResult<()> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let (item, destination, destination_shelf): (i64, i64, Option<i64>) = tx.query_row(
        "SELECT item_id, destination_location_id, destination_shelf_id FROM stock_transfers WHERE id = ?1 AND status = 'in_transit'",
        [id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    ).optional()?.ok_or_else(|| AppError::Conflict("This transfer is not open.".into()))?;
    if receive {
        inventory::active_place(&tx, destination, destination_shelf)?;
    }
    let (status, action, event, summary) = if receive {
        (
            "received",
            "inventory.transfer_received",
            "transfer_received",
            "Received a copy transfer",
        )
    } else {
        (
            "cancelled",
            "inventory.transfer_cancelled",
            "transfer_cancelled",
            "Cancelled a copy transfer",
        )
    };
    tx.execute("UPDATE stock_transfers SET status = ?2, finished_at = strftime('%Y-%m-%dT%H:%M:%SZ','now'),
        finished_by = ?3 WHERE id = ?1", params![id, status, actor.user_id])?;
    tx.execute("UPDATE physical_items SET status = 'available',
        location_id = CASE WHEN ?2 THEN ?3 ELSE location_id END,
        shelf_id = CASE WHEN ?2 THEN ?4 ELSE shelf_id END, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now')
        WHERE id = ?1", params![item, receive, destination, destination_shelf])?;
    history(&tx, actor, item, event, "in_transit", "available")?;
    reservations::refresh_copy(&tx, item)?;
    audit::record(
        &tx,
        Some(actor),
        action,
        "stock_transfer",
        Some(id),
        summary,
    )?;
    tx.commit()?;
    Ok(())
}
