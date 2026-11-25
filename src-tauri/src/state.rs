use crate::error::{AppError, AppResult};
use crate::paths::AppPaths;
use crate::services::auth::{self, Session};
use rusqlite::Connection;
use std::sync::{Arc, Mutex, MutexGuard};
use tracing_appender::non_blocking::WorkerGuard;

/// Shared application state managed by Tauri.
pub struct AppState {
    pub paths: AppPaths,
    pub db: Arc<Mutex<Connection>>,
    pub lan: Arc<Mutex<crate::services::lan_runtime::LanRuntime>>,
    /// The signed-in staff member. Held in memory only: closing the app signs everyone out.
    pub session: Arc<Mutex<Option<Session>>>,
    /// A file that has been previewed and is waiting for the librarian's confirmation.
    pub pending_import: Arc<Mutex<Option<crate::services::import::Pending>>>,
    /// Keeps the log writer alive until exit.
    pub _log_guard: Arc<WorkerGuard>,
}

impl AppState {
    /// Share library ownership without sharing the desktop login or an import preview.
    /// A browser session manager must authenticate/refresh its session before using this context.
    pub fn independent_session(&self, session: Option<Session>) -> Self {
        Self {
            paths: self.paths.clone(),
            db: Arc::clone(&self.db),
            lan: Arc::clone(&self.lan),
            session: Arc::new(Mutex::new(session)),
            pending_import: std::sync::Arc::new(Mutex::new(None)),
            _log_guard: Arc::clone(&self._log_guard),
        }
    }
    pub fn conn(&self) -> AppResult<MutexGuard<'_, Connection>> {
        self.db
            .lock()
            .map_err(|_| AppError::Internal("database lock poisoned".into()))
    }

    /// Runs `f` with the database connection locked for the duration of the call.
    pub fn with_conn<T>(&self, f: impl FnOnce(&Connection) -> AppResult<T>) -> AppResult<T> {
        let conn = self.conn()?;
        f(&conn)
    }

    pub fn current(&self) -> AppResult<Session> {
        let session = self
            .session
            .lock()
            .map_err(|_| AppError::Internal("session lock poisoned".into()))?
            .clone()
            .ok_or_else(|| AppError::Auth("Please sign in.".into()))?;
        self.with_conn(|conn| auth::refresh_session(conn, &session))
    }

    /// The signed-in user, who must hold `permission`. Every data command calls this first.
    pub fn require(&self, permission: &str) -> AppResult<Session> {
        let s = self.current()?;
        s.require(permission)?;
        Ok(s)
    }

    pub fn set_session(&self, s: Option<Session>) -> AppResult<()> {
        *self
            .session
            .lock()
            .map_err(|_| AppError::Internal("session lock poisoned".into()))? = s;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn independent_sessions_share_data_but_not_authentication_or_imports() {
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::ensure(dir.path()).unwrap();
        let conn = crate::db::open(&paths.database).unwrap();
        auth::create_user(&conn, "owner", "Owner", "test-password", "librarian").unwrap();
        auth::create_user(&conn, "reader", "Reader", "test-password", "viewer").unwrap();
        let owner = auth::login(&conn, "owner", "test-password").unwrap();
        let reader = auth::login(&conn, "reader", "test-password").unwrap();
        let host = AppState {
            _log_guard: Arc::new(crate::logging::init(&paths.logs)),
            paths,
            db: Arc::new(Mutex::new(conn)),
            lan: Arc::new(Mutex::new(Default::default())),
            session: Arc::new(Mutex::new(Some(owner.clone()))),
            pending_import: std::sync::Arc::new(Mutex::new(None)),
        };
        let browser = host.independent_session(Some(reader));
        let other = host.independent_session(Some(owner));
        assert!(Arc::ptr_eq(&host.db, &browser.db));
        assert!(Arc::ptr_eq(&host.lan, &browser.lan));
        assert!(host.require("catalogue.write").is_ok());
        assert!(browser.require("catalogue.read").is_ok());
        assert!(browser.require("catalogue.write").is_err());
        *browser.pending_import.lock().unwrap() = Some(crate::services::import::Pending {
            token: "browser-preview".into(),
            source: "test".into(),
            records: vec![],
        });
        assert!(host.pending_import.lock().unwrap().is_none());
        assert!(other.pending_import.lock().unwrap().is_none());
        browser.set_session(None).unwrap();
        assert!(browser.current().is_err());
        assert!(host.current().is_ok());
        assert!(other.current().is_ok());
    }

    #[test]
    fn independent_context_rechecks_revocation_without_changing_host_session() {
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::ensure(dir.path()).unwrap();
        let conn = crate::db::open(&paths.database).unwrap();
        auth::create_user(&conn, "owner", "Owner", "test-password", "librarian").unwrap();
        auth::create_user(&conn, "reader", "Reader", "test-password", "viewer").unwrap();
        let owner = auth::login(&conn, "owner", "test-password").unwrap();
        let reader = auth::login(&conn, "reader", "test-password").unwrap();
        let host = AppState {
            _log_guard: Arc::new(crate::logging::init(&paths.logs)),
            paths,
            db: Arc::new(Mutex::new(conn)),
            lan: Arc::new(Mutex::new(Default::default())),
            session: Arc::new(Mutex::new(Some(owner))),
            pending_import: std::sync::Arc::new(Mutex::new(None)),
        };
        let browser = host.independent_session(Some(reader));
        host.with_conn(|conn| {
            conn.execute(
                "UPDATE staff_users SET active = 0 WHERE username = 'reader'",
                [],
            )?;
            Ok(())
        })
        .unwrap();
        assert!(browser.current().is_err());
        assert!(host.require("catalogue.write").is_ok());
    }
}
