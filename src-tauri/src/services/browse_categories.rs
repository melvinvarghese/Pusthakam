//! Browse categories: member-facing groupings for the public catalogue (OPAC), such as
//! "Malayalam Fiction" or "Physics". They are separate from subject headings and class numbers so
//! members never see DDC numbers. Each category may carry DDC prefix rules that place records
//! automatically; staff can add or remove categories on a record by hand, and a removal sticks
//! (stored as `excluded`) even when a rule would add the category back.

use crate::domain::validate;
use crate::error::{AppError, AppResult};
use crate::services::audit;
use crate::services::auth::Session;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const MAX_RULES: usize = 30;
const MAX_CATEGORIES_PER_RECORD: usize = 20;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowseCategory {
    pub id: i64,
    pub name: String,
    pub ddc_rules: Option<String>,
    pub sort_order: i64,
    pub is_default: bool,
    pub archived: bool,
    /// Active (non-archived) records shown under this category.
    pub record_count: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowseCategoryInput {
    pub id: Option<i64>,
    pub name: String,
    pub ddc_rules: Option<String>,
    pub sort_order: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CategoryRef {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpacCategory {
    pub id: i64,
    pub name: String,
    pub count: i64,
}

/// SQL condition (on alias `r`) for records whose class number may be matched by DDC rules.
const DDC_RECORD: &str = "r.classification_number IS NOT NULL
    AND (r.classification_scheme_id IS NULL
         OR r.classification_scheme_id = (SELECT id FROM classification_schemes WHERE code = 'ddc'))";

/// Validates and normalises a comma-separated list of DDC prefixes such as "530, 894.8123".
pub fn normalise_rules(raw: Option<&str>) -> AppResult<Option<String>> {
    let Some(raw) = raw else { return Ok(None) };
    let mut rules: Vec<String> = Vec::new();
    for part in raw.split([',', ';', '\n']) {
        let p: String = part.chars().filter(|c| !c.is_whitespace()).collect();
        if p.is_empty() {
            continue;
        }
        let (whole, fraction) = match p.split_once('.') {
            Some((w, f)) => (w, Some(f)),
            None => (p.as_str(), None),
        };
        let digits = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit());
        let valid = digits(whole)
            && whole.len() <= 3
            && match fraction {
                None => true,
                Some(f) => whole.len() == 3 && digits(f) && f.len() <= 12,
            };
        if !valid {
            return Err(AppError::Validation(format!(
                "\"{p}\" is not a DDC number. Use numbers such as 530 or 894.8123, separated by commas."
            )));
        }
        if !rules.contains(&p) {
            rules.push(p);
        }
    }
    if rules.len() > MAX_RULES {
        return Err(AppError::Validation(format!(
            "A category can have at most {MAX_RULES} DDC numbers."
        )));
    }
    Ok(if rules.is_empty() {
        None
    } else {
        Some(rules.join(","))
    })
}

pub fn list_categories(
    conn: &Connection,
    include_archived: bool,
) -> AppResult<Vec<BrowseCategory>> {
    let mut stmt = conn.prepare(
        "SELECT c.id, c.name, c.ddc_rules, c.sort_order, c.is_default, c.archived_at IS NOT NULL,
                (SELECT COUNT(*) FROM record_categories rc
                   JOIN bibliographic_records r ON r.id = rc.record_id AND r.archived_at IS NULL
                  WHERE rc.category_id = c.id AND rc.source IN ('rule','manual'))
           FROM browse_categories c
          WHERE ?1 = 1 OR c.archived_at IS NULL
          ORDER BY c.archived_at IS NOT NULL, c.sort_order, c.name COLLATE NOCASE",
    )?;
    let rows = stmt
        .query_map([include_archived as i64], |r| {
            Ok(BrowseCategory {
                id: r.get(0)?,
                name: r.get(1)?,
                ddc_rules: r.get(2)?,
                sort_order: r.get(3)?,
                is_default: r.get(4)?,
                archived: r.get(5)?,
                record_count: r.get(6)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(rows)
}

pub fn save_category(
    conn: &Connection,
    actor: &Session,
    input: &BrowseCategoryInput,
) -> AppResult<i64> {
    let name = validate::required("Category name", &input.name, 80)?;
    let rules = normalise_rules(input.ddc_rules.as_deref())?;
    let tx = conn.unchecked_transaction()?;
    let duplicate: Option<i64> = tx
        .query_row(
            "SELECT id FROM browse_categories WHERE name = ?1 AND (?2 IS NULL OR id <> ?2)",
            params![name, input.id],
            |r| r.get(0),
        )
        .optional()?;
    if duplicate.is_some() {
        return Err(AppError::Conflict(format!(
            "A category called \"{name}\" already exists."
        )));
    }
    let id = match input.id {
        Some(id) => {
            let sort_order = input.sort_order;
            let changed = tx.execute(
                "UPDATE browse_categories
                    SET name = ?1, ddc_rules = ?2, sort_order = COALESCE(?3, sort_order),
                        updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now')
                  WHERE id = ?4",
                params![name, rules, sort_order, id],
            )?;
            if changed == 0 {
                return Err(AppError::NotFound("That category no longer exists.".into()));
            }
            audit::record(
                &tx,
                Some(actor),
                "browse_category.update",
                "browse_categories",
                Some(id),
                &format!("Updated browse category {name}"),
            )?;
            id
        }
        None => {
            let sort_order = match input.sort_order {
                Some(s) => s,
                None => tx.query_row(
                    "SELECT COALESCE(MAX(sort_order), 0) + 10 FROM browse_categories",
                    [],
                    |r| r.get(0),
                )?,
            };
            tx.execute(
                "INSERT INTO browse_categories (name, ddc_rules, sort_order) VALUES (?1, ?2, ?3)",
                params![name, rules, sort_order],
            )?;
            let id = tx.last_insert_rowid();
            audit::record(
                &tx,
                Some(actor),
                "browse_category.create",
                "browse_categories",
                Some(id),
                &format!("Created browse category {name}"),
            )?;
            id
        }
    };
    apply_all_rules(&tx)?;
    tx.commit()?;
    Ok(id)
}

/// Archiving hides a category from staff pickers and the OPAC; its links are kept so restoring it
/// brings the same books back.
pub fn set_category_archived(
    conn: &Connection,
    actor: &Session,
    id: i64,
    archived: bool,
) -> AppResult<()> {
    let tx = conn.unchecked_transaction()?;
    let name: String = tx
        .query_row(
            "SELECT name FROM browse_categories WHERE id = ?1",
            [id],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound("That category no longer exists.".into()))?;
    tx.execute(
        "UPDATE browse_categories
            SET archived_at = CASE WHEN ?1 THEN COALESCE(archived_at, strftime('%Y-%m-%dT%H:%M:%SZ','now')) END,
                updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now')
          WHERE id = ?2",
        params![archived, id],
    )?;
    let (action, verb) = if archived {
        ("browse_category.archive", "Archived")
    } else {
        ("browse_category.restore", "Restored")
    };
    audit::record(
        &tx,
        Some(actor),
        action,
        "browse_categories",
        Some(id),
        &format!("{verb} browse category {name}"),
    )?;
    apply_all_rules(&tx)?;
    tx.commit()?;
    Ok(())
}

/// Recomputes rule-based placements for the whole catalogue. Manual and excluded links are kept.
pub fn apply_all_rules(conn: &Connection) -> AppResult<()> {
    conn.execute("DELETE FROM record_categories WHERE source = 'rule'", [])?;
    conn.execute(
        &format!(
            "INSERT OR IGNORE INTO record_categories (record_id, category_id, source)
             SELECT DISTINCT r.id, br.category_id, 'rule'
               FROM bibliographic_records r
               JOIN browse_category_rules br ON trim(r.classification_number) LIKE br.prefix || '%'
              WHERE {DDC_RECORD}"
        ),
        [],
    )?;
    Ok(())
}

fn rule_matches(conn: &Connection, record_id: i64) -> AppResult<HashSet<i64>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT DISTINCT br.category_id
           FROM bibliographic_records r
           JOIN browse_category_rules br ON trim(r.classification_number) LIKE br.prefix || '%'
          WHERE r.id = ?1 AND {DDC_RECORD}"
    ))?;
    let ids = stmt
        .query_map([record_id], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    Ok(ids)
}

/// Recomputes rule-based placements for one record (after its class number changed).
pub fn apply_rules_for_record(conn: &Connection, record_id: i64) -> AppResult<()> {
    conn.execute(
        "DELETE FROM record_categories WHERE record_id = ?1 AND source = 'rule'",
        [record_id],
    )?;
    for category_id in rule_matches(conn, record_id)? {
        conn.execute(
            "INSERT OR IGNORE INTO record_categories (record_id, category_id, source) VALUES (?1, ?2, 'rule')",
            params![record_id, category_id],
        )?;
    }
    Ok(())
}

/// Sets the record's active categories to exactly `ids`, as chosen by staff. Categories a rule
/// would add but staff left out are stored as `excluded`. Links to archived categories are kept.
pub fn set_record_categories(conn: &Connection, record_id: i64, ids: &[i64]) -> AppResult<()> {
    let mut wanted: Vec<i64> = Vec::new();
    for &id in ids {
        if !wanted.contains(&id) {
            wanted.push(id);
        }
    }
    if wanted.len() > MAX_CATEGORIES_PER_RECORD {
        return Err(AppError::Validation(format!(
            "A book can be in at most {MAX_CATEGORIES_PER_RECORD} categories."
        )));
    }
    for &id in &wanted {
        let active: Option<bool> = conn
            .query_row(
                "SELECT archived_at IS NULL FROM browse_categories WHERE id = ?1",
                [id],
                |r| r.get(0),
            )
            .optional()?;
        if active != Some(true) {
            return Err(AppError::Validation(
                "One of the chosen categories no longer exists. Refresh and try again.".into(),
            ));
        }
    }
    let matches = rule_matches(conn, record_id)?;
    conn.execute(
        "DELETE FROM record_categories
          WHERE record_id = ?1
            AND category_id IN (SELECT id FROM browse_categories WHERE archived_at IS NULL)",
        [record_id],
    )?;
    for &id in &wanted {
        let source = if matches.contains(&id) {
            "rule"
        } else {
            "manual"
        };
        conn.execute(
            "INSERT INTO record_categories (record_id, category_id, source) VALUES (?1, ?2, ?3)",
            params![record_id, id, source],
        )?;
    }
    for id in matches.iter().filter(|id| !wanted.contains(id)) {
        conn.execute(
            "INSERT INTO record_categories (record_id, category_id, source) VALUES (?1, ?2, 'excluded')",
            params![record_id, id],
        )?;
    }
    Ok(())
}

/// Active categories shown for a record.
pub fn record_categories(conn: &Connection, record_id: i64) -> AppResult<Vec<CategoryRef>> {
    let mut stmt = conn.prepare(
        "SELECT c.id, c.name FROM record_categories rc
           JOIN browse_categories c ON c.id = rc.category_id
          WHERE rc.record_id = ?1 AND rc.source IN ('rule','manual') AND c.archived_at IS NULL
          ORDER BY c.sort_order, c.name COLLATE NOCASE",
    )?;
    let rows = stmt
        .query_map([record_id], |r| {
            Ok(CategoryRef {
                id: r.get(0)?,
                name: r.get(1)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(rows)
}

/// Categories for the public catalogue: active categories that hold at least one visible record.
pub fn opac_categories(conn: &Connection) -> AppResult<Vec<OpacCategory>> {
    let mut stmt = conn.prepare(
        "SELECT c.id, c.name, COUNT(DISTINCT rc.record_id)
           FROM browse_categories c
           JOIN record_categories rc ON rc.category_id = c.id AND rc.source IN ('rule','manual')
           JOIN bibliographic_records r ON r.id = rc.record_id AND r.archived_at IS NULL
          WHERE c.archived_at IS NULL
          GROUP BY c.id
          ORDER BY c.sort_order, c.name COLLATE NOCASE",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok(OpacCategory {
                id: r.get(0)?,
                name: r.get(1)?,
                count: r.get(2)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;

    fn db() -> (Connection, Session) {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        migrations::run(&mut conn, migrations::MIGRATIONS).unwrap();
        let actor = Session {
            user_id: 1,
            username: "admin".into(),
            display_name: "Admin".into(),
            role: "administrator".into(),
            permissions: vec!["catalogue.write".into()],
            revision: 1,
        };
        (conn, actor)
    }

    fn id_of(conn: &Connection, name: &str) -> i64 {
        conn.query_row(
            "SELECT id FROM browse_categories WHERE name = ?1",
            [name],
            |r| r.get(0),
        )
        .unwrap()
    }

    fn add_record(conn: &Connection, id: i64, class: Option<&str>) {
        conn.execute(
            "INSERT INTO bibliographic_records (id, title, classification_number) VALUES (?1, ?2, ?3)",
            params![id, format!("Book {id}"), class],
        )
        .unwrap();
    }

    fn names(conn: &Connection, record_id: i64) -> Vec<String> {
        record_categories(conn, record_id)
            .unwrap()
            .into_iter()
            .map(|c| c.name)
            .collect()
    }

    #[test]
    fn ships_default_categories_with_rules() {
        let (conn, _) = db();
        let all = list_categories(&conn, false).unwrap();
        assert!(all.len() >= 25);
        let mf = all.iter().find(|c| c.name == "Malayalam Fiction").unwrap();
        assert_eq!(mf.ddc_rules.as_deref(), Some("894.8123"));
        assert!(all.iter().any(|c| c.name == "Physics"));
        assert!(all.iter().any(|c| c.name == "Management & Business"));
    }

    #[test]
    fn rules_place_books_by_ddc_prefix() {
        let (conn, _) = db();
        add_record(&conn, 1, Some("894.8123 BAS"));
        add_record(&conn, 2, Some(" 530.1"));
        add_record(&conn, 3, Some("658.4"));
        add_record(&conn, 4, None);
        add_record(&conn, 5, Some("823.914"));
        apply_all_rules(&conn).unwrap();
        assert_eq!(names(&conn, 1), vec!["Malayalam Fiction"]);
        assert_eq!(names(&conn, 2), vec!["Physics"]);
        assert_eq!(names(&conn, 3), vec!["Management & Business"]);
        assert!(names(&conn, 4).is_empty());
        assert_eq!(names(&conn, 5), vec!["English Fiction"]);
    }

    #[test]
    fn non_ddc_class_numbers_are_not_matched() {
        let (conn, _) = db();
        let udc: i64 = conn
            .query_row(
                "SELECT id FROM classification_schemes WHERE code = 'udc'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            "INSERT INTO bibliographic_records (id, title, classification_scheme_id, classification_number) VALUES (9, 'UDC book', ?1, '53')",
            [udc],
        )
        .unwrap();
        apply_all_rules(&conn).unwrap();
        assert!(names(&conn, 9).is_empty());
    }

    #[test]
    fn staff_choices_survive_rule_recomputation() {
        let (conn, actor) = db();
        add_record(&conn, 1, Some("530"));
        apply_all_rules(&conn).unwrap();
        let physics = id_of(&conn, "Physics");
        let exams = id_of(&conn, "Competitive Exams");
        // Staff remove Physics and add Competitive Exams.
        set_record_categories(&conn, 1, &[exams]).unwrap();
        assert_eq!(names(&conn, 1), vec!["Competitive Exams"]);
        apply_all_rules(&conn).unwrap();
        apply_rules_for_record(&conn, 1).unwrap();
        assert_eq!(
            names(&conn, 1),
            vec!["Competitive Exams"],
            "removed rule category stays removed"
        );
        // Staff add Physics back: it shows again.
        set_record_categories(&conn, 1, &[physics, exams]).unwrap();
        assert_eq!(names(&conn, 1), vec!["Physics", "Competitive Exams"]);
        // Editing rules re-applies across the catalogue.
        let id = save_category(
            &conn,
            &actor,
            &BrowseCategoryInput {
                id: Some(exams),
                name: "Competitive Exams".into(),
                ddc_rules: Some("530".into()),
                sort_order: None,
            },
        )
        .unwrap();
        assert_eq!(id, exams);
        add_record(&conn, 2, Some("530.4"));
        apply_rules_for_record(&conn, 2).unwrap();
        assert_eq!(names(&conn, 2), vec!["Physics", "Competitive Exams"]);
    }

    #[test]
    fn validates_names_and_rules() {
        let (conn, actor) = db();
        let input = |name: &str, rules: Option<&str>| BrowseCategoryInput {
            id: None,
            name: name.into(),
            ddc_rules: rules.map(Into::into),
            sort_order: None,
        };
        assert!(matches!(
            save_category(&conn, &actor, &input("physics", None)),
            Err(AppError::Conflict(_))
        ));
        assert!(matches!(
            save_category(&conn, &actor, &input("  ", None)),
            Err(AppError::Validation(_))
        ));
        for bad in ["5%", "abc", "53.1", "1234", "894.", ".5"] {
            assert!(
                matches!(
                    save_category(&conn, &actor, &input("Local History", Some(bad))),
                    Err(AppError::Validation(_))
                ),
                "{bad} should be rejected"
            );
        }
        assert_eq!(
            normalise_rules(Some(" 954.83 , 954.83; 3"))
                .unwrap()
                .as_deref(),
            Some("954.83,3")
        );
        assert_eq!(normalise_rules(Some(" , ")).unwrap(), None);
        let id = save_category(&conn, &actor, &input("Kerala History", Some("954.83"))).unwrap();
        let saved = list_categories(&conn, false)
            .unwrap()
            .into_iter()
            .find(|c| c.id == id)
            .unwrap();
        assert!(!saved.is_default);
        assert!(saved.sort_order > 310, "new categories go to the end");
    }

    #[test]
    fn opac_lists_only_active_non_empty_categories() {
        let (conn, actor) = db();
        add_record(&conn, 1, Some("530"));
        add_record(&conn, 2, Some("531"));
        add_record(&conn, 3, Some("540"));
        conn.execute(
            "UPDATE bibliographic_records SET archived_at = '2026-01-01' WHERE id = 3",
            [],
        )
        .unwrap();
        apply_all_rules(&conn).unwrap();
        let cats = opac_categories(&conn).unwrap();
        assert_eq!(cats.len(), 1);
        assert_eq!((cats[0].name.as_str(), cats[0].count), ("Physics", 2));
        let physics = id_of(&conn, "Physics");
        set_category_archived(&conn, &actor, physics, true).unwrap();
        assert!(opac_categories(&conn).unwrap().is_empty());
        assert!(names(&conn, 1).is_empty());
        assert!(matches!(
            set_record_categories(&conn, 1, &[physics]),
            Err(AppError::Validation(_))
        ));
        set_category_archived(&conn, &actor, physics, false).unwrap();
        assert_eq!(opac_categories(&conn).unwrap()[0].count, 2);
    }
}
