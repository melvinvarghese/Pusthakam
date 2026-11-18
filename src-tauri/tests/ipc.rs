//! End-to-end tests of the command layer. Each call goes through Tauri's real IPC path with the exact
//! JSON shape the React frontend sends (camelCase fields, string enum names), so a mismatch between the
//! UI contract and the Rust commands fails here and not in front of a librarian.

use pusthakam_lib::db;
use pusthakam_lib::logging;
use pusthakam_lib::paths::AppPaths;
use pusthakam_lib::services::auth;
use pusthakam_lib::state::AppState;
use serde_json::{json, Value};
use std::sync::Mutex;
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{get_ipc_response, mock_builder, mock_context, noop_assets, INVOKE_KEY};
use tauri::webview::InvokeRequest;
use tauri::{Manager, WebviewWindow};

type Res = Result<Value, Value>;

struct Harness {
    webview: WebviewWindow<tauri::test::MockRuntime>,
    // Keeps the temp folder and app alive for the whole test.
    _dir: tempfile::TempDir,
    _app: tauri::App<tauri::test::MockRuntime>,
}

impl Harness {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::ensure(&dir.path().join("data")).unwrap();
        let conn = db::open(&paths.database).unwrap();
        let guard = logging::init(&paths.logs);
        let app = mock_builder()
            .invoke_handler(pusthakam_lib::app_handlers!())
            .manage(AppState { paths, db: Mutex::new(conn), session: Mutex::new(None), pending_import: Mutex::new(None), _log_guard: guard })
            .build(mock_context(noop_assets()))
            .unwrap();
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default()).build().unwrap();
        Harness { webview, _dir: dir, _app: app }
    }

    fn call(&self, cmd: &str, body: Value) -> Res {
        get_ipc_response(
            &self.webview,
            InvokeRequest {
                cmd: cmd.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: "http://tauri.localhost".parse().unwrap(),
                body: InvokeBody::Json(body),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .map(|b| b.deserialize::<Value>().unwrap())
    }

    fn ok(&self, cmd: &str, body: Value) -> Value {
        self.call(cmd, body).unwrap_or_else(|e| panic!("{cmd} failed: {e}"))
    }

    fn code(&self, cmd: &str, body: Value) -> String {
        match self.call(cmd, body) {
            Err(e) => e["code"].as_str().unwrap_or("<no code>").to_string(),
            Ok(v) => panic!("{cmd} should have failed but returned {v}"),
        }
    }
}

fn setup_body() -> Value {
    json!({ "input": {
        "library": { "name": "Demo Library", "profile": "school", "address": null, "contact": null, "currency": "INR", "locale": "en" },
        "admin": { "username": "admin", "displayName": "Head Librarian", "password": "a long passphrase" },
        "applyProfileDefaults": true,
        "finePerDayMinor": 100
    }})
}

#[test]
fn member_directory_paginates_and_uses_the_selected_category_expiry() {
    let h = Harness::new();
    h.ok("complete_setup", setup_body());
    let config = h.ok("config_overview", json!({}));
    let category = &config["memberCategories"][0];
    for index in 0..26 {
        h.ok("members_save", json!({ "input": {
            "name": format!("Page Reader {index:02}"), "memberNumber": format!("PAGE-{index:02}"),
            "categoryId": category["id"]
        }}));
    }
    let first = h.ok("members_list", json!({ "search": "Page Reader", "page": 1, "archived": false }));
    let second = h.ok("members_list", json!({ "search": "Page Reader", "page": 2, "archived": false }));
    assert_eq!(first["total"], 26);
    assert_eq!(first["rows"].as_array().unwrap().len(), 25);
    assert_eq!(second["rows"].as_array().unwrap().len(), 1);
    assert_eq!(second["rows"][0]["memberNumber"], "PAGE-25");
    let lookup = h.ok("members_list", json!({ "search": "PAGE-25", "page": 1, "archived": false }));
    assert_eq!(lookup["rows"][0]["id"], second["rows"][0]["id"]);
    let expiry: String = {
        let state = h.webview.state::<AppState>();
        let conn = state.conn().unwrap();
        conn.query_row("SELECT date('now','localtime','+' || membership_days || ' days') FROM member_categories WHERE id = ?1",
            [category["id"].as_i64().unwrap()], |row| row.get(0)).unwrap()
    };
    assert_eq!(lookup["rows"][0]["expiresOn"], expiry);
}

#[test]
fn stock_checks_preserve_expected_and_unexpected_copies_across_reopen() {
    let h = Harness::new();
    assert_eq!(h.code("stock_check_list", json!({ "page": 1 })), "AUTH_FAILED");
    h.ok("complete_setup", setup_body());
    let config = h.ok("config_overview", json!({}));
    let location = &config["locations"][0]["id"];
    let record = h.ok("catalogue_save_record", json!({ "input": {
        "title": "Shelf fixture", "contributors": [], "subjects": []
    }}));
    let copies = h.ok("catalogue_add_copies", json!({ "recordId": record, "input": {
        "count": 3, "itemCategoryId": config["itemCategories"][0]["id"], "locationId": location,
        "barcodes": ["SCAN-A", "SCAN-B", "SCAN-C"], "acquisitionSource": "purchase"
    }}));
    h.ok("catalogue_set_copy_status", json!({ "id": copies[2]["id"], "status": "damaged" }));
    assert_eq!(h.code("stock_check_start", json!({ "locationId": location, "shelfId": -1 })), "VALIDATION");
    let id = h.ok("stock_check_start", json!({ "locationId": location }));
    let scan = |code: &str| json!({ "id": id, "barcode": code });
    assert_eq!(h.ok("stock_check_scan", scan("scan-a")), "matched");
    assert_eq!(h.ok("stock_check_scan", scan("SCAN-A")), "duplicate");
    assert_eq!(h.ok("stock_check_scan", scan("SCAN-C")), "unexpected");
    assert_eq!(h.ok("stock_check_scan", scan("UNKNOWN-1")), "unexpected");
    let missing = h.ok("stock_check_review", json!({ "id": id, "view": "missing", "page": 1 }));
    assert_eq!(missing["check"]["expected"], 2);
    assert_eq!(missing["check"]["matched"], 1);
    assert_eq!(missing["check"]["unexpected"], 2);
    assert_eq!(missing["entries"]["rows"][0]["barcode"], "SCAN-B");
    let unexpected = h.ok("stock_check_review", json!({ "id": id, "view": "unexpected", "page": 1 }));
    assert_eq!(unexpected["entries"]["rows"][0]["title"], "Shelf fixture");
    assert!(unexpected["entries"]["rows"][1]["title"].is_null());
    {
        let paths = h.webview.state::<AppState>().paths.clone();
        let reopened = db::open(&paths.database).unwrap();
        let review = pusthakam_lib::services::inventory::review(&reopened, id.as_i64().unwrap(),
            pusthakam_lib::services::inventory::View::Missing, 1).unwrap();
        assert_eq!(serde_json::to_value(review).unwrap(), missing);
    }
    h.ok("stock_check_close", json!({ "id": id }));
    assert_eq!(h.code("stock_check_scan", scan("SCAN-B")), "CONFLICT");
    assert!(h.ok("stock_check_list", json!({ "page": 1 }))["rows"][0]["closedAt"].is_string());
    assert_eq!(h.ok("catalogue_list_copies", json!({ "query": { "status": "available" } }))["total"], 2);
    let destination = h.ok("config_save_named", json!({ "kind": "location", "input": { "name": "Destination reading room" } }));
    assert_eq!(h.code("stock_transfer_send", json!({ "barcode": "SCAN-A", "locationId": location })), "VALIDATION");
    let transfer = h.ok("stock_transfer_send", json!({ "barcode": "SCAN-A", "locationId": destination }));
    assert_eq!(h.ok("stock_transfer_list", json!({ "page": 1 }))["rows"][0]["status"], "in_transit");
    {
        let paths = h.webview.state::<AppState>().paths.clone();
        let reopened = db::open(&paths.database).unwrap();
        let history = pusthakam_lib::services::transfers::list(&reopened, 1).unwrap();
        assert_eq!(serde_json::to_value(history).unwrap()["rows"][0]["id"], transfer);
    }
    assert_eq!(h.code("catalogue_set_copy_status", json!({ "id": copies[0]["id"], "status": "available" })), "CONFLICT");
    assert_eq!(h.code("catalogue_archive_record", json!({ "id": record, "archived": true })), "CONFLICT");
    h.ok("stock_transfer_finish", json!({ "id": transfer, "receive": true }));
    let moved = h.ok("catalogue_get_record", json!({ "id": record }));
    let copy = moved["copies"].as_array().unwrap().iter().find(|copy| copy["barcode"] == "SCAN-A").unwrap();
    assert_eq!(copy["locationId"], destination);
    assert_eq!(copy["status"], "available");
    assert_eq!(h.code("stock_transfer_finish", json!({ "id": transfer, "receive": true })), "CONFLICT");
    let transfer_back = h.ok("stock_transfer_send", json!({ "barcode": "SCAN-A", "locationId": location }));
    h.ok("stock_transfer_finish", json!({ "id": transfer_back, "receive": false }));
    let transfers = h.ok("stock_transfer_list", json!({ "page": 1 }));
    assert_eq!(transfers["rows"][0]["status"], "cancelled");
    assert_eq!(transfers["rows"][1]["destination"], "Destination reading room");
    {
        let state = h.webview.state::<AppState>();
        let conn = state.conn().unwrap();
        auth::create_user(&conn, "viewer", "Read Only", "viewer passphrase", "viewer").unwrap();
    }
    h.ok("login", json!({ "username": "viewer", "password": "viewer passphrase" }));
    assert_eq!(h.ok("stock_check_review", json!({ "id": id, "view": "missing", "page": 1 }))["entries"]["total"], 1);
    assert_eq!(h.code("stock_check_start", json!({ "locationId": location })), "PERMISSION_DENIED");
    assert_eq!(h.code("stock_transfer_send", json!({ "barcode": "SCAN-A", "locationId": location })), "PERMISSION_DENIED");
}

#[test]
fn staff_roles_passwords_and_administrator_recovery_take_effect_without_touching_library_data() {
    let h = Harness::new();
    assert_eq!(h.code("staff_list", json!({})), "AUTH_FAILED");
    let admin = h.ok("complete_setup", setup_body());
    let account = h.ok("staff_create", json!({ "input": {
        "username": "desk", "displayName": "Desk Staff", "role": "viewer", "password": "first test password"
    }}));
    let id = account.as_i64().unwrap();
    let stale = {
        let state = h.webview.state::<AppState>();
        let conn = state.conn().unwrap();
        conn.execute("UPDATE staff_users SET locked_until = '2099-01-01T00:00:00Z' WHERE id = ?1", [id]).unwrap();
        auth::load_session(&conn, id).unwrap()
    };
    assert_eq!(h.ok("staff_list", json!({}))["users"][1]["locked"], true);
    h.ok("staff_unlock", json!({ "id": id }));
    assert_eq!(h.ok("staff_list", json!({}))["users"][1]["locked"], false);
    let changes = |active| json!({ "displayName": "Desk Staff", "role": "circulation", "active": active });
    assert_eq!(h.code("staff_update", json!({ "id": admin["userId"], "input": changes(false) })), "VALIDATION");
    h.ok("staff_update", json!({ "id": id, "input": changes(false) }));
    assert_eq!(h.code("login", json!({ "username": "desk", "password": "first test password" })), "AUTH_FAILED");
    h.ok("staff_update", json!({ "id": id, "input": changes(true) }));
    h.ok("staff_reset_password", json!({ "id": id, "password": "replacement test password" }));
    {
        let state = h.webview.state::<AppState>();
        let conn = state.conn().unwrap();
        assert!(auth::refresh_session(&conn, &stale).is_err(), "an earlier session cannot outlive account changes");
        let hash: String = conn.query_row("SELECT password_hash FROM staff_users WHERE id = ?1", [id], |row| row.get(0)).unwrap();
        assert!(hash.starts_with("$argon2id$"));
    }
    let desk = h.ok("login", json!({ "username": "desk", "password": "replacement test password" }));
    assert_eq!(desk["role"], "circulation");
    assert!(desk.get("revision").is_none());
    assert_eq!(h.code("staff_list", json!({})), "PERMISSION_DENIED");
    assert_eq!(h.code("staff_change_password", json!({ "currentPassword": "wrong", "newPassword": "changed test password" })), "AUTH_FAILED");
    h.ok("staff_change_password", json!({ "currentPassword": "replacement test password", "newPassword": "changed test password" }));
    assert!(h.ok("current_session", json!({})).is_null());
    h.ok("login", json!({ "username": "desk", "password": "changed test password" }));
    h.ok("login", json!({ "username": "admin", "password": "a long passphrase" }));
    let discarded = h.ok("staff_generate_recovery_code", json!({ "currentPassword": "a long passphrase" }));
    let code = h.ok("staff_generate_recovery_code", json!({ "currentPassword": "a long passphrase" }));
    assert_ne!(discarded, code);
    assert_eq!(code.as_str().unwrap().len(), 64);
    {
        let state = h.webview.state::<AppState>();
        let conn = state.conn().unwrap();
        let stored: String = conn.query_row("SELECT code_hash FROM staff_recovery_codes", [], |row| row.get(0)).unwrap();
        assert!(stored.starts_with("$argon2id$"));
        assert_ne!(stored, code.as_str().unwrap());
    }
    h.ok("logout", json!({}));
    assert_eq!(h.code("staff_recover", json!({ "username": "admin", "code": discarded, "newPassword": "recovered password" })), "AUTH_FAILED");
    let recover = json!({ "username": "admin", "code": code, "newPassword": "recovered password" });
    h.ok("staff_recover", recover.clone());
    assert_eq!(h.code("staff_recover", recover), "AUTH_FAILED");
    h.ok("login", json!({ "username": "admin", "password": "recovered password" }));
    assert_eq!(h.ok("config_overview", json!({}))["library"]["name"], "Demo Library");
    assert_eq!(h.ok("audit_search", json!({ "query": { "action": "password_recovered", "page": 1 } }))["total"], 1);
}

#[test]
fn manual_backups_are_permission_checked_complete_and_retained() {
    let h = Harness::new();
    let destination = tempfile::tempdir().unwrap();
    assert_eq!(h.code("backup_list", json!({})), "AUTH_FAILED");
    assert_eq!(h.code("backup_create", json!({})), "AUTH_FAILED");
    assert_eq!(h.code("backup_copy_to_folder", json!({ "name": "pusthakam-none.sqlite3", "folder": destination.path() })), "AUTH_FAILED");
    h.ok("complete_setup", setup_body());
    let initial = h.ok("backup_list", json!({}));
    assert_eq!(initial["files"], json!([]));
    assert_eq!(initial["dailyEnabled"], true);
    let folder = std::path::PathBuf::from(initial["folder"].as_str().unwrap());
    let mut writer = {
        let state = h.webview.state::<AppState>();
        rusqlite::Connection::open(&state.paths.database).unwrap()
    };
    let tx = writer.transaction().unwrap();
    tx.execute("UPDATE libraries SET name = 'Changed Library'", []).unwrap();
    let first = h.ok("backup_create", json!({}));
    tx.commit().unwrap();
    drop(writer);
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        auth::create_user(&conn, "viewer", "Visitor", "visitor passphrase", "viewer").unwrap();
    }
    let second = h.ok("backup_create", json!({}));
    assert_ne!(first["name"], second["name"]);
    assert_eq!(h.code("backup_copy_to_folder", json!({ "name": first["name"], "folder": folder })), "VALIDATION");
    let copied = h.ok("backup_copy_to_folder", json!({ "name": first["name"], "folder": destination.path() }));
    let copied_name = copied["name"].as_str().unwrap();
    assert_eq!(pusthakam_lib::services::backup::verify_saved(destination.path(), copied_name).unwrap(),
        db::migrations::latest_version());
    let exported = rusqlite::Connection::open(destination.path().join(copied_name)).unwrap();
    assert_eq!(exported.query_row("SELECT name FROM libraries", [], |row| row.get::<_, String>(0)).unwrap(), "Demo Library");
    let files = h.ok("backup_list", json!({}));
    assert_eq!(files["files"].as_array().unwrap().len(), 2);
    assert_eq!(h.ok("backup_verify", json!({ "name": first["name"] })), db::migrations::latest_version());
    for (backup, expected_name) in [(first, "Demo Library"), (second, "Changed Library")] {
        let file = folder.join(backup["name"].as_str().unwrap());
        assert_eq!(std::fs::metadata(&file).unwrap().len(), backup["sizeBytes"].as_u64().unwrap());
        let conn = rusqlite::Connection::open_with_flags(&file, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
        assert_eq!(conn.query_row("SELECT name FROM libraries", [], |row| row.get::<_, String>(0)).unwrap(), expected_name);
        assert_eq!(conn.query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0)).unwrap(), "ok");
        assert_eq!(conn.query_row("SELECT max(version) FROM schema_migrations", [], |row| row.get::<_, i64>(0)).unwrap(),
            db::migrations::MIGRATIONS.last().unwrap().version);
    }
    assert_eq!(std::fs::read_dir(&folder).unwrap().count(), 2, "only finalized database files remain");
    assert_eq!(h.code("backup_verify", json!({ "name": "../pusthakam-old.sqlite3" })), "VALIDATION");
    std::fs::write(folder.join("pusthakam-broken.sqlite3"), b"not a SQLite file").unwrap();
    assert_eq!(h.code("backup_verify", json!({ "name": "pusthakam-broken.sqlite3" })), "DATABASE");
    let run_daily = || {
        let state = h.webview.state::<AppState>();
        state.with_conn(|conn| pusthakam_lib::services::backup::run_daily(conn, &state.paths)).unwrap()
    };
    let older = "pusthakam-20000101-daily.sqlite3";
    let queued = "pusthakam-20000102-daily.sqlite3";
    let manual = "pusthakam-20000101T000000Z-manual.sqlite3";
    let source = std::fs::read_dir(&folder).unwrap().map(|entry| entry.unwrap().path())
        .find(|path| path.file_name().unwrap() != "pusthakam-broken.sqlite3").unwrap();
    for name in [older, queued, manual] {
        std::fs::copy(&source, folder.join(name)).unwrap();
    }
    h.ok("backup_request_restore", json!({ "name": queued }));
    assert!(run_daily());
    assert!(!folder.join(older).exists());
    assert!(folder.join(queued).is_file());
    assert!(folder.join(manual).is_file());
    h.ok("backup_cancel_restore", json!({}));
    std::fs::remove_file(folder.join(queued)).unwrap();
    assert!(!run_daily());
    h.ok("backup_set_daily", json!({ "enabled": false }));
    assert_eq!(h.ok("backup_list", json!({}))["dailyEnabled"], false);
    let daily_name = h.ok("backup_list", json!({}))["files"].as_array().unwrap().iter()
        .find(|file| file["name"].as_str().unwrap().ends_with("-daily.sqlite3")).unwrap()["name"]
        .as_str().unwrap().to_owned();
    let parked = folder.join("daily-check.sqlite3");
    std::fs::rename(folder.join(&daily_name), &parked).unwrap();
    assert!(!run_daily());
    std::fs::rename(parked, folder.join(daily_name)).unwrap();
    let list = h.ok("backup_list", json!({}));
    let daily = list["files"].as_array().unwrap().iter()
        .find(|file| file["name"].as_str().unwrap().ends_with("-daily.sqlite3")).unwrap();
    assert_eq!(h.ok("backup_verify", json!({ "name": daily["name"] })), db::migrations::latest_version());
    h.ok("login", json!({ "username": "viewer", "password": "visitor passphrase" }));
    assert_eq!(h.code("backup_list", json!({})), "PERMISSION_DENIED");
    assert_eq!(h.code("backup_create", json!({})), "PERMISSION_DENIED");
    assert_eq!(h.code("backup_copy_to_folder", json!({ "name": copied_name, "folder": destination.path() })), "PERMISSION_DENIED");
    assert_eq!(h.code("backup_set_daily", json!({ "enabled": false })), "PERMISSION_DENIED");
}

#[test]
fn restore_waits_for_other_instances_and_preserves_the_previous_database() {
    use pusthakam_lib::services::backup;
    let h = Harness::new();
    assert_eq!(h.code("backup_request_restore", json!({ "name": "pusthakam-test.sqlite3" })), "AUTH_FAILED");
    h.ok("complete_setup", setup_body());
    let config = h.ok("config_overview", json!({}));
    let policy = &config["policies"][0];
    let member = h.ok("members_save", json!({ "input": {
        "memberNumber": "RESTORE-MEMBER", "name": "Restore Member", "categoryId": policy["memberCategoryId"]
    }}));
    let record = h.ok("catalogue_save_record", json!({ "input": {
        "title": "Restore fixture", "contributors": [], "subjects": []
    }}));
    let copies = h.ok("catalogue_add_copies", json!({ "recordId": record, "input": {
        "count": 1, "itemCategoryId": policy["itemCategoryId"], "acquisitionSource": "purchase"
    }}));
    let code = &copies[0]["barcode"];
    let issue = h.ok("circulation_prepare_issue", json!({ "memberNumber": "RESTORE-MEMBER", "barcode": code }));
    h.ok("circulation_issue", json!({
        "memberNumber": "RESTORE-MEMBER", "barcode": code, "memberId": member,
        "itemId": copies[0]["id"], "dueOn": issue["dueOn"]
    }));
    let loan = h.ok("circulation_prepare_return", json!({ "barcode": code }));
    let saved = h.ok("backup_create", json!({}));
    let name = saved["name"].as_str().unwrap();
    let paths = h.webview.state::<AppState>().paths.clone();
    let (session_lock, exclusive) = backup::startup_lock(&paths).unwrap();
    assert!(!exclusive);
    h.ok("backup_request_restore", json!({ "name": name }));
    assert_eq!(h.ok("backup_list", json!({}))["pendingRestore"], name);
    assert_eq!(h.code("backup_request_restore", json!({ "name": name })), "CONFLICT");
    h.ok("backup_cancel_restore", json!({}));
    assert!(h.ok("backup_list", json!({}))["pendingRestore"].is_null());

    h.ok("backup_request_restore", json!({ "name": name }));
    h.ok("circulation_return", json!({ "loanId": loan["loanId"] }));
    {
        let state = h.webview.state::<AppState>();
        let conn = state.conn().unwrap();
        conn.execute("UPDATE libraries SET name = 'After snapshot'", []).unwrap();
        auth::create_user(&conn, "later", "Later Staff", "a later passphrase", "viewer").unwrap();
    }
    let (second_process, exclusive) = backup::startup_lock(&paths).unwrap();
    assert!(!exclusive, "another open copy defers restore");
    drop(second_process);
    drop(session_lock);
    let (restore_lock, exclusive) = backup::startup_lock(&paths).unwrap();
    assert!(exclusive);
    let mut conn = db::open(&paths.database).unwrap();
    backup::apply_pending_restore(&mut conn, &paths).unwrap();
    let _lock = backup::hold_library_lock(restore_lock, exclusive).unwrap();
    assert_eq!(conn.query_row("SELECT name FROM libraries", [], |row| row.get::<_, String>(0)).unwrap(), "Demo Library");
    assert_eq!(conn.query_row("SELECT count(*) FROM staff_users WHERE username = 'later'", [], |row| row.get::<_, i64>(0)).unwrap(), 0);
    assert_eq!(conn.query_row("SELECT count(*) FROM bibliographic_records WHERE title = 'Restore fixture'", [], |row| row.get::<_, i64>(0)).unwrap(), 1);
    assert_eq!(conn.query_row("SELECT count(*) FROM loans WHERE returned_at IS NULL", [], |row| row.get::<_, i64>(0)).unwrap(), 1);
    let listed = h.ok("backup_list", json!({}));
    assert!(listed["pendingRestore"].is_null());
    assert!(listed["lastRestore"].as_str().unwrap().starts_with("Database restored"));
    let recovery = listed["files"].as_array().unwrap().iter().find(|item| item["name"] != name).unwrap();
    let recovery_conn = rusqlite::Connection::open_with_flags(
        paths.backups.join(recovery["name"].as_str().unwrap()), rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    ).unwrap();
    assert_eq!(recovery_conn.query_row("SELECT name FROM libraries", [], |row| row.get::<_, String>(0)).unwrap(), "After snapshot");
    assert_eq!(recovery_conn.query_row("SELECT count(*) FROM staff_users WHERE username = 'later'", [], |row| row.get::<_, i64>(0)).unwrap(), 1);
    assert_eq!(recovery_conn.query_row("SELECT count(*) FROM loans WHERE returned_at IS NOT NULL", [], |row| row.get::<_, i64>(0)).unwrap(), 1);

    let tampered = "pusthakam-tampered.sqlite3";
    std::fs::copy(paths.backups.join(name), paths.backups.join(tampered)).unwrap();
    h.ok("backup_request_restore", json!({ "name": tampered }));
    std::fs::write(paths.backups.join(tampered), b"corrupted after verification").unwrap();
    backup::apply_pending_restore(&mut conn, &paths).unwrap();
    assert_eq!(conn.query_row("SELECT name FROM libraries", [], |row| row.get::<_, String>(0)).unwrap(), "Demo Library");
    assert!(h.ok("backup_list", json!({}))["lastRestore"].as_str().unwrap().starts_with("Restore failed"));
}

#[test]
fn members_can_be_registered_found_updated_and_restored_through_ipc() {
    use base64::Engine;
    let h = Harness::new();
    let list = json!({ "search": "M%01", "page": 1, "archived": false });
    assert_eq!(h.code("members_list", list.clone()), "AUTH_FAILED");
    assert_eq!(h.code("reports_member_summary", json!({})), "AUTH_FAILED");
    h.ok("complete_setup", setup_body());
    let config = h.ok("config_overview", json!({}));
    let mut input = json!({
        "memberNumber": "M%01",
        "name": "Asha Nair",
        "categoryId": config["memberCategories"][0]["id"],
        "email": "asha@example.test",
        "phone": null,
        "expiresOn": null
    });
    let id = h.ok("members_save", json!({ "input": input.clone() }));
    assert_eq!(h.code("members_save", json!({ "input": input.clone() })), "CONFLICT");
    let row = &h.ok("members_list", list.clone())["rows"][0];
    assert_eq!(row["id"], id);
    assert_eq!(row["memberNumber"], "M%01");
    assert_eq!(row["expiresOn"].as_str().unwrap().len(), 10);
    assert!(h.ok("members_get_photo", json!({ "id": id })).is_null());
    assert_eq!(h.code("members_set_photo", json!({ "id": id, "pngBase64": "eA==" })), "VALIDATION");
    let mut photo = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut photo, 1, 1);
        encoder.set_color(png::ColorType::Rgb);
        encoder.add_text_chunk("Comment".into(), "Camera metadata".into()).unwrap();
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&[20, 60, 90]).unwrap();
        writer.finish().unwrap();
    }
    let photo = base64::engine::general_purpose::STANDARD.encode(photo);
    h.ok("members_set_photo", json!({ "id": id, "pngBase64": photo }));
    let stored = h.ok("members_get_photo", json!({ "id": id }));
    let bytes = base64::engine::general_purpose::STANDARD.decode(stored.as_str().unwrap()).unwrap();
    assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
    assert!(!bytes.windows(b"Camera metadata".len()).any(|part| part == b"Camera metadata"));

    input["id"] = id.clone();
    input["name"] = json!("Asha Ravi");
    h.ok("members_save", json!({ "input": input.clone() }));
    h.ok("members_archive", json!({ "id": id, "archived": true }));
    assert_eq!(h.ok("reports_member_summary", json!({}))[0]["archived"], 1);
    assert_eq!(h.ok("members_list", list.clone())["total"], 0);
    let archived = h.ok("members_list", json!({ "search": "Ravi", "page": 1, "archived": true }));
    assert_eq!(archived["rows"][0]["id"], id);
    h.ok("members_archive", json!({ "id": id, "archived": false }));
    assert_eq!(h.ok("members_list", list.clone())["total"], 1);
    let mut expired = input.clone();
    expired["id"] = Value::Null;
    expired["memberNumber"] = json!("M-EXPIRED");
    expired["expiresOn"] = json!("2000-01-01");
    let expired_id = h.ok("members_save", json!({ "input": expired }));
    assert_eq!(h.ok("reports_member_summary", json!({})), json!([{
        "categoryId": input["categoryId"], "category": config["memberCategories"][0]["name"],
        "archivedCategory": false, "current": 1, "expired": 1, "archived": 0, "total": 2
    }]));
    h.ok("members_archive", json!({ "id": expired_id, "archived": true }));

    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        auth::create_user(&conn, "viewer", "Visitor", "visitor passphrase", "viewer").unwrap();
    }
    h.ok("login", json!({ "username": "viewer", "password": "visitor passphrase" }));
    assert_eq!(h.ok("members_list", list)["total"], 1);
    let dashboard = h.ok("dashboard_overview", json!({}));
    assert_eq!(dashboard["activeMembers"], 1);
    assert_eq!(dashboard["catalogue"]["titles"], 0);
    assert!(dashboard["circulation"].is_null());
    assert_eq!(h.code("members_save", json!({ "input": input })), "PERMISSION_DENIED");
    assert_eq!(h.code("members_archive", json!({ "id": id, "archived": true })), "PERMISSION_DENIED");
    assert_eq!(h.ok("members_get_photo", json!({ "id": id })), stored);
    assert_eq!(h.code("members_set_photo", json!({ "id": id, "pngBase64": null })), "PERMISSION_DENIED");
    h.ok("login", json!({ "username": "admin", "password": "a long passphrase" }));
    h.ok("members_set_photo", json!({ "id": id, "pngBase64": null }));
    assert!(h.ok("members_get_photo", json!({ "id": id })).is_null());
}

#[test]
fn setup_sign_in_and_configuration_work_through_the_real_commands() {
    let h = Harness::new();

    // Fresh install: not configured, and nothing else is usable.
    assert_eq!(h.ok("setup_status", json!({}))["configured"], false);
    assert_eq!(h.code("config_overview", json!({})), "AUTH_FAILED");
    assert_eq!(h.code("get_setting", json!({ "key": "ui.theme" })), "AUTH_FAILED");
    assert_eq!(h.ok("current_session", json!({})), Value::Null);

    // The wizard's request signs the new administrator in.
    let session = h.ok("complete_setup", setup_body());
    assert_eq!(session["role"], "administrator");
    assert_eq!(session["displayName"], "Head Librarian");
    assert!(session["permissions"].as_array().unwrap().iter().any(|p| p == "admin.settings"));
    assert!(session.get("passwordHash").is_none() && session.get("password_hash").is_none());
    assert_eq!(h.ok("setup_status", json!({}))["configured"], true);
    assert_eq!(h.code("complete_setup", setup_body()), "CONFLICT", "setup must only run once");

    // Settings page data arrives in the shape the UI types expect.
    let cfg = h.ok("config_overview", json!({}));
    assert_eq!(cfg["library"]["name"], "Demo Library");
    assert_eq!(cfg["memberCategories"].as_array().unwrap().len(), 3);
    assert_eq!(cfg["policies"][0]["fineGraceDays"], 1);
    assert_eq!(cfg["policies"][0]["finePerDayMinor"], 100);
    assert!(cfg["policies"][0]["fineCapMinor"].is_null());
    assert_eq!(cfg["collections"][0]["archived"], false);

    // Edit flows with the exact payloads the dialogs send.
    let id = h.ok("config_save_named", json!({ "kind": "collection", "input": { "id": null, "name": "Fiction", "description": null } }));
    assert!(id.as_i64().unwrap() > 0);
    assert_eq!(
        h.code("config_save_named", json!({ "kind": "collection", "input": { "id": null, "name": "fiction", "description": null } })),
        "CONFLICT"
    );
    h.ok("config_save_named", json!({ "kind": "itemCategory", "input": { "id": null, "name": "Periodical", "description": "Magazines" } }));
    let loc = h.ok("config_save_named", json!({ "kind": "location", "input": { "id": null, "name": "Annex", "description": null } }));
    h.ok("config_save_shelf", json!({ "input": { "id": null, "locationId": loc, "code": "A1", "description": null } }));
    h.ok("config_save_member_category", json!({ "input": { "id": null, "name": "Visitor", "description": null, "membershipDays": 30 } }));
    h.ok("config_set_archived", json!({ "kind": "collection", "id": id, "archived": true }));
    h.ok("config_update_library", json!({ "input": { "name": "Renamed Library", "profile": "college", "address": "Kochi", "contact": null, "currency": "INR", "locale": "en" } }));

    let cfg = h.ok("config_overview", json!({}));
    assert_eq!(cfg["library"]["name"], "Renamed Library");
    assert_eq!(cfg["collections"].as_array().unwrap().iter().find(|c| c["name"] == "Fiction").unwrap()["archived"], true);

    // Loan rule create, update and delete.
    let member = cfg["memberCategories"].as_array().unwrap().iter().find(|m| m["name"] == "Visitor").unwrap()["id"].clone();
    let item = cfg["itemCategories"].as_array().unwrap().iter().find(|m| m["name"] == "Periodical").unwrap()["id"].clone();
    let rule = |id: Value, loan_days: i64| {
        json!({ "input": {
            "id": id, "memberCategoryId": member, "itemCategoryId": item, "loanDays": loan_days, "maxLoans": 2,
            "maxRenewals": 1, "renewalDays": 7, "finePerDayMinor": 250, "fineGraceDays": 0, "fineCapMinor": 5000,
            "reservationHoldDays": 3
        }})
    };
    let pid = h.ok("config_save_policy", rule(Value::Null, 7));
    assert_eq!(h.code("config_save_policy", rule(Value::Null, 7)), "CONFLICT");
    h.ok("config_save_policy", rule(pid.clone(), 10));
    let cfg = h.ok("config_overview", json!({}));
    let saved = cfg["policies"].as_array().unwrap().iter().find(|p| p["id"] == pid).unwrap().clone();
    assert_eq!((saved["loanDays"].as_i64(), saved["fineCapMinor"].as_i64()), (Some(10), Some(5000)));
    h.ok("config_delete_policy", json!({ "id": pid }));

    // Errors reach the UI as { code, message } without internals.
    let err = h.call("config_save_named", json!({ "kind": "collection", "input": { "id": null, "name": "  ", "description": null } })).unwrap_err();
    assert_eq!(err["code"], "VALIDATION");
    assert!(err["message"].as_str().unwrap().contains("required"));

    // Settings (key/value) are admin-only to change.
    h.ok("set_setting", json!({ "key": "ui.density", "value": "\"compact\"" }));
    assert_eq!(h.ok("get_setting", json!({ "key": "ui.density" })), "\"compact\"");
}

#[test]
fn sign_out_and_in_and_permissions_are_enforced_in_rust() {
    let h = Harness::new();
    h.ok("complete_setup", setup_body());

    // Add a librarian directly (staff management UI comes later) and an inactive-free clerk.
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        auth::create_user(&conn, "rekha", "Rekha Nair", "another passphrase", "librarian").unwrap();
        auth::create_user(&conn, "viewer", "Visitor", "yet another phrase", "viewer").unwrap();
    }

    h.ok("logout", json!({}));
    assert_eq!(h.ok("current_session", json!({})), Value::Null);
    assert_eq!(h.code("config_overview", json!({})), "AUTH_FAILED");

    // Wrong credentials give the generic message and keep the user signed out.
    let err = h.call("login", json!({ "username": "rekha", "password": "nope nope nope" })).unwrap_err();
    assert_eq!(err["code"], "AUTH_FAILED");
    assert_eq!(err["message"], "Incorrect username or password.");
    assert_eq!(h.ok("current_session", json!({})), Value::Null);

    // A librarian can read settings but not change them, and cannot change application settings.
    let s = h.ok("login", json!({ "username": "REKHA", "password": "another passphrase" }));
    assert_eq!(s["role"], "librarian");
    assert_eq!(h.ok("current_session", json!({}))["username"], "rekha");
    h.ok("config_overview", json!({}));
    assert_eq!(
        h.code("config_save_named", json!({ "kind": "collection", "input": { "id": null, "name": "Hack", "description": null } })),
        "PERMISSION_DENIED"
    );
    assert_eq!(h.code("config_set_archived", json!({ "kind": "location", "id": 1, "archived": true })), "PERMISSION_DENIED");
    assert_eq!(h.code("config_delete_policy", json!({ "id": 1 })), "PERMISSION_DENIED");
    assert_eq!(h.code("set_setting", json!({ "key": "a", "value": "b" })), "PERMISSION_DENIED");
    assert_eq!(
        h.code("config_update_library", json!({ "input": { "name": "X", "profile": "school", "address": null, "contact": null, "currency": "INR", "locale": "en" } })),
        "PERMISSION_DENIED"
    );

    // Nothing changed behind the denied calls.
    h.ok("logout", json!({}));
    h.ok("login", json!({ "username": "admin", "password": "a long passphrase" }));
    let cfg = h.ok("config_overview", json!({}));
    assert_eq!(cfg["library"]["name"], "Demo Library");
    assert!(cfg["collections"].as_array().unwrap().iter().all(|c| c["name"] != "Hack"));
}

#[test]
fn catalogue_commands_accept_the_ui_payloads_and_enforce_permissions() {
    let h = Harness::new();
    h.ok("complete_setup", setup_body());
    let cfg = h.ok("config_overview", json!({}));
    let category = cfg["itemCategories"][0]["id"].clone();
    let location = cfg["locations"][0]["id"].clone();
    let collection = cfg["collections"][0]["id"].clone();

    // Create a record exactly as the form sends it.
    let record = json!({ "input": {
        "id": null, "title": "Balyakalasakhi", "subtitle": null, "edition": null, "publisher": "DC Books",
        "publicationPlace": "Kottayam", "publicationYear": 1944, "language": "Malayalam", "physicalDescription": null,
        "pagination": "120 p.", "series": null, "abstract": null, "notes": null, "isbn": "0-306-40615-2",
        "classificationSchemeId": 1, "classificationNumber": "894.8123", "keywords": "classic",
        "contributors": [{ "name": "Vaikom Muhammad Basheer", "role": "author" }],
        "subjects": ["Malayalam fiction"]
    }});
    let id = h.ok("catalogue_save_record", record.clone());
    let got = h.ok("catalogue_get_record", json!({ "id": id }));
    assert_eq!(got["title"], "Balyakalasakhi");
    assert_eq!(h.ok("catalogue_get_cover", json!({ "id": id })), Value::Null);
    assert_eq!(h.code("catalogue_set_cover", json!({ "id": id, "pngBase64": "eA==" })), "VALIDATION");
    let mut cover = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut cover, 1, 1);
        encoder.set_color(png::ColorType::Rgb);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&[10, 20, 30]).unwrap();
    }
    let cover = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, cover);
    h.ok("catalogue_set_cover", json!({ "id": id, "pngBase64": cover }));
    assert!(h.ok("catalogue_get_cover", json!({ "id": id })).as_str().unwrap().starts_with("iVBOR"));
    h.ok("catalogue_set_cover", json!({ "id": id, "pngBase64": null }));
    assert_eq!(h.ok("catalogue_get_cover", json!({ "id": id })), Value::Null);
    assert_eq!(got["isbn13"], "9780306406157");
    assert_eq!(got["contributors"][0]["role"], "author");
    assert_eq!(got["abstract"], Value::Null);
    assert_eq!(got["archived"], false);

    // Duplicate check as the form sends it.
    let hits = h.ok("catalogue_find_duplicates", json!({ "probe": { "id": null, "title": "x", "isbn": "9780306406157", "firstAuthor": null, "publicationYear": null } }));
    assert_eq!(hits[0]["reason"], "Same ISBN");

    // Add copies; accession numbers come from the configured format.
    h.ok("accession_format_set", json!({ "format": { "prefix": "LIB-", "padding": 5, "next": 100 } }));
    assert_eq!(h.ok("accession_format_get", json!({}))["next"], 100);
    let copies = h.ok("catalogue_add_copies", json!({ "recordId": id, "input": {
        "count": 2, "accessionNumbers": null, "barcodes": null, "itemCategoryId": category, "collectionId": collection,
        "locationId": location, "shelfId": null, "callNumber": "894.8123 BAS", "acquisitionDate": "2026-11-02",
        "acquisitionSource": "purchase", "priceMinor": 25000, "vendor": null, "invoiceRef": null, "notes": null
    }}));
    assert_eq!(copies[0]["accessionNo"], "LIB-00100");
    assert_eq!(copies[1]["barcode"], "LIB-00101");
    assert_eq!(copies[0]["status"], "available");
    let copy_id = copies[0]["id"].clone();

    // Search, register and status.
    let found = h.ok("catalogue_search", json!({ "query": { "text": "balya", "page": 1, "pageSize": 25 } }));
    assert_eq!((found["total"].as_i64(), found["rows"][0]["availableCopies"].as_i64()), (Some(1), Some(2)));
    let scan = h.ok("catalogue_search", json!({ "query": { "text": "LIB-00101", "page": 1, "pageSize": 25 } }));
    assert_eq!(scan["rows"][0]["id"], id, "a scanned barcode finds its record");
    h.ok("catalogue_set_copy_status", json!({ "id": copy_id, "status": "damaged", "note": "torn cover" }));
    let register = h.ok("catalogue_list_copies", json!({ "query": { "status": "damaged", "page": 1, "pageSize": 25 } }));
    assert_eq!(register["total"], 1);
    let exported = h.ok("catalogue_export_copies", json!({ "query": { "collectionId": collection, "page": 99, "pageSize": 1 } }));
    let mut csv = csv::Reader::from_reader(exported.as_str().unwrap().as_bytes());
    let rows = csv.records().collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!((&rows[0][0], &rows[0][4], &rows[0][9]), ("LIB-00100", "damaged", "25000"));
    let exported = h.ok("catalogue_export_copies", json!({ "query": { "status": "available", "text": "LIB-00101" } }));
    let mut csv = csv::Reader::from_reader(exported.as_str().unwrap().as_bytes());
    let rows = csv.records().collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(&rows[0][0], "LIB-00101");
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        conn.execute_batch("CREATE TEMP TRIGGER reject_archive_audit BEFORE INSERT ON audit_events
            WHEN NEW.action = 'catalogue.record_archived'
            BEGIN SELECT RAISE(ABORT, 'test audit failure'); END;").unwrap();
    }
    assert_eq!(h.code("catalogue_archive_record", json!({ "id": id, "archived": true })), "DATABASE");
    assert_eq!(h.ok("catalogue_get_record", json!({ "id": id }))["archived"], false);
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        conn.execute_batch("DROP TRIGGER reject_archive_audit").unwrap();
    }
    assert_eq!(h.code("catalogue_archive_records", json!({ "ids": [id, 999999] })), "NOT_FOUND");
    assert_eq!(h.ok("catalogue_get_record", json!({ "id": id }))["archived"], false);
    h.ok("catalogue_archive_records", json!({ "ids": [id] }));
    assert_eq!(h.ok("catalogue_get_record", json!({ "id": id }))["archived"], true);
    h.ok("catalogue_archive_record", json!({ "id": id, "archived": false }));
    assert_eq!(h.ok("catalogue_get_record", json!({ "id": id }))["archived"], false);
    let local_day: String = {
        let state = h.webview.state::<AppState>();
        let conn = state.conn().unwrap();
        conn.query_row("SELECT date('now','localtime')", [], |row| row.get(0)).unwrap()
    };
    let log = h.ok("audit_search", json!({ "query": { "from": local_day, "to": local_day,
        "actor": "head", "action": "record_restored", "page": 1 } }));
    assert_eq!(log["total"], 1);
    assert_eq!(log["rows"][0]["entityId"], id);
    assert_eq!(log["rows"][0]["actor"], "Head Librarian");
    assert_eq!(h.code("audit_search", json!({ "query": { "from": "2000-01-02", "to": "2000-01-01", "page": 1 } })), "VALIDATION");
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        let mut stmt = conn.prepare("SELECT action FROM audit_events WHERE entity_type = 'bibliographic_records'
            AND entity_id = ?1 AND action IN ('catalogue.record_archived', 'catalogue.record_restored') ORDER BY id").unwrap();
        let events = stmt.query_map([id.as_i64().unwrap()], |row| row.get::<_, String>(0))
            .unwrap().collect::<Result<Vec<_>, _>>().unwrap();
        assert_eq!(events, ["catalogue.record_archived", "catalogue.record_restored"]);
    }
    assert_eq!(h.code("catalogue_set_copy_status", json!({ "id": copy_id, "status": "on_loan", "note": null })), "VALIDATION");
    assert_eq!(h.code("catalogue_add_copies", json!({ "recordId": id, "input": {
        "count": 1, "accessionNumbers": ["LIB-00100"], "barcodes": null, "itemCategoryId": category, "collectionId": null,
        "locationId": null, "shelfId": null, "callNumber": null, "acquisitionDate": null, "acquisitionSource": "purchase",
        "priceMinor": null, "vendor": null, "invoiceRef": null, "notes": null }})), "CONFLICT");

    let policy = &cfg["policies"][0];
    let member = h.ok("members_save", json!({ "input": {
        "memberNumber": "MEM-101", "name": "Asha Nair", "categoryId": policy["memberCategoryId"]
    }}));
    let member_loans = json!({ "memberId": member, "page": 1 });
    assert_eq!(h.ok("members_list_loans", member_loans.clone())["total"], 0);
    assert_eq!(h.code("members_list_loans", json!({ "memberId": -1, "page": 1 })), "NOT_FOUND");

    // A clerk can read the catalogue and circulate copies but cannot edit the catalogue.
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        conn.execute("UPDATE circulation_policies SET max_renewals = 1, renewal_days = 7,
            fine_per_day_minor = 150, fine_grace_days = 1, fine_cap_minor = 250 WHERE id = ?1",
            [policy["id"].as_i64().unwrap()]).unwrap();
        conn.execute("UPDATE physical_items SET accession_no = 'ACC-101' WHERE barcode = 'LIB-00101'", []).unwrap();
        conn.execute("UPDATE physical_items SET accession_no = 'LIB-00101' WHERE id = ?1",
            [copy_id.as_i64().unwrap()]).unwrap();
        auth::create_user(&conn, "clerk", "Front Desk", "clerk passphrase!", "circulation").unwrap();
    }
    h.ok("logout", json!({}));
    h.ok("login", json!({ "username": "clerk", "password": "clerk passphrase!" }));
    assert_eq!(h.ok("catalogue_search", json!({ "query": { "text": "balya", "page": 1, "pageSize": 25 } }))["total"], 1);
    h.ok("catalogue_get_record", json!({ "id": id }));
    assert_eq!(h.code("catalogue_save_record", record), "PERMISSION_DENIED");
    assert_eq!(h.code("catalogue_archive_record", json!({ "id": id, "archived": true })), "PERMISSION_DENIED");
    assert_eq!(h.code("catalogue_set_cover", json!({ "id": id, "pngBase64": null })), "PERMISSION_DENIED");
    assert_eq!(h.code("catalogue_archive_records", json!({ "ids": [id] })), "PERMISSION_DENIED");
    assert_eq!(h.code("catalogue_set_copy_status", json!({ "id": copy_id, "status": "available", "note": null })), "PERMISSION_DENIED");
    assert_eq!(h.code("accession_format_set", json!({ "format": { "prefix": "", "padding": 6, "next": 1 } })), "PERMISSION_DENIED");
    assert_eq!(h.code("circulation_prepare_issue", json!({ "memberNumber": "MEM-101", "barcode": "LIB-00101" })), "CONFLICT");
    let lookup = json!({ "memberNumber": "MEM-101", "barcode": "ACC-101" });
    let preview = h.ok("circulation_prepare_issue", lookup.clone());
    assert_eq!(preview["memberName"], "Asha Nair");
    assert_eq!(preview["title"], "Balyakalasakhi");
    let request = json!({ "memberNumber": "MEM-101", "barcode": "ACC-101",
        "memberId": member, "itemId": preview["itemId"], "dueOn": preview["dueOn"] });
    h.ok("circulation_issue", request.clone());
    assert_eq!(h.code("circulation_issue", request), "CONFLICT");
    assert_eq!(h.ok("catalogue_list_copies", json!({ "query": { "status": "on_loan", "page": 1, "pageSize": 25 } }))["total"], 1);
    let barcode = json!({ "barcode": "ACC-101" });
    let initial_renewal = h.ok("circulation_prepare_renewal", barcode.clone());
    assert_eq!(initial_renewal["previousDueOn"], preview["dueOn"]);
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        conn.execute("INSERT INTO holidays (date, name) VALUES (?1, 'Holiday')",
            [initial_renewal["dueOn"].as_str().unwrap()]).unwrap();
    }
    assert_eq!(h.code("circulation_renew", json!({ "confirmation": initial_renewal })), "CONFLICT");
    let renewal = h.ok("circulation_prepare_renewal", barcode.clone());
    assert!(renewal["dueOn"].as_str().unwrap() > initial_renewal["dueOn"].as_str().unwrap());
    let confirm = json!({ "confirmation": renewal });
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        conn.execute_batch("CREATE TEMP TRIGGER reject_renewal_audit BEFORE INSERT ON audit_events
            WHEN NEW.action = 'circulation.renew'
            BEGIN SELECT RAISE(ABORT, 'test audit failure'); END;").unwrap();
    }
    assert_eq!(h.code("circulation_renew", confirm.clone()), "DATABASE");
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        let loan_id = renewal["loanId"].as_i64().unwrap();
        let (due, history): (String, i64) = conn.query_row(
            "SELECT due_on, (SELECT count(*) FROM loan_renewals WHERE loan_id = ?1) FROM loans WHERE id = ?1",
            [loan_id], |row| Ok((row.get(0)?, row.get(1)?)),
        ).unwrap();
        assert_eq!(due, renewal["previousDueOn"].as_str().unwrap());
        assert_eq!(history, 0);
        conn.execute_batch("DROP TRIGGER reject_renewal_audit").unwrap();
    }
    h.ok("circulation_renew", confirm.clone());
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        let (old, new): (String, String) = conn.query_row(
            "SELECT previous_due_on, due_on FROM loan_renewals WHERE loan_id = ?1",
            [renewal["loanId"].as_i64().unwrap()], |row| Ok((row.get(0)?, row.get(1)?)),
        ).unwrap();
        assert_eq!(old, renewal["previousDueOn"].as_str().unwrap());
        assert_eq!(new, renewal["dueOn"].as_str().unwrap());
    }
    assert_eq!(h.code("circulation_renew", confirm), "CONFLICT");
    assert_eq!(h.code("circulation_prepare_renewal", barcode.clone()), "CONFLICT");
    let loan = h.ok("circulation_prepare_return", barcode.clone());
    assert_eq!(loan["dueOn"], renewal["dueOn"]);
    let first_page = json!({ "page": 1 });
    assert_eq!(h.ok("circulation_list_overdue", first_page.clone())["total"], 0);
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        let id = loan["loanId"].as_i64().unwrap();
        conn.execute("UPDATE loans SET due_on = date('now','localtime') WHERE id = ?1", [id]).unwrap();
    }
    assert_eq!(h.ok("circulation_list_overdue", first_page.clone())["total"], 0);
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        conn.execute("UPDATE loans SET due_on = date('now','localtime','-3 days') WHERE id = ?1",
            [loan["loanId"].as_i64().unwrap()]).unwrap();
        conn.execute("UPDATE circulation_policies SET fine_per_day_minor = 0 WHERE id = ?1",
            [policy["id"].as_i64().unwrap()]).unwrap();
    }
    let overdue = h.ok("circulation_list_overdue", json!({ "page": 99 }));
    assert_eq!(overdue["page"], 1);
    assert_eq!(overdue["total"], 1);
    assert_eq!(overdue["rows"][0]["loanId"], loan["loanId"]);
    assert_eq!(overdue["rows"][0]["barcode"], "LIB-00101");
    assert_eq!(overdue["rows"][0]["memberNumber"], "MEM-101");
    assert_eq!(overdue["rows"][0]["daysOverdue"], 3);
    let dashboard = h.ok("dashboard_overview", json!({}));
    assert_eq!(dashboard["catalogue"], json!({ "titles": 1, "copies": 2, "available": 0 }));
    assert_eq!(dashboard["activeMembers"], 1);
    assert_eq!(dashboard["circulation"]["onLoan"], 1);
    assert_eq!(dashboard["circulation"]["overdue"], 1);
    let history = h.ok("members_list_loans", member_loans.clone());
    assert_eq!(history["total"], 1);
    assert_eq!(history["rows"][0]["id"], loan["loanId"]);
    assert_eq!(history["rows"][0]["title"], "Balyakalasakhi");
    assert_eq!(history["rows"][0]["overdue"], true);
    assert!(history["rows"][0]["returnedOn"].is_null());
    let return_preview = h.ok("circulation_prepare_return", barcode.clone());
    assert_eq!(return_preview["estimatedFine"], json!({ "amountMinor": 250, "currency": "INR" }));
    assert_eq!(h.code("circulation_return", json!({ "loanId": loan["loanId"], "condition": "damaged" })), "PERMISSION_DENIED");
    let fine = h.ok("circulation_return", json!({ "loanId": loan["loanId"] }));
    assert_eq!(fine, json!({ "amountMinor": 250, "currency": "INR" }));
    assert_eq!(h.ok("dashboard_overview", json!({}))["circulation"]["outstandingFines"], 1);
    let history = h.ok("members_list_loans", member_loans.clone());
    assert_eq!(history["total"], 1);
    assert_eq!(history["rows"][0]["overdue"], false);
    assert!(history["rows"][0]["returnedOn"].is_string());
    assert_eq!(h.ok("circulation_list_overdue", first_page.clone())["total"], 0);
    assert_eq!(h.code("circulation_return", json!({ "loanId": loan["loanId"] })), "CONFLICT");
    assert_eq!(h.code("circulation_prepare_renewal", barcode), "NOT_FOUND");
    assert_eq!(h.ok("catalogue_list_copies", json!({ "query": { "status": "available", "page": 1, "pageSize": 25 } }))["total"], 1);
    let outstanding = json!({ "page": 1, "includeSettled": false });
    let fines = h.ok("circulation_list_fines", outstanding.clone());
    assert_eq!(fines["total"], 1);
    let fine_id = &fines["rows"][0]["id"];
    let payment = json!({ "fineId": fine_id, "amountMinor": 125, "expectedBalance": 250,
        "entry": { "kind": "payment", "method": "cash" } });
    h.ok("circulation_settle_fine", payment.clone());
    assert_eq!(h.code("circulation_settle_fine", payment), "CONFLICT");
    assert_eq!(h.ok("circulation_list_fines", outstanding.clone())["rows"][0]["outstandingMinor"], 125);
    let waiver = json!({ "fineId": fine_id, "amountMinor": 125, "expectedBalance": 125,
        "entry": { "kind": "waiver", "reason": "Approved by librarian" } });
    assert_eq!(h.code("circulation_settle_fine", waiver.clone()), "PERMISSION_DENIED");
    h.ok("logout", json!({}));
    h.ok("login", json!({ "username": "admin", "password": "a long passphrase" }));
    h.ok("circulation_settle_fine", waiver);
    assert_eq!(h.ok("reports_fine_summary", json!({})), json!([{
        "currency": "INR", "assessed": "250", "paid": "125", "waived": "125", "outstanding": "0"
    }]));
    assert_eq!(h.ok("circulation_list_fines", outstanding)["total"], 0);
    let settled = h.ok("circulation_list_fines", json!({ "page": 1, "includeSettled": true }));
    assert_eq!(settled["rows"][0]["paidMinor"], 125);
    assert_eq!(settled["rows"][0]["waivedMinor"], 125);
    let history_request = json!({ "fineId": fine_id, "page": 1 });
    let entries = h.ok("circulation_fine_history", history_request.clone());
    assert_eq!(entries["total"], 2);
    assert_eq!(entries["rows"][0]["reason"], "Approved by librarian");
    assert_eq!(entries["rows"][1]["method"], "cash");
    h.ok("login", json!({ "username": "clerk", "password": "clerk passphrase!" }));
    let entries = h.ok("circulation_fine_history", history_request);
    assert!(entries["rows"][0]["reason"].is_null());
    // Signed-out callers get nothing.
    h.ok("logout", json!({}));
    assert_eq!(h.code("circulation_list_overdue", first_page), "AUTH_FAILED");
    assert_eq!(h.code("reports_fine_summary", json!({})), "AUTH_FAILED");
    assert_eq!(h.code("audit_search", json!({ "query": { "page": 1 } })), "AUTH_FAILED");
    assert_eq!(h.code("catalogue_export_copies", json!({ "query": {} })), "AUTH_FAILED");
    assert_eq!(h.code("reports_catalogue_growth", json!({ "from": "2026-01-01", "to": "2026-01-01" })), "AUTH_FAILED");
    assert_eq!(h.code("members_list_loans", member_loans), "AUTH_FAILED");
    assert_eq!(h.code("dashboard_overview", json!({})), "AUTH_FAILED");
    assert_eq!(h.code("circulation_prepare_issue", lookup), "AUTH_FAILED");
    assert_eq!(h.code("catalogue_search", json!({ "query": { "text": "balya", "page": 1, "pageSize": 25 } })), "AUTH_FAILED");
}

#[test]
fn import_preview_then_commit_through_the_real_commands() {
    use base64::Engine;
    let h = Harness::new();
    h.ok("complete_setup", setup_body());
    let cfg = h.ok("config_overview", json!({}));
    let category = cfg["itemCategories"][0]["id"].clone();

    let csv = "Title,Author,ISBN,Year,Barcode,Location\nBook One,\"Roy, A\",9780306406157,2001,C1,Main\nBook One,\"Roy, A\",9780306406157,2001,C2,Main\nBad ISBN Book,Someone,123,1999,C3,Annex\n,Nobody,,,C4,Main\n";
    let data = base64::engine::general_purpose::STANDARD.encode(csv);
    let preview = h.ok("import_preview", json!({ "request": { "fileName": "books.csv", "dataBase64": data, "format": "csv", "profile": "generic", "mapping": null } }));
    assert_eq!(preview["totals"]["records"], 3);
    assert_eq!(preview["totals"]["copies"], 4);
    assert_eq!(preview["totals"]["withErrors"], 1);
    assert_eq!(preview["mapping"]["title"], "Title");
    assert!(preview["newLookups"]["locations"].as_array().unwrap().iter().any(|l| l == "Annex"));
    let before = h.ok("catalogue_search", json!({ "query": { "text": null, "page": 1, "pageSize": 25 } }));
    assert_eq!(before["total"], 0, "previewing writes nothing");

    // Wrong token is refused; the preview stays usable.
    let opts = json!({ "duplicatePolicy": "skip", "createMissing": true, "defaultItemCategoryId": category, "barcodeAsAccession": true });
    assert_eq!(h.code("import_commit", json!({ "token": "nope", "options": opts })), "NOT_FOUND");
    let report = h.ok("import_commit", json!({ "token": preview["token"], "options": opts }));
    assert_eq!((report["createdRecords"].as_i64(), report["copiesAdded"].as_i64(), report["failedRecords"].as_i64()), (Some(2), Some(3), Some(1)));
    assert!(report["problems"][0]["message"].as_str().unwrap().contains("No title"));
    let after = h.ok("catalogue_search", json!({ "query": { "text": "C2", "page": 1, "pageSize": 25 } }));
    assert_eq!(after["rows"][0]["title"], "Book One");
    // The preview is used up.
    assert_eq!(h.code("import_commit", json!({ "token": preview["token"], "options": opts })), "NOT_FOUND");

    // Bad input is explained.
    assert_eq!(h.code("import_preview", json!({ "request": { "fileName": "x", "dataBase64": "!!!", "format": "csv", "profile": "generic", "mapping": null } })), "VALIDATION");
    assert_eq!(h.code("import_preview", json!({ "request": { "fileName": "x", "dataBase64": data, "format": "pdf", "profile": "generic", "mapping": null } })), "VALIDATION");

    // Only people who may change the catalogue can import.
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        auth::create_user(&conn, "viewer", "Visitor", "visitor passphrase", "viewer").unwrap();
    }
    h.ok("logout", json!({}));
    h.ok("login", json!({ "username": "viewer", "password": "visitor passphrase" }));
    assert_eq!(h.code("import_preview", json!({ "request": { "fileName": "books.csv", "dataBase64": data, "format": "csv", "profile": "generic", "mapping": null } })), "PERMISSION_DENIED");
    assert_eq!(h.code("import_cancel", json!({})), "PERMISSION_DENIED");
}

#[test]
fn reservation_queue_controls_issue_and_renewal_through_ipc() {
    let h = Harness::new();
    h.ok("complete_setup", setup_body());
    let config = h.ok("config_overview", json!({}));
    let policy = &config["policies"][0];
    let member_category = &policy["memberCategoryId"];
    for (number, name) in [("M-1", "First Member"), ("M-2", "Second Member"), ("M-3", "Third Member")] {
        h.ok("members_save", json!({ "input": { "name": name, "categoryId": member_category, "memberNumber": number } }));
    }
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        conn.execute("INSERT INTO bibliographic_records (title) VALUES ('Reservation test')", []).unwrap();
        conn.execute("INSERT INTO physical_items (record_id, accession_no, barcode, item_category_id)
            VALUES (?1, 'COPY-1', 'COPY-1', ?2)",
            [conn.last_insert_rowid(), policy["itemCategoryId"].as_i64().unwrap()]).unwrap();
        conn.execute("UPDATE circulation_policies SET max_renewals = 1, renewal_days = 7 WHERE id = ?1",
            [policy["id"].as_i64().unwrap()]).unwrap();
    }
    let lookup = |number| json!({ "memberNumber": number, "barcode": "COPY-1" });
    let issue = |number| {
        let preview = h.ok("circulation_prepare_issue", lookup(number));
        h.ok("circulation_issue", json!({
            "memberNumber": number, "barcode": "COPY-1", "memberId": preview["memberId"],
            "itemId": preview["itemId"], "dueOn": preview["dueOn"]
        }));
    };
    let list = || h.ok("circulation_list_reservations", json!({ "page": 1 }));
    issue("M-1");
    h.ok("circulation_place_reservation", lookup("M-2"));
    h.ok("circulation_place_reservation", lookup("M-3"));
    assert_eq!(h.code("circulation_place_reservation", lookup("M-2")), "CONFLICT");
    assert_eq!(list()["rows"][0]["status"], "pending");
    assert_eq!(h.code("circulation_prepare_renewal", json!({ "barcode": "COPY-1" })), "CONFLICT");
    let loan = h.ok("circulation_prepare_return", json!({ "barcode": "COPY-1" }));
    h.ok("circulation_return", json!({ "loanId": loan["loanId"] }));
    let queue = list();
    assert_eq!(queue["rows"][0]["memberNumber"], "M-2");
    assert_eq!(queue["rows"][0]["status"], "ready");
    assert_eq!(h.ok("dashboard_overview", json!({}))["circulation"]["readyHolds"], 1);
    assert!(queue["rows"][0]["expiresAt"].is_string());
    assert_eq!(queue["rows"][1]["status"], "pending");
    assert_eq!(h.code("circulation_prepare_issue", lookup("M-3")), "CONFLICT");
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        conn.execute("UPDATE reservations SET expires_at = '2000-01-01T00:00:00Z' WHERE id = ?1",
            [queue["rows"][0]["id"].as_i64().unwrap()]).unwrap();
    }
    let queue = list();
    assert_eq!(queue["total"], 1);
    assert_eq!(queue["rows"][0]["memberNumber"], "M-3");
    assert_eq!(queue["rows"][0]["status"], "ready");
    h.ok("circulation_place_reservation", lookup("M-1"));
    h.ok("circulation_cancel_reservation", json!({ "id": queue["rows"][0]["id"] }));
    let next = list();
    assert_eq!(next["rows"][0]["memberNumber"], "M-1");
    assert_eq!(next["rows"][0]["status"], "ready");
    h.ok("circulation_cancel_reservation", json!({ "id": next["rows"][0]["id"] }));
    assert_eq!(list()["total"], 0);
    h.ok("circulation_place_reservation", lookup("M-2"));
    issue("M-2");
    assert_eq!(list()["total"], 0);
    let loan = h.ok("circulation_prepare_return", json!({ "barcode": "COPY-1" }));
    h.ok("circulation_place_reservation", lookup("M-3"));
    h.ok("circulation_return", json!({ "loanId": loan["loanId"], "condition": "lost" }));
    assert_eq!(list()["rows"][0]["status"], "pending");
    let copies = h.ok("catalogue_list_copies", json!({ "query": { "status": "lost", "page": 1, "pageSize": 25 } }));
    assert_eq!(copies["total"], 1);
    let today: String = {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        conn.query_row("SELECT date('now','localtime')", [], |r| r.get(0)).unwrap()
    };
    let activity = h.ok("reports_circulation_activity", json!({ "from": today, "to": today }));
    assert_eq!(activity, json!([{
        "date": today, "issued": 2, "returned": 1, "lost": 1, "renewed": 0, "reservations": 5
    }]));
    let popular = h.ok("reports_popular_titles", json!({ "from": today, "to": today }));
    assert_eq!(popular.as_array().unwrap().len(), 1);
    assert_eq!(popular[0]["title"], "Reservation test");
    assert_eq!(popular[0]["issues"], 2);
    let stock = h.ok("reports_stock_by_location", json!({}));
    assert_eq!(stock[0]["location"], Value::Null);
    assert_eq!(stock[0]["lost"], 1);
    assert_eq!(stock[0]["total"], 1);
    assert_eq!(h.ok("reports_catalogue_growth", json!({ "from": today, "to": today })), json!([
        { "date": today, "titlesAdded": 1, "copiesAdded": 1 }
    ]));
    assert_eq!(h.ok("reports_catalogue_growth", json!({ "from": "2000-01-01", "to": "2000-01-02" })), json!([
        { "date": "2000-01-01", "titlesAdded": 0, "copiesAdded": 0 },
        { "date": "2000-01-02", "titlesAdded": 0, "copiesAdded": 0 }
    ]));
    assert_eq!(h.code("reports_catalogue_growth", json!({ "from": "2000-01-02", "to": "2000-01-01" })), "VALIDATION");
    assert_eq!(h.code("reports_circulation_activity", json!({ "from": "2025-01-01", "to": "2027-01-01" })), "VALIDATION");
    let state = h.webview.state::<AppState>();
    let conn = state.db.lock().unwrap();
    let mut stmt = conn.prepare("SELECT status FROM reservations ORDER BY id").unwrap();
    let outcomes = stmt.query_map([], |row| row.get::<_, String>(0)).unwrap().collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(outcomes, ["expired", "cancelled", "cancelled", "fulfilled", "pending"]);
}

#[test]
fn koha_patron_import_previews_and_saves_transactionally_through_ipc() {
    use base64::Engine;
    let h = Harness::new();
    h.ok("complete_setup", setup_body());
    let config = h.ok("config_overview", json!({}));
    let category = &config["memberCategories"][0]["id"];
    let csv = "cardnumber,firstname,surname,categorycode,email,phone,dateexpiry\r\n\
        0001,Asha,Nair,STU,asha@example.test,555-0101,2030-10-22\r\n\
        A-02,അനു,നായർ,STAFF,,,\r\n\
        0001,Duplicate,Member,STU,,,\r\n\
        0003,Invalid,Date,STU,,,2026-02-30\r\n";
    let data = base64::engine::general_purpose::STANDARD.encode(csv);
    let preview_request = json!({ "dataBase64": data });
    let preview = h.ok("members_preview_koha", preview_request.clone());
    assert_eq!(preview["ready"], 2);
    assert_eq!(preview["skipped"], 2);
    assert_eq!(preview["categories"], json!({ "STU": 1, "STAFF": 1 }));
    let list = json!({ "search": "", "page": 1, "archived": false });
    assert_eq!(h.ok("members_list", list.clone())["total"], 0);

    let invalid = json!({ "dataBase64": data, "expectedReady": 2, "mappings": { "STU": category, "STAFF": -1 } });
    assert_eq!(h.code("members_import_koha", invalid), "VALIDATION");
    assert_eq!(h.ok("members_list", list.clone())["total"], 0);
    let request = json!({ "dataBase64": data, "expectedReady": 2, "mappings": { "STU": category, "STAFF": category } });
    assert_eq!(h.ok("members_import_koha", request.clone()), 2);
    assert_eq!(h.code("members_import_koha", request), "CONFLICT");
    assert_eq!(h.ok("members_list", list)["total"], 2);
    let found = h.ok("members_list", json!({ "search": "0001", "page": 1, "archived": false }));
    assert_eq!(found["rows"][0]["memberNumber"], "0001");
    assert_eq!(found["rows"][0]["expiresOn"], "2030-10-22");
    assert_eq!(found["rows"][0]["email"], "asha@example.test");
    let found = h.ok("members_list", json!({ "search": "A-02", "page": 1, "archived": false }));
    assert_eq!(found["rows"][0]["name"], "അനു നായർ");
    h.ok("logout", json!({}));
    assert_eq!(h.code("members_preview_koha", preview_request), "AUTH_FAILED");
}

#[test]
fn app_info_works_before_sign_in_and_reports_the_current_schema() {
    let h = Harness::new();
    let info = h.ok("app_info", json!({}));
    assert_eq!(info["schemaVersion"], db::migrations::latest_version());
    assert!(info["databasePath"].as_str().unwrap().ends_with("pusthakam.db"));
}

#[test]
fn json_catalogue_import_previews_commits_and_rejects_malformed_input() {
    use base64::Engine;
    let h = Harness::new();
    h.ok("complete_setup", setup_body());
    let config = h.ok("config_overview", json!({}));
    let encode = |value: &str| base64::engine::general_purpose::STANDARD.encode(value);
    let request = |value: &str| json!({"request": {"fileName":"books.json", "format":"json", "profile":"generic", "mapping":null, "dataBase64":encode(value)}});
    let data = r#"[{"title":"Animal Farm","contributors":[{"name":"George Orwell","role":"author"}],"subjects":["Fiction"],"copies":[{"barcode":"000123","accession":"000045","priceMinor":12550,"location":"Reading room","status":"damaged"}]}]"#;
    let preview = h.ok("import_preview", request(data));
    assert_eq!(preview["totals"]["records"], 1);
    assert_eq!(preview["totals"]["copies"], 1);
    assert_eq!(h.ok("catalogue_search", json!({"query":{"page":1,"pageSize":25}}))["total"], 0);
    let options = json!({"duplicatePolicy":"skip","createMissing":true,"defaultItemCategoryId":config["itemCategories"][0]["id"],"barcodeAsAccession":false});
    let report = h.ok("import_commit", json!({"token":preview["token"],"options":options}));
    assert_eq!(report["createdRecords"], 1);
    assert_eq!(report["copiesAdded"], 1);
    let found = h.ok("catalogue_search", json!({"query":{"text":"000123","page":1,"pageSize":25}}));
    let record = h.ok("catalogue_get_record", json!({"id":found["rows"][0]["id"]}));
    assert_eq!(record["title"], "Animal Farm");
    assert_eq!(record["copies"][0]["accessionNo"], "000045");
    assert_eq!(record["copies"][0]["priceMinor"], 12550);
    assert_eq!(record["copies"][0]["status"], "damaged");
    assert_eq!(h.code("import_preview", request(r#"[{"title":"Good"},{"title":"Bad","unknown":true}]"#)), "VALIDATION");
    assert_eq!(h.ok("catalogue_search", json!({"query":{"page":1,"pageSize":25}}))["total"], 1);
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        auth::create_user(&conn, "jsonviewer", "Viewer", "viewer passphrase", "viewer").unwrap();
    }
    h.ok("logout", json!({}));
    assert_eq!(h.code("import_preview", request(data)), "AUTH_FAILED");
    h.ok("login", json!({"username":"jsonviewer","password":"viewer passphrase"}));
    assert_eq!(h.code("import_preview", request(data)), "PERMISSION_DENIED");
}
