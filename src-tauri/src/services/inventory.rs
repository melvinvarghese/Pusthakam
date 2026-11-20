use crate::domain::validate;
use crate::error::{AppError, AppResult};
use crate::services::{audit, auth::Session};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};

pub(super) const PAGE_SIZE: i64 = 25;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StockCheck {
    id: i64,
    location: String,
    shelf: Option<String>,
    started_at: String,
    closed_at: Option<String>,
    expected: i64,
    matched: i64,
    unexpected: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Page<T> {
    pub(super) rows: Vec<T>,
    pub(super) total: i64,
    pub(super) page: i64,
    pub(super) page_size: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    barcode: String,
    accession_no: Option<String>,
    title: Option<String>,
}

#[derive(Serialize)]
pub struct Review {
    check: StockCheck,
    entries: Page<Entry>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum View {
    Matched,
    Missing,
    Unexpected,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanOutcome {
    Matched,
    Unexpected,
    Duplicate,
}

const CHECK_SELECT: &str = "SELECT c.id, c.location_name, c.shelf_code, c.started_at, c.closed_at,
    (SELECT count(*) FROM stock_check_expected e WHERE e.check_id = c.id),
    (SELECT count(*) FROM stock_check_scans s WHERE s.check_id = c.id AND matched = 1),
    (SELECT count(*) FROM stock_check_scans s WHERE s.check_id = c.id AND matched = 0)
    FROM stock_checks c";

fn read_check(row: &rusqlite::Row<'_>) -> rusqlite::Result<StockCheck> {
    Ok(StockCheck {
        id: row.get(0)?,
        location: row.get(1)?,
        shelf: row.get(2)?,
        started_at: row.get(3)?,
        closed_at: row.get(4)?,
        expected: row.get(5)?,
        matched: row.get(6)?,
        unexpected: row.get(7)?,
    })
}

pub(super) fn page_number(page: i64, total: i64) -> i64 {
    page.clamp(1, ((total + PAGE_SIZE - 1) / PAGE_SIZE).max(1))
}

pub fn list(conn: &Connection, page: i64) -> AppResult<Page<StockCheck>> {
    let total = conn.query_row("SELECT count(*) FROM stock_checks", [], |row| row.get(0))?;
    let page = page_number(page, total);
    let mut stmt = conn.prepare(&format!(
        "{CHECK_SELECT} ORDER BY c.id DESC LIMIT ?1 OFFSET ?2"
    ))?;
    let rows = stmt
        .query_map(params![PAGE_SIZE, (page - 1) * PAGE_SIZE], read_check)?
        .collect::<Result<_, _>>()?;
    Ok(Page {
        rows,
        total,
        page,
        page_size: PAGE_SIZE,
    })
}

fn check_exists(conn: &Connection, id: i64) -> AppResult<StockCheck> {
    conn.query_row(&format!("{CHECK_SELECT} WHERE c.id = ?1"), [id], read_check)
        .optional()?
        .ok_or_else(|| AppError::NotFound("Stock check not found.".into()))
}

pub fn review(conn: &Connection, id: i64, view: View, page: i64) -> AppResult<Review> {
    let check = check_exists(conn, id)?;
    let (query, total) = match view {
        View::Matched => ("SELECT barcode, accession_no, title FROM stock_check_scans WHERE check_id = ?1 AND matched = 1", check.matched),
        View::Unexpected => ("SELECT barcode, accession_no, title FROM stock_check_scans WHERE check_id = ?1 AND matched = 0", check.unexpected),
        View::Missing => ("SELECT e.barcode, e.accession_no, e.title FROM stock_check_expected e
            WHERE e.check_id = ?1 AND NOT EXISTS (SELECT 1 FROM stock_check_scans s
                WHERE s.check_id = e.check_id AND s.barcode = e.barcode AND s.matched = 1)", check.expected - check.matched),
    };
    let page = page_number(page, total);
    let mut stmt = conn.prepare(&format!(
        "{query} ORDER BY barcode COLLATE NOCASE LIMIT ?2 OFFSET ?3"
    ))?;
    let rows = stmt
        .query_map(params![id, PAGE_SIZE, (page - 1) * PAGE_SIZE], |row| {
            Ok(Entry {
                barcode: row.get(0)?,
                accession_no: row.get(1)?,
                title: row.get(2)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(Review {
        check,
        entries: Page {
            rows,
            total,
            page,
            page_size: PAGE_SIZE,
        },
    })
}

pub(super) fn active_place(
    conn: &Connection,
    location_id: i64,
    shelf_id: Option<i64>,
) -> AppResult<(String, Option<String>)> {
    let location: String = conn
        .query_row(
            "SELECT name FROM locations WHERE id = ?1 AND archived_at IS NULL",
            [location_id],
            |row| row.get(0),
        )
        .optional()?
        .ok_or_else(|| AppError::Validation("Choose an active collection location.".into()))?;
    let shelf: Option<String> = shelf_id.map(|id| {
        conn.query_row("SELECT code FROM shelves WHERE id = ?1 AND location_id = ?2 AND archived_at IS NULL",
            params![id, location_id], |row| row.get(0)).optional()?
            .ok_or_else(|| AppError::Validation("Choose an active shelf at that location.".into()))
    }).transpose()?;
    Ok((location, shelf))
}

pub fn start(
    conn: &mut Connection,
    actor: &Session,
    location_id: i64,
    shelf_id: Option<i64>,
) -> AppResult<i64> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let (location, shelf) = active_place(&tx, location_id, shelf_id)?;
    tx.execute(
        "INSERT INTO stock_checks (location_id, shelf_id, location_name, shelf_code, started_by)
        VALUES (?1, ?2, ?3, ?4, ?5)",
        params![location_id, shelf_id, location, shelf, actor.user_id],
    )?;
    let id = tx.last_insert_rowid();
    tx.execute(
        "INSERT INTO stock_check_expected (check_id, item_id, barcode, accession_no, title)
        SELECT ?1, i.id, i.barcode, i.accession_no, r.title FROM physical_items i
        JOIN bibliographic_records r ON r.id = i.record_id
        WHERE i.location_id = ?2 AND (?3 IS NULL OR i.shelf_id = ?3)
          AND r.archived_at IS NULL AND i.status = 'available'",
        params![id, location_id, shelf_id],
    )?;
    audit::record(
        &tx,
        Some(actor),
        "inventory.started",
        "stock_check",
        Some(id),
        "Started a stock check",
    )?;
    tx.commit()?;
    Ok(id)
}

pub fn scan(
    conn: &mut Connection,
    actor: &Session,
    id: i64,
    barcode: &str,
) -> AppResult<ScanOutcome> {
    let barcode = validate::required("Barcode", barcode, 120)?;
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let closed = tx
        .query_row(
            "SELECT closed_at IS NOT NULL FROM stock_checks WHERE id = ?1",
            [id],
            |row| row.get::<_, bool>(0),
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound("Stock check not found.".into()))?;
    if closed {
        return Err(AppError::Conflict("This stock check is closed.".into()));
    }
    let expected: Option<(String, String)> = tx.query_row(
        "SELECT accession_no, title FROM stock_check_expected WHERE check_id = ?1 AND barcode = ?2",
        params![id, barcode], |row| Ok((row.get(0)?, row.get(1)?)),
    ).optional()?;
    let matched = expected.is_some();
    let item = match expected {
        Some(entry) => Some(entry),
        None => tx.query_row(
            "SELECT i.accession_no, r.title FROM physical_items i JOIN bibliographic_records r ON r.id = i.record_id WHERE i.barcode = ?1",
            [&barcode], |row| Ok((row.get(0)?, row.get(1)?)),
        ).optional()?,
    };
    let (accession, title) = item
        .map(|(accession, title)| (Some(accession), Some(title)))
        .unwrap_or_default();
    let inserted = tx.execute("INSERT INTO stock_check_scans (check_id, barcode, accession_no, title, matched, scanned_by)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6) ON CONFLICT(check_id, barcode) DO NOTHING",
        params![id, barcode, accession, title, matched, actor.user_id])?;
    tx.commit()?;
    Ok(if inserted == 0 {
        ScanOutcome::Duplicate
    } else if matched {
        ScanOutcome::Matched
    } else {
        ScanOutcome::Unexpected
    })
}

pub fn close(conn: &mut Connection, actor: &Session, id: i64) -> AppResult<()> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if tx.execute(
        "UPDATE stock_checks SET closed_at = strftime('%Y-%m-%dT%H:%M:%SZ','now'), closed_by = ?2
        WHERE id = ?1 AND closed_at IS NULL",
        params![id, actor.user_id],
    )? == 0
    {
        return Err(AppError::Conflict(
            "This stock check is already closed or no longer exists.".into(),
        ));
    }
    audit::record(
        &tx,
        Some(actor),
        "inventory.closed",
        "stock_check",
        Some(id),
        "Closed a stock check",
    )?;
    tx.commit()?;
    Ok(())
}
