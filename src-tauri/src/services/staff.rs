use crate::domain::validate;
use crate::error::{AppError, AppResult};
use crate::services::{audit, auth::{self, Session}};
use rand_core::{OsRng, RngCore};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StaffRow {
    id: i64,
    username: String,
    display_name: String,
    role: String,
    active: bool,
    locked: bool,
}

#[derive(Serialize)]
struct Role {
    code: String,
    name: String,
}

#[derive(Serialize)]
pub struct Directory {
    users: Vec<StaffRow>,
    roles: Vec<Role>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewStaff {
    username: String,
    display_name: String,
    role: String,
    password: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffChanges {
    display_name: String,
    role: String,
    active: bool,
}

pub fn list(conn: &Connection) -> AppResult<Directory> {
    let mut statement = conn.prepare(
        "SELECT u.id, u.username, u.display_name, r.code, u.active,
                coalesce(u.locked_until > strftime('%Y-%m-%dT%H:%M:%SZ','now'), 0)
         FROM staff_users u JOIN roles r ON r.id = u.role_id
         ORDER BY u.active DESC, u.username COLLATE NOCASE, u.id",
    )?;
    let users = statement.query_map([], |row| Ok(StaffRow {
        id: row.get(0)?, username: row.get(1)?, display_name: row.get(2)?, role: row.get(3)?,
        active: row.get(4)?, locked: row.get(5)?,
    }))?.collect::<Result<_, _>>()?;
    let mut statement = conn.prepare("SELECT code, name FROM roles ORDER BY id")?;
    let roles = statement.query_map([], |row| Ok(Role { code: row.get(0)?, name: row.get(1)? }))?
        .collect::<Result<_, _>>()?;
    Ok(Directory { users, roles })
}

fn admin_transaction<'a>(conn: &'a mut Connection, actor: &Session) -> AppResult<rusqlite::Transaction<'a>> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    auth::refresh_session(&tx, actor)?.require("admin.staff")?;
    Ok(tx)
}

pub fn create(conn: &mut Connection, actor: &Session, input: &NewStaff) -> AppResult<i64> {
    let tx = admin_transaction(conn, actor)?;
    let id = auth::create_user(&tx, &input.username, &input.display_name, &input.password, &input.role)?;
    audit::record(&tx, Some(actor), "staff.create", "staff_user", Some(id), "Created staff account")?;
    tx.commit()?;
    Ok(id)
}

fn other_account(actor: &Session, id: i64) -> AppResult<()> {
    if actor.user_id == id {
        return Err(AppError::Validation("Use Change your password for your own account. Another administrator can change your role or status.".into()));
    }
    Ok(())
}

fn require_row(changed: usize) -> AppResult<()> {
    if changed == 0 {
        return Err(AppError::NotFound("Staff account not found.".into()));
    }
    Ok(())
}

pub fn update(conn: &mut Connection, actor: &Session, id: i64, changes: &StaffChanges) -> AppResult<()> {
    other_account(actor, id)?;
    let name = validate::required("Name", &changes.display_name, 80)?;
    let tx = admin_transaction(conn, actor)?;
    let role_id: i64 = tx.query_row("SELECT id FROM roles WHERE code = ?1", [&changes.role], |row| row.get(0))
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => AppError::Validation("Unknown role.".into()),
            other => other.into(),
        })?;
    require_row(tx.execute(
        "UPDATE staff_users SET display_name = ?2, role_id = ?3, active = ?4,
         session_revision = session_revision + 1, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE id = ?1",
        params![id, name, role_id, changes.active],
    )?)?;
    if changes.role != "administrator" || !changes.active {
        tx.execute("DELETE FROM staff_recovery_codes WHERE staff_user_id = ?1", [id])?;
    }
    let summary = if changes.active { "Updated active staff account" } else { "Deactivated staff account" };
    audit::record(&tx, Some(actor), "staff.update", "staff_user", Some(id), summary)?;
    tx.commit()?;
    Ok(())
}

fn replace_password(conn: &Connection, id: i64, password: &str) -> AppResult<()> {
    validate::password(password)?;
    let hash = auth::hash_password(password)?;
    require_row(conn.execute(
        "UPDATE staff_users SET password_hash = ?2, failed_attempts = 0, locked_until = NULL,
         session_revision = session_revision + 1, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE id = ?1",
        params![id, hash],
    )?)?;
    conn.execute("DELETE FROM staff_recovery_codes WHERE staff_user_id = ?1", [id])?;
    Ok(())
}

pub fn reset_password(conn: &mut Connection, actor: &Session, id: i64, password: &str) -> AppResult<()> {
    other_account(actor, id)?;
    let tx = admin_transaction(conn, actor)?;
    replace_password(&tx, id, password)?;
    audit::record(&tx, Some(actor), "staff.password_reset", "staff_user", Some(id), "Reset staff password")?;
    tx.commit()?;
    Ok(())
}

pub fn unlock(conn: &mut Connection, actor: &Session, id: i64) -> AppResult<()> {
    let tx = admin_transaction(conn, actor)?;
    require_row(tx.execute("UPDATE staff_users SET failed_attempts = 0, locked_until = NULL WHERE id = ?1", [id])?)?;
    audit::record(&tx, Some(actor), "staff.unlock", "staff_user", Some(id), "Unlocked staff account")?;
    tx.commit()?;
    Ok(())
}

fn verify_current_password(conn: &Connection, actor: &Session, current: &str) -> AppResult<()> {
    let hash: String = conn.query_row("SELECT password_hash FROM staff_users WHERE id = ?1", [actor.user_id], |row| row.get(0))?;
    if !auth::verify_password(&hash, current) {
        return Err(AppError::Auth("The current password is incorrect.".into()));
    }
    Ok(())
}

pub fn change_password(conn: &mut Connection, actor: &Session, current: &str, replacement: &str) -> AppResult<()> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    auth::refresh_session(&tx, actor)?;
    verify_current_password(&tx, actor, current)?;
    replace_password(&tx, actor.user_id, replacement)?;
    audit::record(&tx, Some(actor), "staff.password_changed", "staff_user", Some(actor.user_id), "Changed password")?;
    tx.commit()?;
    Ok(())
}

pub fn generate_recovery_code(conn: &mut Connection, actor: &Session, current: &str) -> AppResult<String> {
    let tx = admin_transaction(conn, actor)?;
    verify_current_password(&tx, actor, current)?;
    let mut random = [0_u8; 32];
    OsRng.try_fill_bytes(&mut random).map_err(|_| AppError::Internal("Could not generate a recovery code.".into()))?;
    let code: String = random.iter().map(|byte| format!("{byte:02x}")).collect();
    let hash = auth::hash_password(&code)?;
    tx.execute(
        "INSERT INTO staff_recovery_codes (staff_user_id, code_hash) VALUES (?1, ?2)
         ON CONFLICT(staff_user_id) DO UPDATE SET code_hash = excluded.code_hash",
        params![actor.user_id, hash],
    )?;
    audit::record(&tx, Some(actor), "staff.recovery_created", "staff_user", Some(actor.user_id), "Generated a recovery code")?;
    tx.commit()?;
    Ok(code)
}

pub fn recover(conn: &mut Connection, username: &str, code: &str, replacement: &str) -> AppResult<()> {
    let bad = || AppError::Auth("The username or recovery code is incorrect.".into());
    let code = code.trim().to_ascii_lowercase();
    if code.len() != 64 || !code.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(bad());
    }
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let account: Option<(i64, String)> = tx.query_row(
        "SELECT u.id, c.code_hash FROM staff_users u JOIN roles r ON r.id = u.role_id
         JOIN staff_recovery_codes c ON c.staff_user_id = u.id WHERE u.username = ?1 AND r.code = 'administrator' AND u.active = 1",
        [username.trim()], |row| Ok((row.get(0)?, row.get(1)?)),
    ).optional()?;
    let hash = account.as_ref().map(|(_, hash)| hash.as_str()).unwrap_or_else(|| auth::dummy_hash());
    let valid = auth::verify_password(hash, &code);
    let (id, _) = account.filter(|_| valid).ok_or_else(bad)?;
    replace_password(&tx, id, replacement)?;
    audit::record(&tx, None, "staff.password_recovered", "staff_user", Some(id), "Used a recovery code to reset the password")?;
    tx.commit()?;
    Ok(())
}
