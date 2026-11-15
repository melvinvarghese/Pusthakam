use crate::error::{AppError, AppResult};
use crate::paths::AppPaths;
use crate::services::auth::Session;
use rusqlite::Connection;
use std::sync::{Mutex, MutexGuard};
use tracing_appender::non_blocking::WorkerGuard;

/// Shared application state managed by Tauri.
pub struct AppState {
    pub paths: AppPaths,
    pub db: Mutex<Connection>,
    /// The signed-in staff member. Held in memory only: closing the app signs everyone out.
    pub session: Mutex<Option<Session>>,
    /// A file that has been previewed and is waiting for the librarian's confirmation.
    pub pending_import: Mutex<Option<crate::services::import::Pending>>,
    /// Keeps the log writer alive until exit.
    pub _log_guard: WorkerGuard,
}

impl AppState {
    pub fn conn(&self) -> AppResult<MutexGuard<'_, Connection>> {
        self.db.lock().map_err(|_| AppError::Internal("database lock poisoned".into()))
    }

    /// Runs `f` with the database connection locked for the duration of the call.
    pub fn with_conn<T>(&self, f: impl FnOnce(&Connection) -> AppResult<T>) -> AppResult<T> {
        let conn = self.conn()?;
        f(&conn)
    }

    pub fn current(&self) -> AppResult<Session> {
        self.session
            .lock()
            .map_err(|_| AppError::Internal("session lock poisoned".into()))?
            .clone()
            .ok_or_else(|| AppError::Auth("Please sign in.".into()))
    }

    /// The signed-in user, who must hold `permission`. Every data command calls this first.
    pub fn require(&self, permission: &str) -> AppResult<Session> {
        let s = self.current()?;
        s.require(permission)?;
        Ok(s)
    }

    pub fn set_session(&self, s: Option<Session>) -> AppResult<()> {
        *self.session.lock().map_err(|_| AppError::Internal("session lock poisoned".into()))? = s;
        Ok(())
    }
}
