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
fn loan_import_preview_is_admin_only_and_read_only() {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let h = Harness::new();
    let data = STANDARD.encode("source_id,member_number,barcode,issued_at,due_on,returned_at,source_staff\na,missing,0002,2026-11-01T10:00:00Z,2026-11-30,,Historical staff\n");
    h.ok("complete_setup", setup_body());
    let p = h.ok("circulation_preview_import", json!({"dataBase64":data}));
    assert_eq!(p["total"],1);
    assert_eq!(p["invalid"],1);
    assert_eq!(h.code("circulation_preview_import",json!({"dataBase64":"!"})),"VALIDATION");
    {
        let state=h.webview.state::<AppState>();
        let conn=state.conn().unwrap();
        assert_eq!(conn.query_row("SELECT count(*) FROM loans",[],|r|r.get::<_,i64>(0)).unwrap(),0);
        auth::create_user(&conn,"migrationviewer","Viewer","viewer passphrase","viewer").unwrap();
    }
    h.ok("logout",json!({}));
    h.ok("login",json!({"username":"migrationviewer","password":"viewer passphrase"}));
    assert_eq!(h.code("circulation_preview_import",json!({"dataBase64":data})),"PERMISSION_DENIED");
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

#[test]
fn marcxml_import_previews_commits_and_enforces_permissions() {
    use base64::Engine;
    let h = Harness::new();
    h.ok("complete_setup", setup_body());
    let config = h.ok("config_overview", json!({}));
    let data = include_str!("../../docs/examples/catalogue-import.xml");
    let request = |value: &str, profile: &str| json!({"request": {"fileName":"books.xml", "format":"marcxml", "profile":profile, "mapping":null, "dataBase64":base64::engine::general_purpose::STANDARD.encode(value)}});
    let generic = h.ok("import_preview", request(data, "generic"));
    assert_eq!(generic["totals"]["copies"], 0);
    h.ok("import_cancel", json!({}));
    let preview = h.ok("import_preview", request(data, "koha"));
    assert_eq!(preview["totals"]["records"], 1);
    assert_eq!(preview["totals"]["copies"], 1);
    assert_eq!(h.ok("catalogue_search", json!({"query":{"page":1,"pageSize":25}}))["total"], 0);
    let options = json!({"duplicatePolicy":"skip","createMissing":true,"defaultItemCategoryId":config["itemCategories"][0]["id"],"barcodeAsAccession":true});
    let report = h.ok("import_commit", json!({"token":preview["token"],"options":options}));
    assert_eq!(report["createdRecords"], 1);
    assert_eq!(report["copiesAdded"], 1);
    let found = h.ok("catalogue_search", json!({"query":{"text":"XML-000123","page":1,"pageSize":25}}));
    let record = h.ok("catalogue_get_record", json!({"id":found["rows"][0]["id"]}));
    assert_eq!(record["title"], "MARCXML verification book");
    assert_eq!(record["copies"][0]["accessionNo"], "XML-000123");
    assert_eq!(record["copies"][0]["priceMinor"], 12550);
    assert_eq!(record["copies"][0]["callNumber"], "020 XML");
    assert_eq!(h.code("import_preview", request(&format!("{data}<record>"), "koha")), "VALIDATION");
    assert_eq!(h.ok("catalogue_search", json!({"query":{"page":1,"pageSize":25}}))["total"], 1);
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        auth::create_user(&conn, "xmlviewer", "Viewer", "viewer passphrase", "viewer").unwrap();
    }
    h.ok("logout", json!({}));
    assert_eq!(h.code("import_preview", request(data, "koha")), "AUTH_FAILED");
    assert_eq!(h.code("import_commit", json!({"token":"stale", "options":options})), "AUTH_FAILED");
    h.ok("login", json!({"username":"xmlviewer","password":"viewer passphrase"}));
    assert_eq!(h.code("import_preview", request(data, "koha")), "PERMISSION_DENIED");
    assert_eq!(h.code("import_commit", json!({"token":"stale", "options":options})), "PERMISSION_DENIED");
}

#[test]
fn xlsx_import_maps_previews_commits_and_rejects_unsafe_workbooks() {
    use base64::Engine;
    let h = Harness::new();
    h.ok("complete_setup", setup_body());
    let config = h.ok("config_overview", json!({}));
    let data = include_bytes!("../../docs/examples/catalogue-import.xlsx");
    let request = |bytes: &[u8]| json!({"request": {"fileName":"books.xlsx", "format":"xlsx", "profile":"generic", "mapping":null, "dataBase64":base64::engine::general_purpose::STANDARD.encode(bytes)}});
    let preview = h.ok("import_preview", request(data));
    assert_eq!(preview["totals"]["records"], 1);
    assert_eq!(preview["totals"]["copies"], 2);
    assert_eq!(preview["mapping"]["barcode"], "Barcode");
    assert!(preview["mappingFields"].as_array().unwrap().iter().any(|v| v == "callNumber"));
    assert_eq!(h.ok("catalogue_search", json!({"query":{"page":1,"pageSize":25}}))["total"], 0);
    let options = json!({"duplicatePolicy":"skip","createMissing":true,"defaultItemCategoryId":config["itemCategories"][0]["id"],"barcodeAsAccession":true});
    let report = h.ok("import_commit", json!({"token":preview["token"],"options":options}));
    assert_eq!(report["createdRecords"], 1);
    assert_eq!(report["copiesAdded"], 2);
    let found = h.ok("catalogue_search", json!({"query":{"text":"000XLSX123","page":1,"pageSize":25}}));
    let record = h.ok("catalogue_get_record", json!({"id":found["rows"][0]["id"]}));
    assert_eq!(record["title"], "XLSX verification book");
    let copies = record["copies"].as_array().unwrap();
    let first = copies.iter().find(|c| c["barcode"] == "000XLSX123").unwrap();
    assert_eq!(first["accessionNo"], "000XLSX123");
    assert_eq!(first["priceMinor"], 12550);
    assert_eq!(first["callNumber"], "020 XLSX");
    assert!(copies.iter().any(|c| c["barcode"] == "000XLSX124" && c["status"] == "damaged"));
    for bad in [b"not a workbook".as_slice(), include_bytes!("../../docs/examples/catalogue-import-formula.xlsx").as_slice()] {
        assert_eq!(h.code("import_preview", request(bad)), "VALIDATION");
    }
    assert_eq!(h.ok("catalogue_search", json!({"query":{"page":1,"pageSize":25}}))["total"], 1);
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        auth::create_user(&conn, "xlsxviewer", "Viewer", "viewer passphrase", "viewer").unwrap();
    }
    h.ok("logout", json!({}));
    assert_eq!(h.code("import_preview", request(data)), "AUTH_FAILED");
    assert_eq!(h.code("import_commit", json!({"token":"stale", "options":options})), "AUTH_FAILED");
    h.ok("login", json!({"username":"xlsxviewer","password":"viewer passphrase"}));
    assert_eq!(h.code("import_preview", request(data)), "PERMISSION_DENIED");
    assert_eq!(h.code("import_commit", json!({"token":"stale", "options":options})), "PERMISSION_DENIED");
}

#[test]
fn selected_catalogue_export_supports_exchange_formats_and_read_permissions() {
    use base64::Engine;
    let h = Harness::new();
    assert_eq!(h.code("catalogue_export_records", json!({"ids":[1],"format":"json"})), "AUTH_FAILED");
    h.ok("complete_setup", setup_body());
    let config = h.ok("config_overview", json!({}));
    let preview = h.ok("import_preview", json!({"request":{"fileName":"sample.xlsx","format":"xlsx","profile":"generic","mapping":null,"dataBase64":base64::engine::general_purpose::STANDARD.encode(include_bytes!("../../docs/examples/catalogue-import.xlsx"))}}));
    h.ok("import_commit", json!({"token":preview["token"],"options":{"duplicatePolicy":"new","createMissing":true,"defaultItemCategoryId":config["itemCategories"][0]["id"],"barcodeAsAccession":true}}));
    let found = h.ok("catalogue_search", json!({"query":{"page":1,"pageSize":25}}));
    let id = &found["rows"][0]["id"];
    for format in ["json","csv","xlsx","marc","marcxml"] {
        let file = h.ok("catalogue_export_records", json!({"ids":[id],"format":format}));
        assert_eq!(file["records"],1);
        assert_eq!(file["copies"],2);
        let bytes = base64::engine::general_purpose::STANDARD.decode(file["dataBase64"].as_str().unwrap()).unwrap();
        assert!(!bytes.is_empty());
        let reimport = h.ok("import_preview", json!({"request":{"fileName":file["fileName"],"format":format,"profile":"koha","mapping":null,"dataBase64":file["dataBase64"]}}));
        assert_eq!(reimport["totals"]["records"],1);
        assert_eq!(reimport["totals"]["copies"],2);
        h.ok("import_cancel",json!({}));
    }
    assert_eq!(h.code("catalogue_export_records", json!({"ids":[],"format":"json"})), "VALIDATION");
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        auth::create_user(&conn, "exportviewer", "Viewer", "viewer passphrase", "viewer").unwrap();
    }
    h.ok("logout",json!({}));
    h.ok("login",json!({"username":"exportviewer","password":"viewer passphrase"}));
    assert_eq!(h.ok("catalogue_export_records",json!({"ids":[id],"format":"json"}))["records"],1);
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        conn.execute("DELETE FROM role_permissions WHERE role_id=(SELECT id FROM roles WHERE code='viewer') AND permission_id=(SELECT id FROM permissions WHERE code='catalogue.read')", []).unwrap();
    }
    assert_eq!(h.code("catalogue_export_records",json!({"ids":[id],"format":"json"})), "PERMISSION_DENIED");
}

#[test]
fn export_save_requires_catalogue_read_and_validates_before_showing_a_dialog() {
    let h = Harness::new();
    let body = json!({"fileName":"pusthakam-catalogue.json","dataBase64":"W10="});
    assert_eq!(h.code("catalogue_save_export",body.clone()),"AUTH_FAILED");
    h.ok("complete_setup",setup_body());
    assert_eq!(h.code("catalogue_save_export",json!({"fileName":"../library.db","dataBase64":"W10="})),"VALIDATION");
    assert_eq!(h.code("catalogue_save_export",json!({"fileName":"pusthakam-catalogue.json","dataBase64":"invalid"})),"VALIDATION");
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        conn.execute("DELETE FROM role_permissions WHERE role_id=(SELECT role_id FROM staff_users WHERE username='admin') AND permission_id=(SELECT id FROM permissions WHERE code='catalogue.read')",[]).unwrap();
    }
    assert_eq!(h.code("catalogue_save_export",body),"PERMISSION_DENIED");
}

#[test]
fn library_calendar_configures_closures_and_rechecks_issue_and_renewal_dates() {
    let h=Harness::new();
    assert_eq!(h.code("config_calendar",json!({})),"AUTH_FAILED");
    assert_eq!(h.code("config_save_closed_weekdays",json!({"days":[0]})),"AUTH_FAILED");
    assert_eq!(h.code("config_save_holiday",json!({"input":{"id":null,"date":"2027-01-01","name":"New year"}})),"AUTH_FAILED");
    assert_eq!(h.code("config_remove_holiday",json!({"id":1})),"AUTH_FAILED");
    h.ok("complete_setup",setup_body());
    let cfg=h.ok("config_overview",json!({}));
    let policy=&cfg["policies"][0];
    let member=h.ok("members_save",json!({"input":{"name":"Calendar reader","memberNumber":"CALENDAR-1","categoryId":policy["memberCategoryId"]}}));
    let record=h.ok("catalogue_save_record",json!({"input":{"title":"Calendar test book","contributors":[],"subjects":[]}}));
    let copies=h.ok("catalogue_add_copies",json!({"recordId":record,"input":{"count":1,"itemCategoryId":policy["itemCategoryId"],"acquisitionSource":"purchase"}}));
    let barcode=&copies[0]["barcode"];
    let initial=h.ok("circulation_prepare_issue",json!({"memberNumber":"CALENDAR-1","barcode":barcode}));
    let (weekday, expected): (i64,String) = {
        let state=h.webview.state::<AppState>(); let conn=state.db.lock().unwrap();
        conn.query_row("SELECT CAST(strftime('%w',?1) AS INTEGER),date(?1,'+2 days')",[initial["dueOn"].as_str().unwrap()],|r|Ok((r.get(0)?,r.get(1)?))).unwrap()
    };
    h.ok("config_save_closed_weekdays",json!({"days":[weekday]}));
    let tomorrow = {
        let state=h.webview.state::<AppState>(); let conn=state.db.lock().unwrap();
        conn.query_row("SELECT date(?1,'+1 day')",[initial["dueOn"].as_str().unwrap()],|r|r.get::<_,String>(0)).unwrap()
    };
    let holiday=h.ok("config_save_holiday",json!({"input":{"id":null,"date":tomorrow,"name":"Calendar closure"}}));
    assert_eq!(h.code("config_save_holiday",json!({"input":{"id":null,"date":tomorrow,"name":"Duplicate"}})),"CONFLICT");
    assert_eq!(h.code("config_save_closed_weekdays",json!({"days":[0,1,2,3,4,5,6]})),"VALIDATION");
    let old_request=json!({"memberNumber":"CALENDAR-1","barcode":barcode,"memberId":member,"itemId":initial["itemId"],"dueOn":initial["dueOn"]});
    assert_eq!(h.code("circulation_issue",old_request),"CONFLICT");
    let issue=h.ok("circulation_prepare_issue",json!({"memberNumber":"CALENDAR-1","barcode":barcode}));
    assert_eq!(issue["dueOn"],expected);
    h.ok("circulation_issue",json!({"memberNumber":"CALENDAR-1","barcode":barcode,"memberId":member,"itemId":issue["itemId"],"dueOn":issue["dueOn"]}));
    let renewal=h.ok("circulation_prepare_renewal",json!({"barcode":barcode}));
    let renewal_holiday=h.ok("config_save_holiday",json!({"input":{"id":null,"date":renewal["dueOn"],"name":"Renewal closure"}}));
    assert_eq!(h.code("circulation_renew",json!({"confirmation":renewal})),"CONFLICT");
    let changed=h.ok("circulation_prepare_renewal",json!({"barcode":barcode}));
    assert!(changed["dueOn"].as_str().unwrap()>renewal["dueOn"].as_str().unwrap());
    h.ok("circulation_renew",json!({"confirmation":changed}));
    h.ok("config_save_holiday",json!({"input":{"id":holiday,"date":"2028-02-29","name":"Edited closure"}}));
    h.ok("config_remove_holiday",json!({"id":renewal_holiday}));
    let cal=h.ok("config_calendar",json!({})); assert_eq!(cal["holidays"][0]["date"],"2028-02-29");
    assert_eq!(h.ok("circulation_prepare_return",json!({"barcode":barcode}))["dueOn"],changed["dueOn"],"calendar edits do not rewrite existing loan dates");
    {
        let state=h.webview.state::<AppState>(); let conn=state.db.lock().unwrap();
        auth::create_user(&conn,"calendarviewer","Viewer","viewer passphrase","viewer").unwrap();
    }
    h.ok("logout",json!({})); h.ok("login",json!({"username":"calendarviewer","password":"viewer passphrase"}));
    h.ok("config_calendar",json!({}));
    assert_eq!(h.code("config_save_closed_weekdays",json!({"days":[]})),"PERMISSION_DENIED");
    assert_eq!(h.code("config_save_holiday",json!({"input":{"id":null,"date":"2027-01-01","name":"Denied"}})),"PERMISSION_DENIED");
    assert_eq!(h.code("config_remove_holiday",json!({"id":holiday})),"PERMISSION_DENIED");
}

#[test]
fn member_fine_accounts_reconcile_payments_waivers_currencies_and_read_permissions() {
    let h=Harness::new();
    assert_eq!(h.code("members_fine_account",json!({"memberId":1,"page":1})),"AUTH_FAILED");
    assert_eq!(h.code("members_fine_history",json!({"memberId":1,"fineId":1,"page":1})),"AUTH_FAILED");
    h.ok("complete_setup",setup_body());
    let cfg=h.ok("config_overview",json!({}));
    let member=h.ok("members_save",json!({"input":{"name":"Account reader","memberNumber":"FINE-A","categoryId":cfg["memberCategories"][0]["id"]}}));
    let other=h.ok("members_save",json!({"input":{"name":"Other reader","memberNumber":"FINE-B","categoryId":cfg["memberCategories"][0]["id"]}}));
    assert_eq!(h.ok("members_fine_account",json!({"memberId":other,"page":1}))["total"],0);
    let record=h.ok("catalogue_save_record",json!({"input":{"title":"Fine account test","contributors":[],"subjects":[]}}));
    let copies=h.ok("catalogue_add_copies",json!({"recordId":record,"input":{"count":1,"itemCategoryId":cfg["itemCategories"][0]["id"],"acquisitionSource":"purchase"}}));
    let first_fine: i64 = {
        let state=h.webview.state::<AppState>(); let conn=state.db.lock().unwrap();
        let staff:i64=conn.query_row("SELECT id FROM staff_users WHERE username='admin'",[],|r|r.get(0)).unwrap();
        let mut first=0;
        for i in 0..27 {
            let mid=if i==26 {other.as_i64().unwrap()} else {member.as_i64().unwrap()};
            conn.execute("INSERT INTO loans (item_id,member_id,item_category_id,issued_by,due_on,returned_at,returned_by) VALUES (?1,?2,?3,?4,'2026-01-01','2026-01-03T00:00:00Z',?4)",rusqlite::params![copies[0]["id"].as_i64().unwrap(),mid,cfg["itemCategories"][0]["id"].as_i64().unwrap(),staff]).unwrap();
            let loan=conn.last_insert_rowid();
            conn.execute("INSERT INTO fines (loan_id,amount_minor,currency) VALUES (?1,?2,?3)",rusqlite::params![loan,if i==25 {500} else if i==26 {900} else {100},if i==25 {"USD"} else {"INR"}]).unwrap();
            if i==0 {first=conn.last_insert_rowid();}
        }
        first
    };
    let request=json!({"memberId":member,"page":1});
    let initial=h.ok("members_fine_account",request.clone());
    assert_eq!(initial["total"],26); assert_eq!(initial["rows"].as_array().unwrap().len(),25);
    assert_eq!(initial["balances"],json!([
        {"currency":"INR","assessedMinor":2500,"paidMinor":0,"waivedMinor":0,"outstandingMinor":2500},
        {"currency":"USD","assessedMinor":500,"paidMinor":0,"waivedMinor":0,"outstandingMinor":500}]));
    h.ok("circulation_settle_fine",json!({"fineId":first_fine,"amountMinor":40,"expectedBalance":100,"entry":{"kind":"payment","method":"cash"}}));
    assert_eq!(h.ok("members_fine_account",request.clone())["balances"][0]["outstandingMinor"],2460);
    h.ok("circulation_settle_fine",json!({"fineId":first_fine,"amountMinor":10,"expectedBalance":60,"entry":{"kind":"waiver","reason":"Approved adjustment"}}));
    let partial=h.ok("members_fine_account",json!({"memberId":member,"page":2}));
    assert_eq!(partial["rows"][0]["outstandingMinor"],50); assert_eq!(partial["balances"][0]["paidMinor"],40);
    assert_eq!(partial["balances"][0]["waivedMinor"],10); assert_eq!(partial["balances"][0]["outstandingMinor"],2450);
    let ledger=h.ok("circulation_list_fines",json!({"page":2,"includeSettled":true}));
    let matched=ledger["rows"].as_array().unwrap().iter().find(|f| f["id"]==first_fine).unwrap();
    assert_eq!(matched["outstandingMinor"],partial["rows"][0]["outstandingMinor"]);
    let history=h.ok("members_fine_history",json!({"memberId":member,"fineId":first_fine,"page":1}));
    assert_eq!(history["total"],2);
 assert_eq!(history["rows"][0]["reason"],"Approved adjustment");
    assert_eq!(h.code("members_fine_history",json!({"memberId":other,"fineId":first_fine,"page":1})),"NOT_FOUND");
    assert_eq!(h.code("members_fine_account",json!({"memberId":999999,"page":1})),"NOT_FOUND");
    h.ok("circulation_settle_fine",json!({"fineId":first_fine,"amountMinor":50,"expectedBalance":50,"entry":{"kind":"payment","method":"card"}}));
    let settled=h.ok("members_fine_account",json!({"memberId":member,"page":999}));
    assert_eq!(settled["page"],2); assert_eq!(settled["rows"][0]["outstandingMinor"],0);
    assert_eq!(settled["balances"][0]["paidMinor"],90); assert_eq!(settled["balances"][0]["outstandingMinor"],2400);
    h.ok("members_archive",json!({"id":member,"archived":true}));
    assert_eq!(h.ok("members_fine_account",request.clone())["total"],26);
    {
        let state=h.webview.state::<AppState>(); let conn=state.db.lock().unwrap();
        auth::create_user(&conn,"fineviewer","Viewer","viewer passphrase","viewer").unwrap();
    }
    h.ok("logout",json!({}));h.ok("login",json!({"username":"fineviewer","password":"viewer passphrase"}));
    assert_eq!(h.ok("members_fine_account",request.clone())["total"],26);
    let redacted=h.ok("members_fine_history",json!({"memberId":member,"fineId":first_fine,"page":1}));
    assert!(redacted["rows"].as_array().unwrap().iter().filter(|r|r["kind"]=="waiver").all(|r|r["reason"].is_null()));
    assert_eq!(h.code("circulation_settle_fine",json!({"fineId":first_fine,"amountMinor":1,"expectedBalance":0,"entry":{"kind":"payment","method":"cash"}})),"PERMISSION_DENIED");
    {
        let state=h.webview.state::<AppState>(); let conn=state.db.lock().unwrap();
        conn.execute("DELETE FROM role_permissions WHERE role_id=(SELECT id FROM roles WHERE code='viewer') AND permission_id=(SELECT id FROM permissions WHERE code='members.read')",[]).unwrap();
    }
    assert_eq!(h.code("members_fine_account",request),"PERMISSION_DENIED");
    assert_eq!(h.code("members_fine_history",json!({"memberId":member,"fineId":first_fine,"page":1})),"PERMISSION_DENIED");
}

#[test]
fn loan_import_commit_preserves_history_and_rolls_back_conflicts() {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let h=Harness::new();
    h.ok("complete_setup",setup_body());
    let cfg=h.ok("config_overview",json!({}));
    let policy=&cfg["policies"][0];
    let member=h.ok("members_save",json!({"input":{"name":"Migration reader","memberNumber":"0001","categoryId":policy["memberCategoryId"]}}));
    let record=h.ok("catalogue_save_record",json!({"input":{"title":"Migration fixture","contributors":[],"subjects":[]}}));
    let copies=h.ok("catalogue_add_copies",json!({"recordId":record,"input":{"count":1,"itemCategoryId":policy["itemCategoryId"],"acquisitionSource":"purchase"}}));
    let barcode=copies[0]["barcode"].as_str().unwrap();
    let file=|rows:String| STANDARD.encode(format!("source_id,member_number,barcode,issued_at,due_on,returned_at,source_staff\n{rows}"));
    let request=|data:&str, fp:&Value, terms:bool|json!({"dataBase64":data,"sourceSystem":"Old library","expectedFingerprint":fp,"useCurrentTerms":terms});
    let data=file(format!("old,0001,{barcode},2020-01-01T10:00:00Z,2020-01-20,2020-01-10T11:00:00Z,Former librarian\nactive,0001,{barcode},2026-11-01T10:00:00Z,2026-11-30,,Former librarian\n"));
    let p=h.ok("circulation_preview_import",json!({"dataBase64":data}));
    assert_eq!(h.code("circulation_commit_import",request(&data,&p["fingerprint"],false)),"VALIDATION");
    // Policy edits invalidate the review, even when row counts stay unchanged.
    {
        let state=h.webview.state::<AppState>();let conn=state.conn().unwrap();
        conn.execute("UPDATE circulation_policies SET fine_per_day_minor=fine_per_day_minor+1",[]).unwrap();
    }
    assert_eq!(h.code("circulation_commit_import",request(&data,&p["fingerprint"],true)),"VALIDATION");
    let p=h.ok("circulation_preview_import",json!({"dataBase64":data}));
    // Fail the audit after loan, provenance, terms and stock writes; all must roll back.
    {
        let state=h.webview.state::<AppState>();let conn=state.conn().unwrap();
        conn.execute_batch("CREATE TRIGGER fail_import_audit BEFORE INSERT ON audit_events WHEN NEW.action='circulation.import' BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();
    }
    assert!(h.call("circulation_commit_import",request(&data,&p["fingerprint"],true)).is_err());
    {
        let state=h.webview.state::<AppState>();let conn=state.conn().unwrap();
        for table in ["loans","loan_import_batches","loan_import_sources","loan_fine_terms"] {
            assert_eq!(conn.query_row(&format!("SELECT count(*) FROM {table}"),[],|r|r.get::<_,i64>(0)).unwrap(),0);
        }
        assert_eq!(conn.query_row("SELECT status FROM physical_items",[],|r|r.get::<_,String>(0)).unwrap(),"available");
        conn.execute_batch("DROP TRIGGER fail_import_audit").unwrap();
    }
    assert_eq!(h.ok("circulation_commit_import",request(&data,&p["fingerprint"],true)),2);
    let history=h.ok("members_list_loans",json!({"memberId":member,"page":1}));
    assert_eq!(history["total"],2);
    assert_eq!(history["rows"][0]["importSource"],"Old library");
    assert_eq!(history["rows"][0]["sourceStaff"],"Former librarian");
    {
        let state=h.webview.state::<AppState>();let conn=state.conn().unwrap();
        let saved:(String,String,String)=conn.query_row("SELECT l.issued_at,l.returned_at,s.source_staff FROM loans l JOIN loan_import_sources s ON s.loan_id=l.id WHERE s.source_id='old'",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
        assert_eq!(saved,("2020-01-01T10:00:00Z".into(),"2020-01-10T11:00:00Z".into(),"Former librarian".into()));
        assert_eq!(conn.query_row("SELECT status FROM physical_items",[],|r|r.get::<_,String>(0)).unwrap(),"on_loan");
        assert_eq!(conn.query_row("SELECT count(*) FROM fines",[],|r|r.get::<_,i64>(0)).unwrap(),0);
        assert_eq!(conn.query_row("SELECT count(*) FROM loan_fine_terms",[],|r|r.get::<_,i64>(0)).unwrap(),1);
        assert!(conn.execute("DELETE FROM loan_import_sources",[]).is_err());
    }
    assert_eq!(h.code("circulation_commit_import",request(&data,&p["fingerprint"],true)),"VALIDATION");
    // A new file with an existing source ID also fails without retaining its batch.
    let repeat=file(format!("old,0001,{barcode},2019-01-01T10:00:00Z,2019-01-20,2019-01-10T11:00:00Z,Former librarian\n"));
    let p=h.ok("circulation_preview_import",json!({"dataBase64":repeat}));
    assert_eq!(h.code("circulation_commit_import",request(&repeat,&p["fingerprint"],false)),"VALIDATION");
    // Overlapping closed history must fail, even with a fresh source ID.
    let overlap=file(format!("overlap,0001,{barcode},2020-01-02T10:00:00Z,2020-01-20,2020-01-03T11:00:00Z,Former librarian\n"));
    let p=h.ok("circulation_preview_import",json!({"dataBase64":overlap}));
    assert_eq!(h.code("circulation_commit_import",request(&overlap,&p["fingerprint"],false)),"VALIDATION");
    {
        let state=h.webview.state::<AppState>();let conn=state.conn().unwrap();
        assert_eq!(conn.query_row("SELECT count(*) FROM loan_import_batches",[],|r|r.get::<_,i64>(0)).unwrap(),1);
        auth::create_user(&conn,"loanviewer","Viewer","viewer passphrase","viewer").unwrap();
    }
    h.ok("login",json!({"username":"loanviewer","password":"viewer passphrase"}));
    assert_eq!(h.code("circulation_commit_import",request(&overlap,&p["fingerprint"],true)),"PERMISSION_DENIED");
}

#[test]
fn renewal_import_validates_chains_preserves_events_and_rolls_back() {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let h=Harness::new();h.ok("complete_setup",setup_body());
    let cfg=h.ok("config_overview",json!({}));let policy=&cfg["policies"][0];
    let member=h.ok("members_save",json!({"input":{"name":"Renewal migration reader","memberNumber":"0001","categoryId":policy["memberCategoryId"]}}));
    let record=h.ok("catalogue_save_record",json!({"input":{"title":"Renewal migration fixture","contributors":[],"subjects":[]}}));
    let copies=h.ok("catalogue_add_copies",json!({"recordId":record,"input":{"count":1,"itemCategoryId":policy["itemCategoryId"],"acquisitionSource":"purchase"}}));
    let barcode=copies[0]["barcode"].as_str().unwrap();
    let loans=STANDARD.encode(format!("source_id,member_number,barcode,issued_at,due_on,returned_at,source_staff\nloan-001,0001,{barcode},2020-01-01T10:00:00Z,2020-01-20,2020-01-25T11:00:00Z,Original staff\n"));
    let p=h.ok("circulation_preview_import",json!({"dataBase64":loans}));
    h.ok("circulation_commit_import",json!({"dataBase64":loans,"sourceSystem":"Legacy","expectedFingerprint":p["fingerprint"],"useCurrentTerms":false}));
    let csv=|rows:&str|STANDARD.encode(format!("source_id,loan_source_id,renewed_at,previous_due_on,due_on,source_staff\n{rows}"));
    let data=csv("r2,loan-001,2020-01-12T10:00:00Z,2020-01-15,2020-01-20,Old staff B\nr1,loan-001,2020-01-05T10:00:00Z,2020-01-10,2020-01-15,Old staff A\n");
    let preview=|data:&str,source:&str|json!({"dataBase64":data,"sourceSystem":source});
    let commit=|data:&str,p:&Value,source:&str|json!({"dataBase64":data,"sourceSystem":source,"expectedFingerprint":p["fingerprint"]});
    for invalid in [
        "bad,missing,2020-01-05T10:00:00Z,2020-01-10,2020-01-20,Staff\n",
        "bad,loan-001,2019-01-05T10:00:00Z,2020-01-10,2020-01-20,Staff\n",
        "bad,loan-001,2020-02-05T10:00:00Z,2020-01-10,2020-01-20,Staff\n",
        "bad,loan-001,2020-01-05T10:00:00Z,2020-01-10,2020-01-15,Staff\n",
        "a,loan-001,2020-01-05T10:00:00Z,2020-01-10,2020-01-15,Staff\nb,loan-001,2020-01-12T10:00:00Z,2020-01-16,2020-01-20,Staff\n",
        "a,loan-001,2020-01-05T10:00:00Z,2020-01-10,2020-01-15,Staff\nb,loan-001,2020-01-05T10:00:00Z,2020-01-15,2020-01-20,Staff\n",
        "a,loan-001,2020-01-05T10:00:00Z,2020-01-10,2020-01-15,Staff\na,loan-001,2020-01-12T10:00:00Z,2020-01-15,2020-01-20,Staff\n"
    ] {
        let bad=csv(invalid);let p=h.ok("circulation_preview_renewal_import",preview(&bad,"Legacy"));
        assert!(p["invalid"].as_u64().unwrap()>0);
        assert_eq!(h.code("circulation_commit_renewal_import",commit(&bad,&p,"Legacy")),"VALIDATION");
    }
    assert_eq!(h.code("circulation_preview_renewal_import",preview("!","Legacy")),"VALIDATION");
    let wrong=h.ok("circulation_preview_renewal_import",preview(&data,"Other source"));assert_eq!(wrong["invalid"],2);
    let p=h.ok("circulation_preview_renewal_import",preview(&data,"Legacy"));
    assert_eq!(p["invalid"],0);assert_eq!(p["total"],2);assert_eq!(p["loans"],1);
    assert_eq!(h.code("circulation_commit_renewal_import",commit(&data,&p,"Other source")),"VALIDATION");
    {
        let state=h.webview.state::<AppState>();let conn=state.conn().unwrap();
        conn.execute_batch("CREATE TRIGGER fail_renewal_import_audit BEFORE INSERT ON audit_events WHEN NEW.action='circulation.import_renewals' BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();
    }
    assert!(h.call("circulation_commit_renewal_import",commit(&data,&p,"Legacy")).is_err());
    {
        let state=h.webview.state::<AppState>();let conn=state.conn().unwrap();
        for table in ["loan_renewals","renewal_import_batches","renewal_import_sources"] {assert_eq!(conn.query_row(&format!("SELECT count(*) FROM {table}"),[],|r|r.get::<_,i64>(0)).unwrap(),0);}
        conn.execute_batch("DROP TRIGGER fail_renewal_import_audit").unwrap();
        conn.execute("UPDATE loans SET due_on='2020-01-21'",[]).unwrap();
    }
    assert_eq!(h.code("circulation_commit_renewal_import",commit(&data,&p,"Legacy")),"VALIDATION");
    {
        let state=h.webview.state::<AppState>();let conn=state.conn().unwrap();conn.execute("UPDATE loans SET due_on='2020-01-20'",[]).unwrap();
    }
    let p=h.ok("circulation_preview_renewal_import",preview(&data,"Legacy"));
    assert_eq!(h.ok("circulation_commit_renewal_import",commit(&data,&p,"Legacy")),2);
    let history=h.ok("members_list_loans",json!({"memberId":member,"page":1}));assert_eq!(history["rows"][0]["renewalCount"],2);assert_eq!(history["rows"][0]["dueOn"],"2020-01-20");
    {
        let state=h.webview.state::<AppState>();let conn=state.conn().unwrap();
        let events=conn.prepare("SELECT n.renewed_at,n.previous_due_on,n.due_on,s.source_staff FROM loan_renewals n JOIN renewal_import_sources s ON s.renewal_id=n.id ORDER BY n.id").unwrap().query_map([],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?))).unwrap().collect::<Result<Vec<_>,_>>().unwrap();
        assert_eq!(events[0],("2020-01-05T10:00:00Z".into(),"2020-01-10".into(),"2020-01-15".into(),"Old staff A".into()));
        assert_eq!(events[1].3,"Old staff B");
        assert_eq!(conn.query_row("SELECT status FROM physical_items",[],|r|r.get::<_,String>(0)).unwrap(),"available");
        assert_eq!(conn.query_row("SELECT count(*) FROM fines",[],|r|r.get::<_,i64>(0)).unwrap(),0);
        assert!(conn.execute("DELETE FROM renewal_import_sources",[]).is_err());
        auth::create_user(&conn,"renewalviewer","Viewer","viewer passphrase","viewer").unwrap();
    }
    let repeat=h.ok("circulation_preview_renewal_import",preview(&data,"Legacy"));assert!(repeat["invalid"].as_u64().unwrap()>0);
    assert_eq!(h.code("circulation_commit_renewal_import",commit(&data,&repeat,"Legacy")),"VALIDATION");
    h.ok("login",json!({"username":"renewalviewer","password":"viewer passphrase"}));
    assert_eq!(h.code("circulation_preview_renewal_import",preview(&data,"Legacy")),"PERMISSION_DENIED");
    assert_eq!(h.code("circulation_commit_renewal_import",commit(&data,&p,"Legacy")),"PERMISSION_DENIED");
}

#[test]
fn history_bundle_import_reconciles_all_five_parts_and_rolls_back() {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let h=Harness::new();h.ok("complete_setup",setup_body());
    let cfg=h.ok("config_overview",json!({}));let policy=&cfg["policies"][0];
    let member=h.ok("members_save",json!({"input":{"name":"Bundle reader","memberNumber":"0001","categoryId":policy["memberCategoryId"]}}));
    h.ok("members_save",json!({"input":{"name":"Second queue reader","memberNumber":"0002","categoryId":policy["memberCategoryId"]}}));
    let record=h.ok("catalogue_save_record",json!({"input":{"title":"Five part migration","contributors":[],"subjects":[]}}));
    let copies=h.ok("catalogue_add_copies",json!({"recordId":record,"input":{"count":1,"itemCategoryId":policy["itemCategoryId"],"acquisitionSource":"purchase"}}));
    let barcode=copies[0]["barcode"].as_str().unwrap();
    let loans=STANDARD.encode(format!("source_id,member_number,barcode,issued_at,due_on,returned_at,source_staff\na,0001,{barcode},2018-01-01T10:00:00Z,2018-01-10,2018-01-20T10:00:00Z,Original staff\nb,0001,{barcode},2019-01-01T10:00:00Z,2019-01-10,2019-01-20T10:00:00Z,Original staff\n"));
    let p=h.ok("circulation_preview_import",json!({"dataBase64":loans}));
    h.ok("circulation_commit_import",json!({"dataBase64":loans,"sourceSystem":"Legacy","expectedFingerprint":p["fingerprint"],"useCurrentTerms":false}));
    let (now,expiry):(String,String)={let state=h.webview.state::<AppState>();let conn=state.conn().unwrap();conn.query_row("SELECT strftime('%Y-%m-%dT%H:%M:%SZ','now'),strftime('%Y-%m-%dT%H:%M:%SZ','now','+7 days')",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap()};
    let bundle=json!({"version":1,"sourceSystem":"Legacy",
      "reservations":[
        {"sourceId":"q2","memberNumber":"0002","barcode":barcode,"placedAt":now,"status":"pending","expiresAt":null,"closedAt":null,"queuePosition":2,"sourceStaff":"Source queue staff"},
        {"sourceId":"oldq","memberNumber":"0001","barcode":barcode,"placedAt":"2017-01-01T10:00:00Z","status":"cancelled","expiresAt":null,"closedAt":"2017-01-02T10:00:00Z","queuePosition":null,"sourceStaff":"Source queue staff"},
        {"sourceId":"q1","memberNumber":"0001","barcode":barcode,"placedAt":now,"status":"ready","expiresAt":expiry,"closedAt":null,"queuePosition":1,"sourceStaff":"Source queue staff"}],
      "fines":[
        {"sourceId":"f1","loanSourceId":"a","amountMinor":1000,"currency":"INR","assessedAt":"2018-01-20T10:00:00Z","sourceStaff":"Assessor A"},
        {"sourceId":"f2","loanSourceId":"b","amountMinor":500,"currency":"USD","assessedAt":"2019-01-20T10:00:00Z","sourceStaff":"Assessor B"}],
      "payments":[
        {"sourceId":"p1","fineSourceId":"f1","amountMinor":300,"currency":"INR","recordedAt":"2018-01-21T10:00:00Z","method":"cash","sourceStaff":"Original cashier"},
        {"sourceId":"p2","fineSourceId":"f2","amountMinor":100,"currency":"USD","recordedAt":"2019-01-21T10:00:00Z","method":"bank_transfer","sourceStaff":"Original cashier"}],
      "waivers":[{"sourceId":"w1","fineSourceId":"f1","amountMinor":200,"currency":"INR","recordedAt":"2018-01-22T10:00:00Z","reason":"Historical approved adjustment","sourceStaff":"Original approver"}],
      "expected":{"counts":{"reservations":3,"fines":2,"payments":2,"waivers":1},"balances":[
        {"currency":"INR","assessedMinor":1000,"paidMinor":300,"waivedMinor":200,"outstandingMinor":500},
        {"currency":"USD","assessedMinor":500,"paidMinor":100,"waivedMinor":0,"outstandingMinor":400}]}});
    let encoded=|b:&Value|STANDARD.encode(serde_json::to_vec(b).unwrap());
    let preview=|b:&Value|h.ok("circulation_preview_history",json!({"dataBase64":encoded(b)}));
    let request=|b:&Value,p:&Value|json!({"dataBase64":encoded(b),"expectedFingerprint":p["fingerprint"]});
    for kind in 0..17 {
      let mut bad=bundle.clone();
      match kind {
        0=>bad["expected"]["balances"][0]["outstandingMinor"]=json!(501),
        1=>bad["payments"][0]["amountMinor"]=json!(900),
        2=>bad["payments"][0]["currency"]=json!("USD"),
        3=>bad["waivers"][0]["reason"]=json!(""),
        4=>bad["fines"][0]["loanSourceId"]=json!("missing"),
        5=>bad["reservations"][0]["queuePosition"]=json!(1),
        6=>bad["reservations"][2]["expiresAt"]=json!("2020-01-01T00:00:00Z"),
        7=>bad["reservations"][0]["memberNumber"]=json!("missing"),
        8=>bad["payments"][0]["recordedAt"]=json!("2010-01-01T00:00:00Z"),
        9=>bad["payments"][0]["sourceId"]=json!("w1"),
        10=>bad["fines"][1]["loanSourceId"]=json!("a"),
        11=>bad["payments"][0]["fineSourceId"]=json!("missing"),
        12=>bad["payments"][0]["method"]=json!("unknown"),
        13=>bad["fines"][0]["amountMinor"]=json!(9007199254740992_i64),
        14=>bad["waivers"][0]["method"]=json!("cash"),
        15=>bad["fines"][0]["amountMinor"]=json!(0),
        _=>bad["fines"][0]["currency"]=json!("inr")
      }
      let p=preview(&bad);assert_eq!(p["canCommit"],false,"bad variant {kind}");
      assert_eq!(h.code("circulation_commit_history",request(&bad,&p)),"VALIDATION");
    }
    let mut unknown=bundle.clone();unknown["fines"][0]["ignoredField"]=json!("not allowed");
    assert_eq!(h.code("circulation_preview_history",json!({"dataBase64":encoded(&unknown)})),"VALIDATION");
    let p=preview(&bundle);assert_eq!(p["canCommit"],true);assert_eq!(p["summary"],bundle["expected"]);
    // Changed member data invalidates the exact preview even when counts remain equal.
    {let state=h.webview.state::<AppState>();let conn=state.conn().unwrap();conn.execute("UPDATE members SET name='Changed' WHERE member_number='0001'",[]).unwrap();}
    assert_eq!(h.code("circulation_commit_history",request(&bundle,&p)),"VALIDATION");
    let p=preview(&bundle);
    // Failure after all five parts were inserted must roll every table and stock change back.
    {let state=h.webview.state::<AppState>();let conn=state.conn().unwrap();conn.execute_batch("CREATE TRIGGER fail_history_audit BEFORE INSERT ON audit_events WHEN NEW.action='circulation.import_history' BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();}
    assert!(h.call("circulation_commit_history",request(&bundle,&p)).is_err());
    {let state=h.webview.state::<AppState>();let conn=state.conn().unwrap();
      for table in ["reservations","fines","fine_settlements","history_import_sources","history_import_batches"]{assert_eq!(conn.query_row(&format!("SELECT count(*) FROM {table}"),[],|r|r.get::<_,i64>(0)).unwrap(),0);}
      assert_eq!(conn.query_row("SELECT status FROM physical_items",[],|r|r.get::<_,String>(0)).unwrap(),"available");conn.execute_batch("DROP TRIGGER fail_history_audit").unwrap();}
    // Re-read destination balances before committing: even an unexpected trigger change rolls back.
    {let state=h.webview.state::<AppState>();let conn=state.conn().unwrap();conn.execute_batch("CREATE TRIGGER alter_imported_fine AFTER INSERT ON fines BEGIN UPDATE fines SET amount_minor=amount_minor+1 WHERE id=NEW.id; END;").unwrap();}
    assert_eq!(h.code("circulation_commit_history",request(&bundle,&p)),"VALIDATION");
    {let state=h.webview.state::<AppState>();let conn=state.conn().unwrap();assert_eq!(conn.query_row("SELECT count(*) FROM fines",[],|r|r.get::<_,i64>(0)).unwrap(),0);conn.execute_batch("DROP TRIGGER alter_imported_fine").unwrap();}
    let saved=h.ok("circulation_commit_history",request(&bundle,&p));assert_eq!(saved,bundle["expected"]);
    assert_eq!(h.ok("circulation_history_imports",json!({}))[0]["summary"],saved);
    let account=h.ok("members_fine_account",json!({"memberId":member,"page":1}));assert_eq!(account["balances"][0]["outstandingMinor"],500);assert_eq!(account["balances"][1]["outstandingMinor"],400);
    let fine=account["rows"].as_array().unwrap().iter().find(|r|r["currency"]=="INR").unwrap()["id"].clone();
    let history=h.ok("members_fine_history",json!({"memberId":member,"fineId":fine,"page":1}));assert_eq!(history["rows"][0]["staffName"],"Original approver (imported source)");assert_eq!(history["rows"][0]["reason"],"Historical approved adjustment");
    let queue=h.ok("circulation_list_reservations",json!({"page":1}));assert_eq!(queue["total"],2);assert_eq!(queue["rows"][0]["memberNumber"],"0001");
    {let state=h.webview.state::<AppState>();let conn=state.conn().unwrap();
      assert_eq!(conn.query_row("SELECT status FROM physical_items",[],|r|r.get::<_,String>(0)).unwrap(),"reserved_hold");
      let summary:String=conn.query_row("SELECT summary_json FROM history_import_batches",[],|r|r.get(0)).unwrap();assert_eq!(serde_json::from_str::<Value>(&summary).unwrap(),saved);
      assert!(conn.execute("DELETE FROM history_import_sources",[]).is_err());
    }
    // Imported queue order is used by normal cancellation/promotion, not just stored metadata.
    h.ok("circulation_cancel_reservation",json!({"id":queue["rows"][0]["id"]}));
    let promoted=h.ok("circulation_list_reservations",json!({"page":1}));assert_eq!(promoted["rows"][0]["memberNumber"],"0002");assert_eq!(promoted["rows"][0]["status"],"ready");
    let repeat=preview(&bundle);assert_eq!(repeat["canCommit"],false);assert_eq!(h.code("circulation_commit_history",request(&bundle,&repeat)),"VALIDATION");
    // Follow-up payment-only batch must reconcile the full affected fine, including prior entries.
    let follow=json!({"version":1,"sourceSystem":"Legacy","reservations":[],"fines":[],"payments":[{"sourceId":"p3","fineSourceId":"f1","amountMinor":500,"currency":"INR","recordedAt":"2018-01-23T10:00:00Z","method":"card","sourceStaff":"Original cashier"}],"waivers":[],"expected":{"counts":{"reservations":0,"fines":0,"payments":1,"waivers":0},"balances":[{"currency":"INR","assessedMinor":1000,"paidMinor":800,"waivedMinor":200,"outstandingMinor":0}]}});
    let p=preview(&follow);assert_eq!(p["canCommit"],true);h.ok("circulation_commit_history",request(&follow,&p));
    // A wholly pending imported queue on an on-loan copy survives until the normal return promotes it.
    h.ok("circulation_cancel_reservation",json!({"id":promoted["rows"][0]["id"]}));
    let issue=h.ok("circulation_prepare_issue",json!({"memberNumber":"0001","barcode":barcode}));
    h.ok("circulation_issue",json!({"memberNumber":"0001","barcode":barcode,"memberId":issue["memberId"],"itemId":issue["itemId"],"dueOn":issue["dueOn"]}));
    let mut pending=json!({"version":1,"sourceSystem":"Legacy","reservations":[{"sourceId":"pending-new","memberNumber":"0002","barcode":barcode,"placedAt":now,"status":"pending","expiresAt":null,"closedAt":null,"queuePosition":1,"sourceStaff":"Queue staff"}],"fines":[],"payments":[],"waivers":[],"expected":{"counts":{"reservations":1,"fines":0,"payments":0,"waivers":0},"balances":[]}});
    let mut wrong=pending.clone();wrong["reservations"][0]["memberNumber"]=json!("0001");assert_eq!(preview(&wrong)["canCommit"],false);
    let p_pending=preview(&pending);assert_eq!(p_pending["canCommit"],true);h.ok("circulation_commit_history",request(&pending,&p_pending));
    assert_eq!(h.ok("circulation_list_reservations",json!({"page":1}))["rows"][0]["status"],"pending");
    let ret=h.ok("circulation_prepare_return",json!({"barcode":barcode}));h.ok("circulation_return",json!({"loanId":ret["loanId"],"condition":"available"}));
    assert_eq!(h.ok("circulation_list_reservations",json!({"page":1}))["rows"][0]["status"],"ready");
    pending["reservations"][0]["sourceId"]=json!("attempted-queue-merge");assert_eq!(preview(&pending)["canCommit"],false);
    {let state=h.webview.state::<AppState>();let conn=state.conn().unwrap();auth::create_user(&conn,"bundleviewer","Viewer","viewer passphrase","viewer").unwrap();}
    h.ok("login",json!({"username":"bundleviewer","password":"viewer passphrase"}));
    assert_eq!(h.code("circulation_preview_history",json!({"dataBase64":encoded(&bundle)})),"PERMISSION_DENIED");assert_eq!(h.code("circulation_commit_history",request(&bundle,&p)),"PERMISSION_DENIED");
    assert_eq!(h.code("circulation_history_imports",json!({})),"PERMISSION_DENIED");
    let redacted=h.ok("members_fine_history",json!({"memberId":member,"fineId":fine,"page":1}));assert!(redacted["rows"].as_array().unwrap().iter().all(|r|r["reason"].is_null()));
}
