//! Importing existing catalogue data (for example from Koha): read a file, show a preview with every
//! problem, then (only after the librarian confirms) write everything in one transaction.
//! Nothing is dropped silently: damaged records, skipped copies and warnings are all reported.

pub mod csv_import;
pub mod marc;
pub mod json_import;
pub mod marcxml;
pub mod xlsx_import;

use crate::domain::{accession, isbn};
use crate::error::{AppError, AppResult};
use crate::services::catalogue::{self, ContributorDto, CopyBatchInput, RecordInput};
use crate::services::{audit, auth::Session};
use rusqlite::{Connection, TransactionBehavior};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub const MAX_FILE_BYTES: usize = 100 * 1024 * 1024;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ImportCopy {
    pub barcode: Option<String>,
    pub accession: Option<String>,
    pub call_number: Option<String>,
    pub location: Option<String>,
    pub shelf: Option<String>,
    pub collection: Option<String>,
    pub item_category: Option<String>,
    pub acquisition_date: Option<String>,
    pub price_minor: Option<i64>,
    pub vendor: Option<String>,
    pub notes: Option<String>,
    /// lost, damaged, withdrawn or in_repair when the source marks the copy that way.
    pub status: Option<String>,
    /// Set by analysis when this copy will not be imported (for example its barcode already exists).
    pub skip_reason: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ImportRecord {
    /// Position in the source (record number for MARC, line number for CSV).
    pub row: usize,
    pub title: String,
    pub subtitle: Option<String>,
    pub contributors: Vec<ContributorDto>,
    pub isbn: Option<String>,
    pub publisher: Option<String>,
    pub place: Option<String>,
    pub year: Option<i64>,
    pub edition: Option<String>,
    pub language: Option<String>,
    pub pagination: Option<String>,
    pub physical: Option<String>,
    pub series: Option<String>,
    pub subjects: Vec<String>,
    pub classification: Option<String>,
    pub classification_scheme: Option<String>,
    pub abstract_text: Option<String>,
    pub notes: Option<String>,
    pub keywords: Option<String>,
    pub copies: Vec<ImportCopy>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    /// Existing record with the same ISBN, if any.
    pub duplicate_of: Option<i64>,
}

/// "250", "250.5", "1,250.00", "₹250", "Rs. 250.00" -> minor units. `None` when it is not an amount.
pub fn parse_price_minor(raw: &str) -> Option<i64> {
    // Keep digits and dots, then drop dots at either end ("Rs. 10" leaves ". 10"; "250." is just 250).
    let kept: String = raw.chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
    let cleaned = kept.trim_matches('.').to_string();
    if cleaned.is_empty() || cleaned.matches('.').count() > 1 {
        return None;
    }
    let (whole, frac) = cleaned.split_once('.').unwrap_or((&cleaned, ""));
    if frac.len() > 2 || whole.is_empty() {
        return None;
    }
    let w: i64 = whole.parse().ok()?;
    let f: i64 = format!("{frac:0<2}").parse().ok()?;
    w.checked_mul(100)?.checked_add(f)
}

// ---------- preview ----------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Totals {
    pub records: usize,
    pub copies: usize,
    pub with_warnings: usize,
    pub with_errors: usize,
    pub duplicates: usize,
    pub copies_skipped: usize,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewLookups {
    pub locations: Vec<String>,
    pub collections: Vec<String>,
    pub item_categories: Vec<String>,
}

fn lower_set(conn: &Connection, sql: &str) -> AppResult<HashSet<String>> {
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], |r| r.get::<_, String>(0))?.collect::<Result<Vec<_>, _>>()?;
    Ok(rows.into_iter().map(|s| s.to_lowercase()).collect())
}

/// Checks every record against the database and the rest of the file: invalid values, ISBNs and barcodes
/// that already exist, and which new locations/collections/categories would be created.
pub fn analyze(conn: &Connection, records: &mut [ImportRecord]) -> AppResult<(Totals, NewLookups)> {
    let existing_barcodes = lower_set(conn, "SELECT barcode FROM physical_items")?;
    let existing_accessions = lower_set(conn, "SELECT accession_no FROM physical_items")?;
    let have_locations = lower_set(conn, "SELECT name FROM locations")?;
    let have_collections = lower_set(conn, "SELECT name FROM collections")?;
    let have_categories = lower_set(conn, "SELECT name FROM item_categories")?;

    let mut seen_barcodes: HashSet<String> = HashSet::new();
    let mut seen_accessions: HashSet<String> = HashSet::new();
    let mut seen_isbns: HashMap<String, usize> = HashMap::new();
    let (mut loc_new, mut col_new, mut cat_new) = (Vec::<String>::new(), Vec::<String>::new(), Vec::<String>::new());
    let note_new = |list: &mut Vec<String>, name: &str, have: &HashSet<String>| {
        let l = name.to_lowercase();
        if !have.contains(&l) && !list.iter().any(|x| x.to_lowercase() == l) {
            list.push(name.to_string());
        }
    };
    let mut totals = Totals { records: records.len(), copies: 0, with_warnings: 0, with_errors: 0, duplicates: 0, copies_skipped: 0 };

    for r in records.iter_mut() {
        r.title = r.title.split_whitespace().collect::<Vec<_>>().join(" ");
        if r.title.is_empty() {
            r.errors.push("No title. This record cannot be imported.".into());
        }
        if let Some(raw) = r.isbn.clone() {
            match isbn::both_forms(&raw) {
                Some((_, i13)) => {
                    r.duplicate_of = conn
                        .query_row("SELECT id FROM bibliographic_records WHERE isbn13 = ?1 LIMIT 1", [&i13], |row| row.get(0))
                        .ok();
                    if let Some(prev) = seen_isbns.insert(i13, r.row) {
                        r.warnings.push(format!("Same ISBN as row {prev} of this file."));
                    }
                }
                None => {
                    r.warnings.push(format!("The ISBN \"{raw}\" is not valid and was left out."));
                    r.isbn = None;
                }
            }
        }
        if r.duplicate_of.is_some() {
            totals.duplicates += 1;
            r.warnings.push("A record with this ISBN is already in the catalogue.".into());
        }
        if r.copies.is_empty() {
            r.warnings.push("No copies in the file: only the title will be imported.".into());
        }
        for c in r.copies.iter_mut() {
            totals.copies += 1;
            if let Some(b) = &c.barcode {
                let l = b.to_lowercase();
                if accession::validate_code("Barcode", b).is_err() {
                    c.skip_reason = Some(format!("Barcode \"{b}\" has spaces or unusual characters."));
                } else if existing_barcodes.contains(&l) || existing_accessions.contains(&l) {
                    c.skip_reason = Some(format!("Barcode {b} is already in the catalogue."));
                } else if !seen_barcodes.insert(l) {
                    c.skip_reason = Some(format!("Barcode {b} appears more than once in this file."));
                }
            }
            if c.skip_reason.is_none() {
                if let Some(a) = &c.accession {
                    let l = a.to_lowercase();
                    if accession::validate_code("Accession number", a).is_err() {
                        c.skip_reason = Some(format!("Accession number \"{a}\" has spaces or unusual characters."));
                    } else if existing_accessions.contains(&l) || !seen_accessions.insert(l) {
                        c.skip_reason = Some(format!("Accession number {a} is already in use."));
                    }
                }
            }
            if let Some(reason) = &c.skip_reason {
                r.warnings.push(format!("Copy skipped: {reason}"));
                totals.copies_skipped += 1;
                continue;
            }
            if let Some(l) = &c.location { note_new(&mut loc_new, l, &have_locations); }
            if let Some(l) = &c.collection { note_new(&mut col_new, l, &have_collections); }
            if let Some(l) = &c.item_category { note_new(&mut cat_new, l, &have_categories); }
        }
        if !r.errors.is_empty() { totals.with_errors += 1; }
        if !r.warnings.is_empty() { totals.with_warnings += 1; }
    }
    Ok((totals, NewLookups { locations: loc_new, collections: col_new, item_categories: cat_new }))
}

// ---------- request / preview ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewRequest {
    pub file_name: String,
    /// The file's bytes, base64 encoded by the UI.
    pub data_base64: String,
    /// "marc" (ISO 2709 .mrc) or "csv".
    pub format: String,
    /// "koha" reads holdings from field 952; "generic" imports titles only. Ignored for CSV.
    pub profile: String,
    /// CSV only: field -> column header, to override automatic column recognition.
    pub mapping: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    pub token: String,
    pub file_name: String,
    pub format: String,
    pub profile: String,
    pub totals: Totals,
    pub new_lookups: NewLookups,
    /// The first records plus records with problems, so the librarian can check real examples.
    pub sample: Vec<ImportRecord>,
    /// Records in the file that could not be read at all.
    pub parse_errors: Vec<String>,
    pub headers: Option<Vec<String>>,
    pub mapping: Option<HashMap<String, String>>,
    /// CSV only: every field a column can be mapped to.
    pub mapping_fields: Option<Vec<String>>,
}

/// A parsed, analysed file waiting for the librarian's confirmation. Held in memory only.
pub struct Pending {
    pub token: String,
    pub source: String,
    pub records: Vec<ImportRecord>,
}

pub fn prepare(conn: &Connection, req: &PreviewRequest) -> AppResult<(Pending, Preview)> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(req.data_base64.trim())
        .map_err(|_| AppError::Validation("The file could not be read. Choose it again.".into()))?;
    if bytes.is_empty() {
        return Err(AppError::Validation("The file is empty.".into()));
    }
    if bytes.len() > MAX_FILE_BYTES {
        return Err(AppError::Validation("The file is larger than 100 MB. Split it into smaller files.".into()));
    }
    let (mut records, parse_errors, headers, mapping, mapping_fields) = match req.format.as_str() {
        "marc" | "marcxml" => {
            let koha = match req.profile.as_str() {
                "koha" => true,
                "generic" => false,
                _ => return Err(AppError::Validation("Choose Koha or a generic MARC file.".into())),
            };
            let (marc_records, errs) = if req.format == "marcxml" {
                (marcxml::parse(&bytes)?, Vec::new())
            } else { marc::read_iso2709(&bytes) };
            let recs = marc_records.iter().enumerate().map(|(i, m)| marc::to_import_record(m, i + 1, koha)).collect::<Vec<_>>();
            (recs, errs, None, None, None)
        }
        "csv" | "xlsx" => {
            let r = if req.format == "xlsx" { xlsx_import::parse(&bytes, req.mapping.as_ref())? }
                else { csv_import::parse_csv(&bytes, req.mapping.as_ref()) };
            let fields = csv_import::FIELDS.iter().map(|(k, _)| (*k).to_string()).collect::<Vec<_>>();
            (r.records, r.errors, Some(r.headers), Some(r.mapping), Some(fields))
        }
        "json" => {
            let records = json_import::parse(&bytes)?;
            (records, Vec::new(), None, None, None)
        }
        _ => return Err(AppError::Validation("Unsupported file type. Use a MARC (.mrc), MARCXML, XLSX, CSV or JSON file.".into())),
    };
    if records.is_empty() && parse_errors.is_empty() {
        return Err(AppError::Validation("No records were found in this file.".into()));
    }
    let (totals, new_lookups) = analyze(conn, &mut records)?;

    let mut idx: Vec<usize> = (0..records.len().min(30)).collect();
    idx.extend((30..records.len()).filter(|&i| !records[i].errors.is_empty() || !records[i].warnings.is_empty()).take(70));
    let sample = idx.into_iter().map(|i| records[i].clone()).collect();

    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let token = format!("{nanos:x}");
    let preview = Preview {
        token: token.clone(), file_name: req.file_name.clone(), format: req.format.clone(), profile: req.profile.clone(),
        totals, new_lookups, sample, parse_errors, headers, mapping, mapping_fields,
    };
    Ok((Pending { token, source: req.file_name.clone(), records }, preview))
}

// ---------- commit ----------

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DuplicatePolicy {
    /// Leave records whose ISBN is already in the catalogue alone.
    Skip,
    /// Add the file's new copies to the existing record.
    Merge,
    /// Create a second record anyway.
    New,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitOptions {
    pub duplicate_policy: DuplicatePolicy,
    /// Create locations, collections and item categories named in the file when they do not exist.
    pub create_missing: bool,
    /// Item category for copies that name none (or whose category is not created).
    pub default_item_category_id: Option<i64>,
    /// Use the barcode as the accession number when the file gives no accession number (keeps old identifiers).
    pub barcode_as_accession: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Problem {
    pub row: usize,
    pub message: String,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub created_records: usize,
    pub merged_records: usize,
    pub skipped_records: usize,
    pub failed_records: usize,
    pub copies_added: usize,
    pub copies_skipped: usize,
    pub created_lookups: usize,
    pub problems: Vec<Problem>,
}

const MAX_PROBLEMS: usize = 500;

#[derive(Default)]
struct Lookups {
    locations: HashMap<String, i64>,
    collections: HashMap<String, i64>,
    categories: HashMap<String, i64>,
    shelves: HashMap<(i64, String), i64>,
}

fn load_map(conn: &Connection, sql: &str) -> AppResult<HashMap<String, i64>> {
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?.to_lowercase(), r.get::<_, i64>(1)?)))?.collect::<Result<Vec<_>, _>>()?;
    Ok(rows.into_iter().collect())
}

fn find_or_create(conn: &Connection, map: &mut HashMap<String, i64>, table: &str, name: &str, create: bool, created: &mut usize) -> AppResult<Option<i64>> {
    let key = name.trim().to_lowercase();
    if key.is_empty() {
        return Ok(None);
    }
    if let Some(id) = map.get(&key) {
        return Ok(Some(*id));
    }
    if !create {
        return Ok(None);
    }
    conn.execute(&format!("INSERT INTO {table} (name) VALUES (?1)"), [name.trim()])?;
    let id = conn.last_insert_rowid();
    map.insert(key, id);
    *created += 1;
    Ok(Some(id))
}

fn record_input(r: &ImportRecord, ddc_id: Option<i64>) -> RecordInput {
    RecordInput {
        id: None, title: r.title.clone(), subtitle: r.subtitle.clone(), edition: r.edition.clone(), publisher: r.publisher.clone(),
        publication_place: r.place.clone(), publication_year: r.year, language: r.language.clone(),
        physical_description: r.physical.clone(), pagination: r.pagination.clone(), series: r.series.clone(),
        abstract_text: r.abstract_text.clone(), notes: r.notes.clone(), isbn: r.isbn.clone(),
        // Imported classification numbers come from MARC 082 (Dewey) or a CSV "DDC" column.
        classification_scheme_id: r.classification.as_ref().and(ddc_id), classification_number: r.classification.clone(),
        keywords: r.keywords.clone(), contributors: r.contributors.clone(), subjects: r.subjects.clone(),
    }
}

/// Writes the analysed records. Everything happens in one transaction; a record that fails is rolled back
/// on its own (savepoint) and reported, the rest continue.
pub fn commit(conn: &mut Connection, actor: Option<&Session>, records: &[ImportRecord], opts: &CommitOptions, source: &str) -> AppResult<Report> {
    if let Some(id) = opts.default_item_category_id {
        let ok: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM item_categories WHERE id = ?1 AND archived_at IS NULL)", [id], |r| r.get(0))?;
        if !ok {
            return Err(AppError::Validation("The default item category no longer exists.".into()));
        }
    }
    let needs_default = records.iter().any(|r| r.copies.iter().any(|c| c.skip_reason.is_none() && c.item_category.is_none()));
    if needs_default && opts.default_item_category_id.is_none() {
        return Err(AppError::Validation("Choose an item category for copies that do not name one.".into()));
    }
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let ddc_id: Option<i64> = tx.query_row("SELECT id FROM classification_schemes WHERE code = 'ddc'", [], |r| r.get(0)).ok();
    let mut lk = Lookups {
        locations: load_map(&tx, "SELECT name, id FROM locations")?,
        collections: load_map(&tx, "SELECT name, id FROM collections")?,
        categories: load_map(&tx, "SELECT name, id FROM item_categories WHERE archived_at IS NULL")?,
        shelves: HashMap::new(),
    };
    {
        let mut stmt = tx.prepare("SELECT location_id, code, id FROM shelves")?;
        for row in stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)?)))? {
            let (l, c, i) = row?;
            lk.shelves.insert((l, c.to_lowercase()), i);
        }
    }
    let mut report = Report::default();
    let problem = |report: &mut Report, row: usize, message: String| {
        if report.problems.len() < MAX_PROBLEMS {
            report.problems.push(Problem { row, message });
        }
    };

    for r in records {
        if !r.errors.is_empty() {
            report.failed_records += 1;
            problem(&mut report, r.row, r.errors.join(" "));
            continue;
        }
        let existing = r.duplicate_of;
        if existing.is_some() && opts.duplicate_policy == DuplicatePolicy::Skip {
            report.skipped_records += 1;
            continue;
        }
        tx.execute_batch("SAVEPOINT import_record")?;
        let attempt = (|| -> AppResult<(i64, bool, usize, usize, usize)> {
            let mut created_lookups = 0usize;
            let scheme_id = match r.classification_scheme.as_deref() {
                Some(code) => Some(tx.query_row("SELECT id FROM classification_schemes WHERE code = ?1", [code], |row| row.get::<_, i64>(0))
                    .map_err(|_| AppError::Validation("The imported classification scheme is not configured.".into()))?),
                None => ddc_id,
            };
            let (record_id, merged) = match (existing, opts.duplicate_policy) {
                (Some(id), DuplicatePolicy::Merge) => (id, true),
                _ => (catalogue::save_record(&tx, actor, &record_input(r, scheme_id))?, false),
            };
            let (mut added, mut skipped) = (0usize, 0usize);
            for c in &r.copies {
                if c.skip_reason.is_some() {
                    skipped += 1;
                    continue;
                }
                let category = match &c.item_category {
                    Some(n) => find_or_create(&tx, &mut lk.categories, "item_categories", n, opts.create_missing, &mut created_lookups)?
                        .or(opts.default_item_category_id),
                    None => opts.default_item_category_id,
                };
                let Some(category) = category else {
                    return Err(AppError::Validation("No item category could be found or created for a copy.".into()));
                };
                let collection = match &c.collection {
                    Some(n) => find_or_create(&tx, &mut lk.collections, "collections", n, opts.create_missing, &mut created_lookups)?,
                    None => None,
                };
                let location = match &c.location {
                    Some(n) => find_or_create(&tx, &mut lk.locations, "locations", n, opts.create_missing, &mut created_lookups)?,
                    None => None,
                };
                let shelf = match (&c.shelf, location) {
                    (Some(code), Some(loc)) => {
                        let key = (loc, code.trim().to_lowercase());
                        match lk.shelves.get(&key) {
                            Some(id) => Some(*id),
                            None if opts.create_missing => {
                                tx.execute("INSERT INTO shelves (location_id, code) VALUES (?1, ?2)", (loc, code.trim()))?;
                                let id = tx.last_insert_rowid();
                                lk.shelves.insert(key, id);
                                created_lookups += 1;
                                Some(id)
                            }
                            None => None,
                        }
                    }
                    _ => None,
                };
                let acc = c.accession.clone().or_else(|| if opts.barcode_as_accession { c.barcode.clone() } else { None });
                let input = CopyBatchInput {
                    count: 1,
                    accession_numbers: acc.clone().map(|a| vec![a]),
                    barcodes: c.barcode.clone().map(|b| vec![b]),
                    item_category_id: category,
                    collection_id: collection,
                    location_id: location,
                    shelf_id: shelf,
                    call_number: c.call_number.clone().or_else(|| r.classification.clone()),
                    acquisition_date: c.acquisition_date.clone(),
                    acquisition_source: "purchase".into(),
                    price_minor: c.price_minor,
                    vendor: c.vendor.clone(),
                    invoice_ref: None,
                    notes: c.notes.clone(),
                };
                // A barcode given without an accession number needs its own accession number.
                let input = if input.accession_numbers.is_none() && input.barcodes.is_some() {
                    CopyBatchInput { accession_numbers: Some(vec![catalogue::next_accession(&tx)?]), ..input }
                } else {
                    input
                };
                match catalogue::add_copies(&tx, actor, record_id, &input) {
                    Ok(copies) => {
                        added += 1;
                        if let (Some(status), Some(copy)) = (&c.status, copies.first()) {
                            catalogue::set_copy_status(&tx, actor, copy.id, status, Some("Imported with this status"))?;
                        }
                    }
                    Err(AppError::Conflict(_)) | Err(AppError::Validation(_)) => skipped += 1,
                    Err(e) => return Err(e),
                }
            }
            Ok((record_id, merged, added, skipped, created_lookups))
        })();
        match attempt {
            Ok((_, merged, added, skipped, created)) => {
                tx.execute_batch("RELEASE import_record")?;
                if merged { report.merged_records += 1 } else { report.created_records += 1 }
                report.copies_added += added;
                report.copies_skipped += skipped;
                report.created_lookups += created;
            }
            Err(e) => {
                tx.execute_batch("ROLLBACK TO import_record; RELEASE import_record")?;
                report.failed_records += 1;
                problem(&mut report, r.row, e.user_message());
            }
        }
    }
    audit::record(
        &tx, actor, "import.completed", "bibliographic_records", None,
        &format!("Imported {source}: {} records, {} copies", report.created_records + report.merged_records, report.copies_added),
    )?;
    tx.commit()?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::services::catalogue::{SearchQuery, ROLES};
    use crate::services::config::{self, NamedInput, NamedKind};

    fn open() -> (tempfile::TempDir, Connection, i64) {
        let tmp = tempfile::tempdir().unwrap();
        let c = db::open(&tmp.path().join("i.db")).unwrap();
        let cat = config::save_named(&c, None, NamedKind::ItemCategory, &NamedInput { id: None, name: "General".into(), description: None }).unwrap();
        (tmp, c, cat)
    }

    fn opts(cat: Option<i64>) -> CommitOptions {
        CommitOptions { duplicate_policy: DuplicatePolicy::Skip, create_missing: true, default_item_category_id: cat, barcode_as_accession: true }
    }

    fn koha_file() -> Vec<u8> {
        let one = |title: &str, isbn: &str, author: &str, items: Vec<Vec<(char, &str)>>| {
            let mut fields: Vec<(&str, Vec<(char, &str)>)> = vec![
                ("020", vec![('a', isbn)]), ("100", vec![('a', author)]), ("245", vec![('a', title)]), ("082", vec![('a', "894.8123")]),
            ];
            for it in items { fields.push(("952", it)); }
            marc::tests::build(&fields, &[])
        };
        let mut file = one("Balyakalasakhi /", "9780306406157", "Basheer, Vaikom Muhammad,", vec![
            vec![('b', "MAIN"), ('c', "FIC"), ('p', "004217"), ('y', "BK"), ('8', "FICTION"), ('g', "250.00")],
            vec![('b', "MAIN"), ('p', "004218"), ('y', "BK"), ('1', "2026-01-01")],
        ]);
        file.extend(one("Wings of Fire", "", "Kalam, A. P. J.", vec![vec![('b', "ANNEX"), ('p', "004219"), ('y', "BK")]]));
        file
    }

    fn analyzed(c: &Connection, file: &[u8], koha: bool) -> Vec<ImportRecord> {
        let (recs, errs) = marc::read_iso2709(file);
        assert!(errs.is_empty(), "{errs:?}");
        let mut out: Vec<ImportRecord> = recs.iter().enumerate().map(|(i, m)| marc::to_import_record(m, i + 1, koha)).collect();
        analyze(c, &mut out).unwrap();
        out
    }

    #[test]
    fn price_parsing() {
        for (s, v) in [("250", Some(25000)), ("250.5", Some(25050)), ("1,250.00", Some(125000)), ("₹ 99.99", Some(9999)), ("Rs. 10", Some(1000)), ("", None), ("abc", None), ("1.234", None), ("1.2.3", None), (".", None), ("-5", Some(500))] {
            assert_eq!(parse_price_minor(s), v, "{s:?}");
        }
    }

    #[test]
    fn analysis_reports_new_lookups_and_problems_before_anything_is_saved() {
        let (_t, c, _) = open();
        let recs = analyzed(&c, &koha_file(), true);
        let (mut recs2, _) = (recs, ());
        let (totals, lookups) = analyze(&c, &mut recs2).unwrap();
        assert_eq!((totals.records, totals.copies), (2, 3));
        assert_eq!(lookups.locations, ["MAIN", "ANNEX"]);
        assert_eq!(lookups.collections, ["FICTION"]);
        assert_eq!(lookups.item_categories, ["BK"]);
        let n: i64 = c.query_row("SELECT count(*) FROM bibliographic_records", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 0, "analysis never writes");
    }

    #[test]
    fn imports_a_koha_file_with_copies_lookups_and_statuses() {
        let (_t, mut c, cat) = open();
        let recs = analyzed(&c, &koha_file(), true);
        let report = commit(&mut c, None, &recs, &opts(Some(cat)), "test.mrc").unwrap();
        assert_eq!((report.created_records, report.copies_added, report.failed_records), (2, 3, 0), "{report:?}");
        assert_eq!(report.created_lookups, 5, "MAIN, FIC shelf, FICTION, BK, ANNEX");

        let found = catalogue::search(&c, &SearchQuery { text: Some("balyakalasakhi".into()), page: 1, page_size: 10, ..Default::default() }).unwrap();
        let rec = catalogue::get_record(&c, found.rows[0].id).unwrap();
        assert_eq!(rec.isbn13.as_deref(), Some("9780306406157"));
        assert_eq!(rec.classification_number.as_deref(), Some("894.8123"));
        assert_eq!(rec.copies.len(), 2);
        let a = rec.copies.iter().find(|x| x.barcode == "004217").unwrap();
        assert_eq!((a.accession_no.as_str(), a.price_minor, a.status.as_str()), ("004217", Some(25000), "available"), "barcode doubles as accession number");
        assert_eq!(rec.copies.iter().find(|x| x.barcode == "004218").unwrap().status, "lost");
        assert!(a.shelf_id.is_some() && a.collection_id.is_some() && a.location_id.is_some());
        // Searchable straight away (full-text index updated).
        assert_eq!(catalogue::search(&c, &SearchQuery { text: Some("wings".into()), page: 1, page_size: 10, ..Default::default() }).unwrap().total, 1);
        assert_eq!(catalogue::search(&c, &SearchQuery { text: Some("004219".into()), page: 1, page_size: 10, ..Default::default() }).unwrap().total, 1);
        assert!(ROLES.contains(&"author"));
    }

    #[test]
    fn running_the_same_import_twice_creates_nothing_new() {
        let (_t, mut c, cat) = open();
        let first = analyzed(&c, &koha_file(), true);
        commit(&mut c, None, &first, &opts(Some(cat)), "a").unwrap();
        let again = analyzed(&c, &koha_file(), true);
        assert!(again.iter().flat_map(|r| &r.copies).all(|x| x.skip_reason.is_some()), "every barcode already exists");
        let report = commit(&mut c, None, &again, &opts(Some(cat)), "b").unwrap();
        assert_eq!(report.copies_added, 0);
        let n: i64 = c.query_row("SELECT count(*) FROM physical_items", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 3);
        // Wings of Fire has no ISBN, so it is imported again as a title (policy cannot detect it); Balyakalasakhi is a duplicate.
        assert_eq!(report.skipped_records, 1);
    }

    #[test]
    fn duplicate_policies_skip_merge_or_create_new() {
        let (_t, mut c, cat) = open();
        let first = analyzed(&c, &koha_file(), true);
        commit(&mut c, None, &first, &opts(Some(cat)), "a").unwrap();
        // A second file: same ISBN, one new copy.
        let file = marc::tests::build(&[("020", vec![('a', "9780306406157")]), ("245", vec![('a', "Balyakalasakhi")]), ("952", vec![('b', "MAIN"), ('p', "NEW-1"), ('y', "BK")])], &[]);
        let recs = analyzed(&c, &file, true);
        let merge = commit(&mut c, None, &recs, &CommitOptions { duplicate_policy: DuplicatePolicy::Merge, ..opts(Some(cat)) }, "m").unwrap();
        assert_eq!((merge.merged_records, merge.created_records, merge.copies_added), (1, 0, 1));
        let isbn_records: i64 = c.query_row("SELECT count(*) FROM bibliographic_records WHERE isbn13 = '9780306406157'", [], |r| r.get(0)).unwrap();
        assert_eq!(isbn_records, 1);
        let file2 = marc::tests::build(&[("020", vec![('a', "9780306406157")]), ("245", vec![('a', "Balyakalasakhi")]), ("952", vec![('b', "MAIN"), ('p', "NEW-2"), ('y', "BK")])], &[]);
        let recs2 = analyzed(&c, &file2, true);
        let new = commit(&mut c, None, &recs2, &CommitOptions { duplicate_policy: DuplicatePolicy::New, ..opts(Some(cat)) }, "n").unwrap();
        assert_eq!((new.created_records, new.copies_added), (1, 1));
        let isbn_records: i64 = c.query_row("SELECT count(*) FROM bibliographic_records WHERE isbn13 = '9780306406157'", [], |r| r.get(0)).unwrap();
        assert_eq!(isbn_records, 2);
    }

    #[test]
    fn barcode_clashes_inside_one_file_skip_only_the_clashing_copy() {
        let (_t, mut c, cat) = open();
        let file = marc::tests::build(&[("245", vec![('a', "A")]), ("952", vec![('p', "X1"), ('y', "BK")]), ("952", vec![('p', "x1"), ('y', "BK")])], &[]);
        let recs = analyzed(&c, &file, true);
        assert!(recs[0].copies[1].skip_reason.as_deref().unwrap().contains("more than once"));
        let report = commit(&mut c, None, &recs, &opts(Some(cat)), "x").unwrap();
        assert_eq!((report.created_records, report.copies_added, report.copies_skipped), (1, 1, 1));
    }

    #[test]
    fn records_without_a_title_fail_and_are_reported_while_others_import() {
        let (_t, mut c, cat) = open();
        let file = {
            let mut f = marc::tests::build(&[("100", vec![('a', "No Title, Author")])], &[]);
            f.extend(marc::tests::build(&[("245", vec![('a', "Good")])], &[]));
            f
        };
        let recs = analyzed(&c, &file, true);
        let report = commit(&mut c, None, &recs, &opts(Some(cat)), "x").unwrap();
        assert_eq!((report.created_records, report.failed_records), (1, 1));
        assert_eq!(report.problems[0].row, 1);
        assert!(report.problems[0].message.contains("No title"));
    }

    #[test]
    fn a_default_item_category_is_required_when_the_file_names_none() {
        let (_t, mut c, cat) = open();
        let file = marc::tests::build(&[("245", vec![('a', "A")]), ("952", vec![('p', "Z1")])], &[]);
        let recs = analyzed(&c, &file, true);
        assert!(matches!(commit(&mut c, None, &recs, &opts(None), "x"), Err(AppError::Validation(_))));
        let n: i64 = c.query_row("SELECT count(*) FROM bibliographic_records", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 0, "nothing was written");
        let ok = commit(&mut c, None, &recs, &opts(Some(cat)), "x").unwrap();
        assert_eq!(ok.copies_added, 1);
        assert!(matches!(commit(&mut c, None, &recs, &opts(Some(9999)), "x"), Err(AppError::Validation(_))));
    }

    #[test]
    fn without_create_missing_unknown_lookups_are_not_created() {
        let (_t, mut c, cat) = open();
        let recs = analyzed(&c, &koha_file(), true);
        let report = commit(&mut c, None, &recs, &CommitOptions { create_missing: false, ..opts(Some(cat)) }, "x").unwrap();
        assert_eq!(report.created_lookups, 0);
        let n: i64 = c.query_row("SELECT count(*) FROM locations", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 0);
        assert_eq!(report.copies_added, 3, "copies fall back to the default category and no location");
    }

    #[test]
    fn marcxml_preview_commit_and_malformed_rejection() {
        use base64::Engine;
        let (_t, mut c, cat) = open();
        let xml = r#"<collection xmlns="http://www.loc.gov/MARC21/slim"><record><datafield tag="245"><subfield code="a">XML Book</subfield></datafield><datafield tag="952"><subfield code="p">000123</subfield><subfield code="g">125.50</subfield></datafield></record></collection>"#;
        let mut req = PreviewRequest { file_name: "books.xml".into(), data_base64: base64::engine::general_purpose::STANDARD.encode(xml), format: "marcxml".into(), profile: "koha".into(), mapping: None };
        let (pending, preview) = prepare(&c, &req).unwrap();
        assert_eq!((preview.totals.records, preview.totals.copies), (1, 1));
        let n: i64 = c.query_row("SELECT count(*) FROM bibliographic_records", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 0);
        let report = commit(&mut c, None, &pending.records, &opts(Some(cat)), "books.xml").unwrap();
        assert_eq!((report.created_records, report.copies_added), (1, 1));
        let barcode: String = c.query_row("SELECT barcode FROM physical_items", [], |r| r.get(0)).unwrap();
        assert_eq!(barcode, "000123");
        req.data_base64 = base64::engine::general_purpose::STANDARD.encode(format!("{xml}<broken>"));
        assert!(prepare(&c, &req).is_err());
        let n: i64 = c.query_row("SELECT count(*) FROM bibliographic_records", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 1);
    }

    #[test]
    fn csv_import_end_to_end() {
        let (_t, mut c, cat) = open();
        let csv = "Title,Author,ISBN,Year,Barcode,Location,Price\nBook One,\"Roy, A\",9780306406157,2001,C1,Main,100\nBook One,\"Roy, A\",9780306406157,2001,C2,Main,100\nBook Two,Someone,not-an-isbn,1999,C3,Annex,\n";
        let parsed = csv_import::parse_csv(csv.as_bytes(), None);
        let mut recs = parsed.records;
        analyze(&c, &mut recs).unwrap();
        assert!(recs[1].warnings.iter().any(|w| w.contains("not valid")), "{:?}", recs[1].warnings);
        let report = commit(&mut c, None, &recs, &opts(Some(cat)), "books.csv").unwrap();
        assert_eq!((report.created_records, report.copies_added), (2, 3));
        let one = catalogue::search(&c, &SearchQuery { text: Some("C2".into()), page: 1, page_size: 5, ..Default::default() }).unwrap();
        assert_eq!(one.rows[0].title, "Book One");
        assert_eq!(one.rows[0].total_copies, 2);
    }

    #[test]
    fn the_whole_import_is_audited_once() {
        let (_t, mut c, cat) = open();
        let recs = analyzed(&c, &koha_file(), true);
        commit(&mut c, None, &recs, &opts(Some(cat)), "x").unwrap();
        let n: i64 = c.query_row("SELECT count(*) FROM audit_events WHERE action = 'import.completed'", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 1);
    }
}
