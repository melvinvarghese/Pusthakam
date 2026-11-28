use crate::db::migrations;
use crate::error::{AppError, AppResult};
use crate::paths::AppPaths;
use crate::services::{settings, setup};
use rusqlite::{
    backup::{Backup, StepResult},
    Connection, OpenFlags, Transaction, TransactionBehavior,
};
use serde::Serialize;
use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub(crate) const DAILY_SETTING: &str = "backup.daily";
const RESTORE_RESULT: &str = "backup.last_restore";
const RESTORE_REQUEST: &str = "restore-request";
const LAST_DAILY: &str = "backup.last_daily";
/// Plain-text file in the data directory naming the folder for daily backups.
/// First chosen in the setup wizard; changeable on the Database Backups page.
const DAILY_FOLDER_FILE: &str = "daily-backup-folder.txt";
/// Daily backups kept in each folder. Making the eighth removes the oldest.
pub const DAILY_KEEP: usize = 7;

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
            Err(rusqlite::Error::SqliteFailure(code, _))
                if code.code == rusqlite::ErrorCode::DatabaseBusy => {}
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
    Ok(LibraryUseLock {
        _connection: Mutex::new(connection),
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupFile {
    name: String,
    size_bytes: u64,
    /// Folder holding this file: the Pusthakam data folder or the daily backup folder.
    folder: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupList {
    folder: String,
    files: Vec<BackupFile>,
    daily_enabled: bool,
    /// Folder chosen for daily backups; `None` means the Pusthakam data folder.
    daily_folder: Option<String>,
    /// False when the chosen folder cannot be reached (for example an unplugged USB drive).
    daily_folder_available: bool,
    daily_keep: usize,
    last_daily: Option<String>,
    pending_restore: Option<String>,
    last_restore: Option<String>,
}

fn daily_enabled(conn: &Connection) -> AppResult<bool> {
    Ok(settings::get(conn, DAILY_SETTING)?.as_deref() != Some("false"))
}

pub fn set_daily(conn: &Connection, enabled: bool) -> AppResult<()> {
    settings::set(conn, DAILY_SETTING, if enabled { "true" } else { "false" })
}

fn is_backup_name(name: &str) -> bool {
    name.starts_with("pusthakam-")
        && name.ends_with(".sqlite3")
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_'))
}

fn backups_in(folder: &Path, files: &mut Vec<BackupFile>) -> AppResult<()> {
    for entry in fs::read_dir(folder)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if is_backup_name(&name)
            && entry.file_type()?.is_file()
            && !files.iter().any(|file| file.name == name)
        {
            files.push(BackupFile {
                name,
                size_bytes: entry.metadata()?.len(),
                folder: folder.to_string_lossy().into_owned(),
            });
        }
    }
    Ok(())
}

pub fn list(conn: &Connection, paths: &AppPaths) -> AppResult<BackupList> {
    let mut files = Vec::new();
    let chosen = daily_folder(paths)?;
    let reachable = chosen.as_deref().filter(|folder| folder.is_dir());
    // The chosen folder comes first so a same-day copy there wins over a fallback copy.
    if let Some(folder) = reachable {
        backups_in(folder, &mut files)?;
    }
    backups_in(&paths.backups, &mut files)?;
    files.sort_by(|left, right| right.name.cmp(&left.name));
    Ok(BackupList {
        folder: paths.backups.to_string_lossy().into_owned(),
        files,
        daily_enabled: daily_enabled(conn)?,
        daily_folder_available: chosen.is_none() || reachable.is_some(),
        daily_folder: chosen.map(|folder| folder.to_string_lossy().into_owned()),
        daily_keep: DAILY_KEEP,
        last_daily: settings::get(conn, LAST_DAILY)?
            .and_then(|raw| serde_json::from_str(&raw).ok()),
        pending_restore: pending_restore(paths)?,
        last_restore: settings::get(conn, RESTORE_RESULT)?
            .and_then(|raw| serde_json::from_str(&raw).ok()),
    })
}

/// The folder chosen for daily backups, or `None` for the Pusthakam data folder.
pub fn daily_folder(paths: &AppPaths) -> AppResult<Option<PathBuf>> {
    match fs::read_to_string(paths.data_dir.join(DAILY_FOLDER_FILE)) {
        Ok(raw) => {
            let folder = raw.trim();
            Ok((!folder.is_empty()).then(|| PathBuf::from(folder)))
        }
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn write_daily_folder(paths: &AppPaths, folder: &Path) -> AppResult<()> {
    let mut file = tempfile::NamedTempFile::new_in(&paths.data_dir)?;
    file.write_all(folder.to_string_lossy().as_bytes())?;
    file.as_file().sync_all()?;
    file.persist(paths.data_dir.join(DAILY_FOLDER_FILE))
        .map_err(|error| error.error)?;
    Ok(())
}

/// Checks a folder offered for daily backups without saving the choice. `Ok(false)` means the
/// backups stay in the Pusthakam data folder (an empty value, or that folder's own backups folder).
pub fn check_daily_folder(paths: &AppPaths, folder: &str) -> AppResult<bool> {
    let folder = folder.trim();
    if folder.is_empty() {
        return Ok(false);
    }
    let path = Path::new(folder);
    if !path.is_absolute() || !path.is_dir() {
        return Err(AppError::Validation(
            "Enter the full path to an existing folder for daily backups.".into(),
        ));
    }
    let canonical = path.canonicalize()?;
    let data_dir = paths.data_dir.canonicalize()?;
    if canonical.starts_with(&data_dir) {
        if canonical == paths.backups.canonicalize()? {
            return Ok(false);
        }
        return Err(AppError::Validation(
            "Choose a folder outside the Pusthakam data directory.".into(),
        ));
    }
    let probe = tempfile::NamedTempFile::new_in(path)
        .map_err(|_| AppError::Validation("Pusthakam cannot save files in that folder.".into()))?;
    drop(probe);
    Ok(true)
}

/// Choose where daily backups go. An empty value returns them to the Pusthakam data folder.
pub fn set_daily_folder(paths: &AppPaths, folder: &str) -> AppResult<Option<String>> {
    let folder = folder.trim();
    if !check_daily_folder(paths, folder)? {
        match fs::remove_file(paths.data_dir.join(DAILY_FOLDER_FILE)) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        return Ok(None);
    }
    write_daily_folder(paths, Path::new(folder))?;
    Ok(Some(folder.to_owned()))
}

/// Where a listed backup lives: the daily folder when it holds the file, else the data folder.
fn locate(paths: &AppPaths, name: &str) -> AppResult<PathBuf> {
    if is_backup_name(name) {
        if let Some(folder) = daily_folder(paths)? {
            if fs::symlink_metadata(folder.join(name)).is_ok_and(|meta| meta.is_file()) {
                return Ok(folder);
            }
        }
    }
    Ok(paths.backups.clone())
}

/// Integrity and schema check for a listed backup, wherever it is saved.
pub fn verify(paths: &AppPaths, name: &str) -> AppResult<i64> {
    verify_saved(&locate(paths, name)?, name)
}

pub fn create(source: &Connection, folder: &Path) -> AppResult<BackupFile> {
    create_snapshot(source, folder, None)
}

pub fn copy_to_folder(paths: &AppPaths, name: &str, folder: &str) -> AppResult<BackupFile> {
    let destination = Path::new(folder.trim());
    if !destination.is_absolute() || !destination.is_dir() {
        return Err(AppError::Validation(
            "Enter the full path to an existing backup destination folder.".into(),
        ));
    }
    let destination = destination.canonicalize()?;
    if destination.starts_with(paths.data_dir.canonicalize()?) {
        return Err(AppError::Validation(
            "Choose a folder outside the Pusthakam data directory.".into(),
        ));
    }
    let source = open_saved(&locate(paths, name)?, name)?;
    check_schema_history(&source)?;
    create_snapshot(&source, &destination, None)
}

/// Makes today's dated backup if it does not exist yet, then keeps the newest `DAILY_KEEP`.
/// If the chosen folder cannot be reached (an unplugged USB drive, a missing network share),
/// the backup goes to the Pusthakam data folder instead so the day is never skipped.
pub fn run_daily(source: &Connection, paths: &AppPaths) -> AppResult<bool> {
    if !daily_enabled(source)? || !setup::status(source)?.configured {
        return Ok(false);
    }
    let date: String =
        source.query_row("SELECT strftime('%Y%m%d','now','localtime')", [], |row| {
            row.get(0)
        })?;
    let name = format!("pusthakam-{date}-daily.sqlite3");
    let chosen = daily_folder(paths)?;
    let unavailable = chosen.as_ref().filter(|folder| !folder.is_dir());
    let folder = match &chosen {
        Some(folder) if unavailable.is_none() => folder.clone(),
        _ => paths.backups.clone(),
    };
    if folder.join(&name).try_exists()? {
        return Ok(false);
    }
    let file = create_snapshot(source, &folder, Some(&name))?;
    // Serialize rotation with restore requests, including requests from other app instances.
    let tx = Transaction::new_unchecked(source, TransactionBehavior::Immediate)?;
    let queued = pending_restore(paths)?;
    prune_daily(&folder, DAILY_KEEP, queued.as_deref())?;
    let message = match unavailable {
        Some(missing) => {
            tracing::warn!("Daily backup folder unavailable; saved in the data folder instead");
            format!(
                "{} could not be reached, so the daily backup {} was saved in the Pusthakam data folder. Reconnect the drive to resume backups there.",
                missing.display(),
                file.name
            )
        }
        None => format!("Daily backup {} saved in {}.", file.name, folder.display()),
    };
    let value =
        serde_json::to_string(&message).map_err(|error| AppError::Internal(error.to_string()))?;
    settings::set(&tx, LAST_DAILY, &value)?;
    tx.commit()?;
    Ok(true)
}

/// Delete dated daily backups beyond the newest `keep`. Manual backups and a backup
/// queued for restore are never removed.
fn prune_daily(folder: &Path, keep: usize, queued: Option<&str>) -> AppResult<()> {
    let mut dailies = Vec::new();
    for entry in fs::read_dir(folder)? {
        let entry = entry?;
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        let Some(date) = name
            .strip_prefix("pusthakam-")
            .and_then(|rest| rest.strip_suffix("-daily.sqlite3"))
        else {
            continue;
        };
        if date.len() == 8
            && date.bytes().all(|byte| byte.is_ascii_digit())
            && entry.file_type()?.is_file()
        {
            dailies.push(name);
        }
    }
    // Names embed YYYYMMDD, so a descending sort puts the newest first.
    dailies.sort_by(|left, right| right.cmp(left));
    for name in dailies.into_iter().skip(keep) {
        if Some(name.as_str()) != queued {
            fs::remove_file(folder.join(name))?;
        }
    }
    Ok(())
}

fn check_integrity(conn: &Connection) -> AppResult<()> {
    if conn.query_row("PRAGMA integrity_check(1)", [], |row| {
        row.get::<_, String>(0)
    })? != "ok"
    {
        return Err(AppError::Validation(
            "The backup did not pass SQLite integrity checking.".into(),
        ));
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
                return Err(AppError::Conflict(
                    "The database is busy. Retry the backup operation.".into(),
                ));
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

/// Open only a listed, regular saved backup for an authorized browser download.
pub fn download_file(paths: &AppPaths, name: &str) -> AppResult<fs::File> {
    let folder = locate(paths, name)?;
    verify_saved(&folder, name)?;
    if !is_backup_name(name) {
        return Err(AppError::Validation("Choose a saved backup.".into()));
    }
    let path = folder.join(name);
    if !fs::symlink_metadata(&path)?.is_file() {
        return Err(AppError::Validation("Choose a regular backup file.".into()));
    }
    Ok(fs::File::open(path)?)
}

fn open_saved(folder: &Path, name: &str) -> AppResult<Connection> {
    if !is_backup_name(name) {
        return Err(AppError::Validation(
            "Choose a saved backup from the list.".into(),
        ));
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
    let mut stmt =
        conn.prepare("SELECT version, checksum FROM schema_migrations ORDER BY version")?;
    let mut rows = stmt.query([])?;
    let mut latest = 0;
    for migration in migrations::MIGRATIONS {
        let Some(row) = rows.next()? else { break };
        let version: i64 = row.get(0)?;
        let checksum: String = row.get(1)?;
        if version != migration.version || checksum != migrations::checksum(migration.sql) {
            return Err(AppError::Validation(
                "This backup has an incompatible migration history.".into(),
            ));
        }
        latest = version;
    }
    if latest == 0 || (latest == migrations::latest_version() && rows.next()?.is_some()) {
        return Err(AppError::Validation(
            "This backup has an incompatible database schema version.".into(),
        ));
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

pub fn request_restore(conn: &Connection, paths: &AppPaths, name: &str) -> AppResult<()> {
    let tx = Transaction::new_unchecked(conn, TransactionBehavior::Immediate)?;
    verify(paths, name)?;
    let mut request = tempfile::NamedTempFile::new_in(&paths.data_dir)?;
    request.write_all(name.as_bytes())?;
    request.as_file().sync_all()?;
    request
        .persist_noclobber(paths.data_dir.join(RESTORE_REQUEST))
        .map_err(|error| {
            if error.error.kind() == ErrorKind::AlreadyExists {
                AppError::Conflict(
                    "Cancel the existing restore request before selecting another backup.".into(),
                )
            } else {
                AppError::Io(error.error)
            }
        })?;
    tx.commit()?;
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
    let source = open_saved(&locate(paths, name)?, name)?;
    check_integrity(&source)?;
    check_schema_history(&source)?;
    let staged = tempfile::NamedTempFile::new_in(&paths.backups)?.into_temp_path();
    let mut candidate = Connection::open_with_flags(&staged, OpenFlags::SQLITE_OPEN_READ_WRITE)?;
    copy_database(&source, &mut candidate)?;
    candidate.pragma_update(None, "foreign_keys", "ON")?;
    migrations::run(&mut candidate, migrations::MIGRATIONS)?;
    // An older snapshot may contain a code that was already used or revoked.
    candidate.execute("DELETE FROM staff_recovery_codes", [])?;
    check_integrity(&candidate)?;
    if candidate
        .prepare("PRAGMA foreign_key_check")?
        .query([])?
        .next()?
        .is_some()
    {
        return Err(AppError::Validation(
            "The backup contains broken database references.".into(),
        ));
    }
    let recovery = create(conn, &paths.backups)?;
    copy_database(&candidate, conn)?;
    Ok(recovery.name)
}

/// Startup-only. No staff session or in-flight library commands exist during replacement.
pub fn apply_pending_restore(conn: &mut Connection, paths: &AppPaths) -> AppResult<()> {
    let Some(name) = pending_restore(paths)? else {
        return Ok(());
    };
    fs::remove_file(paths.data_dir.join(RESTORE_REQUEST))?;
    let message = match restore(conn, paths, &name) {
        Ok(recovery) => {
            format!("Database restored from {name}. Previous database saved as {recovery}.")
        }
        Err(error) => {
            tracing::warn!(%error, "Requested database restore failed");
            format!("Restore failed. {}", error.user_message())
        }
    };
    let value =
        serde_json::to_string(&message).map_err(|error| AppError::Internal(error.to_string()))?;
    settings::set(conn, RESTORE_RESULT, &value)
}

fn create_snapshot(
    source: &Connection,
    folder: &Path,
    filename: Option<&str>,
) -> AppResult<BackupFile> {
    let timestamp: String =
        source.query_row("SELECT strftime('%Y%m%dT%H%M%SZ','now')", [], |row| {
            row.get(0)
        })?;
    let staged = tempfile::Builder::new()
        .prefix(&format!("pusthakam-{timestamp}-"))
        .suffix(".partial")
        .tempfile_in(folder)?
        .into_temp_path();
    {
        let mut destination =
            Connection::open_with_flags(&staged, OpenFlags::SQLITE_OPEN_READ_WRITE)?;
        copy_database(source, &mut destination)?;
        let mode: String =
            destination.query_row("PRAGMA journal_mode = DELETE", [], |row| row.get(0))?;
        if mode != "delete" {
            return Err(AppError::Internal(
                "Could not finalize the backup as a single database file.".into(),
            ));
        }
        check_integrity(&destination)?;
        destination.close().map_err(|(_, error)| error)?;
    }
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&staged)?;
    file.sync_all()?;
    let size_bytes = file.metadata()?.len();
    drop(file);
    let target = filename.map_or_else(
        || staged.with_extension("sqlite3"),
        |name| folder.join(name),
    );
    let name = target
        .file_name()
        .ok_or_else(|| AppError::Internal("Missing backup filename.".into()))?
        .to_string_lossy()
        .into_owned();
    staged
        .persist_noclobber(&target)
        .map_err(|error| error.error)?;
    Ok(BackupFile {
        name,
        size_bytes,
        folder: folder.to_string_lossy().into_owned(),
    })
}
