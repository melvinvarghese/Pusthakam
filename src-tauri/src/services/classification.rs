//! Classification schedule browser (DDC, UDC, LCC, CC, custom schemes) and managed authorities
//! (authors/contributors, publishers, subjects) with linked-record synchronization and controlled MARC mappings.

use crate::domain::validate;
use crate::error::{AppError, AppResult};
use crate::services::audit;
use crate::services::auth::Session;
use crate::services::catalogue;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Types: Schemes & Classification Entries
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassificationScheme {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_custom: bool,
    pub entry_count: i64,
    pub record_count: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassificationSchemeInput {
    pub id: Option<i64>,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassificationEntry {
    pub id: i64,
    pub scheme_id: i64,
    pub scheme_code: String,
    pub code: String,
    pub parent_code: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub is_custom: bool,
    pub record_count: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassificationEntryInput {
    pub id: Option<i64>,
    pub scheme_id: i64,
    pub code: String,
    pub parent_code: Option<String>,
    pub title: String,
    pub description: Option<String>,
}

// ============================================================================
// Types: Authorities (Authors, Publishers, Subjects)
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorAuthority {
    pub id: i64,
    pub name: String,
    pub sort_name: Option<String>,
    pub kind: String, // "personal", "corporate", "meeting", "uniform_title"
    pub birth_year: Option<i64>,
    pub death_year: Option<i64>,
    pub authority_note: Option<String>,
    pub variant_names: Option<String>,
    pub marc_tag: String,
    pub record_count: i64,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorAuthorityInput {
    pub id: Option<i64>,
    pub name: String,
    pub sort_name: Option<String>,
    pub kind: Option<String>,
    pub birth_year: Option<i64>,
    pub death_year: Option<i64>,
    pub authority_note: Option<String>,
    pub variant_names: Option<String>,
    pub marc_tag: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublisherAuthority {
    pub id: i64,
    pub name: String,
    pub place: Option<String>,
    pub notes: Option<String>,
    pub marc_tag: String,
    pub record_count: i64,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublisherAuthorityInput {
    pub id: Option<i64>,
    pub name: String,
    pub place: Option<String>,
    pub notes: Option<String>,
    pub marc_tag: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectAuthority {
    pub id: i64,
    pub heading: String,
    pub scheme: String, // "local", "lcsh", "fast", "mesh"
    pub scope_note: Option<String>,
    pub broader_term: Option<String>,
    pub narrower_terms: Option<String>,
    pub see_also: Option<String>,
    pub marc_tag: String,
    pub record_count: i64,
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectAuthorityInput {
    pub id: Option<i64>,
    pub heading: String,
    pub scheme: Option<String>,
    pub scope_note: Option<String>,
    pub broader_term: Option<String>,
    pub narrower_terms: Option<String>,
    pub see_also: Option<String>,
    pub marc_tag: Option<String>,
}

// ============================================================================
// Types: MARC Authority Mappings & Testing
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarcMapping {
    pub id: i64,
    pub authority_type: String,
    pub marc_tag: String,
    pub subfield_code: String,
    pub description: String,
    pub target_field: String,
    pub is_default: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarcMappingInput {
    pub id: Option<i64>,
    pub authority_type: String,
    pub marc_tag: String,
    pub subfield_code: String,
    pub description: String,
    pub target_field: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarcTestInput {
    pub raw_marc_line: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarcTestResult {
    pub tag: String,
    pub indicators: String,
    pub subfields: HashMap<String, String>,
    pub matched_mappings: Vec<MarcMapping>,
    pub mapped_authority_type: Option<String>,
    pub target_fields: HashMap<String, String>,
}

// ============================================================================
// Scheme & Classification Operations
// ============================================================================

pub fn list_schemes(conn: &Connection) -> AppResult<Vec<ClassificationScheme>> {
    let mut stmt = conn.prepare(
        "SELECT s.id, s.code, s.name, s.description, (s.is_custom = 1) AS is_custom,
                (SELECT count(*) FROM classification_entries e WHERE e.scheme_id = s.id) AS entry_count,
                (SELECT count(*) FROM bibliographic_records r WHERE r.classification_scheme_id = s.id) AS record_count
         FROM classification_schemes s
         ORDER BY s.is_custom ASC, s.id ASC",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok(ClassificationScheme {
                id: r.get(0)?,
                code: r.get(1)?,
                name: r.get(2)?,
                description: r.get(3)?,
                is_custom: r.get(4)?,
                entry_count: r.get(5)?,
                record_count: r.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn save_scheme(
    conn: &Connection,
    actor: &Session,
    input: &ClassificationSchemeInput,
) -> AppResult<i64> {
    let code = validate::required("Scheme code", &input.code, 16)?.to_lowercase();
    let name = validate::required("Scheme name", &input.name, 120)?;
    let desc = validate::optional("Description", input.description.as_deref(), 500)?;

    let tx = conn.unchecked_transaction()?;
    let id = if let Some(id) = input.id {
        tx.execute(
            "UPDATE classification_schemes SET name = ?1, description = ?2 WHERE id = ?3",
            params![name, desc, id],
        )?;
        audit::record(
            &tx,
            Some(actor),
            "classification.scheme_update",
            "classification_schemes",
            Some(id),
            &format!("Updated classification scheme {code} ({name})"),
        )?;
        id
    } else {
        tx.execute(
            "INSERT INTO classification_schemes (code, name, description, is_custom)
             VALUES (?1, ?2, ?3, 1)",
            params![code, name, desc],
        )?;
        let new_id = tx.last_insert_rowid();
        audit::record(
            &tx,
            Some(actor),
            "classification.scheme_create",
            "classification_schemes",
            Some(new_id),
            &format!("Created custom classification scheme {code} ({name})"),
        )?;
        new_id
    };
    tx.commit()?;
    Ok(id)
}

pub fn delete_custom_scheme(conn: &Connection, actor: &Session, id: i64) -> AppResult<()> {
    let tx = conn.unchecked_transaction()?;
    let (code, is_custom): (String, bool) = tx
        .query_row(
            "SELECT code, is_custom = 1 FROM classification_schemes WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound(format!("Classification scheme #{id} not found.")))?;

    if !is_custom {
        return Err(AppError::Validation(
            "Standard classification schemes (DDC, UDC, LCC, CC) cannot be deleted.".into(),
        ));
    }

    let records_using: i64 = tx.query_row(
        "SELECT count(*) FROM bibliographic_records WHERE classification_scheme_id = ?1",
        [id],
        |r| r.get(0),
    )?;
    if records_using > 0 {
        return Err(AppError::Conflict(format!(
            "Cannot delete scheme {code}: it is used by {records_using} catalogue records."
        )));
    }

    tx.execute("DELETE FROM classification_entries WHERE scheme_id = ?1", [id])?;
    tx.execute("DELETE FROM classification_schemes WHERE id = ?1", [id])?;

    audit::record(
        &tx,
        Some(actor),
        "classification.scheme_delete",
        "classification_schemes",
        Some(id),
        &format!("Deleted custom classification scheme {code}"),
    )?;
    tx.commit()?;
    Ok(())
}

pub fn list_entries(
    conn: &Connection,
    scheme_id: i64,
    parent_code: Option<&str>,
    search_query: Option<&str>,
) -> AppResult<Vec<ClassificationEntry>> {
    let mut sql = String::from(
        "SELECT e.id, e.scheme_id, s.code AS scheme_code, e.code, e.parent_code, e.title,
                e.description, (e.is_custom = 1) AS is_custom,
                (SELECT count(*) FROM bibliographic_records r
                 WHERE r.classification_scheme_id = e.scheme_id
                   AND (r.classification_number = e.code
                        OR r.classification_number LIKE e.code || '.%'
                        OR r.classification_number LIKE e.code || '%')
                ) AS record_count
         FROM classification_entries e
         JOIN classification_schemes s ON s.id = e.scheme_id
         WHERE e.scheme_id = ?1 ",
    );

    let clean_search = search_query.map(|s| s.trim()).filter(|s| !s.is_empty());
    let clean_parent = parent_code.map(|p| p.trim()).filter(|p| !p.is_empty());

    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(scheme_id)];

    if let Some(q) = clean_search {
        sql.push_str("AND (e.code LIKE ?2 OR e.title LIKE ?2 OR e.description LIKE ?2) ");
        params_vec.push(Box::new(format!("%{q}%")));
    } else if let Some(p) = clean_parent {
        sql.push_str("AND e.parent_code = ?2 ");
        params_vec.push(Box::new(p.to_string()));
    } else if parent_code.is_some() {
        // Explicitly asking for top-level entries (parent_code is None)
        sql.push_str("AND e.parent_code IS NULL ");
    }

    sql.push_str("ORDER BY e.code ASC");

    let mut stmt = conn.prepare(&sql)?;
    let params_slice: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| &**b as &dyn rusqlite::ToSql).collect();

    let rows = stmt
        .query_map(&params_slice[..], |r| {
            Ok(ClassificationEntry {
                id: r.get(0)?,
                scheme_id: r.get(1)?,
                scheme_code: r.get(2)?,
                code: r.get(3)?,
                parent_code: r.get(4)?,
                title: r.get(5)?,
                description: r.get(6)?,
                is_custom: r.get(7)?,
                record_count: r.get(8)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(rows)
}

pub fn save_entry(
    conn: &Connection,
    actor: &Session,
    input: &ClassificationEntryInput,
) -> AppResult<i64> {
    let code = validate::required("Classification code", &input.code, 64)?;
    let title = validate::required("Classification title", &input.title, 200)?;
    let parent = validate::optional("Parent code", input.parent_code.as_deref(), 64)?;
    let desc = validate::optional("Description", input.description.as_deref(), 500)?;

    let tx = conn.unchecked_transaction()?;
    let id = if let Some(id) = input.id {
        tx.execute(
            "UPDATE classification_entries
             SET code = ?1, parent_code = ?2, title = ?3, description = ?4,
                 updated_at = (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
             WHERE id = ?5",
            params![code, parent, title, desc, id],
        )?;
        audit::record(
            &tx,
            Some(actor),
            "classification.entry_update",
            "classification_entries",
            Some(id),
            &format!("Updated classification code {code} ({title})"),
        )?;
        id
    } else {
        tx.execute(
            "INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
             VALUES (?1, ?2, ?3, ?4, ?5, 1)",
            params![input.scheme_id, code, parent, title, desc],
        )?;
        let new_id = tx.last_insert_rowid();
        audit::record(
            &tx,
            Some(actor),
            "classification.entry_create",
            "classification_entries",
            Some(new_id),
            &format!("Created classification code {code} ({title})"),
        )?;
        new_id
    };
    tx.commit()?;
    Ok(id)
}

pub fn delete_custom_entry(conn: &Connection, actor: &Session, id: i64) -> AppResult<()> {
    let tx = conn.unchecked_transaction()?;
    let (code, is_custom): (String, bool) = tx
        .query_row(
            "SELECT code, is_custom = 1 FROM classification_entries WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound(format!("Classification code #{id} not found.")))?;

    if !is_custom {
        return Err(AppError::Validation(
            "Standard schedule classification codes cannot be deleted.".into(),
        ));
    }

    tx.execute("DELETE FROM classification_entries WHERE id = ?1", [id])?;
    audit::record(
        &tx,
        Some(actor),
        "classification.entry_delete",
        "classification_entries",
        Some(id),
        &format!("Deleted custom classification code {code}"),
    )?;
    tx.commit()?;
    Ok(())
}

// ============================================================================
// Author Authorities
// ============================================================================

pub fn list_authors(
    conn: &Connection,
    search_query: Option<&str>,
    kind_filter: Option<&str>,
) -> AppResult<Vec<AuthorAuthority>> {
    let mut sql = String::from(
        "SELECT c.id, c.name, c.sort_name, COALESCE(c.kind, 'personal') AS kind,
                c.birth_year, c.death_year, c.authority_note, c.variant_names,
                COALESCE(c.marc_tag, '100') AS marc_tag,
                (SELECT count(*) FROM record_contributors rc WHERE rc.contributor_id = c.id) AS record_count,
                c.created_at, c.updated_at
         FROM contributors c
         WHERE 1=1 ",
    );

    let clean_search = search_query.map(|s| s.trim()).filter(|s| !s.is_empty());
    let clean_kind = kind_filter.map(|k| k.trim()).filter(|k| !k.is_empty());

    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(q) = clean_search {
        let idx = params_vec.len() + 1;
        sql.push_str(&format!("AND (c.name LIKE ?{idx} OR c.sort_name LIKE ?{idx} OR c.variant_names LIKE ?{idx}) "));
        params_vec.push(Box::new(format!("%{q}%")));
    }
    if let Some(k) = clean_kind {
        let idx = params_vec.len() + 1;
        sql.push_str(&format!("AND c.kind = ?{idx} "));
        params_vec.push(Box::new(k.to_string()));
    }

    sql.push_str("ORDER BY COALESCE(c.sort_name, c.name) COLLATE NOCASE ASC LIMIT 300");

    let mut stmt = conn.prepare(&sql)?;
    let params_slice: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| &**b as &dyn rusqlite::ToSql).collect();

    let rows = stmt
        .query_map(&params_slice[..], |r| {
            Ok(AuthorAuthority {
                id: r.get(0)?,
                name: r.get(1)?,
                sort_name: r.get(2)?,
                kind: r.get(3)?,
                birth_year: r.get(4)?,
                death_year: r.get(5)?,
                authority_note: r.get(6)?,
                variant_names: r.get(7)?,
                marc_tag: r.get(8)?,
                record_count: r.get(9)?,
                created_at: r.get(10)?,
                updated_at: r.get(11)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(rows)
}

pub fn save_author(
    conn: &Connection,
    actor: &Session,
    input: &AuthorAuthorityInput,
) -> AppResult<i64> {
    let name = validate::required("Author name", &input.name, 250)?;
    let sort_name = validate::optional("Sort name", input.sort_name.as_deref(), 250)?;
    let kind = input.kind.as_deref().unwrap_or("personal");
    let authority_note = validate::optional("Authority note", input.authority_note.as_deref(), 2000)?;
    let variant_names = validate::optional("Variant names", input.variant_names.as_deref(), 1000)?;
    let marc_tag = input.marc_tag.as_deref().unwrap_or("100");

    let tx = conn.unchecked_transaction()?;
    let (id, affected_records): (i64, Vec<i64>) = if let Some(id) = input.id {
        tx.execute(
            "UPDATE contributors
             SET name = ?1, sort_name = ?2, kind = ?3, birth_year = ?4, death_year = ?5,
                 authority_note = ?6, variant_names = ?7, marc_tag = ?8,
                 updated_at = (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
             WHERE id = ?9",
            params![
                name,
                sort_name.or(Some(name.clone())),
                kind,
                input.birth_year,
                input.death_year,
                authority_note,
                variant_names,
                marc_tag,
                id
            ],
        )?;

        let recs = {
            let mut stmt = tx.prepare("SELECT DISTINCT record_id FROM record_contributors WHERE contributor_id = ?1")?;
            let res = stmt.query_map([id], |r| r.get(0))?.collect::<Result<Vec<i64>, _>>()?;
            res
        };

        audit::record(
            &tx,
            Some(actor),
            "authority.author_update",
            "contributors",
            Some(id),
            &format!("Updated author authority {name}"),
        )?;
        (id, recs)
    } else {
        tx.execute(
            "INSERT INTO contributors (name, sort_name, kind, birth_year, death_year, authority_note, variant_names, marc_tag)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                name,
                sort_name.or(Some(name.clone())),
                kind,
                input.birth_year,
                input.death_year,
                authority_note,
                variant_names,
                marc_tag
            ],
        )?;
        let new_id = tx.last_insert_rowid();
        audit::record(
            &tx,
            Some(actor),
            "authority.author_create",
            "contributors",
            Some(new_id),
            &format!("Created author authority {name}"),
        )?;
        (new_id, Vec::new())
    };
    tx.commit()?;

    // Linked-record update: Re-index full-text search for all affected bibliographic records
    for rec_id in affected_records {
        catalogue::reindex(conn, rec_id)?;
    }

    Ok(id)
}

pub fn merge_authors(
    conn: &Connection,
    actor: &Session,
    from_id: i64,
    to_id: i64,
) -> AppResult<()> {
    if from_id == to_id {
        return Err(AppError::Validation("Cannot merge an author into itself.".into()));
    }

    let tx = conn.unchecked_transaction()?;
    let from_name: String = tx
        .query_row("SELECT name FROM contributors WHERE id = ?1", [from_id], |r| r.get(0))
        .optional()?
        .ok_or_else(|| AppError::NotFound(format!("Source author #{from_id} not found.")))?;

    let to_name: String = tx
        .query_row("SELECT name FROM contributors WHERE id = ?1", [to_id], |r| r.get(0))
        .optional()?
        .ok_or_else(|| AppError::NotFound(format!("Target author #{to_id} not found.")))?;

    // Find all records referencing from_id
    let links: Vec<(i64, String, i64)> = {
        let mut stmt = tx.prepare("SELECT record_id, role, position FROM record_contributors WHERE contributor_id = ?1")?;
        let res = stmt.query_map([from_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        res
    };

    let mut affected_records: Vec<i64> = Vec::new();
    for (rec_id, role, pos) in links {
        affected_records.push(rec_id);
        // If target author is already on this record with same role, just remove the source link
        let exists: i64 = tx.query_row(
            "SELECT count(*) FROM record_contributors WHERE record_id = ?1 AND contributor_id = ?2 AND role = ?3",
            params![rec_id, to_id, role],
            |r| r.get(0),
        )?;
        if exists > 0 {
            tx.execute(
                "DELETE FROM record_contributors WHERE record_id = ?1 AND contributor_id = ?2 AND role = ?3",
                params![rec_id, from_id, role],
            )?;
        } else {
            tx.execute(
                "UPDATE record_contributors SET contributor_id = ?1, position = ?2
                 WHERE record_id = ?3 AND contributor_id = ?4 AND role = ?5",
                params![to_id, pos, rec_id, from_id, role],
            )?;
        }
    }

    tx.execute("DELETE FROM contributors WHERE id = ?1", [from_id])?;

    audit::record(
        &tx,
        Some(actor),
        "authority.author_merge",
        "contributors",
        Some(to_id),
        &format!("Merged author authority {from_name} into {to_name}"),
    )?;
    tx.commit()?;

    // Re-index all affected records in FTS5
    affected_records.sort_unstable();
    affected_records.dedup();
    for rec_id in affected_records {
        catalogue::reindex(conn, rec_id)?;
    }

    Ok(())
}

// ============================================================================
// Publisher Authorities
// ============================================================================

pub fn list_publishers(
    conn: &Connection,
    search_query: Option<&str>,
) -> AppResult<Vec<PublisherAuthority>> {
    let mut sql = String::from(
        "SELECT p.id, p.name, p.place, p.notes, COALESCE(p.marc_tag, '264') AS marc_tag,
                (SELECT count(*) FROM bibliographic_records r
                 WHERE r.publisher_id = p.id
                    OR (r.publisher_id IS NULL AND r.publisher = p.name COLLATE NOCASE)
                ) AS record_count,
                p.created_at, p.updated_at
         FROM publishers p
         WHERE 1=1 ",
    );

    let clean_search = search_query.map(|s| s.trim()).filter(|s| !s.is_empty());
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(q) = clean_search {
        sql.push_str("AND (p.name LIKE ?1 OR p.place LIKE ?1 OR p.notes LIKE ?1) ");
        params_vec.push(Box::new(format!("%{q}%")));
    }
    sql.push_str("ORDER BY p.name COLLATE NOCASE ASC LIMIT 300");

    let mut stmt = conn.prepare(&sql)?;
    let params_slice: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| &**b as &dyn rusqlite::ToSql).collect();

    let rows = stmt
        .query_map(&params_slice[..], |r| {
            Ok(PublisherAuthority {
                id: r.get(0)?,
                name: r.get(1)?,
                place: r.get(2)?,
                notes: r.get(3)?,
                marc_tag: r.get(4)?,
                record_count: r.get(5)?,
                created_at: r.get(6)?,
                updated_at: r.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(rows)
}

pub fn save_publisher(
    conn: &Connection,
    actor: &Session,
    input: &PublisherAuthorityInput,
) -> AppResult<i64> {
    let name = validate::required("Publisher name", &input.name, 200)?;
    let place = validate::optional("Publication place", input.place.as_deref(), 200)?;
    let notes = validate::optional("Publisher notes", input.notes.as_deref(), 2000)?;
    let marc_tag = input.marc_tag.as_deref().unwrap_or("264");

    let tx = conn.unchecked_transaction()?;
    let (id, affected_records): (i64, Vec<i64>) = if let Some(id) = input.id {
        let old_name: String = tx.query_row(
            "SELECT name FROM publishers WHERE id = ?1",
            [id],
            |r| r.get(0),
        )?;

        tx.execute(
            "UPDATE publishers
             SET name = ?1, place = ?2, notes = ?3, marc_tag = ?4,
                 updated_at = (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
             WHERE id = ?5",
            params![name, place, notes, marc_tag, id],
        )?;

        // Synchronize linked records: update publisher name and place if matching
        tx.execute(
            "UPDATE bibliographic_records
             SET publisher = ?1, publisher_id = ?2
             WHERE publisher_id = ?2 OR publisher = ?3 COLLATE NOCASE",
            params![name, id, old_name],
        )?;

        let recs = {
            let mut stmt = tx.prepare("SELECT id FROM bibliographic_records WHERE publisher_id = ?1")?;
            let res = stmt.query_map([id], |r| r.get(0))?.collect::<Result<Vec<i64>, _>>()?;
            res
        };

        audit::record(
            &tx,
            Some(actor),
            "authority.publisher_update",
            "publishers",
            Some(id),
            &format!("Updated publisher authority {name}"),
        )?;
        (id, recs)
    } else {
        tx.execute(
            "INSERT INTO publishers (name, place, notes, marc_tag)
             VALUES (?1, ?2, ?3, ?4)",
            params![name, place, notes, marc_tag],
        )?;
        let new_id = tx.last_insert_rowid();

        // Link existing matching unlinked records to this publisher
        tx.execute(
            "UPDATE bibliographic_records
             SET publisher_id = ?1
             WHERE publisher_id IS NULL AND publisher = ?2 COLLATE NOCASE",
            params![new_id, name],
        )?;

        audit::record(
            &tx,
            Some(actor),
            "authority.publisher_create",
            "publishers",
            Some(new_id),
            &format!("Created publisher authority {name}"),
        )?;
        (new_id, Vec::new())
    };
    tx.commit()?;

    // Linked-record update: Re-index full-text search
    for rec_id in affected_records {
        catalogue::reindex(conn, rec_id)?;
    }

    Ok(id)
}

pub fn merge_publishers(
    conn: &Connection,
    actor: &Session,
    from_id: i64,
    to_id: i64,
) -> AppResult<()> {
    if from_id == to_id {
        return Err(AppError::Validation("Cannot merge a publisher into itself.".into()));
    }

    let tx = conn.unchecked_transaction()?;
    let from_name: String = tx
        .query_row("SELECT name FROM publishers WHERE id = ?1", [from_id], |r| r.get(0))
        .optional()?
        .ok_or_else(|| AppError::NotFound(format!("Source publisher #{from_id} not found.")))?;

    let (to_name, to_place): (String, Option<String>) = tx
        .query_row("SELECT name, place FROM publishers WHERE id = ?1", [to_id], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .optional()?
        .ok_or_else(|| AppError::NotFound(format!("Target publisher #{to_id} not found.")))?;

    // Find affected records
    let affected_records: Vec<i64> = {
        let mut stmt = tx.prepare(
            "SELECT id FROM bibliographic_records
             WHERE publisher_id = ?1 OR publisher = ?2 COLLATE NOCASE",
        )?;
        let res = stmt.query_map(params![from_id, from_name], |r| r.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        res
    };

    // Update records to reference target publisher
    tx.execute(
        "UPDATE bibliographic_records
         SET publisher_id = ?1, publisher = ?2
         WHERE publisher_id = ?3 OR publisher = ?4 COLLATE NOCASE",
        params![to_id, to_name, from_id, from_name],
    )?;

    if let Some(ref pl) = to_place {
        tx.execute(
            "UPDATE bibliographic_records SET publication_place = ?1
             WHERE publisher_id = ?2 AND (publication_place IS NULL OR trim(publication_place) = '')",
            params![pl, to_id],
        )?;
    }

    tx.execute("DELETE FROM publishers WHERE id = ?1", [from_id])?;

    audit::record(
        &tx,
        Some(actor),
        "authority.publisher_merge",
        "publishers",
        Some(to_id),
        &format!("Merged publisher authority {from_name} into {to_name}"),
    )?;
    tx.commit()?;

    // Re-index affected records in FTS5
    for rec_id in affected_records {
        catalogue::reindex(conn, rec_id)?;
    }

    Ok(())
}

// ============================================================================
// Subject Authorities
// ============================================================================

pub fn list_subjects(
    conn: &Connection,
    search_query: Option<&str>,
    scheme_filter: Option<&str>,
) -> AppResult<Vec<SubjectAuthority>> {
    let mut sql = String::from(
        "SELECT s.id, s.heading, COALESCE(s.scheme, 'local') AS scheme,
                s.scope_note, s.broader_term, s.narrower_terms, s.see_also,
                COALESCE(s.marc_tag, '650') AS marc_tag,
                (SELECT count(*) FROM record_subjects rs WHERE rs.subject_id = s.id) AS record_count,
                s.updated_at
         FROM subjects s
         WHERE 1=1 ",
    );

    let clean_search = search_query.map(|s| s.trim()).filter(|s| !s.is_empty());
    let clean_scheme = scheme_filter.map(|sc| sc.trim()).filter(|sc| !sc.is_empty());

    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(q) = clean_search {
        let idx = params_vec.len() + 1;
        sql.push_str(&format!("AND (s.heading LIKE ?{idx} OR s.scope_note LIKE ?{idx} OR s.see_also LIKE ?{idx}) "));
        params_vec.push(Box::new(format!("%{q}%")));
    }
    if let Some(sc) = clean_scheme {
        let idx = params_vec.len() + 1;
        sql.push_str(&format!("AND s.scheme = ?{idx} "));
        params_vec.push(Box::new(sc.to_string()));
    }

    sql.push_str("ORDER BY s.heading COLLATE NOCASE ASC LIMIT 300");

    let mut stmt = conn.prepare(&sql)?;
    let params_slice: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| &**b as &dyn rusqlite::ToSql).collect();

    let rows = stmt
        .query_map(&params_slice[..], |r| {
            Ok(SubjectAuthority {
                id: r.get(0)?,
                heading: r.get(1)?,
                scheme: r.get(2)?,
                scope_note: r.get(3)?,
                broader_term: r.get(4)?,
                narrower_terms: r.get(5)?,
                see_also: r.get(6)?,
                marc_tag: r.get(7)?,
                record_count: r.get(8)?,
                updated_at: r.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(rows)
}

pub fn save_subject(
    conn: &Connection,
    actor: &Session,
    input: &SubjectAuthorityInput,
) -> AppResult<i64> {
    let heading = validate::required("Subject heading", &input.heading, 250)?;
    let scheme = input.scheme.as_deref().unwrap_or("local");
    let scope_note = validate::optional("Scope note", input.scope_note.as_deref(), 2000)?;
    let broader_term = validate::optional("Broader term", input.broader_term.as_deref(), 250)?;
    let narrower_terms = validate::optional("Narrower terms", input.narrower_terms.as_deref(), 1000)?;
    let see_also = validate::optional("See also terms", input.see_also.as_deref(), 1000)?;
    let marc_tag = input.marc_tag.as_deref().unwrap_or("650");

    let tx = conn.unchecked_transaction()?;
    let (id, affected_records): (i64, Vec<i64>) = if let Some(id) = input.id {
        tx.execute(
            "UPDATE subjects
             SET heading = ?1, scheme = ?2, scope_note = ?3, broader_term = ?4,
                 narrower_terms = ?5, see_also = ?6, marc_tag = ?7,
                 updated_at = (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
             WHERE id = ?8",
            params![
                heading,
                scheme,
                scope_note,
                broader_term,
                narrower_terms,
                see_also,
                marc_tag,
                id
            ],
        )?;

        let recs = {
            let mut stmt = tx.prepare("SELECT DISTINCT record_id FROM record_subjects WHERE subject_id = ?1")?;
            let res = stmt.query_map([id], |r| r.get(0))?.collect::<Result<Vec<i64>, _>>()?;
            res
        };

        audit::record(
            &tx,
            Some(actor),
            "authority.subject_update",
            "subjects",
            Some(id),
            &format!("Updated subject authority {heading}"),
        )?;
        (id, recs)
    } else {
        tx.execute(
            "INSERT INTO subjects (heading, scheme, scope_note, broader_term, narrower_terms, see_also, marc_tag)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![heading, scheme, scope_note, broader_term, narrower_terms, see_also, marc_tag],
        )?;
        let new_id = tx.last_insert_rowid();
        audit::record(
            &tx,
            Some(actor),
            "authority.subject_create",
            "subjects",
            Some(new_id),
            &format!("Created subject authority {heading}"),
        )?;
        (new_id, Vec::new())
    };
    tx.commit()?;

    // Linked-record update: Re-index full-text search
    for rec_id in affected_records {
        catalogue::reindex(conn, rec_id)?;
    }

    Ok(id)
}

pub fn merge_subjects(
    conn: &Connection,
    actor: &Session,
    from_id: i64,
    to_id: i64,
) -> AppResult<()> {
    if from_id == to_id {
        return Err(AppError::Validation("Cannot merge a subject heading into itself.".into()));
    }

    let tx = conn.unchecked_transaction()?;
    let from_heading: String = tx
        .query_row("SELECT heading FROM subjects WHERE id = ?1", [from_id], |r| r.get(0))
        .optional()?
        .ok_or_else(|| AppError::NotFound(format!("Source subject #{from_id} not found.")))?;

    let to_heading: String = tx
        .query_row("SELECT heading FROM subjects WHERE id = ?1", [to_id], |r| r.get(0))
        .optional()?
        .ok_or_else(|| AppError::NotFound(format!("Target subject #{to_id} not found.")))?;

    let affected_records: Vec<i64> = {
        let mut stmt = tx.prepare("SELECT DISTINCT record_id FROM record_subjects WHERE subject_id = ?1")?;
        let res = stmt.query_map([from_id], |r| r.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        res
    };

    for rec_id in &affected_records {
        let exists: i64 = tx.query_row(
            "SELECT count(*) FROM record_subjects WHERE record_id = ?1 AND subject_id = ?2",
            params![rec_id, to_id],
            |r| r.get(0),
        )?;
        if exists > 0 {
            tx.execute(
                "DELETE FROM record_subjects WHERE record_id = ?1 AND subject_id = ?2",
                params![rec_id, from_id],
            )?;
        } else {
            tx.execute(
                "UPDATE record_subjects SET subject_id = ?1
                 WHERE record_id = ?2 AND subject_id = ?3",
                params![to_id, rec_id, from_id],
            )?;
        }
    }

    tx.execute("DELETE FROM subjects WHERE id = ?1", [from_id])?;

    audit::record(
        &tx,
        Some(actor),
        "authority.subject_merge",
        "subjects",
        Some(to_id),
        &format!("Merged subject authority {from_heading} into {to_heading}"),
    )?;
    tx.commit()?;

    // Re-index affected records
    for rec_id in affected_records {
        catalogue::reindex(conn, rec_id)?;
    }

    Ok(())
}

// ============================================================================
// Controlled MARC Mappings & Parser/Tester
// ============================================================================

pub fn list_marc_mappings(conn: &Connection) -> AppResult<Vec<MarcMapping>> {
    let mut stmt = conn.prepare(
        "SELECT id, authority_type, marc_tag, subfield_code, description, target_field, (is_default = 1) AS is_default
         FROM marc_authority_mappings
         ORDER BY marc_tag ASC, subfield_code ASC",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok(MarcMapping {
                id: r.get(0)?,
                authority_type: r.get(1)?,
                marc_tag: r.get(2)?,
                subfield_code: r.get(3)?,
                description: r.get(4)?,
                target_field: r.get(5)?,
                is_default: r.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn save_marc_mapping(
    conn: &Connection,
    actor: &Session,
    input: &MarcMappingInput,
) -> AppResult<i64> {
    let auth_type = validate::required("Authority type", &input.authority_type, 64)?;
    let tag = validate::required("MARC tag", &input.marc_tag, 10)?;
    let subfield = validate::required("Subfield code", &input.subfield_code, 5)?;
    let desc = validate::required("Description", &input.description, 200)?;
    let target = validate::required("Target field", &input.target_field, 64)?;

    let tx = conn.unchecked_transaction()?;
    let id = if let Some(id) = input.id {
        tx.execute(
            "UPDATE marc_authority_mappings
             SET authority_type = ?1, marc_tag = ?2, subfield_code = ?3, description = ?4, target_field = ?5
             WHERE id = ?6",
            params![auth_type, tag, subfield, desc, target, id],
        )?;
        audit::record(
            &tx,
            Some(actor),
            "marc_mapping.update",
            "marc_authority_mappings",
            Some(id),
            &format!("Updated MARC mapping for tag {tag}${subfield}"),
        )?;
        id
    } else {
        tx.execute(
            "INSERT INTO marc_authority_mappings (authority_type, marc_tag, subfield_code, description, target_field, is_default)
             VALUES (?1, ?2, ?3, ?4, ?5, 0)",
            params![auth_type, tag, subfield, desc, target],
        )?;
        let new_id = tx.last_insert_rowid();
        audit::record(
            &tx,
            Some(actor),
            "marc_mapping.create",
            "marc_authority_mappings",
            Some(new_id),
            &format!("Created custom MARC mapping for tag {tag}${subfield}"),
        )?;
        new_id
    };
    tx.commit()?;
    Ok(id)
}

/// Parses a sample MARC line and resolves against controlled MARC mappings.
/// Supports forms like:
/// - "100 1# $a Basheer, Vaikom Muhammad, $d 1908-1994"
/// - "650 #0 $a Malayalam fiction $x History"
/// - "264 #1 $b DC Books $a Kottayam"
/// - "082 04 $a 894.8123"
pub fn test_marc_mapping(conn: &Connection, input: &MarcTestInput) -> AppResult<MarcTestResult> {
    let line = input.raw_marc_line.trim();
    if line.is_empty() {
        return Err(AppError::Validation("MARC sample line cannot be empty.".into()));
    }

    // Split line into tag + rest: e.g. "100 1# $a Basheer, Vaikom Muhammad, $d 1908-1994"
    let mut parts = line.splitn(2, |c: char| c.is_whitespace() || c == '$');
    let tag_part = parts.next().unwrap_or("").trim();
    let rest = parts.next().unwrap_or("").trim();

    let tag = if tag_part.len() == 3 && tag_part.chars().all(|c| c.is_ascii_alphanumeric()) {
        tag_part.to_string()
    } else if let Some(idx) = line.find('$') {
        let prefix = line[..idx].trim();
        prefix.split_whitespace().next().unwrap_or("100").to_string()
    } else {
        "100".to_string()
    };

    // Extract indicators if present (e.g. "1#", "04", "#0")
    let indicators = if let Some(idx) = rest.find('$') {
        let ind_slice = rest[..idx].trim();
        if ind_slice.len() <= 3 && !ind_slice.is_empty() {
            ind_slice.to_string()
        } else {
            "##".to_string()
        }
    } else {
        "##".to_string()
    };

    // Parse subfields starting with '$' or '‡'
    let mut subfields = HashMap::new();
    let subfield_chunks: Vec<&str> = line.split(&['$', '‡'][..]).collect();
    for chunk in subfield_chunks.iter().skip(1) {
        let chunk = chunk.trim();
        if chunk.is_empty() {
            continue;
        }
        let code = chunk.chars().next().unwrap_or('a').to_string();
        let value = chunk[code.len()..]
            .trim()
            .trim_end_matches(['.', ',', ';', ':'])
            .trim()
            .to_string();
        subfields.insert(code, value);
    }

    // Look up controlled mappings for this tag
    let mut stmt = conn.prepare(
        "SELECT id, authority_type, marc_tag, subfield_code, description, target_field, (is_default = 1) AS is_default
         FROM marc_authority_mappings
         WHERE marc_tag = ?1
         ORDER BY id ASC",
    )?;
    let matched_mappings = stmt
        .query_map([&tag], |r| {
            Ok(MarcMapping {
                id: r.get(0)?,
                authority_type: r.get(1)?,
                marc_tag: r.get(2)?,
                subfield_code: r.get(3)?,
                description: r.get(4)?,
                target_field: r.get(5)?,
                is_default: r.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut target_fields = HashMap::new();
    let mut mapped_auth_type = None;

    for mapping in &matched_mappings {
        if let Some(val) = subfields.get(&mapping.subfield_code) {
            target_fields.insert(mapping.target_field.clone(), val.clone());
            if mapped_auth_type.is_none() {
                mapped_auth_type = Some(mapping.authority_type.clone());
            }
        }
    }

    Ok(MarcTestResult {
        tag,
        indicators,
        subfields,
        matched_mappings,
        mapped_authority_type: mapped_auth_type,
        target_fields,
    })
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use crate::services::catalogue::{save_record, RecordInput};

    fn setup_test_db() -> (Connection, Session) {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        migrations::run(&mut conn, migrations::MIGRATIONS).unwrap();

        let actor = Session {
            user_id: 1,
            username: "admin".into(),
            display_name: "Admin".into(),
            role: "administrator".into(),
            permissions: vec![
                "catalogue.read".into(),
                "catalogue.write".into(),
                "classification.read".into(),
                "classification.write".into(),
            ],
            revision: 1,
        };

        (conn, actor)
    }

    #[test]
    fn browse_classification_schedules_and_custom_entries() {
        let (conn, actor) = setup_test_db();

        let schemes = list_schemes(&conn).unwrap();
        assert!(schemes.len() >= 4); // ddc, udc, lcc, cc
        let ddc = schemes.iter().find(|s| s.code == "ddc").unwrap();
        assert!(!ddc.is_custom);
        assert!(ddc.entry_count > 0);

        // Browse top-level DDC entries
        let top_entries = list_entries(&conn, ddc.id, None, None).unwrap();
        assert!(top_entries.iter().any(|e| e.code == "500" && e.title == "Science"));
        assert!(top_entries.iter().any(|e| e.code == "800" && e.title.contains("Literature")));

        // Browse child entries of 500 (Mathematics 510, Physics 530, etc.)
        let science_children = list_entries(&conn, ddc.id, Some("500"), None).unwrap();
        assert!(science_children.iter().any(|e| e.code == "510" && e.title == "Mathematics"));

        // Search classification
        let search_math = list_entries(&conn, ddc.id, None, Some("Mathematics")).unwrap();
        assert!(search_math.iter().any(|e| e.code == "510"));

        // Add a custom scheme (e.g. school internal classification)
        let custom_scheme_id = save_scheme(
            &conn,
            &actor,
            &ClassificationSchemeInput {
                id: None,
                code: "school".into(),
                name: "School Library Scheme".into(),
                description: Some("Custom school curriculum classification".into()),
            },
        )
        .unwrap();

        // Add custom entry
        let custom_entry_id = save_entry(
            &conn,
            &actor,
            &ClassificationEntryInput {
                id: None,
                scheme_id: custom_scheme_id,
                code: "CURR-10".into(),
                parent_code: None,
                title: "Grade 10 Textbooks".into(),
                description: Some("Secondary school grade 10 curriculum".into()),
            },
        )
        .unwrap();

        let school_entries = list_entries(&conn, custom_scheme_id, None, None).unwrap();
        assert_eq!(school_entries.len(), 1);
        assert_eq!(school_entries[0].code, "CURR-10");

        // Delete custom entry
        delete_custom_entry(&conn, &actor, custom_entry_id).unwrap();
        assert_eq!(list_entries(&conn, custom_scheme_id, None, None).unwrap().len(), 0);

        // Delete custom scheme
        delete_custom_scheme(&conn, &actor, custom_scheme_id).unwrap();
        assert!(list_schemes(&conn).unwrap().iter().all(|s| s.code != "school"));
    }

    #[test]
    fn managed_author_publisher_and_subject_authorities_with_linked_records() {
        let (conn, actor) = setup_test_db();

        // 1. Author Authority with linked record
        let author_id = save_author(
            &conn,
            &actor,
            &AuthorAuthorityInput {
                id: None,
                name: "Basheer, Vaikom Muhammad".into(),
                sort_name: Some("Basheer, Vaikom Muhammad".into()),
                kind: Some("personal".into()),
                birth_year: Some(1908),
                death_year: Some(1994),
                authority_note: Some("Famous Malayalam novelist and freedom fighter".into()),
                variant_names: Some("Beypore Sultan".into()),
                marc_tag: Some("100".into()),
            },
        )
        .unwrap();

        // 2. Publisher Authority
        let publisher_id = save_publisher(
            &conn,
            &actor,
            &PublisherAuthorityInput {
                id: None,
                name: "DC Books".into(),
                place: Some("Kottayam, Kerala".into()),
                notes: Some("Major publisher in Kerala".into()),
                marc_tag: Some("264".into()),
            },
        )
        .unwrap();

        // 3. Subject Authority
        let subject_id = save_subject(
            &conn,
            &actor,
            &SubjectAuthorityInput {
                id: None,
                heading: "Malayalam fiction".into(),
                scheme: Some("lcsh".into()),
                scope_note: Some("Fiction written in the Malayalam language".into()),
                broader_term: Some("Dravidian literature".into()),
                narrower_terms: Some("Malayalam short stories; Malayalam novels".into()),
                see_also: Some("Malayalam literature".into()),
                marc_tag: Some("650".into()),
            },
        )
        .unwrap();

        // Catalogue a book linked to this author, publisher, and subject
        let rec = RecordInput {
            id: None,
            title: "Balyakalasakhi".into(),
            subtitle: Some("A Novel".into()),
            edition: None,
            publisher: Some("DC Books".into()),
            publication_place: Some("Kottayam".into()),
            publication_year: Some(1944),
            language: Some("mal".into()),
            physical_description: None,
            pagination: Some("120 p.".into()),
            series: None,
            abstract_text: Some("Tragic romantic novel".into()),
            notes: None,
            isbn: None,
            classification_scheme_id: Some(1), // ddc
            classification_number: Some("894.8123".into()),
            keywords: Some("romance tragedy".into()),
            contributors: vec![crate::services::catalogue::ContributorDto {
                name: "Basheer, Vaikom Muhammad".into(),
                role: "author".into(),
            }],
            subjects: vec!["Malayalam fiction".into()],
        };
        let record_id = save_record(&conn, Some(&actor), &rec).unwrap();

        // Verify record counts
        let authors = list_authors(&conn, Some("Basheer"), None).unwrap();
        assert_eq!(authors[0].record_count, 1);

        let publishers = list_publishers(&conn, Some("DC Books")).unwrap();
        assert_eq!(publishers[0].record_count, 1);

        let subjects = list_subjects(&conn, Some("Malayalam fiction"), None).unwrap();
        assert_eq!(subjects[0].record_count, 1);

        // Update Author Authority: test linked-record synchronization and FTS
        save_author(
            &conn,
            &actor,
            &AuthorAuthorityInput {
                id: Some(author_id),
                name: "V. M. Basheer".into(),
                sort_name: Some("Basheer, V. M.".into()),
                kind: Some("personal".into()),
                birth_year: Some(1908),
                death_year: Some(1994),
                authority_note: Some("Updated authority record".into()),
                variant_names: Some("Beypore Sultan".into()),
                marc_tag: Some("100".into()),
            },
        )
        .unwrap();

        // FTS search should find record with updated name
        let fts_count: i64 = conn
            .query_row(
                "SELECT count(*) FROM catalogue_fts WHERE catalogue_fts MATCH '\"V. M. Basheer\"'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(fts_count, 1);

        // Update Publisher Authority: test linked-record synchronization
        save_publisher(
            &conn,
            &actor,
            &PublisherAuthorityInput {
                id: Some(publisher_id),
                name: "D.C. Books Kerala".into(),
                place: Some("Kottayam".into()),
                notes: None,
                marc_tag: Some("264".into()),
            },
        )
        .unwrap();

        let updated_rec_publisher: String = conn
            .query_row(
                "SELECT publisher FROM bibliographic_records WHERE id = ?1",
                [record_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(updated_rec_publisher, "D.C. Books Kerala");

        // Update Subject Authority: test linked-record synchronization
        save_subject(
            &conn,
            &actor,
            &SubjectAuthorityInput {
                id: Some(subject_id),
                heading: "Malayalam romantic fiction".into(),
                scheme: Some("local".into()),
                scope_note: None,
                broader_term: None,
                narrower_terms: None,
                see_also: None,
                marc_tag: Some("650".into()),
            },
        )
        .unwrap();

        let fts_subject_count: i64 = conn
            .query_row(
                "SELECT count(*) FROM catalogue_fts WHERE catalogue_fts MATCH '\"romantic fiction\"'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(fts_subject_count, 1);
    }

    #[test]
    fn marc_authority_mappings_and_test_parsing() {
        let (conn, _actor) = setup_test_db();

        let mappings = list_marc_mappings(&conn).unwrap();
        assert!(mappings.iter().any(|m| m.marc_tag == "100" && m.subfield_code == "a"));
        assert!(mappings.iter().any(|m| m.marc_tag == "650" && m.subfield_code == "a"));
        assert!(mappings.iter().any(|m| m.marc_tag == "264" && m.subfield_code == "b"));
        assert!(mappings.iter().any(|m| m.marc_tag == "082" && m.subfield_code == "a"));

        // Test Author 100 line
        let author_test = test_marc_mapping(
            &conn,
            &MarcTestInput {
                raw_marc_line: "100 1# $a Basheer, Vaikom Muhammad, $d 1908-1994".into(),
            },
        )
        .unwrap();
        assert_eq!(author_test.tag, "100");
        assert_eq!(author_test.mapped_authority_type.as_deref(), Some("author_personal"));
        assert_eq!(author_test.target_fields.get("name").map(|s| s.as_str()), Some("Basheer, Vaikom Muhammad"));
        assert_eq!(author_test.target_fields.get("dates").map(|s| s.as_str()), Some("1908-1994"));

        // Test Subject 650 line
        let subject_test = test_marc_mapping(
            &conn,
            &MarcTestInput {
                raw_marc_line: "650 #0 $a Malayalam fiction.".into(),
            },
        )
        .unwrap();
        assert_eq!(subject_test.tag, "650");
        assert_eq!(subject_test.mapped_authority_type.as_deref(), Some("subject_topical"));
        assert_eq!(subject_test.target_fields.get("heading").map(|s| s.as_str()), Some("Malayalam fiction"));

        // Test Publisher 264 line
        let pub_test = test_marc_mapping(
            &conn,
            &MarcTestInput {
                raw_marc_line: "264 #1 $a Kottayam : $b DC Books, $c 1944".into(),
            },
        )
        .unwrap();
        assert_eq!(pub_test.tag, "264");
        assert_eq!(pub_test.mapped_authority_type.as_deref(), Some("publisher"));
        assert_eq!(pub_test.target_fields.get("name").map(|s| s.as_str()), Some("DC Books"));
        assert_eq!(pub_test.target_fields.get("place").map(|s| s.as_str()), Some("Kottayam"));

        // Test Classification 082 line
        let class_test = test_marc_mapping(
            &conn,
            &MarcTestInput {
                raw_marc_line: "082 04 $a 894.8123".into(),
            },
        )
        .unwrap();
        assert_eq!(class_test.tag, "082");
        assert_eq!(class_test.mapped_authority_type.as_deref(), Some("classification_ddc"));
        assert_eq!(class_test.target_fields.get("classification_number").map(|s| s.as_str()), Some("894.8123"));
    }
}
