//! Read-only analysis of the neutral loan-history CSV, not a Koha field mapping.
use crate::domain::validate;
use crate::error::{AppError, AppResult};
use crate::services::{audit, auth::Session};
use base64::{engine::general_purpose::STANDARD, Engine};
use rusqlite::{params, TransactionBehavior};
use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashSet;

pub fn preview_base64(conn: &Connection, encoded: &str) -> AppResult<Preview> {
    if encoded.len() > MAX_BYTES.div_ceil(3) * 4 {
        return Err(AppError::Validation(
            "Loan CSV must be 10 MB or smaller.".into(),
        ));
    }
    let bytes = STANDARD
        .decode(encoded)
        .map_err(|_| AppError::Validation("The loan CSV could not be decoded.".into()))?;
    let tx = conn.unchecked_transaction()?;
    let mut result = preview(&tx, &bytes)?;
    result.fingerprint = fingerprint(&tx, &bytes)?;
    tx.commit()?;
    Ok(result)
}

const MAX_BYTES: usize = 10 * 1024 * 1024;
const MAX_ROWS: usize = 25_000;
const HEADERS: [&str; 7] = [
    "source_id",
    "member_number",
    "barcode",
    "issued_at",
    "due_on",
    "returned_at",
    "source_staff",
];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    pub fingerprint: String,
    pub total: usize,
    pub open: usize,
    pub closed: usize,
    pub invalid: usize,
    pub problems: Vec<String>,
}

pub(super) fn timestamp(value: &str) -> AppResult<()> {
    let b = value.as_bytes();
    if b.len() != 20
        || b[10] != b'T'
        || b[13] != b':'
        || b[16] != b':'
        || b[19] != b'Z'
        || !b[..10].is_ascii()
        || !b[11..19].is_ascii()
    {
        return Err(AppError::Validation(
            "Use UTC timestamps written YYYY-MM-DDTHH:MM:SSZ.".into(),
        ));
    }
    validate::date("Timestamp date", &value[..10])?;
    for (part, max) in [
        (&value[11..13], 23),
        (&value[14..16], 59),
        (&value[17..19], 59),
    ] {
        if !part.bytes().all(|v| v.is_ascii_digit()) || part.parse::<u32>().unwrap_or(100) > max {
            return Err(AppError::Validation("Invalid timestamp time.".into()));
        }
    }
    Ok(())
}

/// No writes, policy substitutions or inferred financial events are performed.
pub fn preview(conn: &Connection, bytes: &[u8]) -> AppResult<Preview> {
    if bytes.len() > MAX_BYTES {
        return Err(AppError::Validation(
            "Loan CSV must be 10 MB or smaller.".into(),
        ));
    }
    let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes);
    let text = std::str::from_utf8(bytes)
        .map_err(|_| AppError::Validation("Loan CSV must be UTF-8.".into()))?;
    let mut reader = csv::ReaderBuilder::new().from_reader(text.as_bytes());
    let header = reader
        .headers()
        .map_err(|_| AppError::Validation("Invalid loan CSV header.".into()))?
        .clone();
    if header.iter().ne(HEADERS) {
        return Err(AppError::Validation(format!(
            "Use exactly these loan CSV columns, in order: {}.",
            HEADERS.join(",")
        )));
    }
    let mut result = Preview {
        fingerprint: String::new(),
        total: 0,
        open: 0,
        closed: 0,
        invalid: 0,
        problems: vec![],
    };
    let mut ids = HashSet::new();
    let mut open_copies = HashSet::new();
    for (index, row) in reader.records().enumerate() {
        if index >= MAX_ROWS {
            return Err(AppError::Validation(
                "Import at most 25,000 loan rows.".into(),
            ));
        }
        result.total += 1;
        let checked = (|| -> AppResult<bool> {
            let row =
                row.map_err(|_| AppError::Validation("Invalid CSV row or column count.".into()))?;
            for (column, max) in [(0, 128), (1, 64), (2, 128), (6, 200)] {
                if validate::required(HEADERS[column], &row[column], max)? != row[column] {
                    return Err(AppError::Validation(
                        "Remove surrounding whitespace from identifiers.".into(),
                    ));
                }
            }
            if !ids.insert(row[0].to_owned()) {
                return Err(AppError::Validation("Duplicate source_id.".into()));
            }
            timestamp(&row[3])?;
            if validate::date("Due date", &row[4])? != row[4] || row[4] < row[3][..10] {
                return Err(AppError::Validation(
                    "Due date precedes issue date or has surrounding whitespace.".into(),
                ));
            }
            let open = row[5].is_empty();
            if !open {
                timestamp(&row[5])?;
                if row[5] < row[3] {
                    return Err(AppError::Validation("Return precedes issue.".into()));
                }
            }
            let member: Option<i64> = conn
                .query_row(
                    "SELECT id FROM members WHERE member_number = ?1",
                    [&row[1]],
                    |r| r.get(0),
                )
                .optional()?;
            if member.is_none() {
                return Err(AppError::Validation(
                    "Member number is not imported.".into(),
                ));
            }
            let item: Option<(i64, String)> = conn
                .query_row(
                    "SELECT id, status FROM physical_items WHERE barcode = ?1",
                    [&row[2]],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional()?;
            let (item_id, status) =
                item.ok_or_else(|| AppError::Validation("Barcode is not imported.".into()))?;
            if open {
                let existing: bool = conn.query_row(
                    "SELECT EXISTS(SELECT 1 FROM loans WHERE item_id = ?1 AND returned_at IS NULL)",
                    [item_id],
                    |r| r.get(0),
                )?;
                if existing || status != "available" || !open_copies.insert(item_id) {
                    return Err(AppError::Validation(
                        "Open loan conflicts with copy availability or another open loan.".into(),
                    ));
                }
            }
            Ok(open)
        })();
        match checked {
            Ok(true) => result.open += 1,
            Ok(false) => result.closed += 1,
            Err(AppError::Validation(message)) => {
                result.invalid += 1;
                if result.problems.len() < 100 {
                    result
                        .problems
                        .push(format!("Record {}: {message}", index + 1));
                }
            }
            Err(error) => return Err(error),
        }
    }
    Ok(result)
}

// Include referenced database state, not just file contents, so a policy/identity/stock
// change after preview requires a new review. Stream rows into the hash, not into memory.
pub(super) fn fingerprint(conn: &Connection, bytes: &[u8]) -> AppResult<String> {
    let mut hash = Sha256::new();
    hash.update(bytes);
    for table in [
        "members",
        "member_categories",
        "physical_items",
        "item_categories",
        "bibliographic_records",
        "circulation_policies",
        "libraries",
        "loans",
        "loan_import_sources",
        "loan_import_batches", "loan_renewals", "renewal_import_batches", "renewal_import_sources",
    ] {
        hash.update(table.as_bytes());
        let mut stmt = conn.prepare(&format!("SELECT * FROM {table} ORDER BY rowid"))?;
        let columns = stmt.column_count();
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            for column in 0..columns {
                let value = format!("{:?}", row.get_ref(column)?);
                hash.update(value.len().to_le_bytes());
                hash.update(value.as_bytes());
            }
        }
    }
    Ok(hash.finalize().iter().map(|b| format!("{b:02x}")).collect::<String>())
}

pub fn commit(
    conn: &mut Connection,
    actor: &Session,
    encoded: &str,
    source_system: &str,
    expected_fingerprint: &str,
    use_current_terms: bool,
) -> AppResult<usize> {
    let source_system = validate::required("Source system", source_system, 128)?;
    if encoded.len() > MAX_BYTES.div_ceil(3) * 4 {
        return Err(AppError::Validation(
            "Loan CSV must be 10 MB or smaller.".into(),
        ));
    }
    let bytes = STANDARD
        .decode(encoded)
        .map_err(|_| AppError::Validation("The loan CSV could not be decoded.".into()))?;
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if expected_fingerprint.is_empty() || fingerprint(&tx, &bytes)? != expected_fingerprint {
        return Err(AppError::Validation(
            "Library data or the file changed. Preview the CSV again before importing.".into(),
        ));
    }
    let checked = preview(&tx, &bytes)?;
    if checked.invalid > 0 || checked.total == 0 {
        return Err(AppError::Validation(
            "Resolve every invalid record before importing a nonempty file.".into(),
        ));
    }
    if checked.open > 0 && !use_current_terms {
        return Err(AppError::Validation(
            "Confirm the use of current fine policies for imported open loans.".into(),
        ));
    }
    let digest = Sha256::digest(&bytes).iter().map(|b| format!("{b:02x}")).collect::<String>();
    let duplicate: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM loan_import_batches WHERE file_digest=?1)",
        [&digest],
        |r| r.get(0),
    )?;
    if duplicate {
        return Err(AppError::Validation(
            "This file has already been imported.".into(),
        ));
    }
    tx.execute("INSERT INTO loan_import_batches(source_system,file_digest,imported_by,row_count) VALUES(?1,?2,?3,?4)", params![source_system,digest,actor.user_id,checked.total as i64])?;
    let batch_id = tx.last_insert_rowid();
    let content = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(&bytes);
    let mut reader = csv::Reader::from_reader(content);
    for row in reader.records() {
        let row = row.map_err(|_| AppError::Validation("Invalid loan CSV record.".into()))?;
        let repeated: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM loan_import_sources WHERE source_system=?1 AND source_id=?2)",params![source_system,&row[0]],|r|r.get(0))?;
        if repeated {
            return Err(AppError::Validation(
                "A source loan identifier has already been imported.".into(),
            ));
        }
        let (member_id, member_category, member_archived): (i64, i64, bool) = tx.query_row(
            "SELECT id,category_id,archived_at IS NOT NULL FROM members WHERE member_number=?1",
            [&row[1]],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?;
        let (item_id, item_category, archived): (i64,i64,bool) = tx.query_row("SELECT i.id,i.item_category_id,r.archived_at IS NOT NULL FROM physical_items i JOIN bibliographic_records r ON r.id=i.record_id WHERE i.barcode=?1",[&row[2]],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?;
        let open = row[5].is_empty();
        if open && (member_archived || archived) {
            return Err(AppError::Validation(
                "An open loan cannot reference an archived member or title.".into(),
            ));
        }
        let end = if open {
            "9999-12-31T23:59:59Z"
        } else {
            &row[5]
        };
        let overlaps: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM loans WHERE item_id=?1 AND issued_at < ?2 AND COALESCE(returned_at,'9999-12-31T23:59:59Z') > ?3)",params![item_id,end,&row[3]],|r|r.get(0))?;
        if overlaps {
            return Err(AppError::Validation(
                "Loan dates overlap another loan for the same copy. Nothing was imported.".into(),
            ));
        }
        tx.execute("INSERT INTO loans(item_id,member_id,item_category_id,issued_by,issued_at,due_on,returned_at,returned_by,return_condition) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,NULL)",params![item_id,member_id,item_category,actor.user_id,&row[3],&row[4],if open {None} else {Some(&row[5])},if open {None} else {Some(actor.user_id)}])?;
        let loan_id = tx.last_insert_rowid();
        if open {
            let policy: Option<(i64,i64,Option<i64>,String)> = tx.query_row("SELECT p.fine_per_day_minor,p.fine_grace_days,p.fine_cap_minor,l.currency FROM circulation_policies p JOIN libraries l ON l.id=1 JOIN member_categories m ON m.id=p.member_category_id JOIN item_categories i ON i.id=p.item_category_id WHERE p.member_category_id=?1 AND p.item_category_id=?2 AND m.archived_at IS NULL AND i.archived_at IS NULL",params![member_category,item_category],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional()?;
            let (rate, grace, cap, currency) = policy.ok_or_else(|| {
                AppError::Validation(
                    "An open loan has no active fine policy. Nothing was imported.".into(),
                )
            })?;
            tx.execute("INSERT INTO loan_fine_terms(loan_id,rate_minor,grace_days,cap_minor,currency) VALUES(?1,?2,?3,?4,?5)",params![loan_id,rate,grace,cap,currency])?;
            tx.execute("UPDATE physical_items SET status='on_loan',updated_at=strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE id=?1",[item_id])?;
            tx.execute("INSERT INTO item_events(item_id,event_type,from_status,to_status,note,staff_user_id) VALUES(?1,'loan_import','available','on_loan','Imported open loan; original dates retained in loan provenance.',?2)",params![item_id,actor.user_id])?;
        }
        let payload = serde_json::to_string(&row.iter().collect::<Vec<_>>())
            .map_err(|_| AppError::Internal("Could not preserve import source.".into()))?;
        tx.execute("INSERT INTO loan_import_sources(loan_id,batch_id,source_system,source_id,source_staff,source_payload,terms_basis) VALUES(?1,?2,?3,?4,?5,?6,?7)",params![loan_id,batch_id,source_system,&row[0],&row[6],payload,if open {"current_policy_confirmed"} else {"closed_history"}])?;
    }
    audit::record(&tx,Some(actor),"circulation.import","loan_import_batch",Some(batch_id),&format!("Imported {} loans ({} open, {} closed); original source labels preserved in provenance.",checked.total,checked.open,checked.closed))?;
    tx.commit()?;
    Ok(checked.total)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn db() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        c.execute_batch("CREATE TABLE members(id INTEGER,member_number TEXT COLLATE NOCASE); INSERT INTO members VALUES(1,'001'); CREATE TABLE physical_items(id INTEGER,barcode TEXT COLLATE NOCASE,status TEXT); INSERT INTO physical_items VALUES(2,'0002','available'); CREATE TABLE loans(item_id INTEGER,returned_at TEXT);").unwrap();
        c
    }
    fn csv(rows: &str) -> String {
        format!("{}\n{rows}", HEADERS.join(","))
    }
    #[test]
    fn resolves_text_identifiers_and_preserves_database() {
        let c = db();
        let p = preview(&c, csv("old,001,0002,2020-02-29T10:00:00Z,2020-03-10,2020-03-01T11:00:00Z,Former librarian\nnew,001,0002,2026-11-01T10:00:00Z,2026-11-30,,Former librarian").as_bytes()).unwrap();
        assert_eq!((p.total, p.open, p.closed, p.invalid), (2, 1, 1, 0));
        assert_eq!(
            c.query_row("SELECT count(*) FROM loans", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
    #[test]
    fn rejects_missing_references_duplicate_ids_and_open_conflicts() {
        let c = db();
        let rows=csv("a,001,0002,2026-11-01T10:00:00Z,2026-11-30,,Staff\nb,001,0002,2026-11-01T10:00:00Z,2026-11-30,,Staff\na,001,0002,2026-11-01T10:00:00Z,2026-11-30,,Staff\nc,1,0002,2026-11-01T10:00:00Z,2026-11-30,,Staff");
        let p = preview(&c, rows.as_bytes()).unwrap();
        assert_eq!((p.open, p.invalid), (1, 3));
        c.execute("INSERT INTO loans VALUES(2,NULL)", []).unwrap();
        assert_eq!(
            preview(
                &c,
                csv("a,001,0002,2026-11-01T10:00:00Z,2026-11-30,,Staff").as_bytes()
            )
            .unwrap()
            .invalid,
            1
        );
    }
    #[test]
    fn rejects_bad_shapes_dates_and_reverse_chronology() {
        let c = db();
        assert!(preview(&c, b"source_id,unknown\na,b").is_err());
        for value in [
            "2025-02-29T00:00:00Z",
            "2026-01-01T24:00:00Z",
            "2026-01-01T00:00:00+05:30",
            "2026-01-01T00:00:00\u{00e9}",
        ] {
            assert!(timestamp(value).is_err());
        }
        let p=preview(&c,csv("a,001,0002,2026-11-01T10:00:00Z,2026-11-30,2026-10-01T10:00:00Z,Staff\nb,001,0002,2026-11-01T10:00:00Z,2026-10-30,,Staff").as_bytes()).unwrap();
        assert_eq!(p.invalid, 2);
    }
}
