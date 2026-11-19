use super::*;
use rusqlite::OptionalExtension;

struct State {
    amount: i64,
    currency: String,
    assessed: String,
    paid: i64,
    waived: i64,
}
type Prepared = (Vec<(Fine, i64)>, Vec<(Settlement, bool)>, Vec<Balance>);

pub(super) fn prepare(
    conn: &Connection,
    b: &Bundle,
    seen: &mut HashSet<(String, String)>,
) -> AppResult<Prepared> {
    let mut states: BTreeMap<String, State> = BTreeMap::new();
    let mut fines = vec![];
    let mut loans = HashSet::new();
    let now: String = conn.query_row("SELECT strftime('%Y-%m-%dT%H:%M:%SZ','now')", [], |r| {
        r.get(0)
    })?;
    for f in &b.fines {
        unique(conn, &b.source_system, "fine", &f.source_id, seen)?;
        exact("source loan ID", &f.loan_source_id, 128)?;
        exact("source staff", &f.source_staff, 200)?;
        money(f.amount_minor)?;
        currency(&f.currency)?;
        loan_import::timestamp(&f.assessed_at)?;
        let loan:Option<(i64,String,Option<String>)>=conn.query_row("SELECT l.id,l.issued_at,l.returned_at FROM loans l JOIN loan_import_sources s ON s.loan_id=l.id WHERE s.source_system=?1 AND s.source_id=?2",params![b.source_system,f.loan_source_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional()?;
        let (loan, issued, returned) = loan.ok_or_else(|| {
            invalid("Fine refers to a loan not imported under this source system.")
        })?;
        if returned.is_none() {
            return Err(invalid("This format supports assessed fines on closed imported loans only. Open-loan charges need a separate mapping; they are not discarded or converted."));
        }
        if f.assessed_at < issued || f.assessed_at > now {
            return Err(invalid(
                "Fine assessment precedes issue or is in the future.",
            ));
        }
        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM fines WHERE loan_id=?1)",
            [loan],
            |r| r.get(0),
        )?;
        if exists || !loans.insert(loan) {
            return Err(invalid("Only one assessed fine per loan is supported; existing/multiple charges require explicit mapping."));
        }
        states.insert(
            f.source_id.clone(),
            State {
                amount: f.amount_minor,
                currency: f.currency.clone(),
                assessed: f.assessed_at.clone(),
                paid: 0,
                waived: 0,
            },
        );
        fines.push((f.clone(), loan));
    }
    let mut settlements = vec![];
    for (rows, payment) in [(&b.payments, true), (&b.waivers, false)] {
        for s in rows {
            unique(conn, &b.source_system, "settlement", &s.source_id, seen)?;
            exact("source fine ID", &s.fine_source_id, 128)?;
            exact("source staff", &s.source_staff, 200)?;
            money(s.amount_minor)?;
            currency(&s.currency)?;
            loan_import::timestamp(&s.recorded_at)?;
            if payment {
                if !s
                    .method
                    .as_deref()
                    .is_some_and(|v| matches!(v, "cash" | "card" | "bank_transfer" | "other"))
                    || s.reason.is_some()
                {
                    return Err(invalid(
                        "Payments require a supported method and no waiver reason.",
                    ));
                }
            } else {
                if s.method.is_some() {
                    return Err(invalid("Waivers must not contain a payment method."));
                }
                exact("waiver reason", s.reason.as_deref().unwrap_or(""), 500)?;
            }
            if !states.contains_key(&s.fine_source_id) {
                let existing:Option<(i64,i64,String,String)>=conn.query_row("SELECT f.id,f.amount_minor,f.currency,f.assessed_at FROM fines f JOIN history_import_sources p ON p.kind='fine' AND p.local_id=f.id WHERE p.source_system=?1 AND p.source_id=?2",params![b.source_system,s.fine_source_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional()?;
                let (id,amount,currency,assessed)=existing.ok_or_else(||invalid("Settlement fine is neither in this bundle nor previously imported under this source."))?;
                let (paid,waived):(i64,i64)=conn.query_row("SELECT coalesce(sum(amount_minor) FILTER(WHERE kind='payment'),0),coalesce(sum(amount_minor) FILTER(WHERE kind='waiver'),0) FROM fine_settlements WHERE fine_id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?)))?;
                states.insert(
                    s.fine_source_id.clone(),
                    State {
                        amount,
                        currency,
                        assessed,
                        paid,
                        waived,
                    },
                );
            }
            let state = states
                .get_mut(&s.fine_source_id)
                .ok_or_else(|| AppError::Internal("Missing fine reconciliation state.".into()))?;
            if state.currency != s.currency || s.recorded_at < state.assessed || s.recorded_at > now
            {
                return Err(invalid(
                    "Settlement currency or recorded date does not match its assessed fine.",
                ));
            }
            if payment {
                state.paid = add(state.paid, s.amount_minor)?;
            } else {
                state.waived = add(state.waived, s.amount_minor)?;
            }
            if add(state.paid, state.waived)? > state.amount {
                return Err(invalid("Payments and waivers exceed the assessed fine. Credits/refunds need a separate mapping."));
            }
            settlements.push((s.clone(), payment));
        }
    }
    let mut currencies: BTreeMap<String, Balance> = BTreeMap::new();
    for state in states.values() {
        let total = currencies.entry(state.currency.clone()).or_insert(Balance {
            currency: state.currency.clone(),
            assessed_minor: 0,
            paid_minor: 0,
            waived_minor: 0,
            outstanding_minor: 0,
        });
        total.assessed_minor = add(total.assessed_minor, state.amount)?;
        total.paid_minor = add(total.paid_minor, state.paid)?;
        total.waived_minor = add(total.waived_minor, state.waived)?;
        total.outstanding_minor = add(
            total.outstanding_minor,
            state.amount - add(state.paid, state.waived)?,
        )?;
    }
    settlements.sort_by(|a, b| {
        a.0.recorded_at
            .cmp(&b.0.recorded_at)
            .then_with(|| a.0.source_id.cmp(&b.0.source_id))
    });
    Ok((fines, settlements, currencies.into_values().collect()))
}
pub(super) fn insert(
    conn: &Connection,
    actor: &Session,
    batch: i64,
    source: &str,
    fines: &[(Fine, i64)],
    settlements: &[(Settlement, bool)],
) -> AppResult<()> {
    for (f, loan) in fines {
        conn.execute(
            "INSERT INTO fines(loan_id,amount_minor,currency,assessed_at) VALUES(?1,?2,?3,?4)",
            params![loan, f.amount_minor, f.currency, f.assessed_at],
        )?;
        provenance(
            conn,
            batch,
            source,
            "fine",
            &f.source_id,
            conn.last_insert_rowid(),
            &f.source_staff,
            f,
        )?;
    }
    for (s, payment) in settlements {
        let id:i64=conn.query_row("SELECT local_id FROM history_import_sources WHERE source_system=?1 AND kind='fine' AND source_id=?2",params![source,s.fine_source_id],|r|r.get(0))?;
        conn.execute("INSERT INTO fine_settlements(fine_id,staff_user_id,kind,amount_minor,method,reason,recorded_at) VALUES(?1,?2,?3,?4,?5,?6,?7)",params![id,actor.user_id,if *payment{"payment"}else{"waiver"},s.amount_minor,s.method,s.reason,s.recorded_at])?;
        provenance(
            conn,
            batch,
            source,
            "settlement",
            &s.source_id,
            conn.last_insert_rowid(),
            &s.source_staff,
            s,
        )?;
    }
    Ok(())
}

pub(super) fn reconcile(conn: &Connection, b: &Bundle, batch: i64) -> AppResult<Summary> {
    let count = |kind: &str| -> AppResult<usize> {
        Ok(conn.query_row(
            "SELECT count(*) FROM history_import_sources WHERE batch_id=?1 AND kind=?2",
            params![batch, kind],
            |r| r.get::<_, i64>(0),
        )? as usize)
    };
    let settlement_count = |kind: &str| -> AppResult<usize> {
        Ok(conn.query_row("SELECT count(*) FROM history_import_sources p JOIN fine_settlements s ON s.id=p.local_id WHERE p.batch_id=?1 AND p.kind='settlement' AND s.kind=?2",params![batch,kind],|r|r.get::<_,i64>(0))? as usize)
    };
    let mut ids = HashSet::new();
    for f in &b.fines {
        ids.insert(f.source_id.as_str());
    }
    for s in b.payments.iter().chain(&b.waivers) {
        ids.insert(s.fine_source_id.as_str());
    }
    let mut totals: BTreeMap<String, Balance> = BTreeMap::new();
    for id in ids {
        let (local,amount,currency):(i64,i64,String)=conn.query_row("SELECT f.id,f.amount_minor,f.currency FROM history_import_sources p JOIN fines f ON f.id=p.local_id WHERE p.source_system=?1 AND p.kind='fine' AND p.source_id=?2",params![b.source_system,id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?;
        let (paid,waived):(i64,i64)=conn.query_row("SELECT coalesce(sum(amount_minor) FILTER(WHERE kind='payment'),0),coalesce(sum(amount_minor) FILTER(WHERE kind='waiver'),0) FROM fine_settlements WHERE fine_id=?1",[local],|r|Ok((r.get(0)?,r.get(1)?)))?;
        if add(paid, waived)? > amount {
            return Err(invalid("Saved fine balance is overallocated."));
        }
        let total = totals.entry(currency.clone()).or_insert(Balance {
            currency,
            assessed_minor: 0,
            paid_minor: 0,
            waived_minor: 0,
            outstanding_minor: 0,
        });
        total.assessed_minor = add(total.assessed_minor, amount)?;
        total.paid_minor = add(total.paid_minor, paid)?;
        total.waived_minor = add(total.waived_minor, waived)?;
        total.outstanding_minor = add(total.outstanding_minor, amount - add(paid, waived)?)?;
    }
    Ok(Summary {
        counts: Counts {
            reservations: count("reservation")?,
            fines: count("fine")?,
            payments: settlement_count("payment")?,
            waivers: settlement_count("waiver")?,
        },
        balances: totals.into_values().collect(),
    })
}
