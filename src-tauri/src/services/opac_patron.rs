use crate::error::{AppError, AppResult};
use crate::services::{auth, reservations, web_http::Head};
use rand_core::{OsRng, RngCore};
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

const COOKIE_LOCAL: &str = "pusthakam_patron_local";
const COOKIE_SECURE: &str = "__Host-pusthakam_patron";
const SESSION_TTL: Duration = Duration::from_secs(2 * 60 * 60);
const MAX_SESSIONS: usize = 4096;

#[derive(Clone)]
struct PatronSession {
    member_id: i64,
    csrf: String,
    expires: Instant,
}

#[derive(Default)]
struct SessionStore {
    sessions: Mutex<HashMap<String, PatronSession>>,
}

static STORE: OnceLock<SessionStore> = OnceLock::new();

fn store() -> &'static SessionStore {
    STORE.get_or_init(SessionStore::default)
}

fn random_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    let mut out = String::with_capacity(64);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(&mut out, "{b:02x}");
    }
    out
}

pub fn is_secure_request(head: &Head) -> bool {
    head.header("x-forwarded-proto")
        .is_some_and(|v| v.eq_ignore_ascii_case("https"))
        || head.header("forwarded").is_some_and(|v| {
            v.split(';')
                .any(|part| part.trim().eq_ignore_ascii_case("proto=https"))
        })
}

pub fn same_origin_ok(head: &Head) -> bool {
    let Some(origin) = head.header("origin") else {
        return true;
    };
    let Some(host) = head.header("host") else {
        return false;
    };
    let Ok(origin_url) = tauri::Url::parse(origin) else {
        return false;
    };
    let Some(origin_host) = origin_url.host_str() else {
        return false;
    };
    let origin_authority = match origin_url.port() {
        Some(port) => format!("{origin_host}:{port}"),
        None => origin_host.to_string(),
    };
    origin_authority.eq_ignore_ascii_case(host)
}

fn read_cookie(head: &Head, secure: bool) -> Option<String> {
    let name = if secure { COOKIE_SECURE } else { COOKIE_LOCAL };
    head.header("cookie")?.split(';').find_map(|part| {
        let (key, value) = part.trim().split_once('=')?;
        (key == name && value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit()))
            .then(|| value.to_string())
    })
}

pub fn current_session(head: &Head) -> Option<(String, i64, String)> {
    let secure = is_secure_request(head);
    let key = read_cookie(head, secure)?;
    let mut map = store().sessions.lock().ok()?;
    let now = Instant::now();
    map.retain(|_, s| s.expires > now);
    map.get(&key).map(|s| (key, s.member_id, s.csrf.clone()))
}

pub fn create_session(member_id: i64, secure: bool) -> AppResult<(String, String, String)> {
    let key = random_token();
    let csrf = random_token();
    let mut map = store()
        .sessions
        .lock()
        .map_err(|_| AppError::Internal("Patron session lock poisoned".into()))?;
    let now = Instant::now();
    map.retain(|_, s| s.expires > now);
    if map.len() >= MAX_SESSIONS {
        if let Some(oldest) = map
            .iter()
            .min_by_key(|(_, s)| s.expires)
            .map(|(k, _)| k.clone())
        {
            map.remove(&oldest);
        }
    }
    map.insert(
        key.clone(),
        PatronSession {
            member_id,
            csrf: csrf.clone(),
            expires: now + SESSION_TTL,
        },
    );
    let cookie_name = if secure { COOKIE_SECURE } else { COOKIE_LOCAL };
    let cookie = format!(
        "{cookie_name}={key}; Path=/; HttpOnly; SameSite=Strict{}",
        if secure { "; Secure" } else { "" }
    );
    Ok((cookie, csrf, key))
}

pub fn clear_session(head: &Head) -> String {
    let secure = is_secure_request(head);
    if let Some(key) = read_cookie(head, secure) {
        if let Ok(mut map) = store().sessions.lock() {
            map.remove(&key);
        }
    }
    let cookie_name = if secure { COOKIE_SECURE } else { COOKIE_LOCAL };
    format!(
        "{cookie_name}=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0{}",
        if secure { "; Secure" } else { "" }
    )
}

fn digits_only(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_digit()).collect()
}

fn verification_matches(
    stored_email: Option<&str>,
    stored_phone: Option<&str>,
    input: &str,
) -> bool {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return false;
    }
    if let Some(email) = stored_email {
        let e = email.trim();
        if !e.is_empty() && e.eq_ignore_ascii_case(trimmed) {
            return true;
        }
    }
    if let Some(phone) = stored_phone {
        let p_digits = digits_only(phone);
        let i_digits = digits_only(trimmed);
        if p_digits.len() >= 6 && i_digits.len() >= 6 {
            if p_digits == i_digits {
                return true;
            }
            // Allow matching local 10-digit mobile number against +91 / country-prefixed number.
            if p_digits.len() >= 10 && i_digits.len() >= 10 {
                let p_tail = &p_digits[p_digits.len() - 10..];
                let i_tail = &i_digits[i_digits.len() - 10..];
                if p_tail == i_tail {
                    return true;
                }
            }
        }
    }
    false
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatronLoginInput {
    pub member_number: String,
    pub verification: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatronHoldInput {
    pub record_id: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatronCancelHoldInput {
    pub reservation_id: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatronProfile {
    pub member_number: String,
    pub name: String,
    pub category_name: String,
    pub expires_on: String,
    pub currency: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatronLoanRow {
    pub id: i64,
    pub title: String,
    pub barcode: String,
    pub issued_on: String,
    pub due_on: String,
    pub returned_on: Option<String>,
    pub overdue: bool,
    pub renewal_count: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatronHoldRow {
    pub id: i64,
    pub title: String,
    pub barcode: String,
    pub status: String,
    pub expires_at: Option<String>,
    pub queue_position: Option<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatronAccount {
    pub signed_in: bool,
    pub csrf_token: Option<String>,
    pub member: Option<PatronProfile>,
    pub unpaid_fine_minor: i64,
    pub loans: Vec<PatronLoanRow>,
    pub reservations: Vec<PatronHoldRow>,
}

pub fn authenticate(conn: &Connection, input: &PatronLoginInput) -> AppResult<i64> {
    let number = input.member_number.trim();
    if number.is_empty() || input.verification.trim().is_empty() {
        return Err(AppError::Validation(
            "Enter your member number or username, plus your password, PIN, or phone/email.".into(),
        ));
    }
    let row: Option<(i64, Option<String>, Option<String>, Option<String>)> = conn
        .query_row(
            "SELECT id, email, phone, opac_password_hash
             FROM members
             WHERE (member_number = ?1 COLLATE NOCASE OR opac_username = ?1 COLLATE NOCASE)
               AND archived_at IS NULL",
            [number],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?;
    let Some((member_id, email, phone, password_hash)) = row else {
        return Err(AppError::Permission(
            "Credentials did not match an active library account.".into(),
        ));
    };
    if let Some(hash) = password_hash.as_deref() {
        if auth::verify_password(hash, input.verification.trim()) {
            return Ok(member_id);
        }
        return Err(AppError::Permission(
            "Credentials did not match an active library account.".into(),
        ));
    }
    if !verification_matches(email.as_deref(), phone.as_deref(), &input.verification) {
        return Err(AppError::Permission(
            "Credentials did not match an active library account.".into(),
        ));
    }
    Ok(member_id)
}

pub fn account_summary(
    conn: &mut Connection,
    member_id: i64,
    csrf_token: String,
) -> AppResult<PatronAccount> {
    let _ = reservations::refresh(conn);
    let profile: PatronProfile = conn
        .query_row(
            "SELECT m.member_number, m.name, c.name, m.expires_on,
                    COALESCE((SELECT currency FROM libraries WHERE id = 1), 'INR')
             FROM members m
             JOIN member_categories c ON c.id = m.category_id
             WHERE m.id = ?1 AND m.archived_at IS NULL",
            [member_id],
            |r| {
                Ok(PatronProfile {
                    member_number: r.get(0)?,
                    name: r.get(1)?,
                    category_name: r.get(2)?,
                    expires_on: r.get(3)?,
                    currency: r.get(4)?,
                })
            },
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound("Member account not found.".into()))?;

    let unpaid_fine_minor: i64 = conn.query_row(
        "SELECT COALESCE(SUM(MAX(0, f.amount_minor - COALESCE(paid.total, 0))), 0)
         FROM fines f
         JOIN loans l ON l.id = f.loan_id
         LEFT JOIN (
             SELECT fine_id, SUM(amount_minor) AS total FROM fine_settlements GROUP BY fine_id
         ) paid ON paid.fine_id = f.id
         WHERE l.member_id = ?1",
        [member_id],
        |r| r.get(0),
    )?;

    let loans = {
        let mut stmt = conn.prepare(
            "SELECT l.id, br.title, i.barcode, date(l.issued_at, 'localtime'), l.due_on,
                    date(l.returned_at, 'localtime'),
                    l.returned_at IS NULL AND l.due_on < date('now', 'localtime'),
                    (SELECT count(*) FROM loan_renewals n WHERE n.loan_id = l.id)
             FROM loans l
             JOIN physical_items i ON i.id = l.item_id
             JOIN bibliographic_records br ON br.id = i.record_id
             WHERE l.member_id = ?1 AND l.returned_at IS NULL
             ORDER BY l.due_on ASC, l.id DESC
             LIMIT 50",
        )?;
        let rows = stmt.query_map([member_id], |r| {
            Ok(PatronLoanRow {
                id: r.get(0)?,
                title: r.get(1)?,
                barcode: r.get(2)?,
                issued_on: r.get(3)?,
                due_on: r.get(4)?,
                returned_on: r.get(5)?,
                overdue: r.get(6)?,
                renewal_count: r.get(7)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    let reservations = {
        let mut stmt = conn.prepare(
            "SELECT r.id, br.title, i.barcode, r.status, r.expires_at,
                    (SELECT count(*) FROM reservations q
                     WHERE q.item_id = r.item_id AND q.closed_at IS NULL AND q.id <= r.id)
             FROM reservations r
             JOIN physical_items i ON i.id = r.item_id
             JOIN bibliographic_records br ON br.id = i.record_id
             WHERE r.member_id = ?1 AND r.closed_at IS NULL
             ORDER BY r.status = 'ready' DESC, r.id ASC",
        )?;
        let rows = stmt.query_map([member_id], |r| {
            Ok(PatronHoldRow {
                id: r.get(0)?,
                title: r.get(1)?,
                barcode: r.get(2)?,
                status: r.get(3)?,
                expires_at: r.get(4)?,
                queue_position: r.get(5)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    Ok(PatronAccount {
        signed_in: true,
        csrf_token: Some(csrf_token),
        member: Some(profile),
        unpaid_fine_minor,
        loans,
        reservations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::services::auth;

    fn seed_db() -> (tempfile::TempDir, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let conn = db::open(&dir.path().join("test.sqlite3")).unwrap();
        auth::create_user(&conn, "admin", "Admin", "admin_pass_123", "administrator").unwrap();
        conn.execute(
            "INSERT INTO libraries (id, name, profile, currency, locale) VALUES (1, 'Town Library', 'public', 'INR', 'en-IN')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO member_categories (id, name) VALUES (1, 'Adult')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO item_categories (id, name) VALUES (1, 'General')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO circulation_policies (member_category_id, item_category_id, loan_days, max_loans, max_renewals, renewal_days, fine_per_day_minor, fine_grace_days, reservation_hold_days)
             VALUES (1, 1, 14, 5, 2, 7, 100, 0, 3)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO members (id, member_number, name, category_id, email, phone, expires_on)
             VALUES (1, 'M-101', 'Ananya Nair', 1, 'ananya@example.com', '+91 98765 43210', date('now','+365 days'))",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO bibliographic_records (id, title) VALUES (1, 'Balyakalasakhi')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO physical_items (id, record_id, accession_no, barcode, item_category_id, status)
             VALUES (1, 1, 'ACC-1', 'BK-001', 1, 'available')",
            [],
        )
        .unwrap();
        (dir, conn)
    }

    #[test]
    fn authenticates_with_email_or_normalized_phone() {
        let (_dir, conn) = seed_db();
        assert_eq!(
            authenticate(
                &conn,
                &PatronLoginInput {
                    member_number: "m-101".into(),
                    verification: "ANANYA@EXAMPLE.COM".into(),
                }
            )
            .unwrap(),
            1
        );
        assert_eq!(
            authenticate(
                &conn,
                &PatronLoginInput {
                    member_number: "M-101".into(),
                    verification: "9876543210".into(),
                }
            )
            .unwrap(),
            1
        );
        assert!(authenticate(
            &conn,
            &PatronLoginInput {
                member_number: "M-101".into(),
                verification: "0000000000".into(),
            }
        )
        .is_err());
    }

    #[test]
    fn authenticates_with_assigned_opac_username_and_password() {
        let (_dir, mut conn) = seed_db();
        let actor = crate::services::auth::Session {
            user_id: 1,
            username: "admin".into(),
            display_name: "Admin".into(),
            role: "administrator".into(),
            permissions: Default::default(),
            revision: 1,
        };
        crate::services::members::set_opac_credentials(
            &mut conn,
            &actor,
            1,
            Some("ananya.nair"),
            Some("4321"),
            false,
        )
        .unwrap();

        assert_eq!(
            authenticate(
                &conn,
                &PatronLoginInput {
                    member_number: "ananya.nair".into(),
                    verification: "4321".into(),
                }
            )
            .unwrap(),
            1
        );
        assert!(authenticate(
            &conn,
            &PatronLoginInput {
                member_number: "ananya.nair".into(),
                verification: "wrong".into(),
            }
        )
        .is_err());
    }

    #[test]
    fn patron_can_place_and_cancel_hold_and_view_account() {
        let (_dir, mut conn) = seed_db();
        let barcode = reservations::place_for_patron_record(&mut conn, 1, 1).unwrap();
        assert_eq!(barcode, "BK-001");

        let acct = account_summary(&mut conn, 1, "csrf".into()).unwrap();
        assert_eq!(acct.reservations.len(), 1);
        assert_eq!(acct.reservations[0].title, "Balyakalasakhi");
        assert_eq!(acct.reservations[0].status, "ready");

        let hold_id = acct.reservations[0].id;
        reservations::cancel_for_patron(&mut conn, 1, hold_id).unwrap();
        let acct2 = account_summary(&mut conn, 1, "csrf".into()).unwrap();
        assert!(acct2.reservations.is_empty());
    }
}
