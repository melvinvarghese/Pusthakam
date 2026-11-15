//! First-launch setup: creates the library, the first administrator and (optionally) the starting
//! configuration for the chosen library type, all in one transaction. It can only run once.

use crate::domain::{profiles, validate};
use crate::error::{AppError, AppResult};
use crate::services::{audit, auth, config, settings};
use rusqlite::{Connection, TransactionBehavior};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupStatus {
    /// True once the library has been created. Until then the app shows the setup wizard.
    pub configured: bool,
    pub profiles: Vec<&'static str>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminInput {
    pub username: String,
    pub display_name: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupInput {
    pub library: config::LibraryInput,
    pub admin: AdminInput,
    /// Create the starting member/item categories, collections, location and policies for the profile.
    pub apply_profile_defaults: bool,
    /// Late fine per day in minor units (e.g. paise/cents). 0 = no fines until configured.
    pub fine_per_day_minor: i64,
}

pub fn status(conn: &Connection) -> AppResult<SetupStatus> {
    let configured: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM libraries)", [], |r| r.get(0))?;
    Ok(SetupStatus { configured, profiles: profiles::PROFILES.to_vec() })
}

/// Runs the whole setup atomically. Returns the new administrator's user id.
pub fn complete(conn: &mut Connection, input: &SetupInput) -> AppResult<i64> {
    // Validate everything before touching the database.
    let name = validate::required("Library name", &input.library.name, 120)?;
    let defaults = profiles::defaults_for(&input.library.profile)
        .ok_or_else(|| AppError::Validation("Choose a library type.".into()))?;
    let address = validate::optional("Address", input.library.address.as_deref(), 300)?;
    let contact = validate::optional("Contact", input.library.contact.as_deref(), 200)?;
    let currency = validate::currency(&input.library.currency)?;
    let locale = validate::required("Language", &input.library.locale, 20)?;
    validate::username(&input.admin.username)?;
    validate::required("Your name", &input.admin.display_name, 80)?;
    validate::password(&input.admin.password)?;
    if input.fine_per_day_minor < 0 {
        return Err(AppError::Validation("The late fine cannot be negative.".into()));
    }

    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if status(&tx)?.configured {
        return Err(AppError::Conflict("This library has already been set up.".into()));
    }

    tx.execute(
        "INSERT INTO libraries (id, name, profile, address, contact, currency, locale) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6)",
        (&name, &input.library.profile, &address, &contact, &currency, &locale),
    )?;
    let admin_id = auth::create_user(
        &tx,
        &input.admin.username,
        &input.admin.display_name,
        &input.admin.password,
        "administrator",
    )?;
    let session = auth::load_session(&tx, admin_id)?;

    if input.apply_profile_defaults {
        let mut member_ids = std::collections::HashMap::new();
        for m in &defaults.member_categories {
            let id = config::save_member_category(
                &tx,
                Some(&session),
                &config::MemberCategoryInput { id: None, name: m.name.into(), description: None, membership_days: m.membership_days },
            )?;
            member_ids.insert(m.name, id);
        }
        let mut item_ids = std::collections::HashMap::new();
        for n in defaults.lendable_items.iter().chain(defaults.reference_items.iter()) {
            let id = config::save_named(&tx, Some(&session), config::NamedKind::ItemCategory,
                &config::NamedInput { id: None, name: (*n).into(), description: None })?;
            item_ids.insert(*n, id);
        }
        for n in &defaults.collections {
            config::save_named(&tx, Some(&session), config::NamedKind::Collection,
                &config::NamedInput { id: None, name: (*n).into(), description: None })?;
        }
        for n in &defaults.locations {
            config::save_named(&tx, Some(&session), config::NamedKind::Location,
                &config::NamedInput { id: None, name: (*n).into(), description: None })?;
        }
        for p in &defaults.policies {
            let (Some(&m), Some(&i)) = (member_ids.get(p.member), item_ids.get(p.item)) else { continue };
            config::save_policy(
                &tx,
                Some(&session),
                &config::PolicyInput {
                    id: None,
                    member_category_id: m,
                    item_category_id: i,
                    loan_days: p.loan_days,
                    max_loans: p.max_loans,
                    max_renewals: p.max_renewals,
                    renewal_days: p.renewal_days,
                    fine_per_day_minor: input.fine_per_day_minor,
                    fine_grace_days: if input.fine_per_day_minor > 0 { 1 } else { 0 },
                    fine_cap_minor: None,
                    reservation_hold_days: 3,
                },
            )?;
        }
    }

    settings::set(&tx, "classification.default", "\"ddc\"")?;
    audit::record(&tx, Some(&session), "setup.completed", "library", Some(1), "Library set up")?;
    tx.commit()?;
    Ok(admin_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn open() -> (tempfile::TempDir, Connection) {
        let tmp = tempfile::tempdir().unwrap();
        let c = db::open(&tmp.path().join("s.db")).unwrap();
        (tmp, c)
    }

    fn input(profile: &str) -> SetupInput {
        SetupInput {
            library: config::LibraryInput {
                name: "Demo Library".into(), profile: profile.into(), address: None, contact: None,
                currency: "INR".into(), locale: "en".into(),
            },
            admin: AdminInput { username: "admin".into(), display_name: "Head Librarian".into(), password: "a long passphrase".into() },
            apply_profile_defaults: true,
            fine_per_day_minor: 100,
        }
    }

    fn count(c: &Connection, table: &str) -> i64 {
        c.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0)).unwrap()
    }

    #[test]
    fn fresh_database_is_not_configured() {
        let (_t, c) = open();
        let s = status(&c).unwrap();
        assert!(!s.configured);
        assert_eq!(s.profiles.len(), 8);
    }

    #[test]
    fn school_setup_creates_library_admin_and_defaults() {
        let (_t, mut c) = open();
        let admin = complete(&mut c, &input("school")).unwrap();
        assert!(status(&c).unwrap().configured);
        let o = config::overview(&c).unwrap();
        assert_eq!(o.library.unwrap().name, "Demo Library");
        let members: Vec<_> = o.member_categories.iter().map(|m| m.name.as_str()).collect();
        assert_eq!(members, ["Staff", "Student", "Teacher"]);
        assert_eq!(o.item_categories.len(), 2);
        assert_eq!(o.collections.len(), 2);
        assert_eq!(o.locations.len(), 1);
        assert_eq!(o.policies.len(), 3);
        assert!(o.policies.iter().all(|p| p.fine_per_day_minor == 100 && p.fine_grace_days == 1));
        // Reference items are not lendable: no policy for them.
        let reference = o.item_categories.iter().find(|i| i.name == "Reference").unwrap().id;
        assert!(o.policies.iter().all(|p| p.item_category_id != reference));
        // The administrator can sign in and has full permissions.
        let s = auth::login(&c, "admin", "a long passphrase").unwrap();
        assert_eq!(s.user_id, admin);
        assert!(s.can("admin.settings") && s.can("admin.staff") && s.can("admin.backup"));
        assert_eq!(settings::get(&c, "classification.default").unwrap().as_deref(), Some("\"ddc\""));
    }

    #[test]
    fn every_profile_can_be_set_up() {
        for p in profiles::PROFILES {
            let (_t, mut c) = open();
            complete(&mut c, &input(p)).unwrap_or_else(|e| panic!("{p}: {e}"));
            assert!(count(&c, "circulation_policies") >= 1, "{p}");
        }
    }

    #[test]
    fn defaults_are_optional() {
        let (_t, mut c) = open();
        let mut i = input("private");
        i.apply_profile_defaults = false;
        complete(&mut c, &i).unwrap();
        assert_eq!(count(&c, "member_categories"), 0);
        assert_eq!(count(&c, "circulation_policies"), 0);
        assert_eq!(count(&c, "libraries"), 1);
    }

    #[test]
    fn zero_fine_means_no_fines_or_grace() {
        let (_t, mut c) = open();
        let mut i = input("college");
        i.fine_per_day_minor = 0;
        complete(&mut c, &i).unwrap();
        let o = config::overview(&c).unwrap();
        assert!(o.policies.iter().all(|p| p.fine_per_day_minor == 0 && p.fine_grace_days == 0));
    }

    #[test]
    fn setup_can_only_run_once() {
        let (_t, mut c) = open();
        complete(&mut c, &input("school")).unwrap();
        let mut again = input("college");
        again.admin.username = "second".into();
        assert!(matches!(complete(&mut c, &again), Err(AppError::Conflict(_))));
        assert_eq!(count(&c, "staff_users"), 1);
        assert_eq!(count(&c, "member_categories"), 3);
    }

    #[test]
    fn invalid_input_leaves_the_database_untouched() {
        let (_t, mut c) = open();
        let bad: Vec<Box<dyn Fn(&mut SetupInput)>> = vec![
            Box::new(|i| i.library.profile = "zoo".into()),
            Box::new(|i| i.library.name = "  ".into()),
            Box::new(|i| i.library.currency = "RS".into()),
            Box::new(|i| i.admin.password = "short".into()),
            Box::new(|i| i.admin.username = "a b".into()),
            Box::new(|i| i.fine_per_day_minor = -5),
        ];
        for mutate in bad {
            let mut i = input("school");
            mutate(&mut i);
            assert!(matches!(complete(&mut c, &i), Err(AppError::Validation(_))));
        }
        assert_eq!(count(&c, "libraries"), 0);
        assert_eq!(count(&c, "staff_users"), 0);
        assert!(!status(&c).unwrap().configured);
    }

    #[test]
    fn a_failure_midway_rolls_everything_back() {
        let (_t, mut c) = open();
        // Make creating the admin fail after the library row was inserted.
        c.execute_batch("CREATE TRIGGER block_staff BEFORE INSERT ON staff_users BEGIN SELECT RAISE(ABORT, 'blocked'); END;")
            .unwrap();
        assert!(complete(&mut c, &input("school")).is_err());
        assert_eq!(count(&c, "libraries"), 0, "library row must roll back with the failed setup");
        assert_eq!(count(&c, "member_categories"), 0);
    }

    #[test]
    fn setup_is_audited_without_the_password() {
        let (_t, mut c) = open();
        complete(&mut c, &input("school")).unwrap();
        let dump: String = c
            .query_row("SELECT group_concat(action || ' ' || summary, '|') FROM audit_events", [], |r| r.get(0))
            .unwrap();
        assert!(dump.contains("setup.completed"));
        assert!(!dump.contains("passphrase"));
    }
}
