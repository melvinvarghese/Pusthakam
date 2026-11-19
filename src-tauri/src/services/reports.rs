use crate::domain::validate;
use crate::error::{AppError, AppResult};
use crate::services::reservations;
use rusqlite::{params, Connection};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CirculationDay {
    date: String,
    issued: i64,
    returned: i64,
    lost: i64,
    renewed: i64,
    reservations: i64,
}

fn date_range(conn: &Connection, from: &str, to: &str) -> AppResult<(String, String)> {
    let from = validate::date("Start date", from)?;
    let to = validate::date("End date", to)?;
    let span: i64 = conn.query_row("SELECT CAST(julianday(?2) - julianday(?1) AS INTEGER)",
        params![from, to], |r| r.get(0))?;
    if !(0..366).contains(&span) {
        return Err(AppError::Validation("Choose a date range of 1 to 366 days.".into()));
    }
    Ok((from, to))
}

pub fn circulation_activity(conn: &Connection, from: &str, to: &str) -> AppResult<Vec<CirculationDay>> {
    let (from, to) = date_range(conn, from, to)?;
    let mut stmt = conn.prepare(
        "WITH RECURSIVE days(day) AS (
            SELECT ?1 UNION ALL SELECT date(day,'+1 day') FROM days WHERE day < ?2
         ), events(day, kind) AS (
            SELECT date(issued_at,'localtime'), 'issue' FROM loans
            UNION ALL
            SELECT date(returned_at,'localtime'), CASE WHEN return_condition = 'lost' THEN 'lost' ELSE 'return' END
            FROM loans WHERE returned_at IS NOT NULL
            UNION ALL
            SELECT date(renewed_at,'localtime'), 'renew' FROM loan_renewals
            UNION ALL
            SELECT date(placed_at,'localtime'), 'reserve' FROM reservations
         ), totals AS (
            SELECT day, count(*) FILTER (WHERE kind = 'issue') AS issued,
                   count(*) FILTER (WHERE kind = 'return') AS returned, count(*) FILTER (WHERE kind = 'lost') AS lost,
                   count(*) FILTER (WHERE kind = 'renew') AS renewed, count(*) FILTER (WHERE kind = 'reserve') AS reservations
            FROM events WHERE day BETWEEN ?1 AND ?2 GROUP BY day
         )
         SELECT days.day, COALESCE(t.issued,0), COALESCE(t.returned,0), COALESCE(t.lost,0),
                COALESCE(t.renewed,0), COALESCE(t.reservations,0)
         FROM days LEFT JOIN totals t ON t.day = days.day ORDER BY days.day",
    )?;
    let rows = stmt.query_map(params![from, to], |r| Ok(CirculationDay {
        date: r.get(0)?, issued: r.get(1)?, returned: r.get(2)?, lost: r.get(3)?, renewed: r.get(4)?, reservations: r.get(5)?,
    }))?.collect::<Result<_, _>>()?;
    Ok(rows)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PopularTitle {
    record_id: i64,
    title: String,
    archived: bool,
    issues: i64,
}

pub fn popular_titles(conn: &Connection, from: &str, to: &str) -> AppResult<Vec<PopularTitle>> {
    let (from, to) = date_range(conn, from, to)?;
    let mut stmt = conn.prepare(
        "SELECT r.id, r.title, r.archived_at IS NOT NULL, count(*) AS issues
         FROM loans l JOIN physical_items i ON i.id = l.item_id
         JOIN bibliographic_records r ON r.id = i.record_id
         WHERE date(l.issued_at,'localtime') BETWEEN ?1 AND ?2
         GROUP BY r.id ORDER BY issues DESC, r.title COLLATE NOCASE, r.id LIMIT 25",
    )?;
    let rows = stmt.query_map(params![from, to], |r| Ok(PopularTitle {
        record_id: r.get(0)?, title: r.get(1)?, archived: r.get(2)?, issues: r.get(3)?,
    }))?.collect::<Result<_, _>>()?;
    Ok(rows)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StockRow {
    location_id: Option<i64>,
    location: Option<String>,
    archived_location: bool,
    available: i64,
    on_loan: i64,
    held: i64,
    lost: i64,
    damaged: i64,
    other: i64,
    total: i64,
}

pub fn stock_by_location(conn: &mut Connection) -> AppResult<Vec<StockRow>> {
    reservations::refresh(conn)?;
    let mut stmt = conn.prepare(
        "SELECT i.location_id, l.name, l.archived_at IS NOT NULL,
                count(*) FILTER (WHERE i.status = 'available'), count(*) FILTER (WHERE i.status = 'on_loan'),
                count(*) FILTER (WHERE i.status = 'reserved_hold'), count(*) FILTER (WHERE i.status = 'lost'),
                count(*) FILTER (WHERE i.status = 'damaged'),
                count(*) FILTER (WHERE i.status IN ('in_repair','in_transit')), count(*)
         FROM physical_items i JOIN bibliographic_records r ON r.id = i.record_id
         LEFT JOIN locations l ON l.id = i.location_id
         WHERE r.archived_at IS NULL AND i.status <> 'withdrawn'
         GROUP BY i.location_id ORDER BY i.location_id IS NULL, l.name COLLATE NOCASE, i.location_id",
    )?;
    let rows = stmt.query_map([], |r| Ok(StockRow {
        location_id: r.get(0)?, location: r.get(1)?, archived_location: r.get(2)?,
        available: r.get(3)?, on_loan: r.get(4)?, held: r.get(5)?, lost: r.get(6)?,
        damaged: r.get(7)?, other: r.get(8)?, total: r.get(9)?,
    }))?.collect::<Result<_, _>>()?;
    Ok(rows)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FineSummary {
    currency: String,
    assessed: String,
    paid: String,
    waived: String,
    outstanding: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberSummary {
    category_id: i64,
    category: String,
    archived_category: bool,
    current: i64,
    expired: i64,
    archived: i64,
    total: i64,
}

pub fn member_summary(conn: &Connection) -> AppResult<Vec<MemberSummary>> {
    let mut stmt = conn.prepare(
        "SELECT c.id, c.name, c.archived_at IS NOT NULL,
                count(*) FILTER (WHERE m.archived_at IS NULL AND m.expires_on >= date('now','localtime')),
                count(*) FILTER (WHERE m.archived_at IS NULL AND m.expires_on < date('now','localtime')),
                count(*) FILTER (WHERE m.archived_at IS NOT NULL), count(*)
         FROM members m JOIN member_categories c ON c.id = m.category_id
         GROUP BY c.id ORDER BY c.archived_at IS NOT NULL, c.name COLLATE NOCASE, c.id",
    )?;
    let rows = stmt.query_map([], |row| Ok(MemberSummary {
        category_id: row.get(0)?,
        category: row.get(1)?,
        archived_category: row.get(2)?,
        current: row.get(3)?,
        expired: row.get(4)?,
        archived: row.get(5)?,
        total: row.get(6)?,
    }))?.collect::<Result<_, _>>()?;
    Ok(rows)
}

pub fn fine_summary(conn: &Connection) -> AppResult<Vec<FineSummary>> {
    let mut stmt = conn.prepare(
        "WITH settlements AS (
            SELECT fine_id, sum(amount_minor) FILTER (WHERE kind = 'payment') AS paid,
                   sum(amount_minor) FILTER (WHERE kind = 'waiver') AS waived
            FROM fine_settlements GROUP BY fine_id
        )
        SELECT f.currency, sum(f.amount_minor), coalesce(sum(s.paid), 0), coalesce(sum(s.waived), 0)
        FROM fines f LEFT JOIN settlements s ON s.fine_id = f.id
        GROUP BY f.currency ORDER BY f.currency",
    )?;
    let rows = stmt.query_map([], |row| {
        let assessed: i64 = row.get(1)?;
        let paid: i64 = row.get(2)?;
        let waived: i64 = row.get(3)?;
        Ok(FineSummary {
            currency: row.get(0)?, assessed: assessed.to_string(), paid: paid.to_string(),
            waived: waived.to_string(), outstanding: (assessed - paid - waived).to_string(),
        })
    })?.collect::<Result<_, _>>()?;
    Ok(rows)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogueGrowthDay {
    date: String,
    titles_added: i64,
    copies_added: i64,
}

pub fn catalogue_growth(conn: &Connection, from: &str, to: &str) -> AppResult<Vec<CatalogueGrowthDay>> {
    let (from, to) = date_range(conn, from, to)?;
    let mut stmt = conn.prepare(
        "WITH RECURSIVE days(day) AS (
            SELECT ?1 UNION ALL SELECT date(day, '+1 day') FROM days WHERE day < ?2
         ), additions(day, kind) AS (
            SELECT date(created_at, 'localtime'), 'title' FROM bibliographic_records
            UNION ALL
            SELECT date(created_at, 'localtime'), 'copy' FROM physical_items
         ), totals AS (
            SELECT day, count(*) FILTER (WHERE kind = 'title') AS titles,
                        count(*) FILTER (WHERE kind = 'copy') AS copies
            FROM additions WHERE day BETWEEN ?1 AND ?2 GROUP BY day
         )
         SELECT days.day, coalesce(t.titles, 0), coalesce(t.copies, 0)
         FROM days LEFT JOIN totals t ON t.day = days.day ORDER BY days.day",
    )?;
    let rows = stmt.query_map(params![from, to], |row| Ok(CatalogueGrowthDay {
        date: row.get(0)?,
        titles_added: row.get(1)?,
        copies_added: row.get(2)?,
    }))?.collect::<Result<_, _>>()?;
    Ok(rows)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverdueRow {
    loan_id: i64,
    member_id: i64,
    member_name: String,
    member_barcode: String,
    item_id: i64,
    accession_no: String,
    barcode: String,
    title: String,
    due_date: String,
    days_overdue: i64,
}

pub fn overdues_list(conn: &Connection, from: &str, to: &str) -> AppResult<Vec<OverdueRow>> {
    let (from, to) = date_range(conn, from, to)?;
    let mut stmt = conn.prepare(
        "SELECT l.id, m.id, m.name, m.member_number, i.id, i.accession_no, i.barcode, r.title,
                l.due_on,
                CAST(julianday('now') - julianday(l.due_on) AS INTEGER)
         FROM loans l
         JOIN members m ON m.id = l.member_id
         JOIN physical_items i ON i.id = l.item_id
         JOIN bibliographic_records r ON r.id = i.record_id
         WHERE l.returned_at IS NULL
           AND date(l.due_on,'localtime') BETWEEN ?1 AND ?2
         ORDER BY l.due_on, r.title COLLATE NOCASE",
    )?;
    let rows = stmt.query_map(params![from, to], |r| Ok(OverdueRow {
        loan_id: r.get(0)?,
        member_id: r.get(1)?,
        member_name: r.get(2)?,
        member_barcode: r.get(3)?,
        item_id: r.get(4)?,
        accession_no: r.get(5)?,
        barcode: r.get(6)?,
        title: r.get(7)?,
        due_date: r.get(8)?,
        days_overdue: r.get(9)?,
    }))?.collect::<Result<_, _>>()?;
    Ok(rows)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessionRow {
    item_id: i64,
    accession_no: String,
    barcode: String,
    title: String,
    acquisition_date: Option<String>,
    acquisition_source: String,
    price_minor: Option<i64>,
    location: Option<String>,
    status: String,
}

pub fn accession_list(conn: &Connection, from: &str, to: &str) -> AppResult<Vec<AccessionRow>> {
    let (from, to) = date_range(conn, from, to)?;
    let mut stmt = conn.prepare(
        "SELECT i.id, i.accession_no, i.barcode, r.title,
                i.acquisition_date, i.acquisition_source, i.price_minor,
                l.name, i.status
         FROM physical_items i
         JOIN bibliographic_records r ON r.id = i.record_id
         LEFT JOIN locations l ON l.id = i.location_id
         WHERE COALESCE(i.acquisition_date, date(i.created_at,'localtime')) BETWEEN ?1 AND ?2
         ORDER BY COALESCE(i.acquisition_date, date(i.created_at,'localtime')), i.accession_no COLLATE NOCASE",
    )?;
    let rows = stmt.query_map(params![from, to], |r| Ok(AccessionRow {
        item_id: r.get(0)?,
        accession_no: r.get(1)?,
        barcode: r.get(2)?,
        title: r.get(3)?,
        acquisition_date: r.get(4)?,
        acquisition_source: r.get(5)?,
        price_minor: r.get(6)?,
        location: r.get(7)?,
        status: r.get(8)?,
    }))?.collect::<Result<_, _>>()?;
    Ok(rows)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpenditureRow {
    acquisition_source: String,
    copies: i64,
    total_minor: i64,
}

pub fn expenditure_summary(conn: &Connection, from: &str, to: &str) -> AppResult<Vec<ExpenditureRow>> {
    let (from, to) = date_range(conn, from, to)?;
    let mut stmt = conn.prepare(
        "SELECT i.acquisition_source, count(*) AS copies, coalesce(sum(i.price_minor), 0) AS total_minor
         FROM physical_items i
         WHERE COALESCE(i.acquisition_date, date(i.created_at,'localtime')) BETWEEN ?1 AND ?2
         GROUP BY i.acquisition_source
         ORDER BY total_minor DESC, copies DESC, i.acquisition_source",
    )?;
    let rows = stmt.query_map(params![from, to], |r| Ok(ExpenditureRow {
        acquisition_source: r.get(0)?,
        copies: r.get(1)?,
        total_minor: r.get(2)?,
    }))?.collect::<Result<_, _>>()?;
    Ok(rows)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LossDamageRow {
    loan_id: i64,
    return_condition: String,
    returned_date: String,
    member_id: i64,
    member_name: String,
    item_id: i64,
    accession_no: String,
    title: String,
}

pub fn loss_damage_list(conn: &Connection, from: &str, to: &str) -> AppResult<Vec<LossDamageRow>> {
    let (from, to) = date_range(conn, from, to)?;
    let mut stmt = conn.prepare(
        "SELECT l.id, l.return_condition, date(l.returned_at,'localtime'),
                m.id, m.name, i.id, i.accession_no, r.title
         FROM loans l
         JOIN members m ON m.id = l.member_id
         JOIN physical_items i ON i.id = l.item_id
         JOIN bibliographic_records r ON r.id = i.record_id
         WHERE l.return_condition IN ('lost','damaged')
           AND date(l.returned_at,'localtime') BETWEEN ?1 AND ?2
         ORDER BY date(l.returned_at,'localtime'), r.title COLLATE NOCASE",
    )?;
    let rows = stmt.query_map(params![from, to], |r| Ok(LossDamageRow {
        loan_id: r.get(0)?,
        return_condition: r.get(1)?,
        returned_date: r.get(2)?,
        member_id: r.get(3)?,
        member_name: r.get(4)?,
        item_id: r.get(5)?,
        accession_no: r.get(6)?,
        title: r.get(7)?,
    }))?.collect::<Result<_, _>>()?;
    Ok(rows)
}
