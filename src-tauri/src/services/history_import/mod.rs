//! Atomic neutral migration bundle: reservations and assessed financial history.
mod finance;
mod reservations;

use crate::domain::validate;
use crate::error::{AppError, AppResult};
use crate::services::{audit, auth::Session, loan_import};
use base64::{engine::general_purpose::STANDARD, Engine};
use rusqlite::{params, Connection, TransactionBehavior};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};

const MAX_BYTES: usize = 10 * 1024 * 1024;
const MAX_MONEY: i64 = 9_007_199_254_740_991;

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Reservation {
    source_id: String,
    member_number: String,
    barcode: String,
    placed_at: String,
    status: String,
    expires_at: Option<String>,
    closed_at: Option<String>,
    queue_position: Option<i64>,
    source_staff: String,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Fine {
    source_id: String,
    loan_source_id: String,
    amount_minor: i64,
    currency: String,
    assessed_at: String,
    source_staff: String,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Settlement {
    source_id: String,
    fine_source_id: String,
    amount_minor: i64,
    currency: String,
    recorded_at: String,
    method: Option<String>,
    reason: Option<String>,
    source_staff: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Counts {
    pub reservations: usize,
    pub fines: usize,
    pub payments: usize,
    pub waivers: usize,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Balance {
    pub currency: String,
    pub assessed_minor: i64,
    pub paid_minor: i64,
    pub waived_minor: i64,
    pub outstanding_minor: i64,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Summary {
    pub counts: Counts,
    pub balances: Vec<Balance>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Bundle {
    version: u32,
    source_system: String,
    reservations: Vec<Reservation>,
    fines: Vec<Fine>,
    payments: Vec<Settlement>,
    waivers: Vec<Settlement>,
    expected: Summary,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    pub source_system: String,
    pub summary: Summary,
    pub expected: Summary,
    pub problems: Vec<String>,
    pub can_commit: bool,
    pub fingerprint: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchSummary {
    id: i64,
    source_system: String,
    imported_at: String,
    summary: Summary,
}

pub fn recent(conn: &Connection) -> AppResult<Vec<BatchSummary>> {
    let mut stmt=conn.prepare("SELECT id,source_system,imported_at,summary_json FROM history_import_batches ORDER BY id DESC LIMIT 20")?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    rows.into_iter()
        .map(|(id, source_system, imported_at, json)| {
            let summary = serde_json::from_str(&json).map_err(|_| {
                AppError::Internal("Saved import summary could not be read.".into())
            })?;
            Ok(BatchSummary {
                id,
                source_system,
                imported_at,
                summary,
            })
        })
        .collect()
}

struct Plan {
    reservations: Vec<(Reservation, i64, i64)>,
    fines: Vec<(Fine, i64)>,
    settlements: Vec<(Settlement, bool)>,
    summary: Summary,
}

fn invalid(message: impl Into<String>) -> AppError {
    AppError::Validation(message.into())
}
fn exact(label: &str, value: &str, max: usize) -> AppResult<()> {
    if validate::required(label, value, max)? != value {
        return Err(invalid(format!(
            "Remove surrounding whitespace from {label}."
        )));
    }
    Ok(())
}
fn money(value: i64) -> AppResult<()> {
    validate::in_range("Amount in minor units", value, 1, MAX_MONEY).map(|_| ())
}
fn add(a: i64, b: i64) -> AppResult<i64> {
    a.checked_add(b)
        .filter(|v| *v <= MAX_MONEY)
        .ok_or_else(|| invalid("Financial total exceeds the supported exact range."))
}
fn currency(value: &str) -> AppResult<()> {
    if validate::currency(value)? != value {
        return Err(invalid("Use uppercase three-letter currencies."));
    }
    Ok(())
}
fn unique(
    conn: &Connection,
    source: &str,
    kind: &str,
    id: &str,
    seen: &mut HashSet<(String, String)>,
) -> AppResult<()> {
    exact("source ID", id, 128)?;
    let exists:bool=conn.query_row("SELECT EXISTS(SELECT 1 FROM history_import_sources WHERE source_system=?1 AND kind=?2 AND source_id=?3)",params![source,kind,id],|r|r.get(0))?;
    if exists || !seen.insert((kind.into(), id.into())) {
        return Err(invalid(format!(
            "Duplicate or previously imported {kind} source ID: {id}."
        )));
    }
    Ok(())
}
fn decode(encoded: &str) -> AppResult<(Vec<u8>, Bundle)> {
    if encoded.len() > MAX_BYTES.div_ceil(3) * 4 {
        return Err(invalid("History JSON must be 10 MB or smaller."));
    }
    let bytes = STANDARD
        .decode(encoded)
        .map_err(|_| invalid("History JSON could not be decoded."))?;
    if bytes.len() > MAX_BYTES {
        return Err(invalid("History JSON must be 10 MB or smaller."));
    }
    let content = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(&bytes);
    let bundle:Bundle=serde_json::from_slice(content).map_err(|_|invalid("Invalid history JSON. Check required fields, types, duplicate fields and unsupported fields."))?;
    if bundle.version != 1 {
        return Err(invalid("Unsupported history bundle version."));
    }
    exact("source system", &bundle.source_system, 128)?;
    let count = bundle.reservations.len()
        + bundle.fines.len()
        + bundle.payments.len()
        + bundle.waivers.len();
    if !(1..=25_000).contains(&count) {
        return Err(invalid("Import between 1 and 25,000 history records."));
    }
    Ok((bytes, bundle))
}
fn counts(b: &Bundle) -> Counts {
    Counts {
        reservations: b.reservations.len(),
        fines: b.fines.len(),
        payments: b.payments.len(),
        waivers: b.waivers.len(),
    }
}
fn plan(conn: &Connection, b: &Bundle) -> AppResult<Plan> {
    let mut seen = HashSet::new();
    let reservations = reservations::prepare(conn, b, &mut seen)?;
    let (fines, settlements, balances) = finance::prepare(conn, b, &mut seen)?;
    Ok(Plan {
        reservations,
        fines,
        settlements,
        summary: Summary {
            counts: counts(b),
            balances,
        },
    })
}
fn expected_matches(b: &Bundle, actual: &Summary) -> bool {
    let mut expected = b.expected.clone();
    expected
        .balances
        .sort_by(|a, b| a.currency.cmp(&b.currency));
    expected == *actual
}
pub fn preview(conn: &Connection, encoded: &str) -> AppResult<Preview> {
    let (bytes, b) = decode(encoded)?;
    let tx = conn.unchecked_transaction()?;
    let (summary, problems) = match plan(&tx, &b) {
        Ok(p) => {
            let errors = if expected_matches(&b, &p.summary) {
                vec![]
            } else {
                vec!["Source expected counts or currency totals do not match the calculated destination summary.".into()]
            };
            (p.summary, errors)
        }
        Err(AppError::Validation(message)) => (
            Summary {
                counts: counts(&b),
                balances: vec![],
            },
            vec![message],
        ),
        Err(e) => return Err(e),
    };
    let fingerprint = loan_import::fingerprint(&tx, &bytes)?;
    tx.commit()?;
    Ok(Preview {
        source_system: b.source_system,
        summary,
        expected: b.expected,
        can_commit: problems.is_empty(),
        problems,
        fingerprint,
    })
}
#[allow(clippy::too_many_arguments)]
fn provenance(
    conn: &Connection,
    batch: i64,
    source: &str,
    kind: &str,
    id: &str,
    local: i64,
    staff: &str,
    payload: &impl Serialize,
) -> AppResult<()> {
    let payload = serde_json::to_string(payload)
        .map_err(|_| AppError::Internal("Could not retain migration source.".into()))?;
    conn.execute("INSERT INTO history_import_sources(batch_id,source_system,kind,source_id,local_id,source_staff,source_payload) VALUES(?1,?2,?3,?4,?5,?6,?7)",params![batch,source,kind,id,local,staff,payload])?;
    Ok(())
}
pub fn commit(
    conn: &mut Connection,
    actor: &Session,
    encoded: &str,
    expected_fingerprint: &str,
) -> AppResult<Summary> {
    let (bytes, b) = decode(encoded)?;
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if expected_fingerprint.is_empty()
        || loan_import::fingerprint(&tx, &bytes)? != expected_fingerprint
    {
        return Err(invalid(
            "Library data or file changed. Preview the history bundle again.",
        ));
    }
    let p = plan(&tx, &b)?;
    if !expected_matches(&b, &p.summary) {
        return Err(invalid(
            "Source counts or balances do not reconcile. Nothing was imported.",
        ));
    }
    let digest: String = Sha256::digest(&bytes)
        .iter()
        .map(|v| format!("{v:02x}"))
        .collect();
    let repeated: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM history_import_batches WHERE file_digest=?1)",
        [&digest],
        |r| r.get(0),
    )?;
    if repeated {
        return Err(invalid("This history file has already been imported."));
    }
    let summary_json = serde_json::to_string(&p.summary)
        .map_err(|_| AppError::Internal("Could not save reconciliation summary.".into()))?;
    tx.execute("INSERT INTO history_import_batches(source_system,file_digest,imported_by,summary_json) VALUES(?1,?2,?3,?4)",params![b.source_system,digest,actor.user_id,summary_json])?;
    let batch = tx.last_insert_rowid();
    reservations::insert(&tx, actor, batch, &b.source_system, &p.reservations)?;
    finance::insert(
        &tx,
        actor,
        batch,
        &b.source_system,
        &p.fines,
        &p.settlements,
    )?;
    let saved = finance::reconcile(&tx, &b, batch)?;
    if saved != p.summary {
        return Err(invalid(
            "Saved records did not reconcile. The entire batch was rolled back.",
        ));
    }
    audit::record(&tx,Some(actor),"circulation.import_history","history_import_batch",Some(batch),"Imported reconciled reservations, assessed fines, payments and waivers; original source labels retained.")?;
    tx.commit()?;
    Ok(p.summary)
}
