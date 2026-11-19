use super::*;
use rusqlite::OptionalExtension;

pub(super) fn prepare(
    conn: &Connection,
    b: &Bundle,
    seen: &mut HashSet<(String, String)>,
) -> AppResult<Vec<(Reservation, i64, i64)>> {
    let now: String = conn.query_row("SELECT strftime('%Y-%m-%dT%H:%M:%SZ','now')", [], |r| {
        r.get(0)
    })?;
    let mut result = vec![];
    let mut groups: BTreeMap<i64, Vec<Reservation>> = BTreeMap::new();
    let mut pairs = HashSet::new();
    for r in &b.reservations {
        unique(conn, &b.source_system, "reservation", &r.source_id, seen)?;
        exact("source staff", &r.source_staff, 200)?;
        exact("member number", &r.member_number, 64)?;
        exact("barcode", &r.barcode, 128)?;
        loan_import::timestamp(&r.placed_at)?;
        if r.placed_at > now {
            return Err(invalid("Reservation placement is in the future."));
        }
        for date in [&r.closed_at, &r.expires_at].into_iter().flatten() {
            loan_import::timestamp(date)?;
            if date < &r.placed_at {
                return Err(invalid("Reservation closure/expiry precedes placement."));
            }
        }
        let active = matches!(r.status.as_str(), "pending" | "ready");
        if !matches!(
            r.status.as_str(),
            "pending" | "ready" | "fulfilled" | "cancelled" | "expired"
        ) || active != r.closed_at.is_none()
        {
            return Err(invalid(
                "Reservation status and closure timestamp disagree.",
            ));
        }
        if r.closed_at.as_ref().is_some_and(|v| v > &now) {
            return Err(invalid("Historical reservation closure is in the future."));
        }
        if active != r.queue_position.is_some() || r.queue_position.is_some_and(|v| v < 1) {
            return Err(invalid(
                "Active reservations need a positive queuePosition; closed history needs null.",
            ));
        }
        if r.status == "ready" && !r.expires_at.as_ref().is_some_and(|v| v > &now) {
            return Err(invalid(
                "A ready reservation requires a future expiry timestamp.",
            ));
        }
        let member: Option<i64> = conn
            .query_row(
                "SELECT id FROM members WHERE member_number=?1",
                [&r.member_number],
                |row| row.get(0),
            )
            .optional()?;
        let member = member.ok_or_else(|| invalid("Reservation member number is not imported."))?;
        let item: Option<i64> = conn
            .query_row(
                "SELECT id FROM physical_items WHERE barcode=?1",
                [&r.barcode],
                |row| row.get(0),
            )
            .optional()?;
        let item=item.ok_or_else(||invalid("Reservation barcode is not imported. Title-level requests need explicit copy mapping."))?;
        if active {
            let eligible:bool=conn.query_row("SELECT EXISTS(SELECT 1 FROM members m JOIN member_categories mc ON mc.id=m.category_id JOIN physical_items i ON i.id=?2 JOIN item_categories ic ON ic.id=i.item_category_id JOIN bibliographic_records br ON br.id=i.record_id JOIN circulation_policies p ON p.member_category_id=mc.id AND p.item_category_id=ic.id WHERE m.id=?1 AND m.archived_at IS NULL AND m.expires_on>=date('now','localtime') AND mc.archived_at IS NULL AND ic.archived_at IS NULL AND br.archived_at IS NULL)",params![member,item],|row|row.get(0))?;
            let conflicts:bool=conn.query_row("SELECT EXISTS(SELECT 1 FROM reservations WHERE item_id=?1 AND closed_at IS NULL) OR EXISTS(SELECT 1 FROM loans WHERE item_id=?1 AND member_id=?2 AND returned_at IS NULL)",params![item,member],|row|row.get(0))?;
            if !eligible || conflicts || !pairs.insert((item, member)) {
                return Err(invalid("Active reservation conflicts with an existing queue/borrower or an ineligible/duplicate member. Existing queues are never merged."));
            }
            groups.entry(item).or_default().push(r.clone());
        }
        result.push((r.clone(), item, member));
    }
    for (item, mut group) in groups {
        group.sort_by_key(|r| r.queue_position);
        for (index, r) in group.iter().enumerate() {
            if r.queue_position != Some(index as i64 + 1) || (r.status == "ready" && index != 0) {
                return Err(invalid("Queue positions must be unique and contiguous from 1; a ready reservation must be first."));
            }
        }
        let status: String = conn.query_row(
            "SELECT status FROM physical_items WHERE id=?1",
            [item],
            |r| r.get(0),
        )?;
        let ready = group[0].status == "ready";
        if (ready && status != "available") || (!ready && status != "on_loan") {
            return Err(invalid("A ready queue requires an available copy; a wholly pending queue requires an on-loan copy. Source statuses are not silently promoted."));
        }
        if !ready {
            let open: bool = conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM loans WHERE item_id=?1 AND returned_at IS NULL)",
                [item],
                |r| r.get(0),
            )?;
            if !open {
                return Err(invalid("Pending reservation copy has no open loan."));
            }
        }
    }
    result.sort_by_key(|(r, item, _)| (*item, r.queue_position.unwrap_or(0)));
    Ok(result)
}

pub(super) fn insert(
    conn: &Connection,
    actor: &Session,
    batch: i64,
    source: &str,
    rows: &[(Reservation, i64, i64)],
) -> AppResult<()> {
    for (r, item, member) in rows {
        conn.execute("INSERT INTO reservations(item_id,member_id,placed_by,placed_at,status,expires_at,closed_at) VALUES(?1,?2,?3,?4,?5,?6,?7)",params![item,member,actor.user_id,r.placed_at,r.status,r.expires_at,r.closed_at])?;
        provenance(
            conn,
            batch,
            source,
            "reservation",
            &r.source_id,
            conn.last_insert_rowid(),
            &r.source_staff,
            r,
        )?;
        if r.status == "ready" {
            conn.execute("UPDATE physical_items SET status='reserved_hold',updated_at=strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE id=?1",[item])?;
            conn.execute("INSERT INTO item_events(item_id,event_type,from_status,to_status,note,staff_user_id) VALUES(?1,'reservation_import','available','reserved_hold','Imported ready reservation; source queue retained.',?2)",params![item,actor.user_id])?;
        }
    }
    Ok(())
}
