//! Import complete historical renewal chains without re-running current renewal policy.
use crate::domain::validate;
use crate::error::{AppError, AppResult};
use crate::services::{audit, auth::Session, loan_import};
use base64::{engine::general_purpose::STANDARD, Engine};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};

const MAX_BYTES: usize = 10 * 1024 * 1024;
const HEADERS: [&str; 6] = [
    "source_id",
    "loan_source_id",
    "renewed_at",
    "previous_due_on",
    "due_on",
    "source_staff",
];

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    pub total: usize,
    pub loans: usize,
    pub problems: Vec<String>,
    pub invalid: usize,
    pub fingerprint: String,
}

struct Event {
    record: usize,
    loan_id: i64,
    fields: Vec<String>,
}

fn decode(encoded: &str) -> AppResult<Vec<u8>> {
    if encoded.len() > MAX_BYTES.div_ceil(3) * 4 {
        return Err(AppError::Validation(
            "Renewal CSV must be 10 MB or smaller.".into(),
        ));
    }
    let bytes = STANDARD
        .decode(encoded)
        .map_err(|_| AppError::Validation("The renewal CSV could not be decoded.".into()))?;
    if bytes.len() > MAX_BYTES {
        return Err(AppError::Validation(
            "Renewal CSV must be 10 MB or smaller.".into(),
        ));
    }
    Ok(bytes)
}

fn analyze(conn: &Connection, bytes: &[u8], source: &str) -> AppResult<(Preview, Vec<Event>)> {
    let source = validate::required("Source system", source, 128)?;
    let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes);
    let text = std::str::from_utf8(bytes)
        .map_err(|_| AppError::Validation("Renewal CSV must be UTF-8.".into()))?;
    let mut reader = csv::Reader::from_reader(text.as_bytes());
    if reader
        .headers()
        .map_err(|_| AppError::Validation("Invalid renewal CSV header.".into()))?
        .iter()
        .ne(HEADERS)
    {
        return Err(AppError::Validation(format!(
            "Use exactly these renewal CSV columns, in order: {}.",
            HEADERS.join(",")
        )));
    }
    let mut p = Preview {
        total: 0,
        loans: 0,
        invalid: 0,
        problems: vec![],
        fingerprint: String::new(),
    };
    let mut ids = HashSet::new();
    let mut groups: BTreeMap<i64, Vec<Event>> = BTreeMap::new();
    for (index, row) in reader.records().enumerate() {
        if index >= 25_000 {
            return Err(AppError::Validation(
                "Import at most 25,000 renewal events.".into(),
            ));
        }
        p.total += 1;
        let checked = (|| -> AppResult<Event> {
            let row = row
                .map_err(|_| AppError::Validation("Invalid CSV record or column count.".into()))?;
            for (column, max) in [(0, 128), (1, 128), (5, 200)] {
                if validate::required(HEADERS[column], &row[column], max)? != row[column] {
                    return Err(AppError::Validation(
                        "Remove surrounding whitespace from identifiers.".into(),
                    ));
                }
            }
            if !ids.insert(row[0].to_owned()) {
                return Err(AppError::Validation("Duplicate renewal source_id.".into()));
            }
            loan_import::timestamp(&row[2])?;
            for column in [3, 4] {
                if validate::date(HEADERS[column], &row[column])? != row[column] {
                    return Err(AppError::Validation(
                        "Remove surrounding whitespace from dates.".into(),
                    ));
                }
            }
            if row[4] <= row[3] {
                return Err(AppError::Validation(
                    "Renewal due date must be later than previous_due_on.".into(),
                ));
            }
            let loan: Option<(i64,String,Option<String>)> = conn.query_row("SELECT l.id,l.issued_at,l.returned_at FROM loan_import_sources s JOIN loans l ON l.id=s.loan_id WHERE s.source_system=?1 AND s.source_id=?2",params![source,&row[1]],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional()?;
            let (loan_id, issued, returned) = loan.ok_or_else(|| {
                AppError::Validation("Source loan is not imported under this source system.".into())
            })?;
            if row[2] < issued[..]
                || returned.as_deref().is_some_and(|date| &row[2] > date)
                || row[3] < issued[..10]
            {
                return Err(AppError::Validation(
                    "Renewal lies outside the loan's issue/return dates.".into(),
                ));
            }
            let repeated: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM renewal_import_sources WHERE source_system=?1 AND source_id=?2)",params![source,&row[0]],|r|r.get(0))?;
            if repeated {
                return Err(AppError::Validation(
                    "Renewal source ID has already been imported.".into(),
                ));
            }
            Ok(Event {
                record: index + 1,
                loan_id,
                fields: row.iter().map(str::to_owned).collect(),
            })
        })();
        match checked {
            Ok(event) => groups.entry(event.loan_id).or_default().push(event),
            Err(AppError::Validation(message)) => {
                p.invalid += 1;
                if p.problems.len() < 100 {
                    p.problems.push(format!("Record {}: {message}", index + 1));
                }
            }
            Err(e) => return Err(e),
        }
    }
    let mut events = vec![];
    p.loans = groups.len();
    for (loan, mut chain) in groups {
        chain.sort_by(|a, b| a.fields[2].cmp(&b.fields[2]));
        let final_due: String =
            conn.query_row("SELECT due_on FROM loans WHERE id=?1", [loan], |r| r.get(0))?;
        let existing: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM loan_renewals WHERE loan_id=?1)",
            [loan],
            |r| r.get(0),
        )?;
        let broken = existing
            || chain.windows(2).any(|pair| {
                pair[0].fields[2] >= pair[1].fields[2] || pair[0].fields[4] != pair[1].fields[3]
            })
            || chain
                .last()
                .is_some_and(|event| event.fields[4] != final_due);
        if broken {
            p.invalid += chain.len();
            if p.problems.len() < 100 {
                p.problems.push(format!("Record {}: supply one complete, chronological renewal chain ending at the imported loan's due date, with no existing renewal events.",chain[0].record));
            }
        } else {
            events.extend(chain);
        }
    }
    Ok((p, events))
}

fn fingerprint(conn: &Connection, bytes: &[u8], source: &str) -> AppResult<String> {
    let state = loan_import::fingerprint(conn, bytes)?;
    let digest = Sha256::digest(format!("{state}:{}", source.trim()).as_bytes());
    Ok(digest.iter().map(|b| format!("{b:02x}")).collect())
}

pub fn preview(conn: &Connection, encoded: &str, source: &str) -> AppResult<Preview> {
    let bytes = decode(encoded)?;
    let tx = conn.unchecked_transaction()?;
    let (mut p, _) = analyze(&tx, &bytes, source)?;
    p.fingerprint = fingerprint(&tx, &bytes, source)?;
    tx.commit()?;
    Ok(p)
}

pub fn commit(
    conn: &mut Connection,
    actor: &Session,
    encoded: &str,
    source: &str,
    expected: &str,
) -> AppResult<usize> {
    let source = validate::required("Source system", source, 128)?;
    let bytes = decode(encoded)?;
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if expected.is_empty() || fingerprint(&tx, &bytes, &source)? != expected {
        return Err(AppError::Validation(
            "Library data, source or file changed. Preview renewals again.".into(),
        ));
    }
    let (p, events) = analyze(&tx, &bytes, &source)?;
    if p.total == 0 || p.invalid > 0 {
        return Err(AppError::Validation(
            "Resolve every invalid renewal before importing a nonempty file.".into(),
        ));
    }
    let digest: String = Sha256::digest(&bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let repeated: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM renewal_import_batches WHERE file_digest=?1)",
        [&digest],
        |r| r.get(0),
    )?;
    if repeated {
        return Err(AppError::Validation(
            "This renewal file has already been imported.".into(),
        ));
    }
    tx.execute("INSERT INTO renewal_import_batches(source_system,file_digest,imported_by,row_count) VALUES(?1,?2,?3,?4)",params![source,digest,actor.user_id,p.total as i64])?;
    let batch = tx.last_insert_rowid();
    for event in events {
        let f = &event.fields;
        tx.execute("INSERT INTO loan_renewals(loan_id,renewed_by,renewed_at,previous_due_on,due_on) VALUES(?1,?2,?3,?4,?5)",params![event.loan_id,actor.user_id,f[2],f[3],f[4]])?;
        let renewal = tx.last_insert_rowid();
        let payload = serde_json::to_string(f)
            .map_err(|_| AppError::Internal("Could not preserve renewal source.".into()))?;
        tx.execute("INSERT INTO renewal_import_sources(renewal_id,batch_id,source_system,source_id,source_staff,source_payload) VALUES(?1,?2,?3,?4,?5,?6)",params![renewal,batch,source,f[0],f[5],payload])?;
    }
    audit::record(
        &tx,
        Some(actor),
        "circulation.import_renewals",
        "renewal_import_batch",
        Some(batch),
        &format!(
            "Imported {} historical renewal events for {} loans; current due dates unchanged.",
            p.total, p.loans
        ),
    )?;
    tx.commit()?;
    Ok(p.total)
}
