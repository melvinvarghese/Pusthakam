use crate::domain::validate;
use crate::error::{AppError, AppResult};
use crate::services::{
    audit,
    auth::Session,
    members::{self, MemberInput},
};
use base64::{engine::general_purpose::STANDARD, Engine};
use rusqlite::{Connection, TransactionBehavior};
use serde::Serialize;
use std::collections::{BTreeMap, HashSet};

const MAX_BYTES: usize = 10 * 1024 * 1024;
const MAX_ROWS: usize = 50_000;
const PROBLEM_LIMIT: usize = 50;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Patron {
    member_number: String,
    name: String,
    category_code: String,
    #[serde(skip)]
    email: Option<String>,
    #[serde(skip)]
    phone: Option<String>,
    expires_on: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    ready: usize,
    skipped: usize,
    categories: BTreeMap<String, usize>,
    sample: Vec<Patron>,
    problems: Vec<String>,
}

struct Analysis {
    patrons: Vec<Patron>,
    skipped: usize,
    problems: Vec<String>,
}

impl Analysis {
    fn skip(&mut self, line: u64, message: &str) {
        self.skipped += 1;
        if self.problems.len() < PROBLEM_LIMIT {
            self.problems.push(format!("Line {line}: {message}"));
        }
    }
}

fn analyze(conn: &Connection, data_base64: &str) -> AppResult<Analysis> {
    if data_base64.len() > MAX_BYTES.div_ceil(3) * 4 {
        return Err(AppError::Validation(
            "The patron CSV must be 10 MB or smaller.".into(),
        ));
    }
    let bytes = STANDARD
        .decode(data_base64)
        .map_err(|_| AppError::Validation("The patron file could not be decoded.".into()))?;
    if bytes.len() > MAX_BYTES {
        return Err(AppError::Validation(
            "The patron CSV must be 10 MB or smaller.".into(),
        ));
    }
    let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(&bytes);
    let text = std::str::from_utf8(bytes)
        .map_err(|_| AppError::Validation("Export the patron CSV as UTF-8.".into()))?;
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(text.as_bytes());
    let headers = reader
        .headers()
        .map_err(|_| AppError::Validation("The patron CSV header could not be read.".into()))?
        .clone();
    let column = |name: &str| {
        headers
            .iter()
            .position(|field| field.eq_ignore_ascii_case(name))
    };
    let card = column("cardnumber")
        .ok_or_else(|| AppError::Validation("The patron CSV needs a cardnumber column.".into()))?;
    let surname = column("surname")
        .ok_or_else(|| AppError::Validation("The patron CSV needs a surname column.".into()))?;
    let category = column("categorycode").ok_or_else(|| {
        AppError::Validation("The patron CSV needs a categorycode column.".into())
    })?;
    let first = column("firstname");
    let email = column("email");
    let phone = column("phone");
    let expiry = column("dateexpiry");
    let mut stmt = conn.prepare("SELECT member_number FROM members")?;
    let mut seen = stmt
        .query_map([], |r| Ok(r.get::<_, String>(0)?.to_ascii_lowercase()))?
        .collect::<Result<HashSet<_>, _>>()?;
    let mut analysis = Analysis {
        patrons: Vec::new(),
        skipped: 0,
        problems: Vec::new(),
    };

    for (index, record) in reader.records().enumerate() {
        if index >= MAX_ROWS {
            return Err(AppError::Validation(
                "Import at most 50,000 patron rows at a time.".into(),
            ));
        }
        let row = match record {
            Ok(row) => row,
            Err(error) => {
                let line = error.position().map_or(index as u64 + 2, |p| p.line());
                analysis.skip(line, "Invalid CSV row.");
                continue;
            }
        };
        if row.iter().all(str::is_empty) {
            continue;
        }
        let line = row.position().map_or(index as u64 + 2, |p| p.line());
        let field = |column: Option<usize>| {
            column
                .and_then(|i| row.get(i))
                .filter(|value| !value.is_empty())
        };
        let member_number = row.get(card).unwrap_or_default();
        let name = [field(first), field(Some(surname))]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" ");
        let category_code = row.get(category).unwrap_or_default();
        let validated = (|| -> AppResult<Patron> {
            Ok(Patron {
                member_number: validate::required("Card number", member_number, 64)?,
                name: validate::required("Patron name", &name, 160)?,
                category_code: validate::required("Category code", category_code, 64)?,
                email: validate::optional("Email", field(email), 254)?,
                phone: validate::optional("Phone", field(phone), 40)?,
                expires_on: field(expiry)
                    .map(|date| validate::date("Expiry date", date))
                    .transpose()?,
            })
        })();
        match validated {
            Ok(patron) if seen.insert(patron.member_number.to_ascii_lowercase()) => {
                analysis.patrons.push(patron)
            }
            Ok(_) => analysis.skip(
                line,
                "Card number already exists or appears earlier in the CSV.",
            ),
            Err(error) => analysis.skip(line, &error.user_message()),
        }
    }
    Ok(analysis)
}

pub fn preview(conn: &Connection, data_base64: &str) -> AppResult<Preview> {
    let analysis = analyze(conn, data_base64)?;
    let mut categories = BTreeMap::new();
    for patron in &analysis.patrons {
        *categories.entry(patron.category_code.clone()).or_insert(0) += 1;
    }
    Ok(Preview {
        ready: analysis.patrons.len(),
        skipped: analysis.skipped,
        categories,
        sample: analysis.patrons.into_iter().take(8).collect(),
        problems: analysis.problems,
    })
}

pub fn commit(
    conn: &mut Connection,
    actor: &Session,
    data_base64: &str,
    mappings: &BTreeMap<String, i64>,
    expected_ready: usize,
) -> AppResult<usize> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let analysis = analyze(&tx, data_base64)?;
    let count = analysis.patrons.len();
    if count != expected_ready || count == 0 {
        return Err(AppError::Conflict(
            "The patrons ready to import changed. Preview the file again.".into(),
        ));
    }
    for patron in analysis.patrons {
        let category_id = mappings
            .get(&patron.category_code)
            .copied()
            .ok_or_else(|| {
                AppError::Validation(
                    "Map every imported Koha category to an active member category.".into(),
                )
            })?;
        members::save_in_transaction(
            &tx,
            actor,
            MemberInput {
                id: None,
                member_number: patron.member_number,
                name: patron.name,
                category_id,
                email: patron.email,
                phone: patron.phone,
                expires_on: patron.expires_on,
                address: None,
                department: None,
                joined_on: None,
                notes: None,
            },
        )?;
    }
    audit::record(
        &tx,
        Some(actor),
        "members.import",
        "members",
        None,
        &format!("Imported {count} Koha patrons"),
    )?;
    tx.commit()?;
    Ok(count)
}
