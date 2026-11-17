//! Catalogue: bibliographic records, contributors, subjects, physical copies, accession numbers,
//! duplicate detection and search. Records are archived, never deleted; copies change status.
//! Every write also updates the full-text index in the same transaction (caller's transaction).

use crate::domain::{accession, isbn, validate};
use crate::error::{AppError, AppResult};
use crate::services::{audit, auth::Session, settings};
use rusqlite::{named_params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const ROLES: [&str; 6] = ["author", "additional_author", "editor", "translator", "illustrator", "other"];
const SOURCES: [&str; 5] = ["purchase", "donation", "transfer", "exchange", "other"];
/// Statuses staff may set by hand. `on_loan` and `reserved_hold` are owned by circulation.
const MANUAL_STATUSES: [&str; 6] = ["available", "in_repair", "lost", "damaged", "withdrawn", "in_transit"];
const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%SZ','now')";

// ---------- DTOs ----------

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ContributorDto {
    pub name: String,
    pub role: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordInput {
    pub id: Option<i64>,
    pub title: String,
    pub subtitle: Option<String>,
    pub edition: Option<String>,
    pub publisher: Option<String>,
    pub publication_place: Option<String>,
    pub publication_year: Option<i64>,
    pub language: Option<String>,
    pub physical_description: Option<String>,
    pub pagination: Option<String>,
    pub series: Option<String>,
    #[serde(rename = "abstract")]
    pub abstract_text: Option<String>,
    pub notes: Option<String>,
    /// ISBN-10 or ISBN-13; both forms are stored when they can be derived.
    pub isbn: Option<String>,
    pub classification_scheme_id: Option<i64>,
    pub classification_number: Option<String>,
    pub keywords: Option<String>,
    pub contributors: Vec<ContributorDto>,
    pub subjects: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Copy {
    pub id: i64,
    pub record_id: i64,
    pub accession_no: String,
    pub barcode: String,
    pub item_category_id: i64,
    pub collection_id: Option<i64>,
    pub location_id: Option<i64>,
    pub shelf_id: Option<i64>,
    pub call_number: Option<String>,
    pub acquisition_date: Option<String>,
    pub acquisition_source: String,
    pub price_minor: Option<i64>,
    pub vendor: Option<String>,
    pub invoice_ref: Option<String>,
    pub notes: Option<String>,
    pub status: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordDetail {
    pub id: i64,
    pub title: String,
    pub subtitle: Option<String>,
    pub edition: Option<String>,
    pub publisher: Option<String>,
    pub publication_place: Option<String>,
    pub publication_year: Option<i64>,
    pub language: Option<String>,
    pub physical_description: Option<String>,
    pub pagination: Option<String>,
    pub series: Option<String>,
    #[serde(rename = "abstract")]
    pub abstract_text: Option<String>,
    pub notes: Option<String>,
    pub isbn10: Option<String>,
    pub isbn13: Option<String>,
    pub classification_scheme_id: Option<i64>,
    pub classification_number: Option<String>,
    pub keywords: Option<String>,
    pub archived: bool,
    pub contributors: Vec<ContributorDto>,
    pub subjects: Vec<String>,
    pub copies: Vec<Copy>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordSummary {
    pub id: i64,
    pub title: String,
    pub subtitle: Option<String>,
    pub authors: String,
    pub publisher: Option<String>,
    pub publication_year: Option<i64>,
    pub isbn13: Option<String>,
    pub classification_number: Option<String>,
    pub classification_scheme_id: Option<i64>,
    pub total_copies: i64,
    pub available_copies: i64,
    pub archived: bool,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct SearchQuery {
    pub text: Option<String>,
    pub collection_id: Option<i64>,
    pub only_available: bool,
    pub include_archived: bool,
    /// "relevance" (default with text), "title" (default without), "year", "recent".
    pub sort: Option<String>,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub rows: Vec<RecordSummary>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyBatchInput {
    pub count: i64,
    pub accession_numbers: Option<Vec<String>>,
    pub barcodes: Option<Vec<String>>,
    pub item_category_id: i64,
    pub collection_id: Option<i64>,
    pub location_id: Option<i64>,
    pub shelf_id: Option<i64>,
    pub call_number: Option<String>,
    pub acquisition_date: Option<String>,
    pub acquisition_source: String,
    pub price_minor: Option<i64>,
    pub vendor: Option<String>,
    pub invoice_ref: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyUpdateInput {
    pub accession_no: String,
    pub barcode: String,
    pub item_category_id: i64,
    pub collection_id: Option<i64>,
    pub location_id: Option<i64>,
    pub shelf_id: Option<i64>,
    pub call_number: Option<String>,
    pub acquisition_date: Option<String>,
    pub acquisition_source: String,
    pub price_minor: Option<i64>,
    pub vendor: Option<String>,
    pub invoice_ref: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct CopyQuery {
    pub text: Option<String>,
    pub status: Option<String>,
    pub collection_id: Option<i64>,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyRow {
    pub id: i64,
    pub record_id: i64,
    pub accession_no: String,
    pub barcode: String,
    pub title: String,
    pub call_number: Option<String>,
    pub status: String,
    pub collection: Option<String>,
    pub location: Option<String>,
    pub shelf: Option<String>,
    pub acquisition_date: Option<String>,
    pub price_minor: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyPage {
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub rows: Vec<CopyRow>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateProbe {
    pub id: Option<i64>,
    pub title: String,
    pub isbn: Option<String>,
    pub first_author: Option<String>,
    pub publication_year: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateHit {
    pub id: i64,
    pub title: String,
    pub authors: String,
    pub publication_year: Option<i64>,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccessionFormat {
    pub prefix: String,
    pub padding: u32,
    pub next: i64,
}

// ---------- small helpers ----------

fn clean(label: &str, v: &Option<String>, max: usize) -> AppResult<Option<String>> {
    validate::optional(label, v.as_deref(), max)
}

fn collapse(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn unique_violation(e: rusqlite::Error, message: impl Into<String>) -> AppError {
    match &e {
        rusqlite::Error::SqliteFailure(f, _) if matches!(f.extended_code, 2067 | 1555) => AppError::Conflict(message.into()),
        rusqlite::Error::SqliteFailure(f, _) if f.extended_code == 787 => {
            AppError::Validation("One of the choices no longer exists. Reload and try again.".into())
        }
        _ => AppError::Database(e),
    }
}

// ---------- accession numbers ----------

pub fn accession_format(conn: &Connection) -> AppResult<AccessionFormat> {
    let get = |k: &str| settings::get(conn, k);
    let prefix = get("accession.prefix")?.and_then(|v| serde_json::from_str::<String>(&v).ok()).unwrap_or_default();
    let padding = get("accession.padding")?.and_then(|v| v.parse::<u32>().ok()).unwrap_or(6);
    let next = get("accession.next")?.and_then(|v| v.parse::<i64>().ok()).unwrap_or(1);
    Ok(AccessionFormat { prefix, padding, next })
}

pub fn set_accession_format(conn: &Connection, actor: Option<&Session>, f: &AccessionFormat) -> AppResult<()> {
    let prefix = accession::validate_prefix(&f.prefix)?;
    validate::in_range("Number of digits", f.padding as i64, 3, 12)?;
    validate::in_range("Next number", f.next, 1, 999_999_999_999)?;
    settings::set(conn, "accession.prefix", &serde_json::to_string(&prefix).unwrap_or_default())?;
    settings::set(conn, "accession.padding", &f.padding.to_string())?;
    settings::set(conn, "accession.next", &f.next.to_string())?;
    audit::record(conn, actor, "config.accession_format", "settings", None, "Accession number format changed")?;
    Ok(())
}

/// Takes the next unused accession number and advances the counter. Skips numbers already in use
/// (for example typed in by hand), so it never fails on a collision.
pub fn next_accession(conn: &Connection) -> AppResult<String> {
    let f = accession_format(conn)?;
    let mut n = f.next;
    loop {
        let candidate = accession::format(&f.prefix, f.padding, n);
        n += 1;
        let taken: bool =
            conn.query_row("SELECT EXISTS(SELECT 1 FROM physical_items WHERE accession_no = ?1)", [&candidate], |r| r.get(0))?;
        if !taken {
            settings::set(conn, "accession.next", &n.to_string())?;
            return Ok(candidate);
        }
    }
}

// ---------- records ----------

fn upsert_contributor(conn: &Connection, name: &str) -> AppResult<i64> {
    conn.execute("INSERT INTO contributors (name) VALUES (?1) ON CONFLICT(name) DO NOTHING", [name])?;
    Ok(conn.query_row("SELECT id FROM contributors WHERE name = ?1", [name], |r| r.get(0))?)
}

fn upsert_subject(conn: &Connection, heading: &str) -> AppResult<i64> {
    conn.execute("INSERT INTO subjects (heading) VALUES (?1) ON CONFLICT(heading) DO NOTHING", [heading])?;
    Ok(conn.query_row("SELECT id FROM subjects WHERE heading = ?1", [heading], |r| r.get(0))?)
}

/// Rewrites the full-text row for one record from its current data.
pub fn reindex(conn: &Connection, record_id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM catalogue_fts WHERE rowid = ?1", [record_id])?;
    conn.execute(
        "INSERT INTO catalogue_fts (rowid, title, subtitle, authors, subjects, publisher, keywords, identifiers, classification)
         SELECT r.id, r.title, COALESCE(r.subtitle,''),
                COALESCE((SELECT group_concat(c.name, ' ') FROM record_contributors rc JOIN contributors c ON c.id = rc.contributor_id WHERE rc.record_id = r.id), ''),
                COALESCE((SELECT group_concat(s.heading, ' ') FROM record_subjects rs JOIN subjects s ON s.id = rs.subject_id WHERE rs.record_id = r.id), ''),
                COALESCE(r.publisher,''), COALESCE(r.keywords,''),
                trim(COALESCE(r.isbn10,'') || ' ' || COALESCE(r.isbn13,'')),
                COALESCE(r.classification_number,'')
         FROM bibliographic_records r WHERE r.id = ?1",
        [record_id],
    )?;
    Ok(())
}

pub fn save_record(conn: &Connection, actor: Option<&Session>, input: &RecordInput) -> AppResult<i64> {
    let title = validate::required("Title", &input.title, 300)?;
    let subtitle = clean("Subtitle", &input.subtitle, 300)?;
    let edition = clean("Edition", &input.edition, 100)?;
    let publisher = clean("Publisher", &input.publisher, 200)?;
    let place = clean("Place of publication", &input.publication_place, 200)?;
    let language = clean("Language", &input.language, 60)?;
    let physical = clean("Physical description", &input.physical_description, 200)?;
    let pagination = clean("Pagination", &input.pagination, 100)?;
    let series = clean("Series", &input.series, 200)?;
    let abstract_text = clean("Abstract", &input.abstract_text, 5000)?;
    let notes = clean("Notes", &input.notes, 5000)?;
    let class_no = clean("Classification number", &input.classification_number, 100)?;
    let keywords = clean("Keywords", &input.keywords, 500)?;
    if let Some(y) = input.publication_year {
        validate::in_range("Publication year", y, 1, 2200)?;
    }
    let (isbn10, isbn13) = match validate::optional("ISBN", input.isbn.as_deref(), 40)? {
        None => (None, None),
        Some(raw) => match isbn::both_forms(&raw) {
            Some((i10, i13)) => (i10, Some(i13)),
            None => return Err(AppError::Validation("That ISBN is not valid. Check the digits.".into())),
        },
    };

    let mut contributors: Vec<ContributorDto> = Vec::new();
    for c in &input.contributors {
        let name = collapse(&c.name);
        if name.is_empty() {
            continue;
        }
        if name.chars().count() > 200 {
            return Err(AppError::Validation("A contributor name is too long.".into()));
        }
        if !ROLES.contains(&c.role.as_str()) {
            return Err(AppError::Validation("Unknown contributor role.".into()));
        }
        if !contributors.iter().any(|x| x.role == c.role && x.name.to_lowercase() == name.to_lowercase()) {
            contributors.push(ContributorDto { name, role: c.role.clone() });
        }
    }
    if contributors.len() > 50 {
        return Err(AppError::Validation("A record can have at most 50 contributors.".into()));
    }
    let mut subjects: Vec<String> = Vec::new();
    for s in &input.subjects {
        let h = collapse(s);
        if h.is_empty() {
            continue;
        }
        if h.chars().count() > 200 {
            return Err(AppError::Validation("A subject heading is too long.".into()));
        }
        if !subjects.iter().any(|x| x.to_lowercase() == h.to_lowercase()) {
            subjects.push(h);
        }
    }
    if subjects.len() > 50 {
        return Err(AppError::Validation("A record can have at most 50 subjects.".into()));
    }

    let id = match input.id {
        Some(id) => {
            let n = conn
                .execute(
                    &format!(
                        "UPDATE bibliographic_records SET title=?1, subtitle=?2, edition=?3, publisher=?4, publication_place=?5,
                         publication_year=?6, language=?7, physical_description=?8, pagination=?9, series=?10, abstract=?11,
                         notes=?12, isbn10=?13, isbn13=?14, classification_scheme_id=?15, classification_number=?16,
                         keywords=?17, updated_at={NOW} WHERE id=?18"
                    ),
                    rusqlite::params![
                        title, subtitle, edition, publisher, place, input.publication_year, language, physical, pagination,
                        series, abstract_text, notes, isbn10, isbn13, input.classification_scheme_id, class_no, keywords, id
                    ],
                )
                .map_err(|e| unique_violation(e, "Could not save the record."))?;
            if n == 0 {
                return Err(AppError::NotFound("That record no longer exists.".into()));
            }
            id
        }
        None => {
            conn.execute(
                "INSERT INTO bibliographic_records (title, subtitle, edition, publisher, publication_place, publication_year,
                   language, physical_description, pagination, series, abstract, notes, isbn10, isbn13,
                   classification_scheme_id, classification_number, keywords)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17)",
                rusqlite::params![
                    title, subtitle, edition, publisher, place, input.publication_year, language, physical, pagination,
                    series, abstract_text, notes, isbn10, isbn13, input.classification_scheme_id, class_no, keywords
                ],
            )
            .map_err(|e| unique_violation(e, "Could not save the record."))?;
            conn.last_insert_rowid()
        }
    };

    conn.execute("DELETE FROM record_contributors WHERE record_id = ?1", [id])?;
    for (pos, c) in contributors.iter().enumerate() {
        let cid = upsert_contributor(conn, &c.name)?;
        conn.execute(
            "INSERT INTO record_contributors (record_id, contributor_id, role, position) VALUES (?1, ?2, ?3, ?4)",
            (id, cid, &c.role, pos as i64),
        )?;
    }
    conn.execute("DELETE FROM record_subjects WHERE record_id = ?1", [id])?;
    for s in &subjects {
        let sid = upsert_subject(conn, s)?;
        conn.execute("INSERT INTO record_subjects (record_id, subject_id) VALUES (?1, ?2)", (id, sid))?;
    }
    reindex(conn, id)?;
    audit::record(conn, actor, "catalogue.record_saved", "bibliographic_records", Some(id), &format!("Saved record \"{title}\""))?;
    Ok(id)
}

fn map_copy(r: &rusqlite::Row<'_>) -> rusqlite::Result<Copy> {
    Ok(Copy {
        id: r.get(0)?, record_id: r.get(1)?, accession_no: r.get(2)?, barcode: r.get(3)?, item_category_id: r.get(4)?,
        collection_id: r.get(5)?, location_id: r.get(6)?, shelf_id: r.get(7)?, call_number: r.get(8)?,
        acquisition_date: r.get(9)?, acquisition_source: r.get(10)?, price_minor: r.get(11)?, vendor: r.get(12)?,
        invoice_ref: r.get(13)?, notes: r.get(14)?, status: r.get(15)?,
    })
}

const COPY_COLS: &str = "id, record_id, accession_no, barcode, item_category_id, collection_id, location_id, shelf_id,
    call_number, acquisition_date, acquisition_source, price_minor, vendor, invoice_ref, notes, status";

pub fn get_copy(conn: &Connection, id: i64) -> AppResult<Copy> {
    conn.query_row(&format!("SELECT {COPY_COLS} FROM physical_items WHERE id = ?1"), [id], map_copy)
        .optional()?
        .ok_or_else(|| AppError::NotFound("That copy no longer exists.".into()))
}

pub fn get_record(conn: &Connection, id: i64) -> AppResult<RecordDetail> {
    let mut rec = conn
        .query_row(
            "SELECT id, title, subtitle, edition, publisher, publication_place, publication_year, language,
                    physical_description, pagination, series, abstract, notes, isbn10, isbn13, classification_scheme_id,
                    classification_number, keywords, archived_at IS NOT NULL
             FROM bibliographic_records WHERE id = ?1",
            [id],
            |r| {
                Ok(RecordDetail {
                    id: r.get(0)?, title: r.get(1)?, subtitle: r.get(2)?, edition: r.get(3)?, publisher: r.get(4)?,
                    publication_place: r.get(5)?, publication_year: r.get(6)?, language: r.get(7)?,
                    physical_description: r.get(8)?, pagination: r.get(9)?, series: r.get(10)?, abstract_text: r.get(11)?,
                    notes: r.get(12)?, isbn10: r.get(13)?, isbn13: r.get(14)?, classification_scheme_id: r.get(15)?,
                    classification_number: r.get(16)?, keywords: r.get(17)?, archived: r.get(18)?,
                    contributors: vec![], subjects: vec![], copies: vec![],
                })
            },
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound("That record no longer exists.".into()))?;
    rec.contributors = conn
        .prepare(
            "SELECT c.name, rc.role FROM record_contributors rc JOIN contributors c ON c.id = rc.contributor_id
             WHERE rc.record_id = ?1 ORDER BY rc.position",
        )?
        .query_map([id], |r| Ok(ContributorDto { name: r.get(0)?, role: r.get(1)? }))?
        .collect::<Result<_, _>>()?;
    rec.subjects = conn
        .prepare("SELECT s.heading FROM record_subjects rs JOIN subjects s ON s.id = rs.subject_id WHERE rs.record_id = ?1 ORDER BY s.heading")?
        .query_map([id], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    rec.copies = conn
        .prepare(&format!("SELECT {COPY_COLS} FROM physical_items WHERE record_id = ?1 ORDER BY accession_no"))?
        .query_map([id], map_copy)?
        .collect::<Result<_, _>>()?;
    Ok(rec)
}

pub fn set_record_archived(conn: &Connection, actor: Option<&Session>, id: i64, archived: bool) -> AppResult<()> {
    if archived {
        let busy: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM physical_items WHERE record_id = ?1 AND status IN ('on_loan','reserved_hold'))",
            [id],
            |r| r.get(0),
        )?;
        if busy {
            return Err(AppError::Conflict("Some copies are on loan or reserved. Return them before archiving this record.".into()));
        }
    }
    let n = conn.execute(
        &format!("UPDATE bibliographic_records SET archived_at = CASE WHEN ?1 THEN {NOW} ELSE NULL END, updated_at = {NOW} WHERE id = ?2"),
        (archived, id),
    )?;
    if n == 0 {
        return Err(AppError::NotFound("That record no longer exists.".into()));
    }
    audit::record(conn, actor, if archived { "catalogue.record_archived" } else { "catalogue.record_restored" }, "bibliographic_records", Some(id), if archived { "Archived record" } else { "Restored record" })?;
    Ok(())
}

// ---------- duplicates ----------

pub fn find_duplicates(conn: &Connection, probe: &DuplicateProbe) -> AppResult<Vec<DuplicateHit>> {
    let mut hits: Vec<DuplicateHit> = Vec::new();
    let exclude = probe.id.unwrap_or(-1);
    let authors_of = |id: i64| -> AppResult<String> {
        Ok(conn
            .query_row(
                "SELECT COALESCE(group_concat(name, '; '), '') FROM (SELECT c.name AS name FROM record_contributors rc
                 JOIN contributors c ON c.id = rc.contributor_id WHERE rc.record_id = ?1 AND rc.role = 'author' ORDER BY rc.position)",
                [id],
                |r| r.get(0),
            )?)
    };
    let push = |id: i64, title: String, year: Option<i64>, reason: &str, hits: &mut Vec<DuplicateHit>| -> AppResult<()> {
        if !hits.iter().any(|h| h.id == id) {
            hits.push(DuplicateHit { id, title, authors: authors_of(id)?, publication_year: year, reason: reason.into() });
        }
        Ok(())
    };

    if let Some((i10, i13)) = probe.isbn.as_deref().and_then(isbn::both_forms) {
        let rows = conn
            .prepare("SELECT id, title, publication_year FROM bibliographic_records WHERE archived_at IS NULL AND id <> ?1 AND (isbn13 = ?2 OR (?3 IS NOT NULL AND isbn10 = ?3))")?
            .query_map((exclude, &i13, &i10), |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, Option<i64>>(2)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        for (id, t, y) in rows {
            push(id, t, y, "Same ISBN", &mut hits)?;
        }
    }
    if let Some(author) = probe.first_author.as_deref().map(collapse).filter(|a| !a.is_empty()) {
        let rows = conn
            .prepare(
                "SELECT r.id, r.title, r.publication_year FROM bibliographic_records r
                 WHERE r.archived_at IS NULL AND r.id <> ?1 AND r.title = ?2
                   AND (?3 IS NULL OR r.publication_year IS NULL OR r.publication_year = ?3)
                   AND EXISTS (SELECT 1 FROM record_contributors rc JOIN contributors c ON c.id = rc.contributor_id
                               WHERE rc.record_id = r.id AND rc.role = 'author' AND c.name = ?4)",
            )?
            .query_map((exclude, collapse(&probe.title), probe.publication_year, &author), |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, Option<i64>>(2)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        for (id, t, y) in rows {
            push(id, t, y, "Same title, author and year", &mut hits)?;
        }
    }
    Ok(hits)
}

// ---------- search ----------

/// Turns what the user typed into a safe FTS5 query: every word is quoted (so punctuation can never
/// be read as query syntax) and matched as a prefix; all words must match.
pub fn fts_expression(text: &str) -> Option<String> {
    let parts: Vec<String> = text
        .split_whitespace()
        .map(|w| w.chars().filter(|c| *c != '"').collect::<String>())
        .filter(|w| w.chars().any(|c| c.is_alphanumeric()))
        .map(|w| format!("\"{w}\"*"))
        .collect();
    if parts.is_empty() { None } else { Some(parts.join(" ")) }
}

/// Exact lookups first: an ISBN, an accession number or a barcode (what a scanner types) goes straight to its record.
fn exact_record_ids(conn: &Connection, text: &str) -> AppResult<Option<Vec<i64>>> {
    let t = text.trim();
    if t.is_empty() {
        return Ok(None);
    }
    let mut ids: Vec<i64> = Vec::new();
    if let Some((i10, i13)) = isbn::both_forms(t) {
        let mut stmt = conn.prepare("SELECT id FROM bibliographic_records WHERE isbn13 = ?1 OR (?2 IS NOT NULL AND isbn10 = ?2)")?;
        ids.extend(stmt.query_map((&i13, &i10), |r| r.get::<_, i64>(0))?.collect::<Result<Vec<_>, _>>()?);
    }
    let mut stmt = conn.prepare("SELECT record_id FROM physical_items WHERE accession_no = ?1 OR barcode = ?1")?;
    ids.extend(stmt.query_map([t], |r| r.get::<_, i64>(0))?.collect::<Result<Vec<_>, _>>()?);
    ids.sort_unstable();
    ids.dedup();
    Ok(if ids.is_empty() { None } else { Some(ids) })
}

pub fn search(conn: &Connection, q: &SearchQuery) -> AppResult<SearchResult> {
    let text = q.text.as_deref().map(str::trim).filter(|t| !t.is_empty());
    let exact = match text { Some(t) => exact_record_ids(conn, t)?, None => None };
    let fts = if exact.is_some() { None } else { text.and_then(fts_expression) };
    let page_size = if q.page_size <= 0 { 25 } else { q.page_size.min(100) };
    let page = q.page.max(1);
    let offset = (page - 1) * page_size;

    // A text that produced no usable words (only punctuation) matches nothing rather than everything.
    let nothing = text.is_some() && exact.is_none() && fts.is_none();
    let ids_json = exact.as_ref().map(|v| serde_json::to_string(v).unwrap_or_else(|_| "[]".into()));

    let join = if fts.is_some() {
        "LEFT JOIN (SELECT rowid AS rid, rank FROM catalogue_fts WHERE catalogue_fts MATCH :fts) f ON f.rid = r.id"
    } else {
        ""
    };
    let filter_fts = if fts.is_some() { "AND f.rid IS NOT NULL" } else { "" };
    let filter_ids = if ids_json.is_some() { "AND r.id IN (SELECT value FROM json_each(:ids))" } else { "" };
    let nothing_clause = if nothing { "AND 0" } else { "" };
    let where_sql = format!(
        "WHERE (:archived = 1 OR r.archived_at IS NULL) {filter_fts} {filter_ids} {nothing_clause}
         AND (:collection IS NULL OR EXISTS (SELECT 1 FROM physical_items i WHERE i.record_id = r.id AND i.collection_id = :collection))
         AND (:only_available = 0 OR EXISTS (SELECT 1 FROM physical_items i WHERE i.record_id = r.id AND i.status = 'available'))"
    );

    let order = match (q.sort.as_deref(), fts.is_some()) {
        (Some("year"), _) => "r.publication_year DESC, r.title COLLATE NOCASE",
        (Some("recent"), _) => "r.id DESC",
        (Some("title"), _) | (None, false) | (Some("relevance"), false) | (Some(_), false) => "r.title COLLATE NOCASE, r.id",
        (_, true) => "f.rank, r.title COLLATE NOCASE",
    };

    // Named parameters present in the SQL must all be bound; unused ones are fine to omit only if absent, so build the list.
    let sql_rows = format!(
        "SELECT r.id, r.title, r.subtitle,
                COALESCE((SELECT group_concat(name, '; ') FROM (SELECT c.name AS name FROM record_contributors rc
                          JOIN contributors c ON c.id = rc.contributor_id WHERE rc.record_id = r.id AND rc.role = 'author'
                          ORDER BY rc.position)), ''),
                r.publisher, r.publication_year, r.isbn13, r.classification_number,
                (SELECT count(*) FROM physical_items i WHERE i.record_id = r.id AND i.status <> 'withdrawn'),
                (SELECT count(*) FROM physical_items i WHERE i.record_id = r.id AND i.status = 'available'),
                r.archived_at IS NOT NULL, r.classification_scheme_id
         FROM bibliographic_records r {join} {where_sql} ORDER BY {order} LIMIT :limit OFFSET :offset"
    );
    let sql_count = format!("SELECT count(*) FROM bibliographic_records r {join} {where_sql}");

    let archived = q.include_archived as i64;
    let only_available = q.only_available as i64;
    let mut base: Vec<(&str, &dyn rusqlite::ToSql)> = vec![
        (":archived", &archived),
        (":collection", &q.collection_id),
        (":only_available", &only_available),
    ];
    if let Some(f) = &fts { base.push((":fts", f)); }
    if let Some(j) = &ids_json { base.push((":ids", j)); }

    let total: i64 = conn.query_row(&sql_count, base.as_slice(), |r| r.get(0))?;
    let mut with_page = base;
    with_page.push((":limit", &page_size));
    with_page.push((":offset", &offset));
    let rows = conn
        .prepare(&sql_rows)?
        .query_map(with_page.as_slice(), |r| {
            Ok(RecordSummary {
                id: r.get(0)?, title: r.get(1)?, subtitle: r.get(2)?, authors: r.get(3)?, publisher: r.get(4)?,
                publication_year: r.get(5)?, isbn13: r.get(6)?, classification_number: r.get(7)?,
                total_copies: r.get(8)?, available_copies: r.get(9)?, archived: r.get(10)?,
                classification_scheme_id: r.get(11)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(SearchResult { total, page, page_size, rows })
}

// ---------- copies ----------

fn exists_active(conn: &Connection, table: &str, id: i64, label: &str) -> AppResult<()> {
    let row: Option<bool> = conn
        .query_row(&format!("SELECT archived_at IS NULL FROM {table} WHERE id = ?1"), [id], |r| r.get(0))
        .optional()?;
    match row {
        Some(true) => Ok(()),
        Some(false) => Err(AppError::Validation(format!("That {label} is archived. Choose another."))),
        None => Err(AppError::Validation(format!("That {label} no longer exists."))),
    }
}

/// Validates shared copy fields and resolves location from shelf. Returns (location_id, acquisition_date, call_number, ...).
struct CopyFields {
    item_category_id: i64,
    collection_id: Option<i64>,
    location_id: Option<i64>,
    shelf_id: Option<i64>,
    call_number: Option<String>,
    acquisition_date: Option<String>,
    source: String,
    price_minor: Option<i64>,
    vendor: Option<String>,
    invoice_ref: Option<String>,
    notes: Option<String>,
}

#[allow(clippy::too_many_arguments)]
fn check_fields(
    conn: &Connection, item_category_id: i64, collection_id: Option<i64>, location_id: Option<i64>, shelf_id: Option<i64>,
    call_number: &Option<String>, acquisition_date: &Option<String>, source: &str, price_minor: Option<i64>,
    vendor: &Option<String>, invoice_ref: &Option<String>, notes: &Option<String>,
) -> AppResult<CopyFields> {
    exists_active(conn, "item_categories", item_category_id, "item category")?;
    if let Some(c) = collection_id { exists_active(conn, "collections", c, "collection")?; }
    if let Some(l) = location_id { exists_active(conn, "locations", l, "location")?; }
    let mut location_id = location_id;
    if let Some(s) = shelf_id {
        exists_active(conn, "shelves", s, "shelf")?;
        let shelf_loc: i64 = conn.query_row("SELECT location_id FROM shelves WHERE id = ?1", [s], |r| r.get(0))?;
        match location_id {
            Some(l) if l != shelf_loc => return Err(AppError::Validation("That shelf is in a different location.".into())),
            _ => location_id = Some(shelf_loc),
        }
    }
    if !SOURCES.contains(&source) {
        return Err(AppError::Validation("Unknown source of acquisition.".into()));
    }
    if let Some(p) = price_minor {
        if p < 0 { return Err(AppError::Validation("The price cannot be negative.".into())); }
    }
    let date = match acquisition_date.as_deref().map(str::trim).filter(|d| !d.is_empty()) {
        None => None,
        Some(d) => Some(validate::date("Acquisition date", d)?),
    };
    Ok(CopyFields {
        item_category_id, collection_id, location_id, shelf_id,
        call_number: clean("Call number", call_number, 100)?,
        acquisition_date: date,
        source: source.to_string(),
        price_minor,
        vendor: clean("Vendor", vendor, 200)?,
        invoice_ref: clean("Invoice reference", invoice_ref, 100)?,
        notes: clean("Notes", notes, 1000)?,
    })
}

pub(super) fn event(conn: &Connection, actor: Option<&Session>, item_id: i64, kind: &str, from: Option<&str>, to: Option<&str>, note: Option<&str>) -> AppResult<()> {
    conn.execute(
        "INSERT INTO item_events (item_id, event_type, from_status, to_status, note, staff_user_id) VALUES (?1,?2,?3,?4,?5,?6)",
        rusqlite::params![item_id, kind, from, to, note, actor.map(|a| a.user_id)],
    )?;
    Ok(())
}

pub fn add_copies(conn: &Connection, actor: Option<&Session>, record_id: i64, input: &CopyBatchInput) -> AppResult<Vec<Copy>> {
    validate::in_range("Number of copies", input.count, 1, 500)?;
    let exists: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM bibliographic_records WHERE id = ?1 AND archived_at IS NULL)", [record_id], |r| r.get(0))?;
    if !exists {
        return Err(AppError::NotFound("That record does not exist or is archived.".into()));
    }
    let f = check_fields(
        conn, input.item_category_id, input.collection_id, input.location_id, input.shelf_id, &input.call_number,
        &input.acquisition_date, &input.acquisition_source, input.price_minor, &input.vendor, &input.invoice_ref, &input.notes,
    )?;
    let count = input.count as usize;
    let numbers: Vec<String> = match &input.accession_numbers {
        Some(v) => {
            if v.len() != count {
                return Err(AppError::Validation("Give one accession number for each copy.".into()));
            }
            v.iter().map(|n| accession::validate_code("Accession number", n)).collect::<AppResult<_>>()?
        }
        None => (0..count).map(|_| next_accession(conn)).collect::<AppResult<_>>()?,
    };
    let barcodes: Vec<String> = match &input.barcodes {
        Some(v) => {
            if v.len() != count {
                return Err(AppError::Validation("Give one barcode for each copy.".into()));
            }
            v.iter().map(|n| accession::validate_code("Barcode", n)).collect::<AppResult<_>>()?
        }
        None => numbers.clone(),
    };
    // An accession number may equal its own barcode, so duplicates are checked within each list separately.
    for list in [&numbers, &barcodes] {
        let mut seen: HashSet<String> = HashSet::new();
        for c in list.iter() {
            if !seen.insert(c.to_lowercase()) {
                return Err(AppError::Conflict(format!("\"{c}\" appears more than once in this batch.")));
            }
        }
    }

    let mut out = Vec::with_capacity(count);
    for (acc, bar) in numbers.iter().zip(barcodes.iter()) {
        let clash: Option<String> = conn
            .query_row(
                "SELECT CASE WHEN accession_no = ?1 COLLATE NOCASE THEN 'Accession number ' || accession_no
                             ELSE 'Barcode ' || barcode END
                 FROM physical_items WHERE accession_no = ?1 OR barcode = ?2 LIMIT 1",
                (acc, bar),
                |r| r.get(0),
            )
            .optional()?;
        if let Some(what) = clash {
            return Err(AppError::Conflict(format!("{what} is already in use.")));
        }
        conn.execute(
            "INSERT INTO physical_items (record_id, accession_no, barcode, item_category_id, collection_id, location_id, shelf_id,
               call_number, acquisition_date, acquisition_source, price_minor, vendor, invoice_ref, notes)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
            rusqlite::params![
                record_id, acc, bar, f.item_category_id, f.collection_id, f.location_id, f.shelf_id, f.call_number,
                f.acquisition_date, f.source, f.price_minor, f.vendor, f.invoice_ref, f.notes
            ],
        )
        .map_err(|e| unique_violation(e, "That accession number or barcode is already in use."))?;
        let id = conn.last_insert_rowid();
        event(conn, actor, id, "acquired", None, Some("available"), None)?;
        out.push(get_copy(conn, id)?);
    }
    audit::record(conn, actor, "catalogue.copies_added", "bibliographic_records", Some(record_id), &format!("Added {count} cop{}", if count == 1 { "y" } else { "ies" }))?;
    Ok(out)
}

pub fn update_copy(conn: &Connection, actor: Option<&Session>, id: i64, input: &CopyUpdateInput) -> AppResult<()> {
    get_copy(conn, id)?;
    let acc = accession::validate_code("Accession number", &input.accession_no)?;
    let bar = accession::validate_code("Barcode", &input.barcode)?;
    let f = check_fields(
        conn, input.item_category_id, input.collection_id, input.location_id, input.shelf_id, &input.call_number,
        &input.acquisition_date, &input.acquisition_source, input.price_minor, &input.vendor, &input.invoice_ref, &input.notes,
    )?;
    conn.execute(
        &format!(
            "UPDATE physical_items SET accession_no=?1, barcode=?2, item_category_id=?3, collection_id=?4, location_id=?5,
             shelf_id=?6, call_number=?7, acquisition_date=?8, acquisition_source=?9, price_minor=?10, vendor=?11,
             invoice_ref=?12, notes=?13, updated_at={NOW} WHERE id=?14"
        ),
        rusqlite::params![acc, bar, f.item_category_id, f.collection_id, f.location_id, f.shelf_id, f.call_number,
            f.acquisition_date, f.source, f.price_minor, f.vendor, f.invoice_ref, f.notes, id],
    )
    .map_err(|e| unique_violation(e, "That accession number or barcode is already used by another copy."))?;
    event(conn, actor, id, "edited", None, None, None)?;
    audit::record(conn, actor, "catalogue.copy_saved", "physical_items", Some(id), &format!("Edited copy {acc}"))?;
    Ok(())
}

pub fn set_copy_status(conn: &Connection, actor: Option<&Session>, id: i64, status: &str, note: Option<&str>) -> AppResult<()> {
    if !MANUAL_STATUSES.contains(&status) {
        return Err(AppError::Validation("That status can only be set by issuing or returning the item.".into()));
    }
    let current = get_copy(conn, id)?;
    if matches!(current.status.as_str(), "on_loan" | "reserved_hold") {
        return Err(AppError::Conflict("This copy is on loan or reserved. Return it first.".into()));
    }
    let note = validate::optional("Note", note, 500)?;
    conn.execute(&format!("UPDATE physical_items SET status = ?1, updated_at = {NOW} WHERE id = ?2"), (status, id))?;
    event(conn, actor, id, "status_changed", Some(&current.status), Some(status), note.as_deref())?;
    audit::record(conn, actor, "catalogue.copy_status", "physical_items", Some(id), &format!("Copy {} changed from {} to {status}", current.accession_no, current.status))?;
    Ok(())
}

fn copy_search_pattern(q: &CopyQuery) -> Option<String> {
    q.text.as_deref().map(str::trim).filter(|t| !t.is_empty()).map(|t| {
        format!("%{}%", t.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_"))
    })
}

const COPY_SEARCH_FROM: &str = "FROM physical_items i JOIN bibliographic_records r ON r.id = i.record_id
                LEFT JOIN collections c ON c.id = i.collection_id
                LEFT JOIN locations l ON l.id = i.location_id
                LEFT JOIN shelves s ON s.id = i.shelf_id
                WHERE (:pattern IS NULL OR i.accession_no LIKE :pattern ESCAPE '\\' OR i.barcode LIKE :pattern ESCAPE '\\'
                       OR r.title LIKE :pattern ESCAPE '\\' OR i.call_number LIKE :pattern ESCAPE '\\')
                  AND (:status IS NULL OR i.status = :status)
                  AND (:collection IS NULL OR i.collection_id = :collection)";

pub fn list_copies(conn: &Connection, q: &CopyQuery) -> AppResult<CopyPage> {
    let page_size = if q.page_size <= 0 { 25 } else { q.page_size.min(100) };
    let page = q.page.max(1);
    let pattern = copy_search_pattern(q);
    let offset = (page - 1).saturating_mul(page_size);
    let total: i64 = conn.query_row(
        &format!("SELECT count(*) {COPY_SEARCH_FROM}"),
        named_params! { ":pattern": pattern, ":status": q.status, ":collection": q.collection_id },
        |r| r.get(0),
    )?;
    let rows = conn
        .prepare(&format!(
            "SELECT i.id, i.record_id, i.accession_no, i.barcode, r.title, i.call_number, i.status, c.name, l.name, s.code,
                    i.acquisition_date, i.price_minor {COPY_SEARCH_FROM}
             ORDER BY i.accession_no COLLATE NOCASE, i.id LIMIT :limit OFFSET :offset"
        ))?
        .query_map(
            named_params! { ":pattern": pattern, ":status": q.status, ":collection": q.collection_id, ":limit": page_size, ":offset": offset },
            |r| {
                Ok(CopyRow {
                    id: r.get(0)?, record_id: r.get(1)?, accession_no: r.get(2)?, barcode: r.get(3)?, title: r.get(4)?,
                    call_number: r.get(5)?, status: r.get(6)?, collection: r.get(7)?, location: r.get(8)?, shelf: r.get(9)?,
                    acquisition_date: r.get(10)?, price_minor: r.get(11)?,
                })
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CopyPage { total, page, page_size, rows })
}

fn spreadsheet_text(value: String) -> String {
    if value.starts_with(['\t', '\r', '\n'])
        || value.trim_start().starts_with(['=', '+', '-', '@', '＝', '＋', '－', '＠'])
    {
        format!("'{value}")
    } else {
        value
    }
}

pub fn export_copies_csv(conn: &Connection, query: &CopyQuery) -> AppResult<String> {
    let mut statement = conn.prepare(&format!(
        "SELECT i.accession_no, i.barcode, r.title, i.call_number, i.status,
                c.name, l.name, s.code, i.acquisition_date, i.price_minor
         {COPY_SEARCH_FROM} ORDER BY i.accession_no COLLATE NOCASE, i.id"
    ))?;
    let pattern = copy_search_pattern(query);
    let mut rows = statement.query(named_params! {
        ":pattern": pattern, ":status": query.status, ":collection": query.collection_id
    })?;
    let mut writer = csv::WriterBuilder::new().terminator(csv::Terminator::CRLF).from_writer(Vec::new());
    let csv_error = |error: csv::Error| AppError::Internal(error.to_string());
    writer.write_record([
        "Accession number", "Barcode", "Title", "Call number", "Status", "Collection",
        "Location", "Shelf", "Acquisition date", "Price (minor units)",
    ]).map_err(csv_error)?;
    while let Some(row) = rows.next()? {
        let mut fields = Vec::with_capacity(10);
        for column in 0..9 {
            fields.push(spreadsheet_text(row.get::<_, Option<String>>(column)?.unwrap_or_default()));
        }
        fields.push(row.get::<_, Option<i64>>(9)?.map(|n| n.to_string()).unwrap_or_default());
        writer.write_record(fields).map_err(csv_error)?;
    }
    let bytes = writer.into_inner().map_err(|error| AppError::Io(error.into_error()))?;
    String::from_utf8(bytes).map_err(|error| AppError::Internal(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::services::config::{self, NamedInput, NamedKind, ShelfInput};

    #[test]
    fn spreadsheet_text_neutralizes_formulas_and_preserves_plain_fields() {
        for input in ["=SUM(1)", "  +1", "@SUM(1)", "\tplain", "＋1"] {
            assert_eq!(spreadsheet_text(input.into()), format!("'{input}"));
        }
        assert_eq!(spreadsheet_text("004217".into()), "004217");
    }

    struct Fx {
        _tmp: tempfile::TempDir,
        c: Connection,
        category: i64,
        collection: i64,
        location: i64,
        shelf: i64,
    }

    fn fx() -> Fx {
        let tmp = tempfile::tempdir().unwrap();
        let c = db::open(&tmp.path().join("cat.db")).unwrap();
        let named = |n: &str| NamedInput { id: None, name: n.into(), description: None };
        let category = config::save_named(&c, None, NamedKind::ItemCategory, &named("General")).unwrap();
        let collection = config::save_named(&c, None, NamedKind::Collection, &named("Fiction")).unwrap();
        let location = config::save_named(&c, None, NamedKind::Location, &named("Main")).unwrap();
        let shelf = config::save_shelf(&c, None, &ShelfInput { id: None, location_id: location, code: "A1".into(), description: None }).unwrap();
        Fx { _tmp: tmp, c, category, collection, location, shelf }
    }

    fn rec(title: &str) -> RecordInput {
        RecordInput {
            id: None, title: title.into(), subtitle: None, edition: None, publisher: None, publication_place: None,
            publication_year: None, language: None, physical_description: None, pagination: None, series: None,
            abstract_text: None, notes: None, isbn: None, classification_scheme_id: None, classification_number: None,
            keywords: None, contributors: vec![], subjects: vec![],
        }
    }

    fn author(name: &str) -> ContributorDto {
        ContributorDto { name: name.into(), role: "author".into() }
    }

    fn batch(f: &Fx, count: i64) -> CopyBatchInput {
        CopyBatchInput {
            count, accession_numbers: None, barcodes: None, item_category_id: f.category, collection_id: Some(f.collection),
            location_id: Some(f.location), shelf_id: Some(f.shelf), call_number: Some("894.8123 BAS".into()),
            acquisition_date: Some("2026-11-02".into()), acquisition_source: "purchase".into(), price_minor: Some(25000),
            vendor: None, invoice_ref: None, notes: None,
        }
    }

    fn q(text: &str) -> SearchQuery {
        SearchQuery { text: Some(text.into()), page: 1, page_size: 25, ..Default::default() }
    }

    fn titles(r: &SearchResult) -> Vec<&str> {
        r.rows.iter().map(|x| x.title.as_str()).collect()
    }

    #[test]
    fn saves_udc_and_lcc_schemes_and_returns_them_in_search() {
        let f = fx();
        for (code, number, title) in [
            ("udc", "004.42", "Programming"),
            ("lcc", "PR9199.3 .R69", "Fiction"),
        ] {
            let scheme_id: i64 = f.c.query_row(
                "SELECT id FROM classification_schemes WHERE code = ?1", [code], |row| row.get(0),
            ).unwrap();
            let mut input = rec(title);
            input.classification_scheme_id = Some(scheme_id);
            input.classification_number = Some(number.into());
            let id = save_record(&f.c, None, &input).unwrap();
            let record = get_record(&f.c, id).unwrap();
            assert_eq!(record.classification_scheme_id, Some(scheme_id));
            assert_eq!(record.classification_number.as_deref(), Some(number));
            let summary = &search(&f.c, &q(title)).unwrap().rows[0];
            assert_eq!(summary.classification_scheme_id, Some(scheme_id));
            assert_eq!(summary.classification_number.as_deref(), Some(number));
        }
    }

    #[test]
    fn saves_a_full_record_and_reads_it_back() {
        let f = fx();
        let mut i = rec("Balyakalasakhi");
        i.subtitle = Some("A novel".into());
        i.publication_year = Some(1944);
        i.isbn = Some("0-306-40615-2".into());
        i.contributors = vec![author("  Vaikom   Muhammad Basheer "), ContributorDto { name: "Someone".into(), role: "translator".into() }];
        i.subjects = vec!["Malayalam fiction".into(), "malayalam FICTION".into(), "Love stories".into()];
        let id = save_record(&f.c, None, &i).unwrap();
        let r = get_record(&f.c, id).unwrap();
        assert_eq!((r.isbn10.as_deref(), r.isbn13.as_deref()), (Some("0306406152"), Some("9780306406157")));
        assert_eq!(r.contributors[0], author("Vaikom Muhammad Basheer"));
        assert_eq!(r.contributors.len(), 2);
        assert_eq!(r.subjects, ["Love stories", "Malayalam fiction"], "subjects are de-duplicated ignoring case");
        assert!(!r.archived && r.copies.is_empty());
    }

    #[test]
    fn validates_input() {
        let f = fx();
        assert!(matches!(save_record(&f.c, None, &rec("   ")), Err(AppError::Validation(_))));
        let mut i = rec("X");
        i.isbn = Some("0306406153".into());
        assert!(matches!(save_record(&f.c, None, &i), Err(AppError::Validation(_))));
        let mut i = rec("X");
        i.publication_year = Some(3000);
        assert!(matches!(save_record(&f.c, None, &i), Err(AppError::Validation(_))));
        let mut i = rec("X");
        i.contributors = vec![ContributorDto { name: "A".into(), role: "wizard".into() }];
        assert!(matches!(save_record(&f.c, None, &i), Err(AppError::Validation(_))));
        assert!(matches!(get_record(&f.c, 999), Err(AppError::NotFound(_))));
    }

    #[test]
    fn editing_replaces_people_and_subjects_and_updates_search() {
        let f = fx();
        let mut i = rec("Old Title");
        i.contributors = vec![author("First Author")];
        i.subjects = vec!["Alpha".into()];
        let id = save_record(&f.c, None, &i).unwrap();
        assert_eq!(search(&f.c, &q("first")).unwrap().total, 1);
        i.id = Some(id);
        i.title = "New Title".into();
        i.contributors = vec![author("Second Author")];
        i.subjects = vec!["Beta".into()];
        save_record(&f.c, None, &i).unwrap();
        assert_eq!(search(&f.c, &q("first")).unwrap().total, 0, "old author no longer matches");
        assert_eq!(search(&f.c, &q("second")).unwrap().total, 1);
        assert_eq!(search(&f.c, &q("alpha")).unwrap().total, 0);
        assert_eq!(search(&f.c, &q("beta")).unwrap().total, 1);
        assert_eq!(search(&f.c, &q("old")).unwrap().total, 0);
        assert_eq!(search(&f.c, &q("new")).unwrap().total, 1);
    }

    #[test]
    fn search_finds_by_title_author_subject_publisher_and_prefix() {
        let f = fx();
        let mut a = rec("The God of Small Things");
        a.contributors = vec![author("Arundhati Roy")];
        a.subjects = vec!["Kerala".into()];
        a.publisher = Some("IndiaInk".into());
        save_record(&f.c, None, &a).unwrap();
        let mut b = rec("Wings of Fire");
        b.contributors = vec![author("A. P. J. Abdul Kalam")];
        save_record(&f.c, None, &b).unwrap();

        assert_eq!(titles(&search(&f.c, &q("god small")).unwrap()), ["The God of Small Things"]);
        assert_eq!(search(&f.c, &q("arund")).unwrap().total, 1, "prefix match");
        assert_eq!(search(&f.c, &q("kerala")).unwrap().total, 1);
        assert_eq!(search(&f.c, &q("indiaink")).unwrap().total, 1);
        assert_eq!(search(&f.c, &q("wings kalam")).unwrap().total, 1, "all words must match, across fields");
        assert_eq!(search(&f.c, &q("wings roy")).unwrap().total, 0);
        assert_eq!(search(&f.c, &SearchQuery { page: 1, page_size: 25, ..Default::default() }).unwrap().total, 2, "no text lists everything");
    }

    #[test]
    fn search_handles_malayalam_and_never_treats_input_as_query_syntax() {
        let f = fx();
        let mut a = rec("ബാല്യകാലസഖി");
        a.contributors = vec![author("വൈക്കം മുഹമ്മദ് ബഷീർ")];
        save_record(&f.c, None, &a).unwrap();
        assert_eq!(search(&f.c, &q("ബാല്യകാലസഖി")).unwrap().total, 1);
        assert_eq!(search(&f.c, &q("ബാല്യ")).unwrap().total, 1, "prefix inside a Malayalam word");
        assert_eq!(search(&f.c, &q("ബഷീർ")).unwrap().total, 1);
        assert_eq!(search(&f.c, &q("മുഹമ്മദ്")).unwrap().total, 1);
        for hostile in ["\"", "*", "NEAR(", "title:", "a OR", "-", "()", "'; DROP TABLE x;--", "^"] {
            assert!(search(&f.c, &q(hostile)).is_ok(), "{hostile}");
        }
        assert_eq!(search(&f.c, &q("***")).unwrap().total, 0, "punctuation alone matches nothing");
    }

    #[test]
    fn exact_lookups_by_isbn_accession_and_barcode() {
        let f = fx();
        let mut a = rec("Alpha");
        a.isbn = Some("978-0-306-40615-7".into());
        let ida = save_record(&f.c, None, &a).unwrap();
        let idb = save_record(&f.c, None, &rec("Beta")).unwrap();
        add_copies(&f.c, None, idb, &CopyBatchInput { accession_numbers: Some(vec!["ACC-9".into()]), barcodes: Some(vec!["BC900".into()]), ..batch(&f, 1) }).unwrap();
        assert_eq!(search(&f.c, &q("0306406152")).unwrap().rows[0].id, ida, "ISBN-10 finds the ISBN-13 record");
        assert_eq!(search(&f.c, &q("978-0-306-40615-7")).unwrap().rows[0].id, ida);
        assert_eq!(search(&f.c, &q("acc-9")).unwrap().rows[0].id, idb);
        assert_eq!(search(&f.c, &q("BC900")).unwrap().rows[0].id, idb);
    }

    #[test]
    fn filters_pagination_and_archiving() {
        let f = fx();
        for n in 1..=30 {
            let id = save_record(&f.c, None, &rec(&format!("Book {n:02}"))).unwrap();
            if n <= 5 {
                add_copies(&f.c, None, id, &batch(&f, 1)).unwrap();
            }
        }
        let mut query = SearchQuery { page: 1, page_size: 10, ..Default::default() };
        let p1 = search(&f.c, &query).unwrap();
        assert_eq!((p1.total, p1.rows.len(), p1.rows[0].title.as_str()), (30, 10, "Book 01"));
        query.page = 3;
        assert_eq!(search(&f.c, &query).unwrap().rows.len(), 10);
        query.page = 4;
        assert_eq!(search(&f.c, &query).unwrap().rows.len(), 0);
        let filtered = SearchQuery { page: 1, page_size: 50, collection_id: Some(f.collection), ..Default::default() };
        assert_eq!(search(&f.c, &filtered).unwrap().total, 5);
        let avail = SearchQuery { page: 1, page_size: 50, only_available: true, ..Default::default() };
        assert_eq!(search(&f.c, &avail).unwrap().total, 5);

        let id = save_record(&f.c, None, &rec("Hidden Gem")).unwrap();
        set_record_archived(&f.c, None, id, true).unwrap();
        assert_eq!(search(&f.c, &q("hidden")).unwrap().total, 0);
        assert_eq!(search(&f.c, &SearchQuery { text: Some("hidden".into()), include_archived: true, page: 1, page_size: 10, ..Default::default() }).unwrap().total, 1);
        set_record_archived(&f.c, None, id, false).unwrap();
        assert_eq!(search(&f.c, &q("hidden")).unwrap().total, 1);
    }

    #[test]
    fn page_size_is_capped() {
        let f = fx();
        let r = search(&f.c, &SearchQuery { page: 1, page_size: 100_000, ..Default::default() }).unwrap();
        assert_eq!(r.page_size, 100);
    }

    #[test]
    fn duplicate_detection_by_isbn_and_by_title_author_year() {
        let f = fx();
        let mut a = rec("Wings of Fire");
        a.isbn = Some("9780306406157".into());
        a.publication_year = Some(1999);
        a.contributors = vec![author("A. P. J. Abdul Kalam")];
        let id = save_record(&f.c, None, &a).unwrap();

        let probe = |isbn: Option<&str>, title: &str, author: Option<&str>, year: Option<i64>, exclude: Option<i64>| DuplicateProbe {
            id: exclude, title: title.into(), isbn: isbn.map(String::from), first_author: author.map(String::from), publication_year: year,
        };
        let hits = find_duplicates(&f.c, &probe(Some("0-306-40615-2"), "Different", None, None, None)).unwrap();
        assert_eq!((hits.len(), hits[0].reason.as_str()), (1, "Same ISBN"));
        let hits = find_duplicates(&f.c, &probe(None, "wings of fire", Some("a. p. j. abdul kalam"), Some(1999), None)).unwrap();
        assert_eq!((hits.len(), hits[0].reason.as_str()), (1, "Same title, author and year"));
        assert!(find_duplicates(&f.c, &probe(None, "Wings of Fire", Some("A. P. J. Abdul Kalam"), Some(2005), None)).unwrap().is_empty());
        assert!(find_duplicates(&f.c, &probe(Some("9780306406157"), "Wings of Fire", Some("A. P. J. Abdul Kalam"), None, Some(id))).unwrap().is_empty(), "a record never duplicates itself");
        assert!(find_duplicates(&f.c, &probe(None, "Unrelated", None, None, None)).unwrap().is_empty());
    }

    #[test]
    fn auto_accession_follows_prefix_padding_and_skips_numbers_in_use() {
        let f = fx();
        set_accession_format(&f.c, None, &AccessionFormat { prefix: "LIB-".into(), padding: 5, next: 7 }).unwrap();
        let id = save_record(&f.c, None, &rec("A")).unwrap();
        // A hand-typed number occupies LIB-00009 so the counter must step over it.
        add_copies(&f.c, None, id, &CopyBatchInput { accession_numbers: Some(vec!["LIB-00009".into()]), ..batch(&f, 1) }).unwrap();
        let copies = add_copies(&f.c, None, id, &batch(&f, 4)).unwrap();
        let numbers: Vec<_> = copies.iter().map(|c| c.accession_no.as_str()).collect();
        assert_eq!(numbers, ["LIB-00007", "LIB-00008", "LIB-00010", "LIB-00011"]);
        assert!(copies.iter().all(|c| c.barcode == c.accession_no), "barcode defaults to the accession number");
        assert_eq!(accession_format(&f.c).unwrap().next, 12);
        assert!(set_accession_format(&f.c, None, &AccessionFormat { prefix: "bad prefix".into(), padding: 6, next: 1 }).is_err());
        assert!(set_accession_format(&f.c, None, &AccessionFormat { prefix: "".into(), padding: 2, next: 1 }).is_err());
    }

    #[test]
    fn copies_are_independent_and_codes_must_be_unique() {
        let f = fx();
        let id = save_record(&f.c, None, &rec("A")).unwrap();
        let other = save_record(&f.c, None, &rec("B")).unwrap();
        let c = add_copies(&f.c, None, id, &batch(&f, 3)).unwrap();
        assert_eq!(c.len(), 3);
        assert_eq!(get_record(&f.c, id).unwrap().copies.len(), 3);

        // Existing accession number or barcode (any case) is refused, on any record.
        let dup_acc = CopyBatchInput { accession_numbers: Some(vec![c[0].accession_no.to_lowercase()]), ..batch(&f, 1) };
        assert!(matches!(add_copies(&f.c, None, other, &dup_acc), Err(AppError::Conflict(_))));
        let dup_bar = CopyBatchInput { barcodes: Some(vec![c[1].barcode.clone()]), ..batch(&f, 1) };
        assert!(matches!(add_copies(&f.c, None, other, &dup_bar), Err(AppError::Conflict(_))));
        // Repeats inside one batch.
        let rep = CopyBatchInput { accession_numbers: Some(vec!["X1".into(), "x1".into()]), ..batch(&f, 2) };
        assert!(matches!(add_copies(&f.c, None, other, &rep), Err(AppError::Conflict(_))));
        // Nothing was added by the failed calls.
        assert!(get_record(&f.c, other).unwrap().copies.is_empty());
        // Wrong counts and bad codes.
        assert!(add_copies(&f.c, None, id, &batch(&f, 0)).is_err());
        assert!(add_copies(&f.c, None, id, &batch(&f, 501)).is_err());
        assert!(add_copies(&f.c, None, id, &CopyBatchInput { accession_numbers: Some(vec!["has space".into()]), ..batch(&f, 1) }).is_err());
        assert!(add_copies(&f.c, None, id, &CopyBatchInput { accession_numbers: Some(vec!["A".into()]), ..batch(&f, 2) }).is_err());
    }

    #[test]
    fn copy_fields_are_checked() {
        let f = fx();
        let id = save_record(&f.c, None, &rec("A")).unwrap();
        let other_loc = config::save_named(&f.c, None, NamedKind::Location, &NamedInput { id: None, name: "Annex".into(), description: None }).unwrap();
        assert!(matches!(add_copies(&f.c, None, id, &CopyBatchInput { location_id: Some(other_loc), ..batch(&f, 1) }), Err(AppError::Validation(_))), "shelf is in another location");
        // Location is taken from the shelf when omitted.
        let c = add_copies(&f.c, None, id, &CopyBatchInput { location_id: None, ..batch(&f, 1) }).unwrap();
        assert_eq!(c[0].location_id, Some(f.location));
        assert!(add_copies(&f.c, None, id, &CopyBatchInput { acquisition_date: Some("2026-02-30".into()), ..batch(&f, 1) }).is_err());
        assert!(add_copies(&f.c, None, id, &CopyBatchInput { acquisition_source: "stolen".into(), ..batch(&f, 1) }).is_err());
        assert!(add_copies(&f.c, None, id, &CopyBatchInput { price_minor: Some(-1), ..batch(&f, 1) }).is_err());
        assert!(add_copies(&f.c, None, id, &CopyBatchInput { item_category_id: 999, ..batch(&f, 1) }).is_err());
        config::set_archived(&f.c, None, config::ArchiveKind::Collection, f.collection, true).unwrap();
        assert!(add_copies(&f.c, None, id, &batch(&f, 1)).is_err(), "archived collections cannot be assigned");
        assert!(matches!(add_copies(&f.c, None, 999, &batch(&f, 1)), Err(AppError::NotFound(_))));
    }

    #[test]
    fn status_changes_are_guarded_and_recorded() {
        let f = fx();
        let id = save_record(&f.c, None, &rec("A")).unwrap();
        let copy = add_copies(&f.c, None, id, &batch(&f, 1)).unwrap().remove(0);
        set_copy_status(&f.c, None, copy.id, "damaged", Some("Water damage")).unwrap();
        assert_eq!(get_copy(&f.c, copy.id).unwrap().status, "damaged");
        set_copy_status(&f.c, None, copy.id, "available", None).unwrap();
        assert!(matches!(set_copy_status(&f.c, None, copy.id, "on_loan", None), Err(AppError::Validation(_))), "only circulation sets loans");
        assert!(matches!(set_copy_status(&f.c, None, copy.id, "nonsense", None), Err(AppError::Validation(_))));

        f.c.execute("UPDATE physical_items SET status='on_loan' WHERE id=?1", [copy.id]).unwrap();
        assert!(matches!(set_copy_status(&f.c, None, copy.id, "withdrawn", None), Err(AppError::Conflict(_))));
        assert!(matches!(set_record_archived(&f.c, None, id, true), Err(AppError::Conflict(_))), "cannot archive a record with a copy on loan");

        let events: i64 = f.c.query_row("SELECT count(*) FROM item_events WHERE item_id=?1", [copy.id], |r| r.get(0)).unwrap();
        assert_eq!(events, 3, "acquired + two status changes");
        assert!(f.c.execute("DELETE FROM item_events", []).is_err(), "history is append-only");
        assert!(f.c.execute("DELETE FROM physical_items WHERE id=?1", [copy.id]).is_err(), "a copy with history cannot be deleted");
    }

    #[test]
    fn copies_can_be_edited_with_unique_codes() {
        let f = fx();
        let id = save_record(&f.c, None, &rec("A")).unwrap();
        let c = add_copies(&f.c, None, id, &batch(&f, 2)).unwrap();
        let upd = |acc: &str, bar: &str| CopyUpdateInput {
            accession_no: acc.into(), barcode: bar.into(), item_category_id: f.category, collection_id: None, location_id: None,
            shelf_id: None, call_number: Some("123.4 XYZ".into()), acquisition_date: None, acquisition_source: "donation".into(),
            price_minor: None, vendor: Some("Friends of the Library".into()), invoice_ref: None, notes: None,
        };
        update_copy(&f.c, None, c[0].id, &upd("NEW-1", "NEW-1")).unwrap();
        let got = get_copy(&f.c, c[0].id).unwrap();
        assert_eq!((got.accession_no.as_str(), got.acquisition_source.as_str(), got.call_number.as_deref()), ("NEW-1", "donation", Some("123.4 XYZ")));
        assert!(got.shelf_id.is_none());
        assert!(matches!(update_copy(&f.c, None, c[0].id, &upd(&c[1].accession_no, "NEW-1")), Err(AppError::Conflict(_))));
        assert!(matches!(update_copy(&f.c, None, 9999, &upd("Z", "Z")), Err(AppError::NotFound(_))));
    }

    #[test]
    fn accession_register_lists_filters_and_pages() {
        let f = fx();
        let id = save_record(&f.c, None, &rec("Register Book")).unwrap();
        let copies = add_copies(&f.c, None, id, &batch(&f, 12)).unwrap();
        set_copy_status(&f.c, None, copies[0].id, "lost", None).unwrap();
        let all = list_copies(&f.c, &CopyQuery { page: 1, page_size: 5, ..Default::default() }).unwrap();
        assert_eq!((all.total, all.rows.len()), (12, 5));
        assert_eq!(all.rows[0].title, "Register Book");
        assert_eq!(all.rows[0].shelf.as_deref(), Some("A1"));
        let lost = list_copies(&f.c, &CopyQuery { status: Some("lost".into()), page: 1, page_size: 5, ..Default::default() }).unwrap();
        assert_eq!(lost.total, 1);
        let by_text = list_copies(&f.c, &CopyQuery { text: Some("000003".into()), page: 1, page_size: 5, ..Default::default() }).unwrap();
        assert_eq!(by_text.total, 1);
        let wild = list_copies(&f.c, &CopyQuery { text: Some("%".into()), page: 1, page_size: 5, ..Default::default() }).unwrap();
        assert_eq!(wild.total, 0, "% is searched literally, not as a wildcard");
    }

    #[test]
    fn catalogue_writes_are_audited() {
        let f = fx();
        let id = save_record(&f.c, None, &rec("Audited")).unwrap();
        add_copies(&f.c, None, id, &batch(&f, 1)).unwrap();
        let n: i64 = f.c.query_row("SELECT count(*) FROM audit_events WHERE action LIKE 'catalogue.%'", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 2);
        assert!(f.location > 0);
    }
}
