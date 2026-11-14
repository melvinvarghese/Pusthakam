//! File logging into the app-data `logs/` directory. Local only; nothing is transmitted.
//! Never log passwords, member details or other personal data.

use std::path::Path;
use tracing_appender::non_blocking::WorkerGuard;

/// Returns the guard; keep it alive for the lifetime of the app so logs are flushed.
pub fn init(logs_dir: &Path) -> WorkerGuard {
    let appender = tracing_appender::rolling::daily(logs_dir, "pusthakam.log");
    let (writer, guard) = tracing_appender::non_blocking(appender);
    // Ignore the error if a subscriber is already installed (e.g. in tests).
    let _ = tracing_subscriber::fmt()
        .with_writer(writer)
        .with_ansi(false)
        .with_max_level(tracing::Level::INFO)
        .try_init();
    guard
}
