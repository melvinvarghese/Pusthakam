use crate::paths::AppPaths;
use rusqlite::Connection;
use std::sync::Mutex;
use tracing_appender::non_blocking::WorkerGuard;

/// Shared application state managed by Tauri.
pub struct AppState {
    pub paths: AppPaths,
    pub db: Mutex<Connection>,
    /// Keeps the log writer alive until exit.
    pub _log_guard: WorkerGuard,
}
