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
