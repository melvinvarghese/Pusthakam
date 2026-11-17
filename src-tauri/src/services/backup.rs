use crate::db::migrations;
use crate::error::{AppError, AppResult};
use crate::paths::AppPaths;
use crate::services::settings;
use rusqlite::{backup::{Backup, StepResult}, Connection, OpenFlags};
use serde::Serialize;
use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub(crate) const DAILY_SETTING: &str = "backup.daily";
const RESTORE_RESULT: &str = "backup.last_restore";
const RESTORE_REQUEST: &str = "restore-request";

/// Every running copy holds a shared lock. Restore takes an exclusive lock before opening the library.
pub struct LibraryUseLock {
    _connection: Mutex<Connection>,
}

pub fn startup_lock(paths: &AppPaths) -> AppResult<(Connection, bool)> {
    let connection = Connection::open(paths.data_dir.join("library-use.sqlite3"))?;
    let pending = paths.data_dir.join(RESTORE_REQUEST).try_exists()?;
    if pending {
        connection.busy_timeout(Duration::ZERO)?;
        match connection.execute_batch("BEGIN EXCLUSIVE") {
            Ok(()) => return Ok((connection, true)),
            Err(rusqlite::Error::SqliteFailure(code, _)) if code.code == rusqlite::ErrorCode::DatabaseBusy => {}
            Err(error) => return Err(error.into()),
        }
    }
    lock_shared(&connection)?;
    Ok((connection, false))
}

fn lock_shared(connection: &Connection) -> AppResult<()> {
    connection.busy_timeout(Duration::from_secs(30))?;
    connection.execute_batch("BEGIN DEFERRED")?;
    connection.query_row("SELECT count(*) FROM sqlite_schema", [], |_| Ok(()))?;
    Ok(())
}

pub fn hold_library_lock(connection: Connection, exclusive: bool) -> AppResult<LibraryUseLock> {
    if exclusive {
        connection.execute_batch("ROLLBACK")?;
        lock_shared(&connection)?;
    }
    Ok(LibraryUseLock { _connection: Mutex::new(connection) })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupFile {
    name: String,
    size_bytes: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupList {
    folder: String,
    files: Vec<BackupFile>,
    daily_enabled: bool,
    pending_restore: Option<String>,
    last_restore: Option<String>,
}

fn daily_enabled(conn: &Connection) -> AppResult<bool> {
    Ok(settings::get(conn, DAILY_SETTING)?.as_deref() == Some("true"))
}

pub fn set_daily(conn: &Connection, enabled: bool) -> AppResult<()> {
    settings::set(conn, DAILY_SETTING, if enabled { "true" } else { "false" })
}

fn is_backup_name(name: &str) -> bool {
    name.starts_with("pusthakam-") && name.ends_with(".sqlite3")
        && name.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_'))
}

pub fn list(conn: &Connection, paths: &AppPaths) -> AppResult<BackupList> {
    let folder = &paths.backups;
    let mut files = Vec::new();
    for entry in fs::read_dir(folder)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if is_backup_name(&name) && entry.file_type()?.is_file() {
            files.push(BackupFile { name, size_bytes: entry.metadata()?.len() });
        }
    }
    files.sort_by(|left, right| right.name.cmp(&left.name));
    Ok(BackupList {
        folder: folder.to_string_lossy().into_owned(), files, daily_enabled: daily_enabled(conn)?,
        pending_restore: pending_restore(paths)?,
        last_restore: settings::get(conn, RESTORE_RESULT)?.and_then(|raw| serde_json::from_str(&raw).ok()),
    })
}

pub fn create(source: &Connection, folder: &Path) -> AppResult<BackupFile> {
    create_snapshot(source, folder, None)
}

pub fn run_daily(source: &Connection, folder: &Path) -> AppResult<bool> {
    if !daily_enabled(source)? {
        return Ok(false);
    }
    let date: String = source.query_row(
        "SELECT strftime('%Y%m%d','now','localtime')", [], |row| row.get(0),
    )?;
    let name = format!("pusthakam-{date}-daily.sqlite3");
    if folder.join(&name).try_exists()? {
        return Ok(false);
    }
    create_snapshot(source, folder, Some(&name))?;
    Ok(true)
}

fn check_integrity(conn: &Connection) -> AppResult<()> {
    if conn.query_row("PRAGMA integrity_check(1)", [], |row| row.get::<_, String>(0))? != "ok" {
        return Err(AppError::Validation("The backup did not pass SQLite integrity checking.".into()));
    }
    Ok(())
}

fn copy_database(source: &Connection, destination: &mut Connection) -> AppResult<()> {
    let backup = Backup::new(source, destination)?;
    let started = Instant::now();
    loop {
        match backup.step(256)? {
            StepResult::Done => return Ok(()),
            StepResult::More => {}
            _ if started.elapsed() >= Duration::from_secs(5) => {
                return Err(AppError::Conflict("The database is busy. Retry the backup operation.".into()));
            }
            _ => std::thread::sleep(Duration::from_millis(25)),
        }
    }
}

pub fn verify_saved(folder: &Path, name: &str) -> AppResult<i64> {
    let conn = open_saved(folder, name)?;
    check_integrity(&conn)?;
    check_schema_history(&conn)
}

fn open_saved(folder: &Path, name: &str) -> AppResult<Connection> {
    if !is_backup_name(name) {
        return Err(AppError::Validation("Choose a saved backup from the list.".into()));
    }
    let path = folder.join(name);
    if !fs::symlink_metadata(&path)?.is_file() {
        return Err(AppError::Validation("Choose a regular backup file.".into()));
    }
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    conn.pragma_update(None, "trusted_schema", "OFF")?;
    Ok(conn)
}

fn check_schema_history(conn: &Connection) -> AppResult<i64> {
    let mut stmt = conn.prepare("SELECT version, checksum FROM schema_migrations ORDER BY version")?;
    let mut rows = stmt.query([])?;
    let mut latest = 0;
    for migration in migrations::MIGRATIONS {
        let Some(row) = rows.next()? else { break };
        let version: i64 = row.get(0)?;
        let checksum: String = row.get(1)?;
        if version != migration.version || checksum != migrations::checksum(migration.sql) {
            return Err(AppError::Validation("This backup has an incompatible migration history.".into()));
        }
        latest = version;
    }
    if latest == 0 || (latest == migrations::latest_version() && rows.next()?.is_some()) {
        return Err(AppError::Validation("This backup has an incompatible database schema version.".into()));
    }
    Ok(latest)
}

fn pending_restore(paths: &AppPaths) -> AppResult<Option<String>> {
    match fs::File::open(paths.data_dir.join(RESTORE_REQUEST)) {
        Ok(file) => {
            let mut name = String::new();
            file.take(256).read_to_string(&mut name)?;
            Ok(Some(name))
        }
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

pub fn request_restore(paths: &AppPaths, name: &str) -> AppResult<()> {
    verify_saved(&paths.backups, name)?;
    let mut request = tempfile::NamedTempFile::new_in(&paths.data_dir)?;
    request.write_all(name.as_bytes())?;
    request.as_file().sync_all()?;
    request.persist_noclobber(paths.data_dir.join(RESTORE_REQUEST)).map_err(|error| {
        if error.error.kind() == ErrorKind::AlreadyExists {
            AppError::Conflict("Cancel the existing restore request before selecting another backup.".into())
        } else {
            AppError::Io(error.error)
        }
    })?;
    Ok(())
}

pub fn cancel_restore(paths: &AppPaths) -> AppResult<()> {
    match fs::remove_file(paths.data_dir.join(RESTORE_REQUEST)) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn restore(conn: &mut Connection, paths: &AppPaths, name: &str) -> AppResult<String> {
    let source = open_saved(&paths.backups, name)?;
    check_integrity(&source)?;
    check_schema_history(&source)?;
    let staged = tempfile::NamedTempFile::new_in(&paths.backups)?.into_temp_path();
    let mut candidate = Connection::open_with_flags(&staged, OpenFlags::SQLITE_OPEN_READ_WRITE)?;
    copy_database(&source, &mut candidate)?;
    candidate.pragma_update(None, "foreign_keys", "ON")?;
    migrations::run(&mut candidate, migrations::MIGRATIONS)?;
    check_integrity(&candidate)?;
    if candidate.prepare("PRAGMA foreign_key_check")?.query([])?.next()?.is_some() {
        return Err(AppError::Validation("The backup contains broken database references.".into()));
    }
    let recovery = create(conn, &paths.backups)?;
    copy_database(&candidate, conn)?;
    Ok(recovery.name)
}

/// Startup-only. No staff session or in-flight library commands exist during replacement.
pub fn apply_pending_restore(conn: &mut Connection, paths: &AppPaths) -> AppResult<()> {
    let Some(name) = pending_restore(paths)? else { return Ok(()) };
    fs::remove_file(paths.data_dir.join(RESTORE_REQUEST))?;
    let message = match restore(conn, paths, &name) {
        Ok(recovery) => format!("Database restored from {name}. Previous database saved as {recovery}."),
        Err(error) => {
            tracing::warn!(%error, "Requested database restore failed");
            format!("Restore failed. {}", error.user_message())
        }
    };
    let value = serde_json::to_string(&message).map_err(|error| AppError::Internal(error.to_string()))?;
    settings::set(conn, RESTORE_RESULT, &value)
}

fn create_snapshot(source: &Connection, folder: &Path, filename: Option<&str>) -> AppResult<BackupFile> {
    let timestamp: String = source.query_row(
        "SELECT strftime('%Y%m%dT%H%M%SZ','now')", [], |row| row.get(0),
    )?;
    let staged = tempfile::Builder::new().prefix(&format!("pusthakam-{timestamp}-"))
        .suffix(".partial").tempfile_in(folder)?.into_temp_path();
    {
        let mut destination = Connection::open_with_flags(&staged, OpenFlags::SQLITE_OPEN_READ_WRITE)?;
        copy_database(source, &mut destination)?;
        let mode: String = destination.query_row("PRAGMA journal_mode = DELETE", [], |row| row.get(0))?;
        if mode != "delete" {
            return Err(AppError::Internal("Could not finalize the backup as a single database file.".into()));
        }
        check_integrity(&destination)?;
        destination.close().map_err(|(_, error)| error)?;
    }
    let file = fs::OpenOptions::new().read(true).write(true).open(&staged)?;
    file.sync_all()?;
    let size_bytes = file.metadata()?.len();
    drop(file);
    let target = filename.map_or_else(|| staged.with_extension("sqlite3"), |name| folder.join(name));
    let name = target.file_name().ok_or_else(|| AppError::Internal("Missing backup filename.".into()))?
        .to_string_lossy().into_owned();
    staged.persist_noclobber(&target).map_err(|error| error.error)?;
    Ok(BackupFile { name, size_bytes })
}
