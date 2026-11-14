pub mod commands;
pub mod db;
pub mod error;
pub mod logging;
pub mod paths;
pub mod services;
pub mod state;

use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // OS application-data directory, e.g. %APPDATA%\app.pusthakam.desktop on Windows.
            let root = app.path().app_data_dir()?;
            let paths = paths::AppPaths::ensure(&root)?;
            let guard = logging::init(&paths.logs);
            tracing::info!(version = env!("CARGO_PKG_VERSION"), "Pusthakam starting");
            let conn = db::open(&paths.database)?;
            app.manage(state::AppState { paths, db: Mutex::new(conn), _log_guard: guard });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app::app_info,
            commands::app::get_setting,
            commands::app::set_setting,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Pusthakam");
}
