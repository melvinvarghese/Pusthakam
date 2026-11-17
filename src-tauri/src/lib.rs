pub mod commands;
pub mod db;
pub mod domain;
pub mod error;
pub mod logging;
pub mod paths;
pub mod services;
pub mod state;

use std::sync::Mutex;
use tauri::Manager;

/// Every command the UI may call. A macro so the real app and the IPC tests register the same list.
#[macro_export]
macro_rules! app_handlers {
    () => {
        ::tauri::generate_handler![
            $crate::commands::app::app_info,
            $crate::commands::app::dashboard_overview,
            $crate::commands::app::get_setting,
            $crate::commands::app::set_setting,
            $crate::commands::auth::setup_status,
            $crate::commands::auth::complete_setup,
            $crate::commands::auth::login,
            $crate::commands::auth::logout,
            $crate::commands::auth::current_session,
            $crate::commands::config::config_overview,
            $crate::commands::config::config_update_library,
            $crate::commands::config::config_save_named,
            $crate::commands::config::config_save_member_category,
            $crate::commands::config::config_save_shelf,
            $crate::commands::config::config_save_policy,
            $crate::commands::config::config_delete_policy,
            $crate::commands::config::config_set_archived,
            $crate::commands::catalogue::catalogue_search,
            $crate::commands::catalogue::catalogue_get_record,
            $crate::commands::catalogue::catalogue_save_record,
            $crate::commands::catalogue::catalogue_find_duplicates,
            $crate::commands::catalogue::catalogue_archive_record,
            $crate::commands::catalogue::catalogue_add_copies,
            $crate::commands::catalogue::catalogue_update_copy,
            $crate::commands::catalogue::catalogue_set_copy_status,
            $crate::commands::catalogue::catalogue_list_copies,
            $crate::commands::catalogue::accession_format_get,
            $crate::commands::catalogue::accession_format_set,
            $crate::commands::import::import_preview,
            $crate::commands::import::import_commit,
            $crate::commands::import::import_cancel,
            $crate::commands::members::members_list,
            $crate::commands::members::members_list_loans,
            $crate::commands::members::members_save,
            $crate::commands::members::members_archive,
            $crate::commands::members::members_preview_koha,
            $crate::commands::members::members_import_koha,
            $crate::commands::members::members_get_photo,
            $crate::commands::members::members_set_photo,
            $crate::commands::circulation::circulation_prepare_issue,
            $crate::commands::circulation::circulation_prepare_return,
            $crate::commands::circulation::circulation_issue,
            $crate::commands::circulation::circulation_return,
            $crate::commands::circulation::circulation_prepare_renewal,
            $crate::commands::circulation::circulation_renew,
            $crate::commands::circulation::circulation_list_overdue,
            $crate::commands::circulation::circulation_list_reservations,
            $crate::commands::circulation::circulation_place_reservation,
            $crate::commands::circulation::circulation_cancel_reservation,
            $crate::commands::circulation::circulation_list_fines,
            $crate::commands::circulation::circulation_settle_fine,
            $crate::commands::circulation::circulation_fine_history,
            $crate::commands::reports::reports_circulation_activity,
            $crate::commands::reports::reports_popular_titles,
            $crate::commands::reports::reports_stock_by_location,
        ]
    };
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // OS application-data directory, e.g. %APPDATA%\app.pusthakam.desktop on Windows.
            let root = app.path().app_data_dir()?;
            let paths = paths::AppPaths::ensure(&root)?;
            let guard = logging::init(&paths.logs);
            tracing::info!(version = env!("CARGO_PKG_VERSION"), "Pusthakam starting");
            let conn = db::open(&paths.database)?;
            app.manage(state::AppState {
                paths,
                db: Mutex::new(conn),
                session: Mutex::new(None),
                pending_import: Mutex::new(None),
                _log_guard: guard,
            });
            Ok(())
        })
        .invoke_handler(app_handlers!())
        .run(tauri::generate_context!())
        .expect("error while running Pusthakam");
}
