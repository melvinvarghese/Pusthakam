//! Library closure configuration and due-date adjustment, shared by issue and renewal.
use crate::{
    domain::validate,
    error::{AppError, AppResult},
    services::{audit, auth::Session},
};
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Holiday {
    pub id: i64,
    pub date: String,
    pub name: String,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Calendar {
    pub closed_weekdays: Vec<i64>,
    pub holidays: Vec<Holiday>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HolidayInput {
    pub id: Option<i64>,
    pub date: String,
    pub name: String,
}

pub fn load(conn: &Connection) -> AppResult<Calendar> {
    let closed_weekdays = conn
        .prepare("SELECT weekday FROM closed_weekdays ORDER BY weekday")?
        .query_map([], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    let holidays = conn
        .prepare("SELECT id, date, name FROM holidays ORDER BY date, id")?
        .query_map([], |r| {
            Ok(Holiday {
                id: r.get(0)?,
                date: r.get(1)?,
                name: r.get(2)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(Calendar {
        closed_weekdays,
        holidays,
    })
}

pub fn save_weekdays(conn: &mut Connection, actor: &Session, days: &[i64]) -> AppResult<()> {
    let mut days = days.to_vec();
    days.sort_unstable();
    if days.len() > 6
        || days.iter().any(|d| !(0..=6).contains(d))
        || days.windows(2).any(|d| d[0] == d[1])
    {
        return Err(AppError::Validation(
            "Choose unique weekdays and leave at least one day open.".into(),
        ));
    }
    let tx = conn.transaction()?;
    let previous = load(&tx)?.closed_weekdays;
    tx.execute("DELETE FROM closed_weekdays", [])?;
    for day in &days {
        tx.execute("INSERT INTO closed_weekdays (weekday) VALUES (?1)", [day])?;
    }
    audit::record(
        &tx,
        Some(actor),
        "calendar.weekdays",
        "closed_weekdays",
        None,
        &format!("Changed weekly closed days from {previous:?} to {days:?} (Sunday=0)"),
    )?;
    tx.commit()?;
    Ok(())
}

pub fn save_holiday(
    conn: &mut Connection,
    actor: &Session,
    input: &HolidayInput,
) -> AppResult<i64> {
    let date = validate::date("Holiday date", &input.date)?;
    let name = validate::required("Holiday name", &input.name, 120)?;
    let tx = conn.transaction()?;
    if tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM holidays WHERE date=?1 AND (?2 IS NULL OR id<>?2))",
        (&date, input.id),
        |r| r.get::<_, bool>(0),
    )? {
        return Err(AppError::Conflict(
            "That date is already in the library calendar. Edit its existing entry.".into(),
        ));
    }
    let summary = if let Some(id) = input.id {
        let old: (String, String) = tx
            .query_row("SELECT date, name FROM holidays WHERE id=?1", [id], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
            .optional()?
            .ok_or_else(|| AppError::NotFound("That holiday no longer exists.".into()))?;
        tx.execute(
            "UPDATE holidays SET date=?2, name=?3 WHERE id=?1",
            (id, &date, &name),
        )?;
        format!("Changed holiday {} ({}) to {date} ({name})", old.0, old.1)
    } else {
        let count: i64 = tx.query_row("SELECT count(*) FROM holidays", [], |r| r.get(0))?;
        if count >= 2000 {
            return Err(AppError::Validation(
                "Remove older holiday entries before adding more.".into(),
            ));
        }
        tx.execute(
            "INSERT INTO holidays (date, name) VALUES (?1, ?2)",
            (&date, &name),
        )?;
        format!("Added holiday {date} ({name})")
    };
    let id = input.id.unwrap_or_else(|| tx.last_insert_rowid());
    audit::record(
        &tx,
        Some(actor),
        "calendar.holiday.save",
        "holidays",
        Some(id),
        &summary,
    )?;
    tx.commit()?;
    Ok(id)
}

pub fn remove_holiday(conn: &mut Connection, actor: &Session, id: i64) -> AppResult<()> {
    let tx = conn.transaction()?;
    let old: (String, String) = tx
        .query_row("SELECT date, name FROM holidays WHERE id=?1", [id], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .optional()?
        .ok_or_else(|| AppError::NotFound("That holiday no longer exists.".into()))?;
    tx.execute("DELETE FROM holidays WHERE id=?1", [id])?;
    audit::record(
        &tx,
        Some(actor),
        "calendar.holiday.remove",
        "holidays",
        Some(id),
        &format!("Removed holiday {} ({})", old.0, old.1),
    )?;
    tx.commit()?;
    Ok(())
}

pub fn next_open_date(conn: &Connection, start: &str) -> AppResult<String> {
    let mut date = validate::date("Due date", start)?;
    let mut query = conn.prepare("SELECT EXISTS(SELECT 1 FROM holidays WHERE date=?1) OR EXISTS(SELECT 1 FROM closed_weekdays WHERE weekday=CAST(strftime('%w',?1) AS INTEGER)), date(?1,'+1 day')")?;
    // Bound bad/imported configurations instead of looping indefinitely over closed dates.
    for _ in 0..3660 {
        let (closed, next): (bool, Option<String>) =
            query.query_row([&date], |r| Ok((r.get(0)?, r.get(1)?)))?;
        if !closed {
            return Ok(date);
        }
        date = next.ok_or_else(|| {
            AppError::Validation("No valid open due date could be calculated.".into())
        })?;
    }
    Err(AppError::Validation(
        "No open date was found in the library calendar. Review the closed days and holidays."
            .into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{db, services::auth};
    #[test]
    fn calendar_reopens_and_skips_weekends_holidays_year_and_leap_boundaries() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("calendar.db");
        let mut conn = db::open(&path).unwrap();
        auth::create_user(&conn, "admin", "Admin", "test password", "administrator").unwrap();
        let actor = auth::login(&conn, "admin", "test password").unwrap();
        save_weekdays(&mut conn, &actor, &[0, 6]).unwrap();
        let id = save_holiday(
            &mut conn,
            &actor,
            &HolidayInput {
                id: None,
                date: "2027-01-01".into(),
                name: "New year".into(),
            },
        )
        .unwrap();
        assert_eq!(next_open_date(&conn, "2027-01-01").unwrap(), "2027-01-04");
        save_holiday(
            &mut conn,
            &actor,
            &HolidayInput {
                id: Some(id),
                date: "2028-02-29".into(),
                name: "Leap day".into(),
            },
        )
        .unwrap();
        assert_eq!(next_open_date(&conn, "2028-02-29").unwrap(), "2028-03-01");
        assert_eq!(next_open_date(&conn, "2026-12-31").unwrap(), "2026-12-31");
        drop(conn);
        let mut conn = db::open(&path).unwrap();
        assert_eq!(load(&conn).unwrap().closed_weekdays, vec![0, 6]);
        assert_eq!(load(&conn).unwrap().holidays[0].date, "2028-02-29");
        remove_holiday(&mut conn, &actor, id).unwrap();
        assert!(load(&conn).unwrap().holidays.is_empty());
    }
    #[test]
    fn invalid_changes_and_audit_failure_leave_calendar_intact() {
        let dir = tempfile::tempdir().unwrap();
        let mut conn = db::open(&dir.path().join("calendar.db")).unwrap();
        auth::create_user(&conn, "admin", "Admin", "test password", "administrator").unwrap();
        let actor = auth::login(&conn, "admin", "test password").unwrap();
        save_weekdays(&mut conn, &actor, &[0]).unwrap();
        for days in [vec![0, 1, 2, 3, 4, 5, 6], vec![7], vec![0, 0]] {
            assert!(save_weekdays(&mut conn, &actor, &days).is_err());
        }
        assert!(save_holiday(
            &mut conn,
            &actor,
            &HolidayInput {
                id: None,
                date: "2027-02-29".into(),
                name: "Bad".into()
            }
        )
        .is_err());
        conn.execute_batch("CREATE TRIGGER fail_calendar_audit BEFORE INSERT ON audit_events WHEN NEW.action LIKE 'calendar.%' BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();
        assert!(save_weekdays(&mut conn, &actor, &[6]).is_err());
        assert!(save_holiday(
            &mut conn,
            &actor,
            &HolidayInput {
                id: None,
                date: "2027-01-01".into(),
                name: "New year".into()
            }
        )
        .is_err());
        assert_eq!(load(&conn).unwrap().closed_weekdays, vec![0]);
        assert!(load(&conn).unwrap().holidays.is_empty());
    }
}
