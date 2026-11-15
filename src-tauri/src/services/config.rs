//! Library configuration: library details, categories, collections, locations, shelves and
//! circulation policies. Everything here is plain editable data (no per-library-type code).
//! Table names come from the enums below, never from user input.

use crate::domain::validate::{in_range, optional, required};
use crate::error::{AppError, AppResult};
use crate::services::audit;
use crate::services::auth::Session;
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

// ---------- output ----------

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    pub name: String,
    pub profile: String,
    pub address: Option<String>,
    pub contact: Option<String>,
    pub currency: String,
    pub locale: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Named {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub archived: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberCategory {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub membership_days: i64,
    pub archived: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Shelf {
    pub id: i64,
    pub location_id: i64,
    pub code: String,
    pub description: Option<String>,
    pub archived: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Policy {
    pub id: i64,
    pub member_category_id: i64,
    pub item_category_id: i64,
    pub loan_days: i64,
    pub max_loans: i64,
    pub max_renewals: i64,
    pub renewal_days: i64,
    pub fine_per_day_minor: i64,
    pub fine_grace_days: i64,
    pub fine_cap_minor: Option<i64>,
    pub reservation_hold_days: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Scheme {
    pub id: i64,
    pub code: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Overview {
    pub library: Option<Library>,
    pub classification_schemes: Vec<Scheme>,
    pub member_categories: Vec<MemberCategory>,
    pub item_categories: Vec<Named>,
    pub collections: Vec<Named>,
    pub locations: Vec<Named>,
    pub shelves: Vec<Shelf>,
    pub policies: Vec<Policy>,
}

// ---------- input ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryInput {
    pub name: String,
    pub profile: String,
    pub address: Option<String>,
    pub contact: Option<String>,
    pub currency: String,
    pub locale: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NamedInput {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberCategoryInput {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub membership_days: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShelfInput {
    pub id: Option<i64>,
    pub location_id: i64,
    pub code: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyInput {
    pub id: Option<i64>,
    pub member_category_id: i64,
    pub item_category_id: i64,
    pub loan_days: i64,
    pub max_loans: i64,
    pub max_renewals: i64,
    pub renewal_days: i64,
    pub fine_per_day_minor: i64,
    pub fine_grace_days: i64,
    pub fine_cap_minor: Option<i64>,
    pub reservation_hold_days: i64,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NamedKind {
    Collection,
    Location,
    ItemCategory,
}

impl NamedKind {
    fn table(self) -> &'static str {
        match self {
            NamedKind::Collection => "collections",
            NamedKind::Location => "locations",
            NamedKind::ItemCategory => "item_categories",
        }
    }
    fn label(self) -> &'static str {
        match self {
            NamedKind::Collection => "collection",
            NamedKind::Location => "location",
            NamedKind::ItemCategory => "item category",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ArchiveKind {
    Collection,
    Location,
    ItemCategory,
    MemberCategory,
    Shelf,
}

impl ArchiveKind {
    fn table(self) -> &'static str {
        match self {
            ArchiveKind::Collection => "collections",
            ArchiveKind::Location => "locations",
            ArchiveKind::ItemCategory => "item_categories",
            ArchiveKind::MemberCategory => "member_categories",
            ArchiveKind::Shelf => "shelves",
        }
    }
}

// ---------- helpers ----------

/// Turns a UNIQUE violation into a friendly conflict; foreign-key failures into validation errors.
fn friendly(e: rusqlite::Error, duplicate: &str) -> AppError {
    if let rusqlite::Error::SqliteFailure(f, _) = &e {
        match f.extended_code {
            2067 | 1555 => return AppError::Conflict(duplicate.to_string()),
            787 => return AppError::Validation("That choice no longer exists. Reload and try again.".into()),
            _ => {}
        }
    }
    AppError::Database(e)
}

const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%SZ','now')";

// ---------- reads ----------

fn named(conn: &Connection, table: &str) -> AppResult<Vec<Named>> {
    let sql = format!("SELECT id, name, description, archived_at IS NOT NULL FROM {table} ORDER BY name");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map([], |r| Ok(Named { id: r.get(0)?, name: r.get(1)?, description: r.get(2)?, archived: r.get(3)? }))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn library(conn: &Connection) -> AppResult<Option<Library>> {
    Ok(conn
        .query_row("SELECT name, profile, address, contact, currency, locale FROM libraries WHERE id = 1", [], |r| {
            Ok(Library {
                name: r.get(0)?,
                profile: r.get(1)?,
                address: r.get(2)?,
                contact: r.get(3)?,
                currency: r.get(4)?,
                locale: r.get(5)?,
            })
        })
        .optional()?)
}

pub fn overview(conn: &Connection) -> AppResult<Overview> {
    let member_categories = conn
        .prepare("SELECT id, name, description, membership_days, archived_at IS NOT NULL FROM member_categories ORDER BY name")?
        .query_map([], |r| {
            Ok(MemberCategory {
                id: r.get(0)?,
                name: r.get(1)?,
                description: r.get(2)?,
                membership_days: r.get(3)?,
                archived: r.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let shelves = conn
        .prepare("SELECT id, location_id, code, description, archived_at IS NOT NULL FROM shelves ORDER BY location_id, code")?
        .query_map([], |r| {
            Ok(Shelf { id: r.get(0)?, location_id: r.get(1)?, code: r.get(2)?, description: r.get(3)?, archived: r.get(4)? })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let policies = conn
        .prepare(
            "SELECT id, member_category_id, item_category_id, loan_days, max_loans, max_renewals, renewal_days,
                    fine_per_day_minor, fine_grace_days, fine_cap_minor, reservation_hold_days
             FROM circulation_policies ORDER BY member_category_id, item_category_id",
        )?
        .query_map([], |r| {
            Ok(Policy {
                id: r.get(0)?,
                member_category_id: r.get(1)?,
                item_category_id: r.get(2)?,
                loan_days: r.get(3)?,
                max_loans: r.get(4)?,
                max_renewals: r.get(5)?,
                renewal_days: r.get(6)?,
                fine_per_day_minor: r.get(7)?,
                fine_grace_days: r.get(8)?,
                fine_cap_minor: r.get(9)?,
                reservation_hold_days: r.get(10)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let classification_schemes = conn
        .prepare("SELECT id, code, name FROM classification_schemes ORDER BY id")?
        .query_map([], |r| Ok(Scheme { id: r.get(0)?, code: r.get(1)?, name: r.get(2)? }))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Overview {
        library: library(conn)?,
        classification_schemes,
        member_categories,
        item_categories: named(conn, "item_categories")?,
        collections: named(conn, "collections")?,
        locations: named(conn, "locations")?,
        shelves,
        policies,
    })
}

// ---------- writes ----------

pub fn update_library(conn: &Connection, actor: Option<&Session>, input: &LibraryInput) -> AppResult<()> {
    let name = required("Library name", &input.name, 120)?;
    if crate::domain::profiles::defaults_for(&input.profile).is_none() {
        return Err(AppError::Validation("Choose a library type.".into()));
    }
    let address = optional("Address", input.address.as_deref(), 300)?;
    let contact = optional("Contact", input.contact.as_deref(), 200)?;
    let currency = crate::domain::validate::currency(&input.currency)?;
    let locale = required("Language", &input.locale, 20)?;
    let n = conn.execute(
        &format!(
            "UPDATE libraries SET name=?1, profile=?2, address=?3, contact=?4, currency=?5, locale=?6,
             updated_at={NOW} WHERE id = 1"
        ),
        (&name, &input.profile, &address, &contact, &currency, &locale),
    )?;
    if n == 0 {
        return Err(AppError::NotFound("The library has not been set up yet.".into()));
    }
    audit::record(conn, actor, "config.library_updated", "library", Some(1), "Library details updated")?;
    Ok(())
}

pub fn save_named(conn: &Connection, actor: Option<&Session>, kind: NamedKind, input: &NamedInput) -> AppResult<i64> {
    let name = required("Name", &input.name, 80)?;
    let description = optional("Description", input.description.as_deref(), 300)?;
    let table = kind.table();
    let dup = format!("A {} named \"{}\" already exists.", kind.label(), name);
    let id = match input.id {
        Some(id) => {
            let n = conn
                .execute(
                    &format!("UPDATE {table} SET name=?1, description=?2, updated_at={NOW} WHERE id=?3"),
                    (&name, &description, id),
                )
                .map_err(|e| friendly(e, &dup))?;
            if n == 0 {
                return Err(AppError::NotFound(format!("That {} no longer exists.", kind.label())));
            }
            id
        }
        None => {
            // `locations` has no updated_at default issue: all three tables share these columns.
            conn.execute(&format!("INSERT INTO {table} (name, description) VALUES (?1, ?2)"), (&name, &description))
                .map_err(|e| friendly(e, &dup))?;
            conn.last_insert_rowid()
        }
    };
    audit::record(conn, actor, "config.saved", table, Some(id), &format!("Saved {} \"{}\"", kind.label(), name))?;
    Ok(id)
}

pub fn save_member_category(conn: &Connection, actor: Option<&Session>, input: &MemberCategoryInput) -> AppResult<i64> {
    let name = required("Name", &input.name, 80)?;
    let description = optional("Description", input.description.as_deref(), 300)?;
    let days = in_range("Membership period (days)", input.membership_days, 1, 36500)?;
    let dup = format!("A member category named \"{name}\" already exists.");
    let id = match input.id {
        Some(id) => {
            let n = conn
                .execute(
                    &format!("UPDATE member_categories SET name=?1, description=?2, membership_days=?3, updated_at={NOW} WHERE id=?4"),
                    (&name, &description, days, id),
                )
                .map_err(|e| friendly(e, &dup))?;
            if n == 0 {
                return Err(AppError::NotFound("That member category no longer exists.".into()));
            }
            id
        }
        None => {
            conn.execute(
                "INSERT INTO member_categories (name, description, membership_days) VALUES (?1, ?2, ?3)",
                (&name, &description, days),
            )
            .map_err(|e| friendly(e, &dup))?;
            conn.last_insert_rowid()
        }
    };
    audit::record(conn, actor, "config.saved", "member_categories", Some(id), &format!("Saved member category \"{name}\""))?;
    Ok(id)
}

pub fn save_shelf(conn: &Connection, actor: Option<&Session>, input: &ShelfInput) -> AppResult<i64> {
    let code = required("Shelf code", &input.code, 40)?;
    let description = optional("Description", input.description.as_deref(), 300)?;
    let dup = format!("Shelf \"{code}\" already exists in this location.");
    let id = match input.id {
        Some(id) => {
            let n = conn
                .execute(
                    "UPDATE shelves SET location_id=?1, code=?2, description=?3 WHERE id=?4",
                    (input.location_id, &code, &description, id),
                )
                .map_err(|e| friendly(e, &dup))?;
            if n == 0 {
                return Err(AppError::NotFound("That shelf no longer exists.".into()));
            }
            id
        }
        None => {
            conn.execute(
                "INSERT INTO shelves (location_id, code, description) VALUES (?1, ?2, ?3)",
                (input.location_id, &code, &description),
            )
            .map_err(|e| friendly(e, &dup))?;
            conn.last_insert_rowid()
        }
    };
    audit::record(conn, actor, "config.saved", "shelves", Some(id), &format!("Saved shelf \"{code}\""))?;
    Ok(id)
}

pub fn save_policy(conn: &Connection, actor: Option<&Session>, p: &PolicyInput) -> AppResult<i64> {
    in_range("Loan period (days)", p.loan_days, 1, 365)?;
    in_range("Maximum loans", p.max_loans, 1, 500)?;
    in_range("Maximum renewals", p.max_renewals, 0, 50)?;
    in_range("Renewal period (days)", p.renewal_days, 0, 365)?;
    in_range("Fine grace days", p.fine_grace_days, 0, 365)?;
    in_range("Reservation hold (days)", p.reservation_hold_days, 1, 60)?;
    if p.fine_per_day_minor < 0 || p.fine_cap_minor.is_some_and(|c| c < 0) {
        return Err(AppError::Validation("Fine amounts cannot be negative.".into()));
    }
    if p.max_renewals > 0 && p.renewal_days == 0 {
        return Err(AppError::Validation("Set a renewal period when renewals are allowed.".into()));
    }
    let dup = "A policy for that member category and item category already exists.";
    let id = match p.id {
        Some(id) => {
            let n = conn
                .execute(
                    &format!(
                        "UPDATE circulation_policies SET member_category_id=?1, item_category_id=?2, loan_days=?3,
                         max_loans=?4, max_renewals=?5, renewal_days=?6, fine_per_day_minor=?7, fine_grace_days=?8,
                         fine_cap_minor=?9, reservation_hold_days=?10, updated_at={NOW} WHERE id=?11"
                    ),
                    rusqlite::params![
                        p.member_category_id, p.item_category_id, p.loan_days, p.max_loans, p.max_renewals,
                        p.renewal_days, p.fine_per_day_minor, p.fine_grace_days, p.fine_cap_minor,
                        p.reservation_hold_days, id
                    ],
                )
                .map_err(|e| friendly(e, dup))?;
            if n == 0 {
                return Err(AppError::NotFound("That policy no longer exists.".into()));
            }
            id
        }
        None => {
            conn.execute(
                "INSERT INTO circulation_policies (member_category_id, item_category_id, loan_days, max_loans,
                   max_renewals, renewal_days, fine_per_day_minor, fine_grace_days, fine_cap_minor, reservation_hold_days)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                rusqlite::params![
                    p.member_category_id, p.item_category_id, p.loan_days, p.max_loans, p.max_renewals,
                    p.renewal_days, p.fine_per_day_minor, p.fine_grace_days, p.fine_cap_minor, p.reservation_hold_days
                ],
            )
            .map_err(|e| friendly(e, dup))?;
            conn.last_insert_rowid()
        }
    };
    audit::record(conn, actor, "config.saved", "circulation_policies", Some(id), "Saved circulation policy")?;
    Ok(id)
}

/// Policies hold no history themselves (loans keep their own snapshot), so they may be deleted.
pub fn delete_policy(conn: &Connection, actor: Option<&Session>, id: i64) -> AppResult<()> {
    let n = conn.execute("DELETE FROM circulation_policies WHERE id = ?1", [id])?;
    if n == 0 {
        return Err(AppError::NotFound("That policy no longer exists.".into()));
    }
    audit::record(conn, actor, "config.deleted", "circulation_policies", Some(id), "Deleted circulation policy")?;
    Ok(())
}

pub fn set_archived(conn: &Connection, actor: Option<&Session>, kind: ArchiveKind, id: i64, archived: bool) -> AppResult<()> {
    let table = kind.table();
    let n = conn.execute(
        &format!("UPDATE {table} SET archived_at = CASE WHEN ?1 THEN {NOW} ELSE NULL END WHERE id = ?2"),
        (archived, id),
    )?;
    if n == 0 {
        return Err(AppError::NotFound("That item no longer exists.".into()));
    }
    let action = if archived { "config.archived" } else { "config.restored" };
    audit::record(conn, actor, action, table, Some(id), if archived { "Archived" } else { "Restored" })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn conn() -> (tempfile::TempDir, Connection) {
        let tmp = tempfile::tempdir().unwrap();
        let c = db::open(&tmp.path().join("c.db")).unwrap();
        c.execute(
            "INSERT INTO libraries (id, name, profile, currency, locale) VALUES (1, 'Demo Library', 'school', 'INR', 'en')",
            [],
        )
        .unwrap();
        (tmp, c)
    }

    fn named_input(name: &str) -> NamedInput {
        NamedInput { id: None, name: name.into(), description: None }
    }

    fn policy(member: i64, item: i64) -> PolicyInput {
        PolicyInput {
            id: None, member_category_id: member, item_category_id: item, loan_days: 14, max_loans: 3,
            max_renewals: 1, renewal_days: 7, fine_per_day_minor: 100, fine_grace_days: 1, fine_cap_minor: None,
            reservation_hold_days: 3,
        }
    }

    fn make_categories(c: &Connection) -> (i64, i64) {
        let m = save_member_category(
            c, None, &MemberCategoryInput { id: None, name: "Student".into(), description: None, membership_days: 365 },
        )
        .unwrap();
        let i = save_named(c, None, NamedKind::ItemCategory, &named_input("General")).unwrap();
        (m, i)
    }

    #[test]
    fn names_are_trimmed_unique_case_insensitively_and_validated() {
        let (_t, c) = conn();
        let id = save_named(&c, None, NamedKind::Collection, &named_input("  Fiction ")).unwrap();
        assert!(matches!(save_named(&c, None, NamedKind::Collection, &named_input("FICTION")), Err(AppError::Conflict(_))));
        assert!(matches!(save_named(&c, None, NamedKind::Collection, &named_input("   ")), Err(AppError::Validation(_))));
        // Same name is fine in a different kind of list.
        save_named(&c, None, NamedKind::Location, &named_input("Fiction")).unwrap();
        // Rename and update.
        save_named(&c, None, NamedKind::Collection, &NamedInput { id: Some(id), name: "Novels".into(), description: Some("Stories".into()) }).unwrap();
        let o = overview(&c).unwrap();
        assert_eq!(o.collections[0].name, "Novels");
        assert_eq!(o.collections[0].description.as_deref(), Some("Stories"));
        assert!(matches!(
            save_named(&c, None, NamedKind::Collection, &NamedInput { id: Some(9999), name: "X".into(), description: None }),
            Err(AppError::NotFound(_))
        ));
    }

    #[test]
    fn archiving_hides_nothing_but_flags_and_can_be_undone() {
        let (_t, c) = conn();
        let id = save_named(&c, None, NamedKind::Location, &named_input("Annex")).unwrap();
        set_archived(&c, None, ArchiveKind::Location, id, true).unwrap();
        assert!(overview(&c).unwrap().locations[0].archived);
        set_archived(&c, None, ArchiveKind::Location, id, false).unwrap();
        assert!(!overview(&c).unwrap().locations[0].archived);
        assert!(matches!(set_archived(&c, None, ArchiveKind::Shelf, 77, true), Err(AppError::NotFound(_))));
    }

    #[test]
    fn shelf_codes_are_unique_per_location_only() {
        let (_t, c) = conn();
        let a = save_named(&c, None, NamedKind::Location, &named_input("Main")).unwrap();
        let b = save_named(&c, None, NamedKind::Location, &named_input("Annex")).unwrap();
        let shelf = |loc, code: &str| ShelfInput { id: None, location_id: loc, code: code.into(), description: None };
        save_shelf(&c, None, &shelf(a, "A1")).unwrap();
        save_shelf(&c, None, &shelf(b, "A1")).unwrap();
        assert!(matches!(save_shelf(&c, None, &shelf(a, "a1")), Err(AppError::Conflict(_))));
        assert!(matches!(save_shelf(&c, None, &shelf(999, "Z")), Err(AppError::Validation(_))));
    }

    #[test]
    fn policies_are_validated_and_unique_per_category_pair() {
        let (_t, c) = conn();
        let (m, i) = make_categories(&c);
        let id = save_policy(&c, None, &policy(m, i)).unwrap();
        assert!(matches!(save_policy(&c, None, &policy(m, i)), Err(AppError::Conflict(_))));

        let mut bad = policy(m, i);
        bad.loan_days = 0;
        assert!(matches!(save_policy(&c, None, &bad), Err(AppError::Validation(_))));
        let mut bad = policy(m, i);
        bad.max_renewals = 2;
        bad.renewal_days = 0;
        assert!(matches!(save_policy(&c, None, &bad), Err(AppError::Validation(_))));
        let mut bad = policy(m, i);
        bad.fine_per_day_minor = -1;
        assert!(matches!(save_policy(&c, None, &bad), Err(AppError::Validation(_))));
        assert!(matches!(save_policy(&c, None, &policy(999, i)), Err(AppError::Validation(_))));

        let mut upd = policy(m, i);
        upd.id = Some(id);
        upd.loan_days = 21;
        upd.fine_cap_minor = Some(5000);
        save_policy(&c, None, &upd).unwrap();
        let p = &overview(&c).unwrap().policies[0];
        assert_eq!((p.loan_days, p.fine_cap_minor), (21, Some(5000)));

        delete_policy(&c, None, id).unwrap();
        assert!(overview(&c).unwrap().policies.is_empty());
        assert!(matches!(delete_policy(&c, None, id), Err(AppError::NotFound(_))));
    }

    #[test]
    fn member_category_membership_period_is_bounded() {
        let (_t, c) = conn();
        let bad = MemberCategoryInput { id: None, name: "X".into(), description: None, membership_days: 0 };
        assert!(matches!(save_member_category(&c, None, &bad), Err(AppError::Validation(_))));
    }

    #[test]
    fn library_details_can_be_updated_and_are_validated() {
        let (_t, c) = conn();
        let mut input = LibraryInput {
            name: "  St. Mary's School Library ".into(), profile: "school".into(), address: Some("Kochi".into()),
            contact: None, currency: "inr".into(), locale: "en".into(),
        };
        update_library(&c, None, &input).unwrap();
        let l = library(&c).unwrap().unwrap();
        assert_eq!((l.name.as_str(), l.currency.as_str(), l.address.as_deref()), ("St. Mary's School Library", "INR", Some("Kochi")));
        input.profile = "zoo".into();
        assert!(matches!(update_library(&c, None, &input), Err(AppError::Validation(_))));
    }

    #[test]
    fn every_change_is_audited() {
        let (_t, c) = conn();
        save_named(&c, None, NamedKind::Collection, &named_input("Fiction")).unwrap();
        let n: i64 = c.query_row("SELECT count(*) FROM audit_events WHERE action = 'config.saved'", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 1);
    }
}
