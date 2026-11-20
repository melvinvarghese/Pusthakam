use crate::error::AppResult;
use crate::services::{auth::Session, reservations};
use rusqlite::Connection;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CatalogueCounts {
    titles: i64,
    copies: i64,
    available: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CirculationCounts {
    on_loan: i64,
    due_today: i64,
    overdue: i64,
    ready_holds: i64,
    outstanding_fines: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Overview {
    catalogue: Option<CatalogueCounts>,
    active_members: Option<i64>,
    circulation: Option<CirculationCounts>,
}

pub fn overview(conn: &mut Connection, session: &Session) -> AppResult<Overview> {
    let can_read_catalogue = session.can("catalogue.read");
    let can_read_circulation = session.can("circulation.issue");
    if can_read_catalogue || can_read_circulation {
        reservations::refresh(conn)?;
    }
    let catalogue = if can_read_catalogue {
        Some(conn.query_row(
            "SELECT (SELECT count(*) FROM bibliographic_records WHERE archived_at IS NULL),
                    count(*), count(*) FILTER (WHERE i.status = 'available')
             FROM physical_items i JOIN bibliographic_records r ON r.id = i.record_id
             WHERE r.archived_at IS NULL AND i.status <> 'withdrawn'",
            [],
            |r| {
                Ok(CatalogueCounts {
                    titles: r.get(0)?,
                    copies: r.get(1)?,
                    available: r.get(2)?,
                })
            },
        )?)
    } else {
        None
    };
    let active_members = if session.can("members.read") {
        Some(conn.query_row(
            "SELECT count(*) FROM members m JOIN member_categories c ON c.id = m.category_id
             WHERE m.archived_at IS NULL AND c.archived_at IS NULL AND m.expires_on >= date('now','localtime')",
            [], |r| r.get(0),
        )?)
    } else {
        None
    };
    let circulation = if can_read_circulation {
        Some(conn.query_row(
            "SELECT count(*), count(*) FILTER (WHERE due_on = date('now','localtime')),
                    count(*) FILTER (WHERE due_on < date('now','localtime')),
                    (SELECT count(*) FROM reservations WHERE status = 'ready'),
                    (SELECT count(*) FROM fines f WHERE f.amount_minor > COALESCE(
                        (SELECT sum(s.amount_minor) FROM fine_settlements s WHERE s.fine_id = f.id), 0
                    ))
             FROM loans WHERE returned_at IS NULL",
            [], |r| Ok(CirculationCounts {
                on_loan: r.get(0)?, due_today: r.get(1)?, overdue: r.get(2)?,
                ready_holds: r.get(3)?, outstanding_fines: r.get(4)?,
            }),
        )?)
    } else {
        None
    };
    Ok(Overview {
        catalogue,
        active_members,
        circulation,
    })
}
