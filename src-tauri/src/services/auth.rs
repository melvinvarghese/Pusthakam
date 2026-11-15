//! Local staff accounts. Passwords are hashed with argon2id (RustCrypto `argon2` crate, per-user
//! random salt) and stored as PHC strings. Sessions live only in memory (see `state.rs`).

use crate::domain::validate;
use crate::error::{AppError, AppResult};
use crate::services::audit;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use rand_core::OsRng;
use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;
use std::sync::OnceLock;

const MAX_FAILED_ATTEMPTS: i64 = 5;
const LOCK_MINUTES: i64 = 5;
const BAD_LOGIN: &str = "Incorrect username or password.";

/// The signed-in staff member and what they may do. Also sent to the UI (no secrets inside).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub user_id: i64,
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub permissions: Vec<String>,
}

impl Session {
    pub fn can(&self, permission: &str) -> bool {
        self.permissions.iter().any(|p| p == permission)
    }

    pub fn require(&self, permission: &str) -> AppResult<()> {
        if self.can(permission) {
            Ok(())
        } else {
            Err(AppError::Permission("You do not have permission to do this.".into()))
        }
    }
}

pub fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|_| AppError::Internal("password hashing failed".into()))
}

pub fn verify_password(hash: &str, password: &str) -> bool {
    match PasswordHash::new(hash) {
        Ok(parsed) => Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok(),
        Err(_) => false,
    }
}

/// A hash to verify against when the username does not exist, so timing does not reveal valid names.
fn dummy_hash() -> &'static str {
    static HASH: OnceLock<String> = OnceLock::new();
    HASH.get_or_init(|| hash_password("pusthakam-dummy-password").unwrap_or_default())
}

/// Creates a staff account. `role_code` must be an existing role.
pub fn create_user(
    conn: &Connection,
    username: &str,
    display_name: &str,
    password: &str,
    role_code: &str,
) -> AppResult<i64> {
    let username = validate::username(username)?;
    let display_name = validate::required("Name", display_name, 80)?;
    validate::password(password)?;
    let role_id: i64 = conn
        .query_row("SELECT id FROM roles WHERE code = ?1", [role_code], |r| r.get(0))
        .optional()?
        .ok_or_else(|| AppError::Validation("Unknown role.".into()))?;
    let hash = hash_password(password)?;
    conn.execute(
        "INSERT INTO staff_users (username, display_name, password_hash, role_id) VALUES (?1, ?2, ?3, ?4)",
        (&username, &display_name, &hash, role_id),
    )
    .map_err(|e| match e {
        rusqlite::Error::SqliteFailure(f, _) if f.code == rusqlite::ErrorCode::ConstraintViolation => {
            AppError::Conflict("That username is already taken.".into())
        }
        other => AppError::Database(other),
    })?;
    Ok(conn.last_insert_rowid())
}

pub fn load_session(conn: &Connection, user_id: i64) -> AppResult<Session> {
    let (username, display_name, role): (String, String, String) = conn.query_row(
        "SELECT u.username, u.display_name, r.code FROM staff_users u JOIN roles r ON r.id = u.role_id
         WHERE u.id = ?1 AND u.active = 1",
        [user_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    let mut stmt = conn.prepare(
        "SELECT p.code FROM permissions p
         JOIN role_permissions rp ON rp.permission_id = p.id
         JOIN roles r ON r.id = rp.role_id
         JOIN staff_users u ON u.role_id = r.id
         WHERE u.id = ?1 ORDER BY p.code",
    )?;
    let permissions = stmt.query_map([user_id], |r| r.get::<_, String>(0))?.collect::<Result<Vec<_>, _>>()?;
    Ok(Session { user_id, username, display_name, role, permissions })
}

/// Checks credentials with lockout after repeated failures. Wrong user, wrong password and
/// inactive accounts all give the same message.
pub fn login(conn: &Connection, username: &str, password: &str) -> AppResult<Session> {
    type Row = (i64, String, i64, Option<String>);
    let row: Option<Row> = conn
        .query_row(
            "SELECT id, password_hash, active,
                    CASE WHEN locked_until IS NOT NULL AND locked_until > strftime('%Y-%m-%dT%H:%M:%SZ','now')
                         THEN locked_until END
             FROM staff_users WHERE username = ?1",
            [username.trim()],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?;

    let Some((id, hash, active, locked)) = row else {
        verify_password(dummy_hash(), password);
        return Err(AppError::Auth(BAD_LOGIN.into()));
    };

    if locked.is_some() {
        return Err(AppError::Auth(format!(
            "Too many failed attempts. Try again in about {LOCK_MINUTES} minutes."
        )));
    }

    if active != 1 || !verify_password(&hash, password) {
        if active == 1 {
            conn.execute(
                "UPDATE staff_users SET
                   failed_attempts = CASE WHEN failed_attempts + 1 >= ?2 THEN 0 ELSE failed_attempts + 1 END,
                   locked_until = CASE WHEN failed_attempts + 1 >= ?2
                        THEN strftime('%Y-%m-%dT%H:%M:%SZ','now', '+' || ?3 || ' minutes') ELSE locked_until END
                 WHERE id = ?1",
                (id, MAX_FAILED_ATTEMPTS, LOCK_MINUTES),
            )?;
            audit::record(conn, None, "auth.login_failed", "staff_user", Some(id), "Failed sign-in attempt")?;
        }
        return Err(AppError::Auth(BAD_LOGIN.into()));
    }

    conn.execute(
        "UPDATE staff_users SET failed_attempts = 0, locked_until = NULL,
            last_login_at = strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE id = ?1",
        [id],
    )?;
    let session = load_session(conn, id)?;
    audit::record(conn, Some(&session), "auth.login", "staff_user", Some(id), "Signed in")?;
    Ok(session)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn conn() -> (tempfile::TempDir, Connection) {
        let tmp = tempfile::tempdir().unwrap();
        let c = db::open(&tmp.path().join("a.db")).unwrap();
        (tmp, c)
    }

    #[test]
    fn hashes_are_argon2id_salted_and_verifiable() {
        let a = hash_password("correct horse battery").unwrap();
        let b = hash_password("correct horse battery").unwrap();
        assert!(a.starts_with("$argon2id$"));
        assert_ne!(a, b, "each hash needs its own salt");
        assert!(!a.contains("correct horse"));
        assert!(verify_password(&a, "correct horse battery"));
        assert!(!verify_password(&a, "Correct horse battery"));
        assert!(!verify_password("not a hash", "x"));
    }

    #[test]
    fn create_user_validates_and_rejects_duplicates_case_insensitively() {
        let (_t, c) = conn();
        create_user(&c, "Rekha", "Rekha Nair", "long enough password", "librarian").unwrap();
        assert!(matches!(
            create_user(&c, "rekha", "Other", "long enough password", "viewer"),
            Err(AppError::Conflict(_))
        ));
        assert!(matches!(create_user(&c, "ab", "X", "long enough password", "viewer"), Err(AppError::Validation(_))));
        assert!(matches!(create_user(&c, "newuser", "X", "short", "viewer"), Err(AppError::Validation(_))));
        assert!(matches!(
            create_user(&c, "newuser", "X", "long enough password", "wizard"),
            Err(AppError::Validation(_))
        ));
        let stored: String =
            c.query_row("SELECT password_hash FROM staff_users WHERE username='rekha'", [], |r| r.get(0)).unwrap();
        assert!(stored.starts_with("$argon2id$"));
    }

    #[test]
    fn login_returns_role_permissions() {
        let (_t, c) = conn();
        create_user(&c, "admin", "Admin", "long enough password", "administrator").unwrap();
        create_user(&c, "rekha", "Rekha", "long enough password", "librarian").unwrap();
        let admin = login(&c, "ADMIN", "long enough password").unwrap();
        assert!(admin.can("admin.settings") && admin.can("circulation.issue"));
        let lib = login(&c, "rekha", "long enough password").unwrap();
        assert!(lib.can("circulation.issue") && lib.can("catalogue.write"));
        assert!(!lib.can("admin.settings"));
        assert!(matches!(lib.require("admin.settings"), Err(AppError::Permission(_))));
    }

    #[test]
    fn built_in_roles_follow_least_privilege() {
        let (_t, c) = conn();
        create_user(&c, "clerk", "Clerk", "long enough password", "circulation").unwrap();
        create_user(&c, "viewer", "Viewer", "long enough password", "viewer").unwrap();
        let clerk = login(&c, "clerk", "long enough password").unwrap();
        assert!(clerk.can("circulation.issue") && !clerk.can("catalogue.write") && !clerk.can("circulation.manage"));
        let viewer = login(&c, "viewer", "long enough password").unwrap();
        assert!(viewer.can("catalogue.read") && !viewer.can("circulation.issue"));
        assert!(viewer.permissions.iter().all(|p| p.ends_with(".read")));
    }

    #[test]
    fn wrong_credentials_give_one_generic_message() {
        let (_t, c) = conn();
        create_user(&c, "rekha", "Rekha", "long enough password", "librarian").unwrap();
        c.execute("UPDATE staff_users SET active = 0", []).unwrap();
        for (u, p) in [("nobody", "x"), ("rekha", "long enough password"), ("rekha", "wrong")] {
            match login(&c, u, p) {
                Err(AppError::Auth(m)) => assert_eq!(m, BAD_LOGIN),
                other => panic!("expected auth failure, got {other:?}"),
            }
        }
    }

    #[test]
    fn five_failures_lock_the_account_even_for_the_right_password() {
        let (_t, c) = conn();
        create_user(&c, "rekha", "Rekha", "long enough password", "librarian").unwrap();
        for _ in 0..MAX_FAILED_ATTEMPTS {
            assert!(login(&c, "rekha", "wrong password!").is_err());
        }
        match login(&c, "rekha", "long enough password") {
            Err(AppError::Auth(m)) => assert!(m.contains("Too many")),
            other => panic!("expected lockout, got {other:?}"),
        }
        // Lock expires.
        c.execute("UPDATE staff_users SET locked_until = '2000-01-01T00:00:00Z'", []).unwrap();
        assert!(login(&c, "rekha", "long enough password").is_ok());
        let attempts: i64 = c.query_row("SELECT failed_attempts FROM staff_users", [], |r| r.get(0)).unwrap();
        assert_eq!(attempts, 0);
    }

    #[test]
    fn successful_login_is_audited_without_secrets() {
        let (_t, c) = conn();
        create_user(&c, "rekha", "Rekha", "long enough password", "librarian").unwrap();
        login(&c, "rekha", "long enough password").unwrap();
        let (action, summary): (String, String) = c
            .query_row("SELECT action, summary FROM audit_events ORDER BY id DESC LIMIT 1", [], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
            .unwrap();
        assert_eq!(action, "auth.login");
        assert!(!summary.contains("password"));
    }
}
