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

#[path = "support/staff_client.rs"]
mod staff_client;

struct Harness {
    http: Option<staff_client::StaffClient>,
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
            .manage(AppState {
                paths,
                db: std::sync::Arc::new(Mutex::new(conn)),
                lan: std::sync::Arc::new(Mutex::new(Default::default())),
                session: std::sync::Arc::new(Mutex::new(None)),
                pending_import: std::sync::Arc::new(Mutex::new(None)),
                _log_guard: std::sync::Arc::new(guard),
            })
            .build(mock_context(noop_assets()))
            .unwrap();
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        let http = (std::env::var("PUSTHAKAM_TEST_TRANSPORT").as_deref() == Ok("http"))
            .then(|| staff_client::StaffClient::start(&app.state::<AppState>()));
        Harness {
            http,
            webview,
            _dir: dir,
            _app: app,
        }
    }

    fn call(&self, cmd: &str, body: Value) -> Res {
        if let Some(http) = &self.http {
            return http.call(cmd, body);
        }
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
        self.call(cmd, body)
            .unwrap_or_else(|e| panic!("{cmd} failed: {e}"))
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
    assert_eq!(p["total"], 1);
    assert_eq!(p["invalid"], 1);
    assert_eq!(
        h.code("circulation_preview_import", json!({"dataBase64":"!"})),
        "VALIDATION"
    );
    {
        let state = h.webview.state::<AppState>();
        let conn = state.conn().unwrap();
        assert_eq!(
            conn.query_row("SELECT count(*) FROM loans", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            0
        );
        auth::create_user(
            &conn,
            "migrationviewer",
            "Viewer",
            "viewer passphrase",
            "viewer",
        )
        .unwrap();
    }
    h.ok("logout", json!({}));
    h.ok(
        "login",
        json!({"username":"migrationviewer","password":"viewer passphrase"}),
    );
    assert_eq!(
        h.code("circulation_preview_import", json!({"dataBase64":data})),
        "PERMISSION_DENIED"
    );
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
    let first = h.ok(
        "members_list",
        json!({ "search": "Page Reader", "page": 1, "archived": false }),
    );
    let second = h.ok(
        "members_list",
        json!({ "search": "Page Reader", "page": 2, "archived": false }),
    );
    assert_eq!(first["total"], 26);
    assert_eq!(first["rows"].as_array().unwrap().len(), 25);
    assert_eq!(second["rows"].as_array().unwrap().len(), 1);
    assert_eq!(second["rows"][0]["memberNumber"], "PAGE-25");
    let lookup = h.ok(
        "members_list",
        json!({ "search": "PAGE-25", "page": 1, "archived": false }),
    );
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
    assert_eq!(
        h.code("stock_check_list", json!({ "page": 1 })),
        "AUTH_FAILED"
    );
    h.ok("complete_setup", setup_body());
    let config = h.ok("config_overview", json!({}));
    let location = &config["locations"][0]["id"];
    let record = h.ok(
        "catalogue_save_record",
        json!({ "input": {
            "title": "Shelf fixture", "contributors": [], "subjects": []
        }}),
    );
    let copies = h.ok(
        "catalogue_add_copies",
        json!({ "recordId": record, "input": {
            "count": 3, "itemCategoryId": config["itemCategories"][0]["id"], "locationId": location,
            "barcodes": ["SCAN-A", "SCAN-B", "SCAN-C"], "acquisitionSource": "purchase"
        }}),
    );
    h.ok(
        "catalogue_set_copy_status",
        json!({ "id": copies[2]["id"], "status": "damaged" }),
    );
    assert_eq!(
        h.code(
            "stock_check_start",
            json!({ "locationId": location, "shelfId": -1 })
        ),
        "VALIDATION"
    );
    let id = h.ok("stock_check_start", json!({ "locationId": location }));
    let scan = |code: &str| json!({ "id": id, "barcode": code });
    assert_eq!(h.ok("stock_check_scan", scan("scan-a")), "matched");
    assert_eq!(h.ok("stock_check_scan", scan("SCAN-A")), "duplicate");
    assert_eq!(h.ok("stock_check_scan", scan("SCAN-C")), "unexpected");
    assert_eq!(h.ok("stock_check_scan", scan("UNKNOWN-1")), "unexpected");
    let missing = h.ok(
        "stock_check_review",
        json!({ "id": id, "view": "missing", "page": 1 }),
    );
    assert_eq!(missing["check"]["expected"], 2);
    assert_eq!(missing["check"]["matched"], 1);
    assert_eq!(missing["check"]["unexpected"], 2);
    assert_eq!(missing["entries"]["rows"][0]["barcode"], "SCAN-B");
    let unexpected = h.ok(
        "stock_check_review",
        json!({ "id": id, "view": "unexpected", "page": 1 }),
    );
    assert_eq!(unexpected["entries"]["rows"][0]["title"], "Shelf fixture");
    assert!(unexpected["entries"]["rows"][1]["title"].is_null());
    {
        let paths = h.webview.state::<AppState>().paths.clone();
        let reopened = db::open(&paths.database).unwrap();
        let review = pusthakam_lib::services::inventory::review(
            &reopened,
            id.as_i64().unwrap(),
            pusthakam_lib::services::inventory::View::Missing,
            1,
        )
        .unwrap();
        assert_eq!(serde_json::to_value(review).unwrap(), missing);
    }
    h.ok("stock_check_close", json!({ "id": id }));
    assert_eq!(h.code("stock_check_scan", scan("SCAN-B")), "CONFLICT");
    assert!(h.ok("stock_check_list", json!({ "page": 1 }))["rows"][0]["closedAt"].is_string());
    assert_eq!(
        h.ok(
            "catalogue_list_copies",
            json!({ "query": { "status": "available" } })
        )["total"],
        2
    );
    let destination = h.ok(
        "config_save_named",
        json!({ "kind": "location", "input": { "name": "Destination reading room" } }),
    );
    assert_eq!(
        h.code(
            "stock_transfer_send",
            json!({ "barcode": "SCAN-A", "locationId": location })
        ),
        "VALIDATION"
    );
    let transfer = h.ok(
        "stock_transfer_send",
        json!({ "barcode": "SCAN-A", "locationId": destination }),
    );
    assert_eq!(
        h.ok("stock_transfer_list", json!({ "page": 1 }))["rows"][0]["status"],
        "in_transit"
    );
    {
        let paths = h.webview.state::<AppState>().paths.clone();
        let reopened = db::open(&paths.database).unwrap();
        let history = pusthakam_lib::services::transfers::list(&reopened, 1).unwrap();
        assert_eq!(
            serde_json::to_value(history).unwrap()["rows"][0]["id"],
            transfer
        );
    }
    assert_eq!(
        h.code(
            "catalogue_set_copy_status",
            json!({ "id": copies[0]["id"], "status": "available" })
        ),
        "CONFLICT"
    );
    assert_eq!(
        h.code(
            "catalogue_archive_record",
            json!({ "id": record, "archived": true })
        ),
        "CONFLICT"
    );
    h.ok(
        "stock_transfer_finish",
        json!({ "id": transfer, "receive": true }),
    );
    let moved = h.ok("catalogue_get_record", json!({ "id": record }));
    let copy = moved["copies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|copy| copy["barcode"] == "SCAN-A")
        .unwrap();
    assert_eq!(copy["locationId"], destination);
    assert_eq!(copy["status"], "available");
    assert_eq!(
        h.code(
            "stock_transfer_finish",
            json!({ "id": transfer, "receive": true })
        ),
        "CONFLICT"
    );
    let transfer_back = h.ok(
        "stock_transfer_send",
        json!({ "barcode": "SCAN-A", "locationId": location }),
    );
    h.ok(
        "stock_transfer_finish",
        json!({ "id": transfer_back, "receive": false }),
    );
    let transfers = h.ok("stock_transfer_list", json!({ "page": 1 }));
    assert_eq!(transfers["rows"][0]["status"], "cancelled");
    assert_eq!(
        transfers["rows"][1]["destination"],
        "Destination reading room"
    );
    {
        let state = h.webview.state::<AppState>();
        let conn = state.conn().unwrap();
        auth::create_user(&conn, "viewer", "Read Only", "viewer passphrase", "viewer").unwrap();
    }
    h.ok(
        "login",
        json!({ "username": "viewer", "password": "viewer passphrase" }),
    );
    assert_eq!(
        h.ok(
            "stock_check_review",
            json!({ "id": id, "view": "missing", "page": 1 })
        )["entries"]["total"],
        1
    );
    assert_eq!(
        h.code("stock_check_start", json!({ "locationId": location })),
        "PERMISSION_DENIED"
    );
    assert_eq!(
        h.code(
            "stock_transfer_send",
            json!({ "barcode": "SCAN-A", "locationId": location })
        ),
        "PERMISSION_DENIED"
    );
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
        conn.execute(
            "UPDATE staff_users SET locked_until = '2099-01-01T00:00:00Z' WHERE id = ?1",
            [id],
        )
        .unwrap();
        auth::load_session(&conn, id).unwrap()
    };
    assert_eq!(h.ok("staff_list", json!({}))["users"][1]["locked"], true);
    h.ok("staff_unlock", json!({ "id": id }));
    assert_eq!(h.ok("staff_list", json!({}))["users"][1]["locked"], false);
    let changes =
        |active| json!({ "displayName": "Desk Staff", "role": "circulation", "active": active });
    assert_eq!(
        h.code(
            "staff_update",
            json!({ "id": admin["userId"], "input": changes(false) })
        ),
        "VALIDATION"
    );
    h.ok("staff_update", json!({ "id": id, "input": changes(false) }));
    assert_eq!(
        h.code(
            "login",
            json!({ "username": "desk", "password": "first test password" })
        ),
        "AUTH_FAILED"
    );
    h.ok("staff_update", json!({ "id": id, "input": changes(true) }));
    h.ok(
        "staff_reset_password",
        json!({ "id": id, "password": "replacement test password" }),
    );
    {
        let state = h.webview.state::<AppState>();
        let conn = state.conn().unwrap();
        assert!(
            auth::refresh_session(&conn, &stale).is_err(),
            "an earlier session cannot outlive account changes"
        );
        let hash: String = conn
            .query_row(
                "SELECT password_hash FROM staff_users WHERE id = ?1",
                [id],
                |row| row.get(0),
            )
            .unwrap();
        assert!(hash.starts_with("$argon2id$"));
    }
    let desk = h.ok(
        "login",
        json!({ "username": "desk", "password": "replacement test password" }),
    );
    assert_eq!(desk["role"], "circulation");
    assert!(desk.get("revision").is_none());
    assert_eq!(h.code("staff_list", json!({})), "PERMISSION_DENIED");
    assert_eq!(
        h.code(
            "staff_change_password",
            json!({ "currentPassword": "wrong", "newPassword": "changed test password" })
        ),
        "AUTH_FAILED"
    );
    h.ok("staff_change_password", json!({ "currentPassword": "replacement test password", "newPassword": "changed test password" }));
    assert!(h.ok("current_session", json!({})).is_null());
    h.ok(
        "login",
        json!({ "username": "desk", "password": "changed test password" }),
    );
    h.ok(
        "login",
        json!({ "username": "admin", "password": "a long passphrase" }),
    );
    let discarded = h.ok(
        "staff_generate_recovery_code",
        json!({ "currentPassword": "a long passphrase" }),
    );
    let code = h.ok(
        "staff_generate_recovery_code",
        json!({ "currentPassword": "a long passphrase" }),
    );
    assert_ne!(discarded, code);
    assert_eq!(code.as_str().unwrap().len(), 64);
    {
        let state = h.webview.state::<AppState>();
        let conn = state.conn().unwrap();
        let stored: String = conn
            .query_row("SELECT code_hash FROM staff_recovery_codes", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert!(stored.starts_with("$argon2id$"));
        assert_ne!(stored, code.as_str().unwrap());
    }
    h.ok("logout", json!({}));
    assert_eq!(
        h.code(
            "staff_recover",
            json!({ "username": "admin", "code": discarded, "newPassword": "recovered password" })
        ),
        "AUTH_FAILED"
    );
    let recover = json!({ "username": "admin", "code": code, "newPassword": "recovered password" });
    h.ok("staff_recover", recover.clone());
    assert_eq!(h.code("staff_recover", recover), "AUTH_FAILED");
    h.ok(
        "login",
        json!({ "username": "admin", "password": "recovered password" }),
    );
    assert_eq!(
        h.ok("config_overview", json!({}))["library"]["name"],
        "Demo Library"
    );
    assert_eq!(
        h.ok(
            "audit_search",
            json!({ "query": { "action": "password_recovered", "page": 1 } })
        )["total"],
        1
    );
}

#[test]
fn manual_backups_are_permission_checked_complete_and_retained() {
    let h = Harness::new();
    let destination = tempfile::tempdir().unwrap();
    assert_eq!(h.code("backup_list", json!({})), "AUTH_FAILED");
    assert_eq!(h.code("backup_create", json!({})), "AUTH_FAILED");
    assert_eq!(
        h.code(
            "backup_copy_to_folder",
            json!({ "name": "pusthakam-none.sqlite3", "folder": destination.path() })
        ),
        "AUTH_FAILED"
    );
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
    tx.execute("UPDATE libraries SET name = 'Changed Library'", [])
        .unwrap();
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
    assert_eq!(
        h.code(
            "backup_copy_to_folder",
            json!({ "name": first["name"], "folder": folder })
        ),
        "VALIDATION"
    );
    let copied = h.ok(
        "backup_copy_to_folder",
        json!({ "name": first["name"], "folder": destination.path() }),
    );
    let copied_name = copied["name"].as_str().unwrap();
    assert_eq!(
        pusthakam_lib::services::backup::verify_saved(destination.path(), copied_name).unwrap(),
        db::migrations::latest_version()
    );
    let exported = rusqlite::Connection::open(destination.path().join(copied_name)).unwrap();
    assert_eq!(
        exported
            .query_row("SELECT name FROM libraries", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        "Demo Library"
    );
    let files = h.ok("backup_list", json!({}));
    assert_eq!(files["files"].as_array().unwrap().len(), 2);
    assert_eq!(
        h.ok("backup_verify", json!({ "name": first["name"] })),
        db::migrations::latest_version()
    );
    for (backup, expected_name) in [(first, "Demo Library"), (second, "Changed Library")] {
        let file = folder.join(backup["name"].as_str().unwrap());
        assert_eq!(
            std::fs::metadata(&file).unwrap().len(),
            backup["sizeBytes"].as_u64().unwrap()
        );
        let conn = rusqlite::Connection::open_with_flags(
            &file,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .unwrap();
        assert_eq!(
            conn.query_row("SELECT name FROM libraries", [], |row| row
                .get::<_, String>(0))
                .unwrap(),
            expected_name
        );
        assert_eq!(
            conn.query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))
                .unwrap(),
            "ok"
        );
        assert_eq!(
            conn.query_row("SELECT max(version) FROM schema_migrations", [], |row| row
                .get::<_, i64>(
                0
            ))
            .unwrap(),
            db::migrations::MIGRATIONS.last().unwrap().version
        );
    }
    assert_eq!(
        std::fs::read_dir(&folder).unwrap().count(),
        2,
        "only finalized database files remain"
    );
    assert_eq!(
        h.code(
            "backup_verify",
            json!({ "name": "../pusthakam-old.sqlite3" })
        ),
        "VALIDATION"
    );
    std::fs::write(
        folder.join("pusthakam-broken.sqlite3"),
        b"not a SQLite file",
    )
    .unwrap();
    assert_eq!(
        h.code(
            "backup_verify",
            json!({ "name": "pusthakam-broken.sqlite3" })
        ),
        "DATABASE"
    );
    let run_daily = || {
        let state = h.webview.state::<AppState>();
        state
            .with_conn(|conn| pusthakam_lib::services::backup::run_daily(conn, &state.paths))
            .unwrap()
    };
    let older = "pusthakam-20000101-daily.sqlite3";
    let queued = "pusthakam-20000102-daily.sqlite3";
    let manual = "pusthakam-20000101T000000Z-manual.sqlite3";
    let source = std::fs::read_dir(&folder)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| path.file_name().unwrap() != "pusthakam-broken.sqlite3")
        .unwrap();
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
    let daily_name = h.ok("backup_list", json!({}))["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|file| file["name"].as_str().unwrap().ends_with("-daily.sqlite3"))
        .unwrap()["name"]
        .as_str()
        .unwrap()
        .to_owned();
    let parked = folder.join("daily-check.sqlite3");
    std::fs::rename(folder.join(&daily_name), &parked).unwrap();
    assert!(!run_daily());
    std::fs::rename(parked, folder.join(daily_name)).unwrap();
    let list = h.ok("backup_list", json!({}));
    let daily = list["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|file| file["name"].as_str().unwrap().ends_with("-daily.sqlite3"))
        .unwrap();
    assert_eq!(
        h.ok("backup_verify", json!({ "name": daily["name"] })),
        db::migrations::latest_version()
    );
    h.ok(
        "login",
        json!({ "username": "viewer", "password": "visitor passphrase" }),
    );
    assert_eq!(h.code("backup_list", json!({})), "PERMISSION_DENIED");
    assert_eq!(h.code("backup_create", json!({})), "PERMISSION_DENIED");
    assert_eq!(
        h.code(
            "backup_copy_to_folder",
            json!({ "name": copied_name, "folder": destination.path() })
        ),
        "PERMISSION_DENIED"
    );
    assert_eq!(
        h.code("backup_set_daily", json!({ "enabled": false })),
        "PERMISSION_DENIED"
    );
}

#[test]
fn restore_waits_for_other_instances_and_preserves_the_previous_database() {
    use pusthakam_lib::services::backup;
    let h = Harness::new();
    assert_eq!(
        h.code(
            "backup_request_restore",
            json!({ "name": "pusthakam-test.sqlite3" })
        ),
        "AUTH_FAILED"
    );
    h.ok("complete_setup", setup_body());
    let config = h.ok("config_overview", json!({}));
    let policy = &config["policies"][0];
    let member = h.ok("members_save", json!({ "input": {
        "memberNumber": "RESTORE-MEMBER", "name": "Restore Member", "categoryId": policy["memberCategoryId"]
    }}));
    let record = h.ok(
        "catalogue_save_record",
        json!({ "input": {
            "title": "Restore fixture", "contributors": [], "subjects": []
        }}),
    );
    let copies = h.ok(
        "catalogue_add_copies",
        json!({ "recordId": record, "input": {
            "count": 1, "itemCategoryId": policy["itemCategoryId"], "acquisitionSource": "purchase"
        }}),
    );
    let code = &copies[0]["barcode"];
    let issue = h.ok(
        "circulation_prepare_issue",
        json!({ "memberNumber": "RESTORE-MEMBER", "barcode": code }),
    );
    h.ok(
        "circulation_issue",
        json!({
            "memberNumber": "RESTORE-MEMBER", "barcode": code, "memberId": member,
            "itemId": copies[0]["id"], "dueOn": issue["dueOn"]
        }),
    );
    let loan = h.ok("circulation_prepare_return", json!({ "barcode": code }));
    let saved = h.ok("backup_create", json!({}));
    let name = saved["name"].as_str().unwrap();
    let paths = h.webview.state::<AppState>().paths.clone();
    let (session_lock, exclusive) = backup::startup_lock(&paths).unwrap();
    assert!(!exclusive);
    h.ok("backup_request_restore", json!({ "name": name }));
    assert_eq!(h.ok("backup_list", json!({}))["pendingRestore"], name);
    assert_eq!(
        h.code("backup_request_restore", json!({ "name": name })),
        "CONFLICT"
    );
    h.ok("backup_cancel_restore", json!({}));
    assert!(h.ok("backup_list", json!({}))["pendingRestore"].is_null());

    h.ok("backup_request_restore", json!({ "name": name }));
    h.ok("circulation_return", json!({ "loanId": loan["loanId"] }));
    {
        let state = h.webview.state::<AppState>();
        let conn = state.conn().unwrap();
        conn.execute("UPDATE libraries SET name = 'After snapshot'", [])
            .unwrap();
        auth::create_user(
            &conn,
            "later",
            "Later Staff",
            "a later passphrase",
            "viewer",
        )
        .unwrap();
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
    assert_eq!(
        conn.query_row("SELECT name FROM libraries", [], |row| row
            .get::<_, String>(0))
            .unwrap(),
        "Demo Library"
    );
    assert_eq!(
        conn.query_row(
            "SELECT count(*) FROM staff_users WHERE username = 'later'",
            [],
            |row| row.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    assert_eq!(
        conn.query_row(
            "SELECT count(*) FROM bibliographic_records WHERE title = 'Restore fixture'",
            [],
            |row| row.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
    assert_eq!(
        conn.query_row(
            "SELECT count(*) FROM loans WHERE returned_at IS NULL",
            [],
            |row| row.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
    let listed = h.ok("backup_list", json!({}));
    assert!(listed["pendingRestore"].is_null());
    assert!(listed["lastRestore"]
        .as_str()
        .unwrap()
        .starts_with("Database restored"));
    let recovery = listed["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["name"] != name)
        .unwrap();
    let recovery_conn = rusqlite::Connection::open_with_flags(
        paths.backups.join(recovery["name"].as_str().unwrap()),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    assert_eq!(
        recovery_conn
            .query_row("SELECT name FROM libraries", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        "After snapshot"
    );
    assert_eq!(
        recovery_conn
            .query_row(
                "SELECT count(*) FROM staff_users WHERE username = 'later'",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        1
    );
    assert_eq!(
        recovery_conn
            .query_row(
                "SELECT count(*) FROM loans WHERE returned_at IS NOT NULL",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        1
    );

    let tampered = "pusthakam-tampered.sqlite3";
    std::fs::copy(paths.backups.join(name), paths.backups.join(tampered)).unwrap();
    h.ok("backup_request_restore", json!({ "name": tampered }));
    std::fs::write(
        paths.backups.join(tampered),
        b"corrupted after verification",
    )
    .unwrap();
    backup::apply_pending_restore(&mut conn, &paths).unwrap();
    assert_eq!(
        conn.query_row("SELECT name FROM libraries", [], |row| row
            .get::<_, String>(0))
            .unwrap(),
        "Demo Library"
    );
    assert!(h.ok("backup_list", json!({}))["lastRestore"]
        .as_str()
        .unwrap()
        .starts_with("Restore failed"));
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
    assert_eq!(
        h.code("members_save", json!({ "input": input.clone() })),
        "CONFLICT"
    );
    let row = &h.ok("members_list", list.clone())["rows"][0];
    assert_eq!(row["id"], id);
    assert_eq!(row["memberNumber"], "M%01");
    assert_eq!(row["expiresOn"].as_str().unwrap().len(), 10);
    assert!(h.ok("members_get_photo", json!({ "id": id })).is_null());
    assert_eq!(
        h.code(
            "members_set_photo",
            json!({ "id": id, "pngBase64": "eA==" })
        ),
        "VALIDATION"
    );
    let mut photo = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut photo, 1, 1);
        encoder.set_color(png::ColorType::Rgb);
        encoder
            .add_text_chunk("Comment".into(), "Camera metadata".into())
            .unwrap();
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&[20, 60, 90]).unwrap();
        writer.finish().unwrap();
    }
    let photo = base64::engine::general_purpose::STANDARD.encode(photo);
    h.ok("members_set_photo", json!({ "id": id, "pngBase64": photo }));
    let stored = h.ok("members_get_photo", json!({ "id": id }));
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(stored.as_str().unwrap())
        .unwrap();
    assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
    assert!(!bytes
        .windows(b"Camera metadata".len())
        .any(|part| part == b"Camera metadata"));

    input["id"] = id.clone();
    input["name"] = json!("Asha Ravi");
    h.ok("members_save", json!({ "input": input.clone() }));
    h.ok("members_archive", json!({ "id": id, "archived": true }));
    assert_eq!(h.ok("reports_member_summary", json!({}))[0]["archived"], 1);
    assert_eq!(h.ok("members_list", list.clone())["total"], 0);
    let archived = h.ok(
        "members_list",
        json!({ "search": "Ravi", "page": 1, "archived": true }),
    );
    assert_eq!(archived["rows"][0]["id"], id);
    h.ok("members_archive", json!({ "id": id, "archived": false }));
    assert_eq!(h.ok("members_list", list.clone())["total"], 1);
    let mut expired = input.clone();
    expired["id"] = Value::Null;
    expired["memberNumber"] = json!("M-EXPIRED");
    expired["expiresOn"] = json!("2000-01-01");
    let expired_id = h.ok("members_save", json!({ "input": expired }));
    assert_eq!(
        h.ok("reports_member_summary", json!({})),
        json!([{
            "categoryId": input["categoryId"], "category": config["memberCategories"][0]["name"],
            "archivedCategory": false, "current": 1, "expired": 1, "archived": 0, "total": 2
        }])
    );
    h.ok(
        "members_archive",
        json!({ "id": expired_id, "archived": true }),
    );

    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        auth::create_user(&conn, "viewer", "Visitor", "visitor passphrase", "viewer").unwrap();
    }
    h.ok(
        "login",
        json!({ "username": "viewer", "password": "visitor passphrase" }),
    );
    assert_eq!(h.ok("members_list", list)["total"], 1);
    let dashboard = h.ok("dashboard_overview", json!({}));
    assert_eq!(dashboard["activeMembers"], 1);
    assert_eq!(dashboard["catalogue"]["titles"], 0);
    assert!(dashboard["circulation"].is_null());
    assert_eq!(
        h.code("members_save", json!({ "input": input })),
        "PERMISSION_DENIED"
    );
    assert_eq!(
        h.code("members_archive", json!({ "id": id, "archived": true })),
        "PERMISSION_DENIED"
    );
    assert_eq!(h.ok("members_get_photo", json!({ "id": id })), stored);
    assert_eq!(
        h.code("members_set_photo", json!({ "id": id, "pngBase64": null })),
        "PERMISSION_DENIED"
    );
    h.ok(
        "login",
        json!({ "username": "admin", "password": "a long passphrase" }),
    );
    h.ok("members_set_photo", json!({ "id": id, "pngBase64": null }));
    assert!(h.ok("members_get_photo", json!({ "id": id })).is_null());
}

#[test]
fn setup_sign_in_and_configuration_work_through_the_real_commands() {
    let h = Harness::new();

    // Fresh install: not configured, and nothing else is usable.
    assert_eq!(h.ok("setup_status", json!({}))["configured"], false);
    assert_eq!(h.code("config_overview", json!({})), "AUTH_FAILED");
    assert_eq!(
        h.code("get_setting", json!({ "key": "ui.theme" })),
        "AUTH_FAILED"
    );
    assert_eq!(h.ok("current_session", json!({})), Value::Null);

    // The wizard's request signs the new administrator in.
    let session = h.ok("complete_setup", setup_body());
    assert_eq!(session["role"], "administrator");
    assert_eq!(session["displayName"], "Head Librarian");
    assert!(session["permissions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p == "admin.settings"));
    assert!(session.get("passwordHash").is_none() && session.get("password_hash").is_none());
    assert_eq!(h.ok("setup_status", json!({}))["configured"], true);
    assert_eq!(
        h.code("complete_setup", setup_body()),
        "CONFLICT",
        "setup must only run once"
    );

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
    h.ok(
        "config_save_shelf",
        json!({ "input": { "id": null, "locationId": loc, "code": "A1", "description": null } }),
    );
    h.ok("config_save_member_category", json!({ "input": { "id": null, "name": "Visitor", "description": null, "membershipDays": 30 } }));
    h.ok(
        "config_set_archived",
        json!({ "kind": "collection", "id": id, "archived": true }),
    );
    h.ok("config_update_library", json!({ "input": { "name": "Renamed Library", "profile": "college", "address": "Kochi", "contact": null, "currency": "INR", "locale": "en" } }));

    let cfg = h.ok("config_overview", json!({}));
    assert_eq!(cfg["library"]["name"], "Renamed Library");
    assert_eq!(
        cfg["collections"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == "Fiction")
            .unwrap()["archived"],
        true
    );

    // Loan rule create, update and delete.
    let member = cfg["memberCategories"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["name"] == "Visitor")
        .unwrap()["id"]
        .clone();
    let item = cfg["itemCategories"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["name"] == "Periodical")
        .unwrap()["id"]
        .clone();
    let rule = |id: Value, loan_days: i64| {
        json!({ "input": {
            "id": id, "memberCategoryId": member, "itemCategoryId": item, "loanDays": loan_days, "maxLoans": 2,
            "maxRenewals": 1, "renewalDays": 7, "finePerDayMinor": 250, "fineGraceDays": 0, "fineCapMinor": 5000,
            "reservationHoldDays": 3
        }})
    };
    let pid = h.ok("config_save_policy", rule(Value::Null, 7));
    assert_eq!(
        h.code("config_save_policy", rule(Value::Null, 7)),
        "CONFLICT"
    );
    h.ok("config_save_policy", rule(pid.clone(), 10));
    let cfg = h.ok("config_overview", json!({}));
    let saved = cfg["policies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == pid)
        .unwrap()
        .clone();
    assert_eq!(
        (saved["loanDays"].as_i64(), saved["fineCapMinor"].as_i64()),
        (Some(10), Some(5000))
    );
    h.ok("config_delete_policy", json!({ "id": pid }));

    // Errors reach the UI as { code, message } without internals.
    let err = h.call("config_save_named", json!({ "kind": "collection", "input": { "id": null, "name": "  ", "description": null } })).unwrap_err();
    assert_eq!(err["code"], "VALIDATION");
    assert!(err["message"].as_str().unwrap().contains("required"));

    // Settings (key/value) are admin-only to change.
    h.ok(
        "set_setting",
        json!({ "key": "ui.density", "value": "\"compact\"" }),
    );
    assert_eq!(
        h.ok("get_setting", json!({ "key": "ui.density" })),
        "\"compact\""
    );
}

#[test]
fn sign_out_and_in_and_permissions_are_enforced_in_rust() {
    let h = Harness::new();
    h.ok("complete_setup", setup_body());

    // Add a librarian directly (staff management UI comes later) and an inactive-free clerk.
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        auth::create_user(
            &conn,
            "rekha",
            "Rekha Nair",
            "another passphrase",
            "librarian",
        )
        .unwrap();
        auth::create_user(&conn, "viewer", "Visitor", "yet another phrase", "viewer").unwrap();
    }

    h.ok("logout", json!({}));
    assert_eq!(h.ok("current_session", json!({})), Value::Null);
    assert_eq!(h.code("config_overview", json!({})), "AUTH_FAILED");

    // Wrong credentials give the generic message and keep the user signed out.
    let err = h
        .call(
            "login",
            json!({ "username": "rekha", "password": "nope nope nope" }),
        )
        .unwrap_err();
    assert_eq!(err["code"], "AUTH_FAILED");
    assert_eq!(err["message"], "Incorrect username or password.");
    assert_eq!(h.ok("current_session", json!({})), Value::Null);

    // A librarian can read settings but not change them, and cannot change application settings.
    let s = h.ok(
        "login",
        json!({ "username": "REKHA", "password": "another passphrase" }),
    );
    assert_eq!(s["role"], "librarian");
    assert_eq!(h.ok("current_session", json!({}))["username"], "rekha");
    h.ok("config_overview", json!({}));
    assert_eq!(
        h.code("config_save_named", json!({ "kind": "collection", "input": { "id": null, "name": "Hack", "description": null } })),
        "PERMISSION_DENIED"
    );
    assert_eq!(
        h.code(
            "config_set_archived",
            json!({ "kind": "location", "id": 1, "archived": true })
        ),
        "PERMISSION_DENIED"
    );
    assert_eq!(
        h.code("config_delete_policy", json!({ "id": 1 })),
        "PERMISSION_DENIED"
    );
    assert_eq!(
        h.code("set_setting", json!({ "key": "a", "value": "b" })),
        "PERMISSION_DENIED"
    );
    assert_eq!(
        h.code("config_update_library", json!({ "input": { "name": "X", "profile": "school", "address": null, "contact": null, "currency": "INR", "locale": "en" } })),
        "PERMISSION_DENIED"
    );

    // Nothing changed behind the denied calls.
    h.ok("logout", json!({}));
    h.ok(
        "login",
        json!({ "username": "admin", "password": "a long passphrase" }),
    );
    let cfg = h.ok("config_overview", json!({}));
    assert_eq!(cfg["library"]["name"], "Demo Library");
    assert!(cfg["collections"]
        .as_array()
        .unwrap()
        .iter()
        .all(|c| c["name"] != "Hack"));
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
    assert_eq!(
        h.ok("catalogue_get_cover", json!({ "id": id })),
        Value::Null
    );
    assert_eq!(
        h.code(
            "catalogue_set_cover",
            json!({ "id": id, "pngBase64": "eA==" })
        ),
        "VALIDATION"
    );
    let mut cover = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut cover, 1, 1);
        encoder.set_color(png::ColorType::Rgb);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&[10, 20, 30]).unwrap();
    }
    let cover = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, cover);
    h.ok(
        "catalogue_set_cover",
        json!({ "id": id, "pngBase64": cover }),
    );
    assert!(h
        .ok("catalogue_get_cover", json!({ "id": id }))
        .as_str()
        .unwrap()
        .starts_with("iVBOR"));
    h.ok(
        "catalogue_set_cover",
        json!({ "id": id, "pngBase64": null }),
    );
    assert_eq!(
        h.ok("catalogue_get_cover", json!({ "id": id })),
        Value::Null
    );
    assert_eq!(got["isbn13"], "9780306406157");
    assert_eq!(got["contributors"][0]["role"], "author");
    assert_eq!(got["abstract"], Value::Null);
    assert_eq!(got["archived"], false);

    // Duplicate check as the form sends it.
    let hits = h.ok("catalogue_find_duplicates", json!({ "probe": { "id": null, "title": "x", "isbn": "9780306406157", "firstAuthor": null, "publicationYear": null } }));
    assert_eq!(hits[0]["reason"], "Same ISBN");

    // Add copies; accession numbers come from the configured format.
    h.ok(
        "accession_format_set",
        json!({ "format": { "prefix": "LIB-", "padding": 5, "next": 100 } }),
    );
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
    let found = h.ok(
        "catalogue_search",
        json!({ "query": { "text": "balya", "page": 1, "pageSize": 25 } }),
    );
    assert_eq!(
        (
            found["total"].as_i64(),
            found["rows"][0]["availableCopies"].as_i64()
        ),
        (Some(1), Some(2))
    );
    let scan = h.ok(
        "catalogue_search",
        json!({ "query": { "text": "LIB-00101", "page": 1, "pageSize": 25 } }),
    );
    assert_eq!(
        scan["rows"][0]["id"], id,
        "a scanned barcode finds its record"
    );
    h.ok(
        "catalogue_set_copy_status",
        json!({ "id": copy_id, "status": "damaged", "note": "torn cover" }),
    );
    let register = h.ok(
        "catalogue_list_copies",
        json!({ "query": { "status": "damaged", "page": 1, "pageSize": 25 } }),
    );
    assert_eq!(register["total"], 1);
    let exported = h.ok(
        "catalogue_export_copies",
        json!({ "query": { "collectionId": collection, "page": 99, "pageSize": 1 } }),
    );
    let mut csv = csv::Reader::from_reader(exported.as_str().unwrap().as_bytes());
    let rows = csv.records().collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(
        (&rows[0][0], &rows[0][4], &rows[0][9]),
        ("LIB-00100", "damaged", "25000")
    );
    let exported = h.ok(
        "catalogue_export_copies",
        json!({ "query": { "status": "available", "text": "LIB-00101" } }),
    );
    let mut csv = csv::Reader::from_reader(exported.as_str().unwrap().as_bytes());
    let rows = csv.records().collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(&rows[0][0], "LIB-00101");
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        conn.execute_batch(
            "CREATE TEMP TRIGGER reject_archive_audit BEFORE INSERT ON audit_events
            WHEN NEW.action = 'catalogue.record_archived'
            BEGIN SELECT RAISE(ABORT, 'test audit failure'); END;",
        )
        .unwrap();
    }
    assert_eq!(
        h.code(
            "catalogue_archive_record",
            json!({ "id": id, "archived": true })
        ),
        "DATABASE"
    );
    assert_eq!(
        h.ok("catalogue_get_record", json!({ "id": id }))["archived"],
        false
    );
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        conn.execute_batch("DROP TRIGGER reject_archive_audit")
            .unwrap();
    }
    assert_eq!(
        h.code("catalogue_archive_records", json!({ "ids": [id, 999999] })),
        "NOT_FOUND"
    );
    assert_eq!(
        h.ok("catalogue_get_record", json!({ "id": id }))["archived"],
        false
    );
    h.ok("catalogue_archive_records", json!({ "ids": [id] }));
    assert_eq!(
        h.ok("catalogue_get_record", json!({ "id": id }))["archived"],
        true
    );
    h.ok(
        "catalogue_archive_record",
        json!({ "id": id, "archived": false }),
    );
    assert_eq!(
        h.ok("catalogue_get_record", json!({ "id": id }))["archived"],
        false
    );
    let local_day: String = {
        let state = h.webview.state::<AppState>();
        let conn = state.conn().unwrap();
        conn.query_row("SELECT date('now','localtime')", [], |row| row.get(0))
            .unwrap()
    };
    let log = h.ok(
        "audit_search",
        json!({ "query": { "from": local_day, "to": local_day,
        "actor": "head", "action": "record_restored", "page": 1 } }),
    );
    assert_eq!(log["total"], 1);
    assert_eq!(log["rows"][0]["entityId"], id);
    assert_eq!(log["rows"][0]["actor"], "Head Librarian");
    assert_eq!(
        h.code(
            "audit_search",
            json!({ "query": { "from": "2000-01-02", "to": "2000-01-01", "page": 1 } })
        ),
        "VALIDATION"
    );
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        let mut stmt = conn.prepare("SELECT action FROM audit_events WHERE entity_type = 'bibliographic_records'
            AND entity_id = ?1 AND action IN ('catalogue.record_archived', 'catalogue.record_restored') ORDER BY id").unwrap();
        let events = stmt
            .query_map([id.as_i64().unwrap()], |row| row.get::<_, String>(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(
            events,
            ["catalogue.record_archived", "catalogue.record_restored"]
        );
    }
    assert_eq!(
        h.code(
            "catalogue_set_copy_status",
            json!({ "id": copy_id, "status": "on_loan", "note": null })
        ),
        "VALIDATION"
    );
    assert_eq!(h.code("catalogue_add_copies", json!({ "recordId": id, "input": {
        "count": 1, "accessionNumbers": ["LIB-00100"], "barcodes": null, "itemCategoryId": category, "collectionId": null,
        "locationId": null, "shelfId": null, "callNumber": null, "acquisitionDate": null, "acquisitionSource": "purchase",
        "priceMinor": null, "vendor": null, "invoiceRef": null, "notes": null }})), "CONFLICT");

    let policy = &cfg["policies"][0];
    let member = h.ok(
        "members_save",
        json!({ "input": {
            "memberNumber": "MEM-101", "name": "Asha Nair", "categoryId": policy["memberCategoryId"]
        }}),
    );
    let member_loans = json!({ "memberId": member, "page": 1 });
    assert_eq!(h.ok("members_list_loans", member_loans.clone())["total"], 0);
    assert_eq!(
        h.code("members_list_loans", json!({ "memberId": -1, "page": 1 })),
        "NOT_FOUND"
    );

    // A clerk can read the catalogue and circulate copies but cannot edit the catalogue.
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        conn.execute(
            "UPDATE circulation_policies SET max_renewals = 1, renewal_days = 7,
            fine_per_day_minor = 150, fine_grace_days = 1, fine_cap_minor = 250 WHERE id = ?1",
            [policy["id"].as_i64().unwrap()],
        )
        .unwrap();
        conn.execute(
            "UPDATE physical_items SET accession_no = 'ACC-101' WHERE barcode = 'LIB-00101'",
            [],
        )
        .unwrap();
        conn.execute(
            "UPDATE physical_items SET accession_no = 'LIB-00101' WHERE id = ?1",
            [copy_id.as_i64().unwrap()],
        )
        .unwrap();
        auth::create_user(
            &conn,
            "clerk",
            "Front Desk",
            "clerk passphrase!",
            "circulation",
        )
        .unwrap();
    }
    h.ok("logout", json!({}));
    h.ok(
        "login",
        json!({ "username": "clerk", "password": "clerk passphrase!" }),
    );
    assert_eq!(
        h.ok(
            "catalogue_search",
            json!({ "query": { "text": "balya", "page": 1, "pageSize": 25 } })
        )["total"],
        1
    );
    h.ok("catalogue_get_record", json!({ "id": id }));
    assert_eq!(h.code("catalogue_save_record", record), "PERMISSION_DENIED");
    assert_eq!(
        h.code(
            "catalogue_archive_record",
            json!({ "id": id, "archived": true })
        ),
        "PERMISSION_DENIED"
    );
    assert_eq!(
        h.code(
            "catalogue_set_cover",
            json!({ "id": id, "pngBase64": null })
        ),
        "PERMISSION_DENIED"
    );
    assert_eq!(
        h.code("catalogue_archive_records", json!({ "ids": [id] })),
        "PERMISSION_DENIED"
    );
    assert_eq!(
        h.code(
            "catalogue_set_copy_status",
            json!({ "id": copy_id, "status": "available", "note": null })
        ),
        "PERMISSION_DENIED"
    );
    assert_eq!(
        h.code(
            "accession_format_set",
            json!({ "format": { "prefix": "", "padding": 6, "next": 1 } })
        ),
        "PERMISSION_DENIED"
    );
    assert_eq!(
        h.code(
            "circulation_prepare_issue",
            json!({ "memberNumber": "MEM-101", "barcode": "LIB-00101" })
        ),
        "CONFLICT"
    );
    let lookup = json!({ "memberNumber": "MEM-101", "barcode": "ACC-101" });
    let preview = h.ok("circulation_prepare_issue", lookup.clone());
    assert_eq!(preview["memberName"], "Asha Nair");
    assert_eq!(preview["title"], "Balyakalasakhi");
    let request = json!({ "memberNumber": "MEM-101", "barcode": "ACC-101",
        "memberId": member, "itemId": preview["itemId"], "dueOn": preview["dueOn"] });
    h.ok("circulation_issue", request.clone());
    assert_eq!(h.code("circulation_issue", request), "CONFLICT");
    assert_eq!(
        h.ok(
            "catalogue_list_copies",
            json!({ "query": { "status": "on_loan", "page": 1, "pageSize": 25 } })
        )["total"],
        1
    );
    let barcode = json!({ "barcode": "ACC-101" });
    let initial_renewal = h.ok("circulation_prepare_renewal", barcode.clone());
    assert_eq!(initial_renewal["previousDueOn"], preview["dueOn"]);
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        conn.execute(
            "INSERT INTO holidays (date, name) VALUES (?1, 'Holiday')",
            [initial_renewal["dueOn"].as_str().unwrap()],
        )
        .unwrap();
    }
    assert_eq!(
        h.code(
            "circulation_renew",
            json!({ "confirmation": initial_renewal })
        ),
        "CONFLICT"
    );
    let renewal = h.ok("circulation_prepare_renewal", barcode.clone());
    assert!(renewal["dueOn"].as_str().unwrap() > initial_renewal["dueOn"].as_str().unwrap());
    let confirm = json!({ "confirmation": renewal });
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        conn.execute_batch(
            "CREATE TEMP TRIGGER reject_renewal_audit BEFORE INSERT ON audit_events
            WHEN NEW.action = 'circulation.renew'
            BEGIN SELECT RAISE(ABORT, 'test audit failure'); END;",
        )
        .unwrap();
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
        conn.execute_batch("DROP TRIGGER reject_renewal_audit")
            .unwrap();
    }
    h.ok("circulation_renew", confirm.clone());
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        let (old, new): (String, String) = conn
            .query_row(
                "SELECT previous_due_on, due_on FROM loan_renewals WHERE loan_id = ?1",
                [renewal["loanId"].as_i64().unwrap()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(old, renewal["previousDueOn"].as_str().unwrap());
        assert_eq!(new, renewal["dueOn"].as_str().unwrap());
    }
    assert_eq!(h.code("circulation_renew", confirm), "CONFLICT");
    assert_eq!(
        h.code("circulation_prepare_renewal", barcode.clone()),
        "CONFLICT"
    );
    let loan = h.ok("circulation_prepare_return", barcode.clone());
    assert_eq!(loan["dueOn"], renewal["dueOn"]);
    for search in ["", "balyak", "ASHA", "MEM-101", "LIB-00101"] {
        let open = h.ok(
            "circulation_list_open_loans",
            json!({"search":search,"page":99}),
        );
        assert_eq!(open["total"], 1);
        assert_eq!(open["page"], 1);
        assert_eq!(open["rows"][0]["loanId"], loan["loanId"]);
        assert_eq!(open["rows"][0]["barcode"], "LIB-00101");
        assert!(open["rows"][0]["accessionNo"].is_string());
    }
    assert_eq!(
        h.ok(
            "circulation_list_open_loans",
            json!({"search":"no-match","page":1})
        )["total"],
        0
    );

    let first_page = json!({ "page": 1 });
    assert_eq!(
        h.ok("circulation_list_overdue", first_page.clone())["total"],
        0
    );
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        let id = loan["loanId"].as_i64().unwrap();
        conn.execute(
            "UPDATE loans SET due_on = date('now','localtime') WHERE id = ?1",
            [id],
        )
        .unwrap();
    }
    assert_eq!(
        h.ok("circulation_list_overdue", first_page.clone())["total"],
        0
    );
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        conn.execute(
            "UPDATE loans SET due_on = date('now','localtime','-3 days') WHERE id = ?1",
            [loan["loanId"].as_i64().unwrap()],
        )
        .unwrap();
        conn.execute(
            "UPDATE circulation_policies SET fine_per_day_minor = 0 WHERE id = ?1",
            [policy["id"].as_i64().unwrap()],
        )
        .unwrap();
    }
    let overdue = h.ok("circulation_list_overdue", json!({ "page": 99 }));
    assert_eq!(overdue["page"], 1);
    assert_eq!(overdue["total"], 1);
    assert_eq!(overdue["rows"][0]["loanId"], loan["loanId"]);
    assert_eq!(overdue["rows"][0]["barcode"], "LIB-00101");
    assert_eq!(overdue["rows"][0]["memberNumber"], "MEM-101");
    assert_eq!(overdue["rows"][0]["daysOverdue"], 3);
    let dashboard = h.ok("dashboard_overview", json!({}));
    assert_eq!(
        dashboard["catalogue"],
        json!({ "titles": 1, "copies": 2, "available": 0 })
    );
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
    assert_eq!(
        return_preview["estimatedFine"],
        json!({ "amountMinor": 250, "currency": "INR" })
    );
    assert_eq!(
        h.code(
            "circulation_return",
            json!({ "loanId": loan["loanId"], "condition": "damaged" })
        ),
        "PERMISSION_DENIED"
    );
    let fine = h.ok("circulation_return", json!({ "loanId": loan["loanId"] }));
    assert_eq!(fine, json!({ "amountMinor": 250, "currency": "INR" }));
    assert_eq!(
        h.ok("circulation_list_open_loans", json!({"search":"","page":1}))["total"],
        0
    );
    assert_eq!(
        h.ok("dashboard_overview", json!({}))["circulation"]["outstandingFines"],
        1
    );
    let history = h.ok("members_list_loans", member_loans.clone());
    assert_eq!(history["total"], 1);
    assert_eq!(history["rows"][0]["overdue"], false);
    assert!(history["rows"][0]["returnedOn"].is_string());
    assert_eq!(
        h.ok("circulation_list_overdue", first_page.clone())["total"],
        0
    );
    assert_eq!(
        h.code("circulation_return", json!({ "loanId": loan["loanId"] })),
        "CONFLICT"
    );
    assert_eq!(h.code("circulation_prepare_renewal", barcode), "NOT_FOUND");
    assert_eq!(
        h.ok(
            "catalogue_list_copies",
            json!({ "query": { "status": "available", "page": 1, "pageSize": 25 } })
        )["total"],
        1
    );
    let outstanding = json!({ "page": 1, "includeSettled": false });
    let fines = h.ok("circulation_list_fines", outstanding.clone());
    assert_eq!(fines["total"], 1);
    let fine_id = &fines["rows"][0]["id"];
    let payment = json!({ "fineId": fine_id, "amountMinor": 125, "expectedBalance": 250,
        "entry": { "kind": "payment", "method": "cash" } });
    h.ok("circulation_settle_fine", payment.clone());
    assert_eq!(h.code("circulation_settle_fine", payment), "CONFLICT");
    assert_eq!(
        h.ok("circulation_list_fines", outstanding.clone())["rows"][0]["outstandingMinor"],
        125
    );
    let waiver = json!({ "fineId": fine_id, "amountMinor": 125, "expectedBalance": 125,
        "entry": { "kind": "waiver", "reason": "Approved by librarian" } });
    assert_eq!(
        h.code("circulation_settle_fine", waiver.clone()),
        "PERMISSION_DENIED"
    );
    h.ok("logout", json!({}));
    h.ok(
        "login",
        json!({ "username": "admin", "password": "a long passphrase" }),
    );
    h.ok("circulation_settle_fine", waiver);
    assert_eq!(
        h.ok("reports_fine_summary", json!({})),
        json!([{
            "currency": "INR", "assessed": "250", "paid": "125", "waived": "125", "outstanding": "0"
        }])
    );
    assert_eq!(h.ok("circulation_list_fines", outstanding)["total"], 0);
    let settled = h.ok(
        "circulation_list_fines",
        json!({ "page": 1, "includeSettled": true }),
    );
    assert_eq!(settled["rows"][0]["paidMinor"], 125);
    assert_eq!(settled["rows"][0]["waivedMinor"], 125);
    let history_request = json!({ "fineId": fine_id, "page": 1 });
    let entries = h.ok("circulation_fine_history", history_request.clone());
    assert_eq!(entries["total"], 2);
    assert_eq!(entries["rows"][0]["reason"], "Approved by librarian");
    assert_eq!(entries["rows"][1]["method"], "cash");
    h.ok(
        "login",
        json!({ "username": "clerk", "password": "clerk passphrase!" }),
    );
    let entries = h.ok("circulation_fine_history", history_request);
    assert!(entries["rows"][0]["reason"].is_null());
    // Signed-out callers get nothing.
    h.ok("logout", json!({}));
    assert_eq!(
        h.code("circulation_list_overdue", first_page),
        "AUTH_FAILED"
    );
    assert_eq!(h.code("reports_fine_summary", json!({})), "AUTH_FAILED");
    assert_eq!(
        h.code("audit_search", json!({ "query": { "page": 1 } })),
        "AUTH_FAILED"
    );
    assert_eq!(
        h.code("catalogue_export_copies", json!({ "query": {} })),
        "AUTH_FAILED"
    );
    assert_eq!(
        h.code(
            "reports_catalogue_growth",
            json!({ "from": "2026-01-01", "to": "2026-01-01" })
        ),
        "AUTH_FAILED"
    );
    assert_eq!(h.code("members_list_loans", member_loans), "AUTH_FAILED");
    assert_eq!(h.code("dashboard_overview", json!({})), "AUTH_FAILED");
    assert_eq!(h.code("circulation_prepare_issue", lookup), "AUTH_FAILED");
    assert_eq!(
        h.code(
            "catalogue_search",
            json!({ "query": { "text": "balya", "page": 1, "pageSize": 25 } })
        ),
        "AUTH_FAILED"
    );
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
    assert!(preview["newLookups"]["locations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|l| l == "Annex"));
    let before = h.ok(
        "catalogue_search",
        json!({ "query": { "text": null, "page": 1, "pageSize": 25 } }),
    );
    assert_eq!(before["total"], 0, "previewing writes nothing");

    // Wrong token is refused; the preview stays usable.
    let opts = json!({ "duplicatePolicy": "skip", "createMissing": true, "defaultItemCategoryId": category, "barcodeAsAccession": true });
    assert_eq!(
        h.code("import_commit", json!({ "token": "nope", "options": opts })),
        "NOT_FOUND"
    );
    let report = h.ok(
        "import_commit",
        json!({ "token": preview["token"], "options": opts }),
    );
    assert_eq!(
        (
            report["createdRecords"].as_i64(),
            report["copiesAdded"].as_i64(),
            report["failedRecords"].as_i64()
        ),
        (Some(2), Some(3), Some(1))
    );
    assert!(report["problems"][0]["message"]
        .as_str()
        .unwrap()
        .contains("No title"));
    let after = h.ok(
        "catalogue_search",
        json!({ "query": { "text": "C2", "page": 1, "pageSize": 25 } }),
    );
    assert_eq!(after["rows"][0]["title"], "Book One");
    // The preview is used up.
    assert_eq!(
        h.code(
            "import_commit",
            json!({ "token": preview["token"], "options": opts })
        ),
        "NOT_FOUND"
    );

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
    h.ok(
        "login",
        json!({ "username": "viewer", "password": "visitor passphrase" }),
    );
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
    for (number, name) in [
        ("M-1", "First Member"),
        ("M-2", "Second Member"),
        ("M-3", "Third Member"),
    ] {
        h.ok("members_save", json!({ "input": { "name": name, "categoryId": member_category, "memberNumber": number } }));
    }
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        conn.execute(
            "INSERT INTO bibliographic_records (title) VALUES ('Reservation test')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO physical_items (record_id, accession_no, barcode, item_category_id)
            VALUES (?1, 'COPY-1', 'COPY-1', ?2)",
            [
                conn.last_insert_rowid(),
                policy["itemCategoryId"].as_i64().unwrap(),
            ],
        )
        .unwrap();
        conn.execute(
            "UPDATE circulation_policies SET max_renewals = 1, renewal_days = 7 WHERE id = ?1",
            [policy["id"].as_i64().unwrap()],
        )
        .unwrap();
    }
    let lookup = |number| json!({ "memberNumber": number, "barcode": "COPY-1" });
    let issue = |number| {
        let preview = h.ok("circulation_prepare_issue", lookup(number));
        h.ok(
            "circulation_issue",
            json!({
                "memberNumber": number, "barcode": "COPY-1", "memberId": preview["memberId"],
                "itemId": preview["itemId"], "dueOn": preview["dueOn"]
            }),
        );
    };
    let list = || h.ok("circulation_list_reservations", json!({ "page": 1 }));
    issue("M-1");
    h.ok("circulation_place_reservation", lookup("M-2"));
    h.ok("circulation_place_reservation", lookup("M-3"));
    assert_eq!(
        h.code("circulation_place_reservation", lookup("M-2")),
        "CONFLICT"
    );
    assert_eq!(list()["rows"][0]["status"], "pending");
    assert_eq!(
        h.code(
            "circulation_prepare_renewal",
            json!({ "barcode": "COPY-1" })
        ),
        "CONFLICT"
    );
    let loan = h.ok("circulation_prepare_return", json!({ "barcode": "COPY-1" }));
    h.ok("circulation_return", json!({ "loanId": loan["loanId"] }));
    let queue = list();
    assert_eq!(queue["rows"][0]["memberNumber"], "M-2");
    assert_eq!(queue["rows"][0]["status"], "ready");
    assert_eq!(
        h.ok("dashboard_overview", json!({}))["circulation"]["readyHolds"],
        1
    );
    assert!(queue["rows"][0]["expiresAt"].is_string());
    assert_eq!(queue["rows"][1]["status"], "pending");
    assert_eq!(
        h.code("circulation_prepare_issue", lookup("M-3")),
        "CONFLICT"
    );
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        conn.execute(
            "UPDATE reservations SET expires_at = '2000-01-01T00:00:00Z' WHERE id = ?1",
            [queue["rows"][0]["id"].as_i64().unwrap()],
        )
        .unwrap();
    }
    let queue = list();
    assert_eq!(queue["total"], 1);
    assert_eq!(queue["rows"][0]["memberNumber"], "M-3");
    assert_eq!(queue["rows"][0]["status"], "ready");
    h.ok("circulation_place_reservation", lookup("M-1"));
    h.ok(
        "circulation_cancel_reservation",
        json!({ "id": queue["rows"][0]["id"] }),
    );
    let next = list();
    assert_eq!(next["rows"][0]["memberNumber"], "M-1");
    assert_eq!(next["rows"][0]["status"], "ready");
    h.ok(
        "circulation_cancel_reservation",
        json!({ "id": next["rows"][0]["id"] }),
    );
    assert_eq!(list()["total"], 0);
    h.ok("circulation_place_reservation", lookup("M-2"));
    issue("M-2");
    assert_eq!(list()["total"], 0);
    let loan = h.ok("circulation_prepare_return", json!({ "barcode": "COPY-1" }));
    h.ok("circulation_place_reservation", lookup("M-3"));
    h.ok(
        "circulation_return",
        json!({ "loanId": loan["loanId"], "condition": "lost" }),
    );
    assert_eq!(list()["rows"][0]["status"], "pending");
    let copies = h.ok(
        "catalogue_list_copies",
        json!({ "query": { "status": "lost", "page": 1, "pageSize": 25 } }),
    );
    assert_eq!(copies["total"], 1);
    let today: String = {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        conn.query_row("SELECT date('now','localtime')", [], |r| r.get(0))
            .unwrap()
    };
    let activity = h.ok(
        "reports_circulation_activity",
        json!({ "from": today, "to": today }),
    );
    assert_eq!(
        activity,
        json!([{
            "date": today, "issued": 2, "returned": 1, "lost": 1, "renewed": 0, "reservations": 5
        }])
    );
    let popular = h.ok(
        "reports_popular_titles",
        json!({ "from": today, "to": today }),
    );
    assert_eq!(popular.as_array().unwrap().len(), 1);
    assert_eq!(popular[0]["title"], "Reservation test");
    assert_eq!(popular[0]["issues"], 2);
    let stock = h.ok("reports_stock_by_location", json!({}));
    assert_eq!(stock[0]["location"], Value::Null);
    assert_eq!(stock[0]["lost"], 1);
    assert_eq!(stock[0]["total"], 1);
    assert_eq!(
        h.ok(
            "reports_catalogue_growth",
            json!({ "from": today, "to": today })
        ),
        json!([
            { "date": today, "titlesAdded": 1, "copiesAdded": 1 }
        ])
    );
    assert_eq!(
        h.ok(
            "reports_catalogue_growth",
            json!({ "from": "2000-01-01", "to": "2000-01-02" })
        ),
        json!([
            { "date": "2000-01-01", "titlesAdded": 0, "copiesAdded": 0 },
            { "date": "2000-01-02", "titlesAdded": 0, "copiesAdded": 0 }
        ])
    );
    assert_eq!(
        h.code(
            "reports_catalogue_growth",
            json!({ "from": "2000-01-02", "to": "2000-01-01" })
        ),
        "VALIDATION"
    );
    assert_eq!(
        h.code(
            "reports_circulation_activity",
            json!({ "from": "2025-01-01", "to": "2027-01-01" })
        ),
        "VALIDATION"
    );
    let loss_report = h.ok(
        "reports_loss_damage_list",
        json!({ "from": today, "to": today }),
    );
    assert_eq!(loss_report.as_array().unwrap().len(), 1);
    assert_eq!(loss_report[0]["returnCondition"], "lost");
    assert_eq!(loss_report[0]["title"], "Reservation test");
    let accession_report = h.ok(
        "reports_accession_list",
        json!({ "from": today, "to": today }),
    );
    assert_eq!(accession_report.as_array().unwrap().len(), 1);
    assert_eq!(accession_report[0]["barcode"], "COPY-1");
    assert_eq!(accession_report[0]["status"], "lost");
    let exp_report = h.ok(
        "reports_expenditure_summary",
        json!({ "from": today, "to": today }),
    );
    assert_eq!(exp_report.as_array().unwrap().len(), 1);
    assert_eq!(exp_report[0]["copies"], 1);
    let overdue_report = h.ok(
        "reports_overdues_list",
        json!({ "from": today, "to": today }),
    );
    assert_eq!(overdue_report.as_array().unwrap().len(), 0);
    assert_eq!(
        h.code(
            "reports_overdues_list",
            json!({ "from": "2000-01-02", "to": "2000-01-01" })
        ),
        "VALIDATION"
    );
    assert_eq!(
        h.code(
            "reports_accession_list",
            json!({ "from": "2000-01-02", "to": "2000-01-01" })
        ),
        "VALIDATION"
    );
    let xlsx_export = h.ok(
        "reports_export_file",
        json!({
            "format": "xlsx",
            "fileName": "test.xlsx",
            "title": "Circulation",
            "subtitle": "2026-11-19",
            "headers": ["Date", "Issues"],
            "rows": [["2026-11-19", "5"], ["Total", "5"]],
            "alignments": ["left", "right"]
        }),
    );
    assert_eq!(xlsx_export["fileName"], "test.xlsx");
    assert_eq!(
        xlsx_export["mimeType"],
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
    );
    assert!(!xlsx_export["dataBase64"].as_str().unwrap().is_empty());

    let pdf_export = h.ok(
        "reports_export_file",
        json!({
            "format": "pdf",
            "fileName": "test.pdf",
            "title": "Circulation",
            "subtitle": "2026-11-19",
            "headers": ["Date", "Issues"],
            "rows": [["2026-11-19", "5"], ["Total", "5"]],
            "alignments": ["left", "right"]
        }),
    );
    assert_eq!(pdf_export["fileName"], "test.pdf");
    assert_eq!(pdf_export["mimeType"], "application/pdf");
    assert!(!pdf_export["dataBase64"].as_str().unwrap().is_empty());

    assert_eq!(
        h.code(
            "reports_export_file",
            json!({
                "format": "unknown",
                "fileName": "test.bin",
                "title": "T",
                "subtitle": "",
                "headers": [],
                "rows": [],
                "alignments": []
            })
        ),
        "VALIDATION"
    );
    let state = h.webview.state::<AppState>();
    let conn = state.db.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT status FROM reservations ORDER BY id")
        .unwrap();
    let outcomes = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(
        outcomes,
        ["expired", "cancelled", "cancelled", "fulfilled", "pending"]
    );
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
    let found = h.ok(
        "members_list",
        json!({ "search": "0001", "page": 1, "archived": false }),
    );
    assert_eq!(found["rows"][0]["memberNumber"], "0001");
    assert_eq!(found["rows"][0]["expiresOn"], "2030-10-22");
    assert_eq!(found["rows"][0]["email"], "asha@example.test");
    let found = h.ok(
        "members_list",
        json!({ "search": "A-02", "page": 1, "archived": false }),
    );
    assert_eq!(found["rows"][0]["name"], "അനു നായർ");
    h.ok("logout", json!({}));
    assert_eq!(
        h.code("members_preview_koha", preview_request),
        "AUTH_FAILED"
    );
}

#[test]
fn app_info_works_before_sign_in_and_reports_the_current_schema() {
    let h = Harness::new();
    let info = h.ok("app_info", json!({}));
    assert_eq!(info["schemaVersion"], db::migrations::latest_version());
    assert!(info["databasePath"]
        .as_str()
        .unwrap()
        .ends_with("pusthakam.db"));
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
    assert_eq!(
        h.ok(
            "catalogue_search",
            json!({"query":{"page":1,"pageSize":25}})
        )["total"],
        0
    );
    let options = json!({"duplicatePolicy":"skip","createMissing":true,"defaultItemCategoryId":config["itemCategories"][0]["id"],"barcodeAsAccession":false});
    let report = h.ok(
        "import_commit",
        json!({"token":preview["token"],"options":options}),
    );
    assert_eq!(report["createdRecords"], 1);
    assert_eq!(report["copiesAdded"], 1);
    let found = h.ok(
        "catalogue_search",
        json!({"query":{"text":"000123","page":1,"pageSize":25}}),
    );
    let record = h.ok("catalogue_get_record", json!({"id":found["rows"][0]["id"]}));
    assert_eq!(record["title"], "Animal Farm");
    assert_eq!(record["copies"][0]["accessionNo"], "000045");
    assert_eq!(record["copies"][0]["priceMinor"], 12550);
    assert_eq!(record["copies"][0]["status"], "damaged");
    assert_eq!(
        h.code(
            "import_preview",
            request(r#"[{"title":"Good"},{"title":"Bad","unknown":true}]"#)
        ),
        "VALIDATION"
    );
    assert_eq!(
        h.ok(
            "catalogue_search",
            json!({"query":{"page":1,"pageSize":25}})
        )["total"],
        1
    );
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        auth::create_user(&conn, "jsonviewer", "Viewer", "viewer passphrase", "viewer").unwrap();
    }
    h.ok("logout", json!({}));
    assert_eq!(h.code("import_preview", request(data)), "AUTH_FAILED");
    h.ok(
        "login",
        json!({"username":"jsonviewer","password":"viewer passphrase"}),
    );
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
    assert_eq!(
        h.ok(
            "catalogue_search",
            json!({"query":{"page":1,"pageSize":25}})
        )["total"],
        0
    );
    let options = json!({"duplicatePolicy":"skip","createMissing":true,"defaultItemCategoryId":config["itemCategories"][0]["id"],"barcodeAsAccession":true});
    let report = h.ok(
        "import_commit",
        json!({"token":preview["token"],"options":options}),
    );
    assert_eq!(report["createdRecords"], 1);
    assert_eq!(report["copiesAdded"], 1);
    let found = h.ok(
        "catalogue_search",
        json!({"query":{"text":"XML-000123","page":1,"pageSize":25}}),
    );
    let record = h.ok("catalogue_get_record", json!({"id":found["rows"][0]["id"]}));
    assert_eq!(record["title"], "MARCXML verification book");
    assert_eq!(record["copies"][0]["accessionNo"], "XML-000123");
    assert_eq!(record["copies"][0]["priceMinor"], 12550);
    assert_eq!(record["copies"][0]["callNumber"], "020 XML");
    assert_eq!(
        h.code(
            "import_preview",
            request(&format!("{data}<record>"), "koha")
        ),
        "VALIDATION"
    );
    assert_eq!(
        h.ok(
            "catalogue_search",
            json!({"query":{"page":1,"pageSize":25}})
        )["total"],
        1
    );
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        auth::create_user(&conn, "xmlviewer", "Viewer", "viewer passphrase", "viewer").unwrap();
    }
    h.ok("logout", json!({}));
    assert_eq!(
        h.code("import_preview", request(data, "koha")),
        "AUTH_FAILED"
    );
    assert_eq!(
        h.code("import_commit", json!({"token":"stale", "options":options})),
        "AUTH_FAILED"
    );
    h.ok(
        "login",
        json!({"username":"xmlviewer","password":"viewer passphrase"}),
    );
    assert_eq!(
        h.code("import_preview", request(data, "koha")),
        "PERMISSION_DENIED"
    );
    assert_eq!(
        h.code("import_commit", json!({"token":"stale", "options":options})),
        "PERMISSION_DENIED"
    );
}

#[test]
fn koha_bibliographic_migration_reconciliation_and_benchmark() {
    use base64::Engine;
    use std::time::Instant;

    let h = Harness::new();
    h.ok("complete_setup", setup_body());
    let config = h.ok("config_overview", json!({}));

    // 1. Reconciliation Test with representative anonymised Koha export
    let data = include_str!("../../docs/examples/koha-anonymised-export.xml");
    let request = |value: &str| {
        json!({
            "request": {
                "fileName": "koha-export.xml",
                "format": "marcxml",
                "profile": "koha",
                "mapping": null,
                "dataBase64": base64::engine::general_purpose::STANDARD.encode(value)
            }
        })
    };

    let preview = h.ok("import_preview", request(data));
    assert_eq!(
        preview["totals"]["records"], 3,
        "Expected 3 bibliographic records in preview"
    );
    assert_eq!(
        preview["totals"]["copies"], 6,
        "Expected 6 copies across 3 records in preview"
    );

    let options = json!({
        "duplicatePolicy": "skip",
        "createMissing": true,
        "defaultItemCategoryId": config["itemCategories"][0]["id"],
        "barcodeAsAccession": false
    });

    let report = h.ok(
        "import_commit",
        json!({
            "token": preview["token"],
            "options": options
        }),
    );

    assert_eq!(report["createdRecords"], 3);
    assert_eq!(report["copiesAdded"], 6);
    assert_eq!(report["failedRecords"], 0);

    // Reconcile Record 1: Effective Java (Joshua Bloch)
    let r1_search = h.ok(
        "catalogue_search",
        json!({"query": {"text": "Effective Java", "page": 1, "pageSize": 10}}),
    );
    assert_eq!(r1_search["total"], 1);
    let r1 = h.ok(
        "catalogue_get_record",
        json!({"id": r1_search["rows"][0]["id"]}),
    );
    assert!(r1["title"].as_str().unwrap().contains("Effective Java"));
    assert_eq!(r1["contributors"][0]["name"], "Bloch, Joshua");
    assert_eq!(r1["copies"].as_array().unwrap().len(), 3);

    let c1a = r1["copies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["barcode"] == "KOHA-BC-001A")
        .expect("KOHA-BC-001A must exist");
    assert_eq!(c1a["accessionNo"], "ACC-K-001A");
    assert_eq!(c1a["status"], "available");
    assert_eq!(c1a["priceMinor"], 4500);
    assert_eq!(c1a["callNumber"], "005.133 BLO");

    let c1b = r1["copies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["barcode"] == "KOHA-BC-001B")
        .expect("KOHA-BC-001B must exist");
    assert_eq!(c1b["accessionNo"], "ACC-K-001B");
    assert_eq!(c1b["status"], "lost");
    assert!(c1b["notes"]
        .as_str()
        .unwrap()
        .contains("Reported lost by patron in 2022."));

    let c1c = r1["copies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["barcode"] == "KOHA-BC-001C")
        .expect("KOHA-BC-001C must exist");
    assert_eq!(c1c["accessionNo"], "ACC-K-001C");
    assert_eq!(c1c["status"], "damaged");
    assert!(c1c["notes"]
        .as_str()
        .unwrap()
        .contains("Water damage to back cover."));

    // Reconcile Record 2: International encyclopedia of information and library science
    let r2_search = h.ok(
        "catalogue_search",
        json!({"query": {"text": "encyclopedia", "page": 1, "pageSize": 10}}),
    );
    assert_eq!(r2_search["total"], 1);
    let r2 = h.ok(
        "catalogue_get_record",
        json!({"id": r2_search["rows"][0]["id"]}),
    );
    assert!(r2["title"]
        .as_str()
        .unwrap()
        .contains("International encyclopedia"));
    assert_eq!(r2["copies"].as_array().unwrap().len(), 2);

    let c2a = r2["copies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["barcode"] == "KOHA-BC-002A")
        .expect("KOHA-BC-002A must exist");
    assert_eq!(c2a["accessionNo"], "ACC-K-002A");
    assert!(c2a["notes"]
        .as_str()
        .unwrap()
        .contains("Marked not for loan in Koha."));

    let c2b = r2["copies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["barcode"] == "KOHA-BC-002B")
        .expect("KOHA-BC-002B must exist");
    assert_eq!(c2b["accessionNo"], "ACC-K-002B");
    assert_eq!(c2b["status"], "withdrawn");
    assert!(c2b["notes"]
        .as_str()
        .unwrap()
        .contains("Superseded edition withdrawn."));

    // Reconcile Record 3: Malayalam literature title
    let r3_search = h.ok(
        "catalogue_search",
        json!({"query": {"text": "ബാല്യകാലസഖി", "page": 1, "pageSize": 10}}),
    );
    assert_eq!(r3_search["total"], 1);
    let r3 = h.ok(
        "catalogue_get_record",
        json!({"id": r3_search["rows"][0]["id"]}),
    );
    assert!(r3["title"].as_str().unwrap().contains("ബാല്യകാലസഖി"));
    assert_eq!(r3["copies"].as_array().unwrap().len(), 1);
    let c3a = &r3["copies"][0];
    assert_eq!(c3a["barcode"], "KOHA-BC-003A");
    assert_eq!(c3a["accessionNo"], "ACC-K-003A");
    assert_eq!(c3a["status"], "available");
    assert_eq!(c3a["callNumber"], "894.8123 BAS");

    // 2. Large Export Migration Benchmark
    // Construct a synthetic 200-record / 400-copy Koha MARCXML export in memory
    let mut large_xml = String::with_capacity(512 * 1024);
    large_xml.push_str(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<collection xmlns=\"http://www.loc.gov/MARC21/slim\">\n",
    );
    for i in 1..=200 {
        use std::fmt::Write;
        write!(
            large_xml,
            r#"<record>
  <leader>01142cam a2200301 a 4500</leader>
  <controlfield tag="001">BENCH-{i:04}</controlfield>
  <datafield tag="020" ind1=" " ind2=" "><subfield code="a">97800000{i:05}</subfield></datafield>
  <datafield tag="082" ind1="0" ind2="0"><subfield code="a">004.{i:03}</subfield></datafield>
  <datafield tag="100" ind1="1" ind2=" "><subfield code="a">Author, Benchmark {i}</subfield></datafield>
  <datafield tag="245" ind1="1" ind2="0"><subfield code="a">Benchmark Volume {i}: Systems and Performance /</subfield><subfield code="c">B. Author.</subfield></datafield>
  <datafield tag="260" ind1=" " ind2=" "><subfield code="a">Trivandrum :</subfield><subfield code="b">Library Press,</subfield><subfield code="c">2026.</subfield></datafield>
  <datafield tag="942" ind1=" " ind2=" "><subfield code="c">BK</subfield></datafield>
  <datafield tag="952" ind1=" " ind2=" ">
    <subfield code="a">MAIN</subfield><subfield code="b">MAIN</subfield><subfield code="c">STACKS</subfield>
    <subfield code="o">004.{i:03} AUT</subfield><subfield code="p">BENCH-BC-{i:04}A</subfield>
    <subfield code="i">BENCH-ACC-{i:04}A</subfield><subfield code="y">BK</subfield><subfield code="d">2026-01-15</subfield>
    <subfield code="g">500.00</subfield>
  </datafield>
  <datafield tag="952" ind1=" " ind2=" ">
    <subfield code="a">MAIN</subfield><subfield code="b">MAIN</subfield><subfield code="c">STACKS</subfield>
    <subfield code="o">004.{i:03} AUT</subfield><subfield code="p">BENCH-BC-{i:04}B</subfield>
    <subfield code="i">BENCH-ACC-{i:04}B</subfield><subfield code="y">BK</subfield><subfield code="d">2026-01-15</subfield>
    <subfield code="g">500.00</subfield>
  </datafield>
</record>
"#
        )
        .unwrap();
    }
    large_xml.push_str("</collection>\n");

    let t0 = Instant::now();
    let bench_preview = h.ok("import_preview", request(&large_xml));
    let preview_duration = t0.elapsed();

    assert_eq!(bench_preview["totals"]["records"], 200);
    assert_eq!(bench_preview["totals"]["copies"], 400);

    let t1 = Instant::now();
    let bench_report = h.ok(
        "import_commit",
        json!({
            "token": bench_preview["token"],
            "options": options
        }),
    );
    let commit_duration = t1.elapsed();

    assert_eq!(bench_report["createdRecords"], 200);
    assert_eq!(bench_report["copiesAdded"], 400);
    assert_eq!(bench_report["failedRecords"], 0);

    let total_duration = t0.elapsed();
    println!(
        "KOHA_BENCHMARK: 200 records / 400 copies. Preview: {:?}, Commit: {:?}, Total: {:?}",
        preview_duration, commit_duration, total_duration
    );
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
    assert!(preview["mappingFields"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| v == "callNumber"));
    assert_eq!(
        h.ok(
            "catalogue_search",
            json!({"query":{"page":1,"pageSize":25}})
        )["total"],
        0
    );
    let options = json!({"duplicatePolicy":"skip","createMissing":true,"defaultItemCategoryId":config["itemCategories"][0]["id"],"barcodeAsAccession":true});
    let report = h.ok(
        "import_commit",
        json!({"token":preview["token"],"options":options}),
    );
    assert_eq!(report["createdRecords"], 1);
    assert_eq!(report["copiesAdded"], 2);
    let found = h.ok(
        "catalogue_search",
        json!({"query":{"text":"000XLSX123","page":1,"pageSize":25}}),
    );
    let record = h.ok("catalogue_get_record", json!({"id":found["rows"][0]["id"]}));
    assert_eq!(record["title"], "XLSX verification book");
    let copies = record["copies"].as_array().unwrap();
    let first = copies
        .iter()
        .find(|c| c["barcode"] == "000XLSX123")
        .unwrap();
    assert_eq!(first["accessionNo"], "000XLSX123");
    assert_eq!(first["priceMinor"], 12550);
    assert_eq!(first["callNumber"], "020 XLSX");
    assert!(copies
        .iter()
        .any(|c| c["barcode"] == "000XLSX124" && c["status"] == "damaged"));
    for bad in [
        b"not a workbook".as_slice(),
        include_bytes!("../../docs/examples/catalogue-import-formula.xlsx").as_slice(),
    ] {
        assert_eq!(h.code("import_preview", request(bad)), "VALIDATION");
    }
    assert_eq!(
        h.ok(
            "catalogue_search",
            json!({"query":{"page":1,"pageSize":25}})
        )["total"],
        1
    );
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        auth::create_user(&conn, "xlsxviewer", "Viewer", "viewer passphrase", "viewer").unwrap();
    }
    h.ok("logout", json!({}));
    assert_eq!(h.code("import_preview", request(data)), "AUTH_FAILED");
    assert_eq!(
        h.code("import_commit", json!({"token":"stale", "options":options})),
        "AUTH_FAILED"
    );
    h.ok(
        "login",
        json!({"username":"xlsxviewer","password":"viewer passphrase"}),
    );
    assert_eq!(h.code("import_preview", request(data)), "PERMISSION_DENIED");
    assert_eq!(
        h.code("import_commit", json!({"token":"stale", "options":options})),
        "PERMISSION_DENIED"
    );
}

#[test]
fn selected_catalogue_export_supports_exchange_formats_and_read_permissions() {
    use base64::Engine;
    let h = Harness::new();
    assert_eq!(
        h.code(
            "catalogue_export_records",
            json!({"ids":[1],"format":"json"})
        ),
        "AUTH_FAILED"
    );
    h.ok("complete_setup", setup_body());
    let config = h.ok("config_overview", json!({}));
    let preview = h.ok("import_preview", json!({"request":{"fileName":"sample.xlsx","format":"xlsx","profile":"generic","mapping":null,"dataBase64":base64::engine::general_purpose::STANDARD.encode(include_bytes!("../../docs/examples/catalogue-import.xlsx"))}}));
    h.ok("import_commit", json!({"token":preview["token"],"options":{"duplicatePolicy":"new","createMissing":true,"defaultItemCategoryId":config["itemCategories"][0]["id"],"barcodeAsAccession":true}}));
    let found = h.ok(
        "catalogue_search",
        json!({"query":{"page":1,"pageSize":25}}),
    );
    let id = &found["rows"][0]["id"];
    for format in ["json", "csv", "xlsx", "marc", "marcxml"] {
        let file = h.ok(
            "catalogue_export_records",
            json!({"ids":[id],"format":format}),
        );
        assert_eq!(file["records"], 1);
        assert_eq!(file["copies"], 2);
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(file["dataBase64"].as_str().unwrap())
            .unwrap();
        assert!(!bytes.is_empty());
        let reimport = h.ok("import_preview", json!({"request":{"fileName":file["fileName"],"format":format,"profile":"koha","mapping":null,"dataBase64":file["dataBase64"]}}));
        assert_eq!(reimport["totals"]["records"], 1);
        assert_eq!(reimport["totals"]["copies"], 2);
        h.ok("import_cancel", json!({}));
    }
    assert_eq!(
        h.code(
            "catalogue_export_records",
            json!({"ids":[],"format":"json"})
        ),
        "VALIDATION"
    );
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        auth::create_user(
            &conn,
            "exportviewer",
            "Viewer",
            "viewer passphrase",
            "viewer",
        )
        .unwrap();
    }
    h.ok("logout", json!({}));
    h.ok(
        "login",
        json!({"username":"exportviewer","password":"viewer passphrase"}),
    );
    assert_eq!(
        h.ok(
            "catalogue_export_records",
            json!({"ids":[id],"format":"json"})
        )["records"],
        1
    );
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        conn.execute("DELETE FROM role_permissions WHERE role_id=(SELECT id FROM roles WHERE code='viewer') AND permission_id=(SELECT id FROM permissions WHERE code='catalogue.read')", []).unwrap();
    }
    assert_eq!(
        h.code(
            "catalogue_export_records",
            json!({"ids":[id],"format":"json"})
        ),
        "PERMISSION_DENIED"
    );
}

#[test]
fn export_save_requires_catalogue_read_and_validates_before_showing_a_dialog() {
    let h = Harness::new();
    let body = json!({"fileName":"pusthakam-catalogue.json","dataBase64":"W10="});
    assert_eq!(h.code("catalogue_save_export", body.clone()), "AUTH_FAILED");
    h.ok("complete_setup", setup_body());
    assert_eq!(
        h.code(
            "catalogue_save_export",
            json!({"fileName":"../library.db","dataBase64":"W10="})
        ),
        "VALIDATION"
    );
    assert_eq!(
        h.code(
            "catalogue_save_export",
            json!({"fileName":"pusthakam-catalogue.json","dataBase64":"invalid"})
        ),
        "VALIDATION"
    );
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        conn.execute("DELETE FROM role_permissions WHERE role_id=(SELECT role_id FROM staff_users WHERE username='admin') AND permission_id=(SELECT id FROM permissions WHERE code='catalogue.read')",[]).unwrap();
    }
    assert_eq!(h.code("catalogue_save_export", body), "PERMISSION_DENIED");
}

#[test]
fn library_calendar_configures_closures_and_rechecks_issue_and_renewal_dates() {
    let h = Harness::new();
    assert_eq!(h.code("config_calendar", json!({})), "AUTH_FAILED");
    assert_eq!(
        h.code("config_save_closed_weekdays", json!({"days":[0]})),
        "AUTH_FAILED"
    );
    assert_eq!(
        h.code(
            "config_save_holiday",
            json!({"input":{"id":null,"date":"2027-01-01","name":"New year"}})
        ),
        "AUTH_FAILED"
    );
    assert_eq!(
        h.code("config_remove_holiday", json!({"id":1})),
        "AUTH_FAILED"
    );
    h.ok("complete_setup", setup_body());
    let cfg = h.ok("config_overview", json!({}));
    let policy = &cfg["policies"][0];
    let member=h.ok("members_save",json!({"input":{"name":"Calendar reader","memberNumber":"CALENDAR-1","categoryId":policy["memberCategoryId"]}}));
    let record = h.ok(
        "catalogue_save_record",
        json!({"input":{"title":"Calendar test book","contributors":[],"subjects":[]}}),
    );
    let copies=h.ok("catalogue_add_copies",json!({"recordId":record,"input":{"count":1,"itemCategoryId":policy["itemCategoryId"],"acquisitionSource":"purchase"}}));
    let barcode = &copies[0]["barcode"];
    let initial = h.ok(
        "circulation_prepare_issue",
        json!({"memberNumber":"CALENDAR-1","barcode":barcode}),
    );
    let (weekday, expected): (i64, String) = {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        conn.query_row(
            "SELECT CAST(strftime('%w',?1) AS INTEGER),date(?1,'+2 days')",
            [initial["dueOn"].as_str().unwrap()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap()
    };
    h.ok("config_save_closed_weekdays", json!({"days":[weekday]}));
    let tomorrow = {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        conn.query_row(
            "SELECT date(?1,'+1 day')",
            [initial["dueOn"].as_str().unwrap()],
            |r| r.get::<_, String>(0),
        )
        .unwrap()
    };
    let holiday = h.ok(
        "config_save_holiday",
        json!({"input":{"id":null,"date":tomorrow,"name":"Calendar closure"}}),
    );
    assert_eq!(
        h.code(
            "config_save_holiday",
            json!({"input":{"id":null,"date":tomorrow,"name":"Duplicate"}})
        ),
        "CONFLICT"
    );
    assert_eq!(
        h.code(
            "config_save_closed_weekdays",
            json!({"days":[0,1,2,3,4,5,6]})
        ),
        "VALIDATION"
    );
    let old_request = json!({"memberNumber":"CALENDAR-1","barcode":barcode,"memberId":member,"itemId":initial["itemId"],"dueOn":initial["dueOn"]});
    assert_eq!(h.code("circulation_issue", old_request), "CONFLICT");
    let issue = h.ok(
        "circulation_prepare_issue",
        json!({"memberNumber":"CALENDAR-1","barcode":barcode}),
    );
    assert_eq!(issue["dueOn"], expected);
    h.ok("circulation_issue",json!({"memberNumber":"CALENDAR-1","barcode":barcode,"memberId":member,"itemId":issue["itemId"],"dueOn":issue["dueOn"]}));
    let renewal = h.ok("circulation_prepare_renewal", json!({"barcode":barcode}));
    let renewal_holiday = h.ok(
        "config_save_holiday",
        json!({"input":{"id":null,"date":renewal["dueOn"],"name":"Renewal closure"}}),
    );
    assert_eq!(
        h.code("circulation_renew", json!({"confirmation":renewal})),
        "CONFLICT"
    );
    let changed = h.ok("circulation_prepare_renewal", json!({"barcode":barcode}));
    assert!(changed["dueOn"].as_str().unwrap() > renewal["dueOn"].as_str().unwrap());
    h.ok("circulation_renew", json!({"confirmation":changed}));
    h.ok(
        "config_save_holiday",
        json!({"input":{"id":holiday,"date":"2028-02-29","name":"Edited closure"}}),
    );
    h.ok("config_remove_holiday", json!({"id":renewal_holiday}));
    let cal = h.ok("config_calendar", json!({}));
    assert_eq!(cal["holidays"][0]["date"], "2028-02-29");
    assert_eq!(
        h.ok("circulation_prepare_return", json!({"barcode":barcode}))["dueOn"],
        changed["dueOn"],
        "calendar edits do not rewrite existing loan dates"
    );
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        auth::create_user(
            &conn,
            "calendarviewer",
            "Viewer",
            "viewer passphrase",
            "viewer",
        )
        .unwrap();
    }
    h.ok("logout", json!({}));
    h.ok(
        "login",
        json!({"username":"calendarviewer","password":"viewer passphrase"}),
    );
    h.ok("config_calendar", json!({}));
    assert_eq!(
        h.code("config_save_closed_weekdays", json!({"days":[]})),
        "PERMISSION_DENIED"
    );
    assert_eq!(
        h.code(
            "config_save_holiday",
            json!({"input":{"id":null,"date":"2027-01-01","name":"Denied"}})
        ),
        "PERMISSION_DENIED"
    );
    assert_eq!(
        h.code("config_remove_holiday", json!({"id":holiday})),
        "PERMISSION_DENIED"
    );
}

#[test]
fn member_fine_accounts_reconcile_payments_waivers_currencies_and_read_permissions() {
    let h = Harness::new();
    assert_eq!(
        h.code("members_fine_account", json!({"memberId":1,"page":1})),
        "AUTH_FAILED"
    );
    assert_eq!(
        h.code(
            "members_fine_history",
            json!({"memberId":1,"fineId":1,"page":1})
        ),
        "AUTH_FAILED"
    );
    h.ok("complete_setup", setup_body());
    let cfg = h.ok("config_overview", json!({}));
    let member=h.ok("members_save",json!({"input":{"name":"Account reader","memberNumber":"FINE-A","categoryId":cfg["memberCategories"][0]["id"]}}));
    let other=h.ok("members_save",json!({"input":{"name":"Other reader","memberNumber":"FINE-B","categoryId":cfg["memberCategories"][0]["id"]}}));
    assert_eq!(
        h.ok("members_fine_account", json!({"memberId":other,"page":1}))["total"],
        0
    );
    let record = h.ok(
        "catalogue_save_record",
        json!({"input":{"title":"Fine account test","contributors":[],"subjects":[]}}),
    );
    let copies=h.ok("catalogue_add_copies",json!({"recordId":record,"input":{"count":1,"itemCategoryId":cfg["itemCategories"][0]["id"],"acquisitionSource":"purchase"}}));
    let first_fine: i64 = {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        let staff: i64 = conn
            .query_row(
                "SELECT id FROM staff_users WHERE username='admin'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let mut first = 0;
        for i in 0..27 {
            let mid = if i == 26 {
                other.as_i64().unwrap()
            } else {
                member.as_i64().unwrap()
            };
            conn.execute("INSERT INTO loans (item_id,member_id,item_category_id,issued_by,due_on,returned_at,returned_by) VALUES (?1,?2,?3,?4,'2026-01-01','2026-01-03T00:00:00Z',?4)",rusqlite::params![copies[0]["id"].as_i64().unwrap(),mid,cfg["itemCategories"][0]["id"].as_i64().unwrap(),staff]).unwrap();
            let loan = conn.last_insert_rowid();
            conn.execute(
                "INSERT INTO fines (loan_id,amount_minor,currency) VALUES (?1,?2,?3)",
                rusqlite::params![
                    loan,
                    if i == 25 {
                        500
                    } else if i == 26 {
                        900
                    } else {
                        100
                    },
                    if i == 25 { "USD" } else { "INR" }
                ],
            )
            .unwrap();
            if i == 0 {
                first = conn.last_insert_rowid();
            }
        }
        first
    };
    let request = json!({"memberId":member,"page":1});
    let initial = h.ok("members_fine_account", request.clone());
    assert_eq!(initial["total"], 26);
    assert_eq!(initial["rows"].as_array().unwrap().len(), 25);
    assert_eq!(
        initial["balances"],
        json!([
        {"currency":"INR","assessedMinor":2500,"paidMinor":0,"waivedMinor":0,"outstandingMinor":2500},
        {"currency":"USD","assessedMinor":500,"paidMinor":0,"waivedMinor":0,"outstandingMinor":500}])
    );
    h.ok("circulation_settle_fine",json!({"fineId":first_fine,"amountMinor":40,"expectedBalance":100,"entry":{"kind":"payment","method":"cash"}}));
    assert_eq!(
        h.ok("members_fine_account", request.clone())["balances"][0]["outstandingMinor"],
        2460
    );
    h.ok("circulation_settle_fine",json!({"fineId":first_fine,"amountMinor":10,"expectedBalance":60,"entry":{"kind":"waiver","reason":"Approved adjustment"}}));
    let partial = h.ok("members_fine_account", json!({"memberId":member,"page":2}));
    assert_eq!(partial["rows"][0]["outstandingMinor"], 50);
    assert_eq!(partial["balances"][0]["paidMinor"], 40);
    assert_eq!(partial["balances"][0]["waivedMinor"], 10);
    assert_eq!(partial["balances"][0]["outstandingMinor"], 2450);
    let ledger = h.ok(
        "circulation_list_fines",
        json!({"page":2,"includeSettled":true}),
    );
    let matched = ledger["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["id"] == first_fine)
        .unwrap();
    assert_eq!(
        matched["outstandingMinor"],
        partial["rows"][0]["outstandingMinor"]
    );
    let history = h.ok(
        "members_fine_history",
        json!({"memberId":member,"fineId":first_fine,"page":1}),
    );
    assert_eq!(history["total"], 2);
    assert_eq!(history["rows"][0]["reason"], "Approved adjustment");
    assert_eq!(
        h.code(
            "members_fine_history",
            json!({"memberId":other,"fineId":first_fine,"page":1})
        ),
        "NOT_FOUND"
    );
    assert_eq!(
        h.code("members_fine_account", json!({"memberId":999999,"page":1})),
        "NOT_FOUND"
    );
    h.ok("circulation_settle_fine",json!({"fineId":first_fine,"amountMinor":50,"expectedBalance":50,"entry":{"kind":"payment","method":"card"}}));
    let settled = h.ok(
        "members_fine_account",
        json!({"memberId":member,"page":999}),
    );
    assert_eq!(settled["page"], 2);
    assert_eq!(settled["rows"][0]["outstandingMinor"], 0);
    assert_eq!(settled["balances"][0]["paidMinor"], 90);
    assert_eq!(settled["balances"][0]["outstandingMinor"], 2400);
    h.ok("members_archive", json!({"id":member,"archived":true}));
    assert_eq!(h.ok("members_fine_account", request.clone())["total"], 26);
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        auth::create_user(&conn, "fineviewer", "Viewer", "viewer passphrase", "viewer").unwrap();
    }
    h.ok("logout", json!({}));
    h.ok(
        "login",
        json!({"username":"fineviewer","password":"viewer passphrase"}),
    );
    assert_eq!(h.ok("members_fine_account", request.clone())["total"], 26);
    let redacted = h.ok(
        "members_fine_history",
        json!({"memberId":member,"fineId":first_fine,"page":1}),
    );
    assert!(redacted["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["kind"] == "waiver")
        .all(|r| r["reason"].is_null()));
    assert_eq!(h.code("circulation_settle_fine",json!({"fineId":first_fine,"amountMinor":1,"expectedBalance":0,"entry":{"kind":"payment","method":"cash"}})),"PERMISSION_DENIED");
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        conn.execute("DELETE FROM role_permissions WHERE role_id=(SELECT id FROM roles WHERE code='viewer') AND permission_id=(SELECT id FROM permissions WHERE code='members.read')",[]).unwrap();
    }
    assert_eq!(h.code("members_fine_account", request), "PERMISSION_DENIED");
    assert_eq!(
        h.code(
            "members_fine_history",
            json!({"memberId":member,"fineId":first_fine,"page":1})
        ),
        "PERMISSION_DENIED"
    );
}

#[test]
fn loan_import_commit_preserves_history_and_rolls_back_conflicts() {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let h = Harness::new();
    h.ok("complete_setup", setup_body());
    let cfg = h.ok("config_overview", json!({}));
    let policy = &cfg["policies"][0];
    let member=h.ok("members_save",json!({"input":{"name":"Migration reader","memberNumber":"0001","categoryId":policy["memberCategoryId"]}}));
    let record = h.ok(
        "catalogue_save_record",
        json!({"input":{"title":"Migration fixture","contributors":[],"subjects":[]}}),
    );
    let copies=h.ok("catalogue_add_copies",json!({"recordId":record,"input":{"count":1,"itemCategoryId":policy["itemCategoryId"],"acquisitionSource":"purchase"}}));
    let barcode = copies[0]["barcode"].as_str().unwrap();
    let file = |rows: String| {
        STANDARD.encode(format!(
            "source_id,member_number,barcode,issued_at,due_on,returned_at,source_staff\n{rows}"
        ))
    };
    let request = |data: &str, fp: &Value, terms: bool| json!({"dataBase64":data,"sourceSystem":"Old library","expectedFingerprint":fp,"useCurrentTerms":terms});
    let data=file(format!("old,0001,{barcode},2020-01-01T10:00:00Z,2020-01-20,2020-01-10T11:00:00Z,Former librarian\nactive,0001,{barcode},2026-11-01T10:00:00Z,2026-11-30,,Former librarian\n"));
    let p = h.ok("circulation_preview_import", json!({"dataBase64":data}));
    assert_eq!(
        h.code(
            "circulation_commit_import",
            request(&data, &p["fingerprint"], false)
        ),
        "VALIDATION"
    );
    // Policy edits invalidate the review, even when row counts stay unchanged.
    {
        let state = h.webview.state::<AppState>();
        let conn = state.conn().unwrap();
        conn.execute(
            "UPDATE circulation_policies SET fine_per_day_minor=fine_per_day_minor+1",
            [],
        )
        .unwrap();
    }
    assert_eq!(
        h.code(
            "circulation_commit_import",
            request(&data, &p["fingerprint"], true)
        ),
        "VALIDATION"
    );
    let p = h.ok("circulation_preview_import", json!({"dataBase64":data}));
    // Fail the audit after loan, provenance, terms and stock writes; all must roll back.
    {
        let state = h.webview.state::<AppState>();
        let conn = state.conn().unwrap();
        conn.execute_batch("CREATE TRIGGER fail_import_audit BEFORE INSERT ON audit_events WHEN NEW.action='circulation.import' BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();
    }
    assert!(h
        .call(
            "circulation_commit_import",
            request(&data, &p["fingerprint"], true)
        )
        .is_err());
    {
        let state = h.webview.state::<AppState>();
        let conn = state.conn().unwrap();
        for table in [
            "loans",
            "loan_import_batches",
            "loan_import_sources",
            "loan_fine_terms",
        ] {
            assert_eq!(
                conn.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r
                    .get::<_, i64>(0))
                    .unwrap(),
                0
            );
        }
        assert_eq!(
            conn.query_row("SELECT status FROM physical_items", [], |r| r
                .get::<_, String>(0))
                .unwrap(),
            "available"
        );
        conn.execute_batch("DROP TRIGGER fail_import_audit")
            .unwrap();
    }
    assert_eq!(
        h.ok(
            "circulation_commit_import",
            request(&data, &p["fingerprint"], true)
        ),
        2
    );
    let history = h.ok("members_list_loans", json!({"memberId":member,"page":1}));
    assert_eq!(history["total"], 2);
    assert_eq!(history["rows"][0]["importSource"], "Old library");
    assert_eq!(history["rows"][0]["sourceStaff"], "Former librarian");
    {
        let state = h.webview.state::<AppState>();
        let conn = state.conn().unwrap();
        let saved:(String,String,String)=conn.query_row("SELECT l.issued_at,l.returned_at,s.source_staff FROM loans l JOIN loan_import_sources s ON s.loan_id=l.id WHERE s.source_id='old'",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
        assert_eq!(
            saved,
            (
                "2020-01-01T10:00:00Z".into(),
                "2020-01-10T11:00:00Z".into(),
                "Former librarian".into()
            )
        );
        assert_eq!(
            conn.query_row("SELECT status FROM physical_items", [], |r| r
                .get::<_, String>(0))
                .unwrap(),
            "on_loan"
        );
        assert_eq!(
            conn.query_row("SELECT count(*) FROM fines", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            0
        );
        assert_eq!(
            conn.query_row("SELECT count(*) FROM loan_fine_terms", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert!(conn.execute("DELETE FROM loan_import_sources", []).is_err());
    }
    assert_eq!(
        h.code(
            "circulation_commit_import",
            request(&data, &p["fingerprint"], true)
        ),
        "VALIDATION"
    );
    // A new file with an existing source ID also fails without retaining its batch.
    let repeat=file(format!("old,0001,{barcode},2019-01-01T10:00:00Z,2019-01-20,2019-01-10T11:00:00Z,Former librarian\n"));
    let p = h.ok("circulation_preview_import", json!({"dataBase64":repeat}));
    assert_eq!(
        h.code(
            "circulation_commit_import",
            request(&repeat, &p["fingerprint"], false)
        ),
        "VALIDATION"
    );
    // Overlapping closed history must fail, even with a fresh source ID.
    let overlap=file(format!("overlap,0001,{barcode},2020-01-02T10:00:00Z,2020-01-20,2020-01-03T11:00:00Z,Former librarian\n"));
    let p = h.ok("circulation_preview_import", json!({"dataBase64":overlap}));
    assert_eq!(
        h.code(
            "circulation_commit_import",
            request(&overlap, &p["fingerprint"], false)
        ),
        "VALIDATION"
    );
    {
        let state = h.webview.state::<AppState>();
        let conn = state.conn().unwrap();
        assert_eq!(
            conn.query_row("SELECT count(*) FROM loan_import_batches", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            1
        );
        auth::create_user(&conn, "loanviewer", "Viewer", "viewer passphrase", "viewer").unwrap();
    }
    h.ok(
        "login",
        json!({"username":"loanviewer","password":"viewer passphrase"}),
    );
    assert_eq!(
        h.code(
            "circulation_commit_import",
            request(&overlap, &p["fingerprint"], true)
        ),
        "PERMISSION_DENIED"
    );
}

#[test]
fn loan_import_commit_supports_v2_return_conditions() {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let h = Harness::new();
    h.ok("complete_setup", setup_body());
    let cfg = h.ok("config_overview", json!({}));
    let policy = &cfg["policies"][0];
    let member = h.ok("members_save", json!({"input":{"name":"Condition reader","memberNumber":"0001","categoryId":policy["memberCategoryId"]}}));
    let record = h.ok(
        "catalogue_save_record",
        json!({"input":{"title":"Condition fixture","contributors":[],"subjects":[]}}),
    );
    let copies = h.ok("catalogue_add_copies", json!({"recordId":record,"input":{"count":3,"itemCategoryId":policy["itemCategoryId"],"acquisitionSource":"purchase"}}));
    let b1 = copies[0]["barcode"].as_str().unwrap();
    let b2 = copies[1]["barcode"].as_str().unwrap();
    let b3 = copies[2]["barcode"].as_str().unwrap();
    let file = |rows: String| {
        STANDARD.encode(format!("source_id,member_number,barcode,issued_at,due_on,returned_at,source_staff,return_condition\n{rows}"))
    };
    let request = |data: &str, fp: &Value, terms: bool| json!({"dataBase64":data,"sourceSystem":"Condition library","expectedFingerprint":fp,"useCurrentTerms":terms});
    let data = file(format!(
        "l_damaged,0001,{b1},2020-01-01T10:00:00Z,2020-01-20,2020-01-10T11:00:00Z,Staff A,damaged\n\
         l_lost,0001,{b2},2020-02-01T10:00:00Z,2020-02-20,2020-02-10T11:00:00Z,Staff B,lost\n\
         l_avail,0001,{b3},2020-03-01T10:00:00Z,2020-03-20,2020-03-10T11:00:00Z,Staff C,available\n"
    ));
    let p = h.ok("circulation_preview_import", json!({"dataBase64": data}));
    assert_eq!(p["total"], 3);
    assert_eq!(p["closed"], 3);
    assert_eq!(p["invalid"], 0);
    let imported = h.ok(
        "circulation_commit_import",
        request(&data, &p["fingerprint"], false),
    );
    assert_eq!(imported, 3);
    let history = h.ok("members_list_loans", json!({"memberId": member, "page": 1}));
    assert_eq!(history["total"], 3);
    let rows = history["rows"].as_array().unwrap();
    assert!(rows
        .iter()
        .any(|r| r["returnCondition"].as_str() == Some("damaged")));
    assert!(rows
        .iter()
        .any(|r| r["returnCondition"].as_str() == Some("lost")));
    assert!(rows
        .iter()
        .any(|r| r["returnCondition"].as_str() == Some("available")));
}

#[test]
fn renewal_import_validates_chains_preserves_events_and_rolls_back() {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let h = Harness::new();
    h.ok("complete_setup", setup_body());
    let cfg = h.ok("config_overview", json!({}));
    let policy = &cfg["policies"][0];
    let member=h.ok("members_save",json!({"input":{"name":"Renewal migration reader","memberNumber":"0001","categoryId":policy["memberCategoryId"]}}));
    let record = h.ok(
        "catalogue_save_record",
        json!({"input":{"title":"Renewal migration fixture","contributors":[],"subjects":[]}}),
    );
    let copies=h.ok("catalogue_add_copies",json!({"recordId":record,"input":{"count":1,"itemCategoryId":policy["itemCategoryId"],"acquisitionSource":"purchase"}}));
    let barcode = copies[0]["barcode"].as_str().unwrap();
    let loans=STANDARD.encode(format!("source_id,member_number,barcode,issued_at,due_on,returned_at,source_staff\nloan-001,0001,{barcode},2020-01-01T10:00:00Z,2020-01-20,2020-01-25T11:00:00Z,Original staff\n"));
    let p = h.ok("circulation_preview_import", json!({"dataBase64":loans}));
    h.ok("circulation_commit_import",json!({"dataBase64":loans,"sourceSystem":"Legacy","expectedFingerprint":p["fingerprint"],"useCurrentTerms":false}));
    let csv = |rows: &str| {
        STANDARD.encode(format!(
            "source_id,loan_source_id,renewed_at,previous_due_on,due_on,source_staff\n{rows}"
        ))
    };
    let data=csv("r2,loan-001,2020-01-12T10:00:00Z,2020-01-15,2020-01-20,Old staff B\nr1,loan-001,2020-01-05T10:00:00Z,2020-01-10,2020-01-15,Old staff A\n");
    let preview = |data: &str, source: &str| json!({"dataBase64":data,"sourceSystem":source});
    let commit = |data: &str, p: &Value, source: &str| json!({"dataBase64":data,"sourceSystem":source,"expectedFingerprint":p["fingerprint"]});
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
    assert_eq!(
        h.code("circulation_preview_renewal_import", preview("!", "Legacy")),
        "VALIDATION"
    );
    let wrong = h.ok(
        "circulation_preview_renewal_import",
        preview(&data, "Other source"),
    );
    assert_eq!(wrong["invalid"], 2);
    let p = h.ok(
        "circulation_preview_renewal_import",
        preview(&data, "Legacy"),
    );
    assert_eq!(p["invalid"], 0);
    assert_eq!(p["total"], 2);
    assert_eq!(p["loans"], 1);
    assert_eq!(
        h.code(
            "circulation_commit_renewal_import",
            commit(&data, &p, "Other source")
        ),
        "VALIDATION"
    );
    {
        let state = h.webview.state::<AppState>();
        let conn = state.conn().unwrap();
        conn.execute_batch("CREATE TRIGGER fail_renewal_import_audit BEFORE INSERT ON audit_events WHEN NEW.action='circulation.import_renewals' BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();
    }
    assert!(h
        .call(
            "circulation_commit_renewal_import",
            commit(&data, &p, "Legacy")
        )
        .is_err());
    {
        let state = h.webview.state::<AppState>();
        let conn = state.conn().unwrap();
        for table in [
            "loan_renewals",
            "renewal_import_batches",
            "renewal_import_sources",
        ] {
            assert_eq!(
                conn.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r
                    .get::<_, i64>(0))
                    .unwrap(),
                0
            );
        }
        conn.execute_batch("DROP TRIGGER fail_renewal_import_audit")
            .unwrap();
        conn.execute("UPDATE loans SET due_on='2020-01-21'", [])
            .unwrap();
    }
    assert_eq!(
        h.code(
            "circulation_commit_renewal_import",
            commit(&data, &p, "Legacy")
        ),
        "VALIDATION"
    );
    {
        let state = h.webview.state::<AppState>();
        let conn = state.conn().unwrap();
        conn.execute("UPDATE loans SET due_on='2020-01-20'", [])
            .unwrap();
    }
    let p = h.ok(
        "circulation_preview_renewal_import",
        preview(&data, "Legacy"),
    );
    assert_eq!(
        h.ok(
            "circulation_commit_renewal_import",
            commit(&data, &p, "Legacy")
        ),
        2
    );
    let history = h.ok("members_list_loans", json!({"memberId":member,"page":1}));
    assert_eq!(history["rows"][0]["renewalCount"], 2);
    assert_eq!(history["rows"][0]["dueOn"], "2020-01-20");
    {
        let state = h.webview.state::<AppState>();
        let conn = state.conn().unwrap();
        let events=conn.prepare("SELECT n.renewed_at,n.previous_due_on,n.due_on,s.source_staff FROM loan_renewals n JOIN renewal_import_sources s ON s.renewal_id=n.id ORDER BY n.id").unwrap().query_map([],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?))).unwrap().collect::<Result<Vec<_>,_>>().unwrap();
        assert_eq!(
            events[0],
            (
                "2020-01-05T10:00:00Z".into(),
                "2020-01-10".into(),
                "2020-01-15".into(),
                "Old staff A".into()
            )
        );
        assert_eq!(events[1].3, "Old staff B");
        assert_eq!(
            conn.query_row("SELECT status FROM physical_items", [], |r| r
                .get::<_, String>(0))
                .unwrap(),
            "available"
        );
        assert_eq!(
            conn.query_row("SELECT count(*) FROM fines", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            0
        );
        assert!(conn
            .execute("DELETE FROM renewal_import_sources", [])
            .is_err());
        auth::create_user(
            &conn,
            "renewalviewer",
            "Viewer",
            "viewer passphrase",
            "viewer",
        )
        .unwrap();
    }
    let repeat = h.ok(
        "circulation_preview_renewal_import",
        preview(&data, "Legacy"),
    );
    assert!(repeat["invalid"].as_u64().unwrap() > 0);
    assert_eq!(
        h.code(
            "circulation_commit_renewal_import",
            commit(&data, &repeat, "Legacy")
        ),
        "VALIDATION"
    );
    h.ok(
        "login",
        json!({"username":"renewalviewer","password":"viewer passphrase"}),
    );
    assert_eq!(
        h.code(
            "circulation_preview_renewal_import",
            preview(&data, "Legacy")
        ),
        "PERMISSION_DENIED"
    );
    assert_eq!(
        h.code(
            "circulation_commit_renewal_import",
            commit(&data, &p, "Legacy")
        ),
        "PERMISSION_DENIED"
    );
}

#[test]
fn history_bundle_import_reconciles_all_five_parts_and_rolls_back() {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let h = Harness::new();
    h.ok("complete_setup", setup_body());
    let cfg = h.ok("config_overview", json!({}));
    let policy = &cfg["policies"][0];
    let member=h.ok("members_save",json!({"input":{"name":"Bundle reader","memberNumber":"0001","categoryId":policy["memberCategoryId"]}}));
    h.ok("members_save",json!({"input":{"name":"Second queue reader","memberNumber":"0002","categoryId":policy["memberCategoryId"]}}));
    let record = h.ok(
        "catalogue_save_record",
        json!({"input":{"title":"Five part migration","contributors":[],"subjects":[]}}),
    );
    let copies=h.ok("catalogue_add_copies",json!({"recordId":record,"input":{"count":1,"itemCategoryId":policy["itemCategoryId"],"acquisitionSource":"purchase"}}));
    let barcode = copies[0]["barcode"].as_str().unwrap();
    let loans=STANDARD.encode(format!("source_id,member_number,barcode,issued_at,due_on,returned_at,source_staff\na,0001,{barcode},2018-01-01T10:00:00Z,2018-01-10,2018-01-20T10:00:00Z,Original staff\nb,0001,{barcode},2019-01-01T10:00:00Z,2019-01-10,2019-01-20T10:00:00Z,Original staff\n"));
    let p = h.ok("circulation_preview_import", json!({"dataBase64":loans}));
    h.ok("circulation_commit_import",json!({"dataBase64":loans,"sourceSystem":"Legacy","expectedFingerprint":p["fingerprint"],"useCurrentTerms":false}));
    let (now, expiry): (String, String) = {
        let state = h.webview.state::<AppState>();
        let conn = state.conn().unwrap();
        conn.query_row("SELECT strftime('%Y-%m-%dT%H:%M:%SZ','now'),strftime('%Y-%m-%dT%H:%M:%SZ','now','+7 days')",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap()
    };
    let bundle = json!({"version":1,"sourceSystem":"Legacy",
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
    let encoded = |b: &Value| STANDARD.encode(serde_json::to_vec(b).unwrap());
    let preview = |b: &Value| {
        h.ok(
            "circulation_preview_history",
            json!({"dataBase64":encoded(b)}),
        )
    };
    let request = |b: &Value, p: &Value| json!({"dataBase64":encoded(b),"expectedFingerprint":p["fingerprint"]});
    for kind in 0..17 {
        let mut bad = bundle.clone();
        match kind {
            0 => bad["expected"]["balances"][0]["outstandingMinor"] = json!(501),
            1 => bad["payments"][0]["amountMinor"] = json!(900),
            2 => bad["payments"][0]["currency"] = json!("USD"),
            3 => bad["waivers"][0]["reason"] = json!(""),
            4 => bad["fines"][0]["loanSourceId"] = json!("missing"),
            5 => bad["reservations"][0]["queuePosition"] = json!(1),
            6 => bad["reservations"][2]["expiresAt"] = json!("2020-01-01T00:00:00Z"),
            7 => bad["reservations"][0]["memberNumber"] = json!("missing"),
            8 => bad["payments"][0]["recordedAt"] = json!("2010-01-01T00:00:00Z"),
            9 => bad["payments"][0]["sourceId"] = json!("w1"),
            10 => bad["fines"][1]["loanSourceId"] = json!("a"),
            11 => bad["payments"][0]["fineSourceId"] = json!("missing"),
            12 => bad["payments"][0]["method"] = json!("unknown"),
            13 => bad["fines"][0]["amountMinor"] = json!(9007199254740992_i64),
            14 => bad["waivers"][0]["method"] = json!("cash"),
            15 => bad["fines"][0]["amountMinor"] = json!(0),
            _ => bad["fines"][0]["currency"] = json!("inr"),
        }
        let p = preview(&bad);
        assert_eq!(p["canCommit"], false, "bad variant {kind}");
        assert_eq!(
            h.code("circulation_commit_history", request(&bad, &p)),
            "VALIDATION"
        );
    }
    let mut unknown = bundle.clone();
    unknown["fines"][0]["ignoredField"] = json!("not allowed");
    assert_eq!(
        h.code(
            "circulation_preview_history",
            json!({"dataBase64":encoded(&unknown)})
        ),
        "VALIDATION"
    );
    let p = preview(&bundle);
    assert_eq!(p["canCommit"], true);
    assert_eq!(p["summary"], bundle["expected"]);
    // Changed member data invalidates the exact preview even when counts remain equal.
    {
        let state = h.webview.state::<AppState>();
        let conn = state.conn().unwrap();
        conn.execute(
            "UPDATE members SET name='Changed' WHERE member_number='0001'",
            [],
        )
        .unwrap();
    }
    assert_eq!(
        h.code("circulation_commit_history", request(&bundle, &p)),
        "VALIDATION"
    );
    let p = preview(&bundle);
    // Failure after all five parts were inserted must roll every table and stock change back.
    {
        let state = h.webview.state::<AppState>();
        let conn = state.conn().unwrap();
        conn.execute_batch("CREATE TRIGGER fail_history_audit BEFORE INSERT ON audit_events WHEN NEW.action='circulation.import_history' BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();
    }
    assert!(h
        .call("circulation_commit_history", request(&bundle, &p))
        .is_err());
    {
        let state = h.webview.state::<AppState>();
        let conn = state.conn().unwrap();
        for table in [
            "reservations",
            "fines",
            "fine_settlements",
            "history_import_sources",
            "history_import_batches",
        ] {
            assert_eq!(
                conn.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r
                    .get::<_, i64>(0))
                    .unwrap(),
                0
            );
        }
        assert_eq!(
            conn.query_row("SELECT status FROM physical_items", [], |r| r
                .get::<_, String>(0))
                .unwrap(),
            "available"
        );
        conn.execute_batch("DROP TRIGGER fail_history_audit")
            .unwrap();
    }
    // Re-read destination balances before committing: even an unexpected trigger change rolls back.
    {
        let state = h.webview.state::<AppState>();
        let conn = state.conn().unwrap();
        conn.execute_batch("CREATE TRIGGER alter_imported_fine AFTER INSERT ON fines BEGIN UPDATE fines SET amount_minor=amount_minor+1 WHERE id=NEW.id; END;").unwrap();
    }
    assert_eq!(
        h.code("circulation_commit_history", request(&bundle, &p)),
        "VALIDATION"
    );
    {
        let state = h.webview.state::<AppState>();
        let conn = state.conn().unwrap();
        assert_eq!(
            conn.query_row("SELECT count(*) FROM fines", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            0
        );
        conn.execute_batch("DROP TRIGGER alter_imported_fine")
            .unwrap();
    }
    let saved = h.ok("circulation_commit_history", request(&bundle, &p));
    assert_eq!(saved, bundle["expected"]);
    assert_eq!(
        h.ok("circulation_history_imports", json!({}))[0]["summary"],
        saved
    );
    let account = h.ok("members_fine_account", json!({"memberId":member,"page":1}));
    assert_eq!(account["balances"][0]["outstandingMinor"], 500);
    assert_eq!(account["balances"][1]["outstandingMinor"], 400);
    let fine = account["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["currency"] == "INR")
        .unwrap()["id"]
        .clone();
    let history = h.ok(
        "members_fine_history",
        json!({"memberId":member,"fineId":fine,"page":1}),
    );
    assert_eq!(
        history["rows"][0]["staffName"],
        "Original approver (imported source)"
    );
    assert_eq!(
        history["rows"][0]["reason"],
        "Historical approved adjustment"
    );
    let queue = h.ok("circulation_list_reservations", json!({"page":1}));
    assert_eq!(queue["total"], 2);
    assert_eq!(queue["rows"][0]["memberNumber"], "0001");
    {
        let state = h.webview.state::<AppState>();
        let conn = state.conn().unwrap();
        assert_eq!(
            conn.query_row("SELECT status FROM physical_items", [], |r| r
                .get::<_, String>(0))
                .unwrap(),
            "reserved_hold"
        );
        let summary: String = conn
            .query_row("SELECT summary_json FROM history_import_batches", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(serde_json::from_str::<Value>(&summary).unwrap(), saved);
        assert!(conn
            .execute("DELETE FROM history_import_sources", [])
            .is_err());
    }
    // Imported queue order is used by normal cancellation/promotion, not just stored metadata.
    h.ok(
        "circulation_cancel_reservation",
        json!({"id":queue["rows"][0]["id"]}),
    );
    let promoted = h.ok("circulation_list_reservations", json!({"page":1}));
    assert_eq!(promoted["rows"][0]["memberNumber"], "0002");
    assert_eq!(promoted["rows"][0]["status"], "ready");
    let repeat = preview(&bundle);
    assert_eq!(repeat["canCommit"], false);
    assert_eq!(
        h.code("circulation_commit_history", request(&bundle, &repeat)),
        "VALIDATION"
    );
    // Follow-up payment-only batch must reconcile the full affected fine, including prior entries.
    let follow = json!({"version":1,"sourceSystem":"Legacy","reservations":[],"fines":[],"payments":[{"sourceId":"p3","fineSourceId":"f1","amountMinor":500,"currency":"INR","recordedAt":"2018-01-23T10:00:00Z","method":"card","sourceStaff":"Original cashier"}],"waivers":[],"expected":{"counts":{"reservations":0,"fines":0,"payments":1,"waivers":0},"balances":[{"currency":"INR","assessedMinor":1000,"paidMinor":800,"waivedMinor":200,"outstandingMinor":0}]}});
    let p = preview(&follow);
    assert_eq!(p["canCommit"], true);
    h.ok("circulation_commit_history", request(&follow, &p));
    // A wholly pending imported queue on an on-loan copy survives until the normal return promotes it.
    h.ok(
        "circulation_cancel_reservation",
        json!({"id":promoted["rows"][0]["id"]}),
    );
    let issue = h.ok(
        "circulation_prepare_issue",
        json!({"memberNumber":"0001","barcode":barcode}),
    );
    h.ok("circulation_issue",json!({"memberNumber":"0001","barcode":barcode,"memberId":issue["memberId"],"itemId":issue["itemId"],"dueOn":issue["dueOn"]}));
    let mut pending = json!({"version":1,"sourceSystem":"Legacy","reservations":[{"sourceId":"pending-new","memberNumber":"0002","barcode":barcode,"placedAt":now,"status":"pending","expiresAt":null,"closedAt":null,"queuePosition":1,"sourceStaff":"Queue staff"}],"fines":[],"payments":[],"waivers":[],"expected":{"counts":{"reservations":1,"fines":0,"payments":0,"waivers":0},"balances":[]}});
    let mut wrong = pending.clone();
    wrong["reservations"][0]["memberNumber"] = json!("0001");
    assert_eq!(preview(&wrong)["canCommit"], false);
    let p_pending = preview(&pending);
    assert_eq!(p_pending["canCommit"], true);
    h.ok("circulation_commit_history", request(&pending, &p_pending));
    assert_eq!(
        h.ok("circulation_list_reservations", json!({"page":1}))["rows"][0]["status"],
        "pending"
    );
    let ret = h.ok("circulation_prepare_return", json!({"barcode":barcode}));
    h.ok(
        "circulation_return",
        json!({"loanId":ret["loanId"],"condition":"available"}),
    );
    assert_eq!(
        h.ok("circulation_list_reservations", json!({"page":1}))["rows"][0]["status"],
        "ready"
    );
    pending["reservations"][0]["sourceId"] = json!("attempted-queue-merge");
    assert_eq!(preview(&pending)["canCommit"], false);
    {
        let state = h.webview.state::<AppState>();
        let conn = state.conn().unwrap();
        auth::create_user(
            &conn,
            "bundleviewer",
            "Viewer",
            "viewer passphrase",
            "viewer",
        )
        .unwrap();
    }
    h.ok(
        "login",
        json!({"username":"bundleviewer","password":"viewer passphrase"}),
    );
    assert_eq!(
        h.code(
            "circulation_preview_history",
            json!({"dataBase64":encoded(&bundle)})
        ),
        "PERMISSION_DENIED"
    );
    assert_eq!(
        h.code("circulation_commit_history", request(&bundle, &p)),
        "PERMISSION_DENIED"
    );
    assert_eq!(
        h.code("circulation_history_imports", json!({})),
        "PERMISSION_DENIED"
    );
    let redacted = h.ok(
        "members_fine_history",
        json!({"memberId":member,"fineId":fine,"page":1}),
    );
    assert!(redacted["rows"]
        .as_array()
        .unwrap()
        .iter()
        .all(|r| r["reason"].is_null()));
}

#[test]
fn history_bundle_import_supports_open_loan_fines_and_subsequent_return() {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let h = Harness::new();
    h.ok("complete_setup", setup_body());
    let cfg = h.ok("config_overview", json!({}));
    let policy = &cfg["policies"][0];
    let member = h.ok(
        "members_save",
        json!({
            "input": {
                "name": "Open fine borrower",
                "memberNumber": "M-OPEN-01",
                "categoryId": policy["memberCategoryId"]
            }
        }),
    );
    let record = h.ok(
        "catalogue_save_record",
        json!({
            "input": {
                "title": "Open loan fine migration",
                "contributors": [],
                "subjects": []
            }
        }),
    );
    let copies = h.ok(
        "catalogue_add_copies",
        json!({
            "recordId": record,
            "input": {
                "count": 1,
                "itemCategoryId": policy["itemCategoryId"],
                "acquisitionSource": "purchase"
            }
        }),
    );
    let barcode = copies[0]["barcode"].as_str().unwrap();

    // Import an open loan (empty returned_at)
    let loans = STANDARD.encode(format!(
        "source_id,member_number,barcode,issued_at,due_on,returned_at,source_staff\nopen_1,M-OPEN-01,{barcode},2020-01-01T10:00:00Z,2020-01-10,,Original staff\n"
    ));
    let p = h.ok("circulation_preview_import", json!({ "dataBase64": loans }));
    assert_eq!(p["open"], 1);
    assert_eq!(p["closed"], 0);
    h.ok(
        "circulation_commit_import",
        json!({
            "dataBase64": loans,
            "sourceSystem": "LegacySys",
            "expectedFingerprint": p["fingerprint"],
            "useCurrentTerms": true
        }),
    );

    // Verify copy is on_loan
    {
        let state = h.webview.state::<AppState>();
        let conn = state.conn().unwrap();
        assert_eq!(
            conn.query_row(
                "SELECT status FROM physical_items WHERE barcode=?1",
                [barcode],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
            "on_loan"
        );
    }

    // Now import a history bundle attaching an assessed fine to this open loan, plus a partial payment
    let bundle = json!({
        "version": 1,
        "sourceSystem": "LegacySys",
        "reservations": [],
        "fines": [
            {
                "sourceId": "f-open-1",
                "loanSourceId": "open_1",
                "amountMinor": 500,
                "currency": "INR",
                "assessedAt": "2020-01-15T10:00:00Z",
                "sourceStaff": "Assessor Staff"
            }
        ],
        "payments": [
            {
                "sourceId": "p-open-1",
                "fineSourceId": "f-open-1",
                "amountMinor": 200,
                "currency": "INR",
                "recordedAt": "2020-01-16T10:00:00Z",
                "method": "cash",
                "sourceStaff": "Cashier Staff"
            }
        ],
        "waivers": [],
        "expected": {
            "counts": { "reservations": 0, "fines": 1, "payments": 1, "waivers": 0 },
            "balances": [
                { "currency": "INR", "assessedMinor": 500, "paidMinor": 200, "waivedMinor": 0, "outstandingMinor": 300 }
            ]
        }
    });

    let encoded = STANDARD.encode(serde_json::to_vec(&bundle).unwrap());
    let p_hist = h.ok(
        "circulation_preview_history",
        json!({ "dataBase64": encoded }),
    );
    assert_eq!(p_hist["canCommit"], true);
    assert_eq!(p_hist["summary"], bundle["expected"]);

    let committed = h.ok(
        "circulation_commit_history",
        json!({
            "dataBase64": encoded,
            "expectedFingerprint": p_hist["fingerprint"]
        }),
    );
    assert_eq!(committed, bundle["expected"]);

    // Verify member account reflects the imported assessed fine and partial payment
    let account = h.ok(
        "members_fine_account",
        json!({ "memberId": member, "page": 1 }),
    );
    assert_eq!(account["balances"][0]["assessedMinor"], 500);
    assert_eq!(account["balances"][0]["paidMinor"], 200);
    assert_eq!(account["balances"][0]["outstandingMinor"], 300);

    // Prepare return for this open loan at circulation desk: preview reflects the existing assessed fine
    let ret_preview = h.ok("circulation_prepare_return", json!({ "barcode": barcode }));
    assert_eq!(ret_preview["estimatedFine"]["amountMinor"], 500);
    assert_eq!(ret_preview["estimatedFine"]["currency"], "INR");

    // Return the copy at circulation desk: returns cleanly without UNIQUE constraint failure or duplicate insertion
    let ret_result = h.ok(
        "circulation_return",
        json!({
            "loanId": ret_preview["loanId"],
            "condition": "available"
        }),
    );
    assert_eq!(ret_result["amountMinor"], 500);
    assert_eq!(ret_result["currency"], "INR");

    // Copy status is now available
    {
        let state = h.webview.state::<AppState>();
        let conn = state.conn().unwrap();
        assert_eq!(
            conn.query_row(
                "SELECT status FROM physical_items WHERE barcode=?1",
                [barcode],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
            "available"
        );
        // Only 1 fine exists in database for this loan
        let fine_count: i64 = conn
            .query_row(
                "SELECT count(*) FROM fines WHERE loan_id=?1",
                [ret_preview["loanId"].as_i64().unwrap()],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(fine_count, 1);
    }

    // Verify fine can now be settled for the remaining 300 outstanding
    let fines = h.ok(
        "circulation_list_fines",
        json!({ "page": 1, "includeSettled": false }),
    );
    assert_eq!(fines["total"], 1);
    let fine_id = fines["rows"][0]["id"].as_i64().unwrap();
    assert_eq!(fines["rows"][0]["outstandingMinor"], 300);

    h.ok(
        "circulation_settle_fine",
        json!({
            "fineId": fine_id,
            "amountMinor": 300,
            "expectedBalance": 300,
            "entry": { "kind": "payment", "method": "cash" }
        }),
    );

    // Fine is fully settled
    let outstanding_fines = h.ok(
        "circulation_list_fines",
        json!({ "page": 1, "includeSettled": false }),
    );
    assert_eq!(outstanding_fines["total"], 0);

    let updated_account = h.ok(
        "members_fine_account",
        json!({ "memberId": member, "page": 1 }),
    );
    assert_eq!(updated_account["balances"][0]["assessedMinor"], 500);
    assert_eq!(updated_account["balances"][0]["paidMinor"], 500);
    assert_eq!(updated_account["balances"][0]["outstandingMinor"], 0);
}

#[test]
fn acquisitions_full_workflow_via_ipc() {
    let h = Harness::new();
    h.ok("complete_setup", setup_body());

    // 1. Create vendor
    let vendor_id = h
        .ok(
            "acquisitions_save_vendor",
            json!({
                "input": {
                    "name": "Prabhat Books",
                    "code": "PRABHAT",
                    "contactName": "Raman Nair",
                    "email": "raman@prabhat.example",
                    "phone": "+91 471 2334455",
                    "address": "Trivandrum",
                    "notes": "Literature supplier"
                }
            }),
        )
        .as_i64()
        .unwrap();

    let vendors = h.ok(
        "acquisitions_list_vendors",
        json!({ "includeArchived": false }),
    );
    assert_eq!(vendors.as_array().unwrap().len(), 1);
    assert_eq!(vendors[0]["name"], "Prabhat Books");

    // 2. Create budget
    let budget_id = h
        .ok(
            "acquisitions_save_budget",
            json!({
                "input": {
                    "name": "Literature 2026-27",
                    "code": "LIT-26",
                    "amountMinor": 10000000,
                    "currency": "INR",
                    "fiscalYear": "2026-27"
                }
            }),
        )
        .as_i64()
        .unwrap();

    let budgets = h.ok(
        "acquisitions_list_budgets",
        json!({ "includeArchived": false }),
    );
    assert_eq!(budgets.as_array().unwrap().len(), 1);
    assert_eq!(budgets[0]["allocatedMinor"], 0);

    // 3. Create acquisition request
    let req_id = h
        .ok(
            "acquisitions_create_request",
            json!({
                "input": {
                    "title": "Aadujeevitham",
                    "author": "Benyamin",
                    "isbn": "9788126419999",
                    "publisher": "Green Books",
                    "publicationYear": 2008,
                    "estimatedPriceMinor": 25000,
                    "currency": "INR",
                    "copies": 3,
                    "requestedBy": "Malayalam Department",
                    "requestReason": "Prescribed syllabus"
                }
            }),
        )
        .as_i64()
        .unwrap();

    let requests = h.ok("acquisitions_list_requests", json!({ "status": null }));
    assert_eq!(requests.as_array().unwrap().len(), 1);
    assert_eq!(requests[0]["status"], "pending");

    // 4. Approve request with budget
    h.ok(
        "acquisitions_approve_request",
        json!({
            "id": req_id,
            "budgetId": budget_id
        }),
    );

    let approved_requests = h.ok(
        "acquisitions_list_requests",
        json!({ "status": "approved" }),
    );
    assert_eq!(approved_requests.as_array().unwrap().len(), 1);
    assert_eq!(approved_requests[0]["budgetId"], budget_id);

    // 5. Create purchase order with this item
    let po_id = h
        .ok(
            "acquisitions_create_order",
            json!({
                "input": {
                    "orderNumber": "PO-2026-0001",
                    "vendorId": vendor_id,
                    "budgetId": budget_id,
                    "invoiceRef": "PROFORMA-101",
                    "expectedAt": "2026-12-01",
                    "notes": "Direct delivery to central library",
                    "items": [
                        {
                            "requestId": req_id,
                            "title": "Aadujeevitham",
                            "author": "Benyamin",
                            "isbn": "9788126419999",
                            "quantity": 3,
                            "unitPriceMinor": 24000,
                            "notes": null
                        }
                    ]
                }
            }),
        )
        .as_i64()
        .unwrap();

    // 6. Verify purchase order detail
    let po = h.ok("acquisitions_get_order", json!({ "id": po_id }));
    assert_eq!(po["orderNumber"], "PO-2026-0001");
    assert_eq!(po["vendorName"], "Prabhat Books");
    assert_eq!(po["totalMinor"], 72000); // 3 * 24000
    assert_eq!(po["items"].as_array().unwrap().len(), 1);

    // 7. Verify budget now reflects allocated amount
    let budgets_after = h.ok(
        "acquisitions_list_budgets",
        json!({ "includeArchived": false }),
    );
    assert_eq!(budgets_after[0]["allocatedMinor"], 72000);

    // 8. Verify request status transitioned to 'ordered'
    let ordered_requests = h.ok("acquisitions_list_requests", json!({ "status": "ordered" }));
    assert_eq!(ordered_requests.as_array().unwrap().len(), 1);

    // 9. Update order status to 'ordered'
    h.ok(
        "acquisitions_update_order_status",
        json!({
            "id": po_id,
            "status": "ordered",
            "invoiceRef": "INV-OFFICIAL-99"
        }),
    );

    let updated_po = h.ok("acquisitions_get_order", json!({ "id": po_id }));
    assert_eq!(updated_po["status"], "ordered");
    assert_eq!(updated_po["invoiceRef"], "INV-OFFICIAL-99");

    // 10. Receive 2 copies (partial order)
    let item_id = updated_po["items"][0]["id"].as_i64().unwrap();
    let recv1 = h.ok(
        "acquisitions_receive_items",
        json!({
            "input": {
                "orderId": po_id,
                "itemId": item_id,
                "quantity": 2
            }
        }),
    );
    assert_eq!(recv1["orderStatus"], "partially_received");
    assert_eq!(recv1["receivedQuantity"], 2);
    assert_eq!(recv1["createdCopies"].as_array().unwrap().len(), 2);

    // 11. Receive remaining 1 copy (full order)
    let recv2 = h.ok(
        "acquisitions_receive_items",
        json!({
            "input": {
                "orderId": po_id,
                "itemId": item_id,
                "quantity": 1
            }
        }),
    );
    assert_eq!(recv2["orderStatus"], "received");
    assert_eq!(recv2["receivedQuantity"], 3);

    // 12. Duplicate receipt rejected
    assert_eq!(
        h.code(
            "acquisitions_receive_items",
            json!({
                "input": {
                    "orderId": po_id,
                    "itemId": item_id,
                    "quantity": 1
                }
            })
        ),
        "VALIDATION"
    );

    // 13. Verify request is now marked 'received'
    let received_requests = h.ok(
        "acquisitions_list_requests",
        json!({ "status": "received" }),
    );
    assert_eq!(received_requests.as_array().unwrap().len(), 1);
}

#[test]
fn serials_full_workflow_via_ipc() {
    let h = Harness::new();
    h.ok("complete_setup", setup_body());

    // 1. Create quarterly subscription with auto issue generation
    let sub_id = h
        .ok(
            "serials_save_subscription",
            json!({
                "input": {
                    "id": null,
                    "title": "National Geographic",
                    "issn": "0027-9358",
                    "publisher": "National Geographic Society",
                    "frequency": "quarterly",
                    "vendorId": null,
                    "budgetId": null,
                    "costMinor": 360000,
                    "currency": "INR",
                    "startDate": "2026-01-01",
                    "endDate": "2026-12-31",
                    "notes": "Annual subscription",
                    "autoGenerateIssues": true,
                    "issuePatternPrefix": "Q"
                }
            }),
        )
        .as_i64()
        .unwrap();

    // 2. Fetch subscription details
    let sub = h.ok("serials_get_subscription", json!({ "id": sub_id }));
    assert_eq!(sub["title"], "National Geographic");
    assert_eq!(sub["frequency"], "quarterly");
    assert_eq!(sub["totalIssuesCount"], 4);
    assert_eq!(sub["expectedIssuesCount"], 4);

    // 3. List issues
    let issues = h.ok(
        "serials_list_issues",
        json!({
            "filter": {
                "subscriptionId": sub_id
            }
        }),
    );
    let issue_array = issues.as_array().unwrap();
    assert_eq!(issue_array.len(), 4);
    assert_eq!(issue_array[0]["issueNumber"], "Q1");
    assert_eq!(issue_array[0]["expectedDate"], "2026-01-01");
    assert_eq!(issue_array[0]["status"], "expected");

    let issue_1_id = issue_array[0]["id"].as_i64().unwrap();
    let issue_2_id = issue_array[1]["id"].as_i64().unwrap();

    // 4. Check in issue 1 with accessioning
    let checkin1 = h.ok(
        "serials_check_in_issue",
        json!({
            "input": {
                "issueId": issue_1_id,
                "receivedDate": "2026-01-02",
                "accessionCopy": true,
                "notes": "Issue received in mint condition"
            }
        }),
    );
    assert_eq!(checkin1["status"], "received");
    assert_eq!(checkin1["receivedDate"], "2026-01-02");
    assert!(checkin1["accessionNo"].is_string());
    assert!(checkin1["copyId"].is_number());

    // 5. Duplicate check-in rejection
    assert_eq!(
        h.code(
            "serials_check_in_issue",
            json!({
                "input": {
                    "issueId": issue_1_id
                }
            })
        ),
        "CONFLICT"
    );

    // 6. Update issue 2 status to missing, then claimed
    h.ok(
        "serials_update_issue_status",
        json!({
            "issueId": issue_2_id,
            "status": "missing"
        }),
    );
    h.ok(
        "serials_update_issue_status",
        json!({
            "issueId": issue_2_id,
            "status": "claimed"
        }),
    );

    let missing = h.ok("serials_list_missing_issues", json!({}));
    assert!(missing
        .as_array()
        .unwrap()
        .iter()
        .any(|i| i["id"] == issue_2_id && i["status"] == "claimed"));

    // 7. Add ad-hoc manual issue (supplement)
    let manual_id = h
        .ok(
            "serials_add_issue",
            json!({
                "input": {
                    "subscriptionId": sub_id,
                    "issueNumber": "Special Wildlife Atlas",
                    "expectedDate": "2026-05-15",
                    "notes": "Supplementary edition"
                }
            }),
        )
        .as_i64()
        .unwrap();
    assert!(manual_id > 0);

    // 8. Renew subscription for 2027
    let renewal_id = h
        .ok(
            "serials_renew_subscription",
            json!({
                "input": {
                    "subscriptionId": sub_id,
                    "startDate": "2027-01-01",
                    "endDate": "2027-12-31",
                    "costMinor": 380000,
                    "currency": "INR",
                    "budgetId": null,
                    "notes": "Early bird renewal discount",
                    "autoGenerateIssues": true,
                    "issuePatternPrefix": "2027-Q"
                }
            }),
        )
        .as_i64()
        .unwrap();
    assert!(renewal_id > 0);

    let renewals = h.ok("serials_list_renewals", json!({ "subscriptionId": sub_id }));
    assert_eq!(renewals.as_array().unwrap().len(), 1);
    assert_eq!(renewals[0]["costMinor"], 380000);

    // 9. Reconcile expenditure via IPC for date range
    let exp_2026 = h.ok(
        "serials_get_expenditure",
        json!({
            "fromDate": "2026-01-01",
            "toDate": "2026-12-31"
        }),
    );
    assert_eq!(exp_2026["totalSpendMinor"], 360000);
    assert_eq!(exp_2026["items"].as_array().unwrap().len(), 1);
    assert_eq!(exp_2026["items"][0]["itemType"], "initial");

    let exp_2027 = h.ok(
        "serials_get_expenditure",
        json!({
            "fromDate": "2027-01-01",
            "toDate": "2027-12-31"
        }),
    );
    assert_eq!(exp_2027["totalSpendMinor"], 380000);
    assert_eq!(exp_2027["items"].as_array().unwrap().len(), 1);
    assert_eq!(exp_2027["items"][0]["itemType"], "renewal");

    let exp_all = h.ok("serials_get_expenditure", json!({}));
    assert_eq!(exp_all["totalSpendMinor"], 740000);
    assert_eq!(exp_all["items"].as_array().unwrap().len(), 2);

    // 10. Test due reminders / expiring subscriptions via IPC
    let exp_sub_id = h
        .ok(
            "serials_save_subscription",
            json!({
                "input": {
                    "id": null,
                    "title": "Weekly Chronicle",
                    "issn": "1234-5679",
                    "publisher": "Chronicle Media",
                    "frequency": "weekly",
                    "costMinor": 15000,
                    "currency": "INR",
                    "startDate": "2026-01-01",
                    "endDate": "2026-11-25",
                    "autoGenerateIssues": false
                }
            }),
        )
        .as_i64()
        .unwrap();
    assert!(exp_sub_id > 0);

    let expiring = h.ok("serials_list_expiring", json!({ "daysAhead": 60 }));
    assert!(expiring
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["subscription"]["id"] == exp_sub_id));

    // 11. Cancel subscription
    h.ok("serials_cancel_subscription", json!({ "id": sub_id }));
    let cancelled_sub = h.ok("serials_get_subscription", json!({ "id": sub_id }));
    assert_eq!(cancelled_sub["status"], "cancelled");
}

#[test]
fn classification_full_workflow_via_ipc() {
    let h = Harness::new();
    h.ok("complete_setup", setup_body());

    // 1. Classification schemes: list built-in schemes
    let schemes = h
        .ok("classification_list_schemes", json!({}))
        .as_array()
        .unwrap()
        .clone();
    assert!(schemes.len() >= 4);
    let ddc = schemes.iter().find(|s| s["code"] == "ddc").unwrap();
    let ddc_id = ddc["id"].as_i64().unwrap();
    assert_eq!(ddc["isCustom"], false);

    // 2. Browse DDC entries: top-level
    let top_ddc = h
        .ok(
            "classification_list_entries",
            json!({
                "schemeId": ddc_id,
                "parentCode": null,
                "searchQuery": null
            }),
        )
        .as_array()
        .unwrap()
        .clone();
    assert!(top_ddc
        .iter()
        .any(|e| e["code"] == "500" && e["title"] == "Science"));
    assert!(top_ddc.iter().any(|e| e["code"] == "800"));

    // Browse child entries of 500
    let math_entries = h
        .ok(
            "classification_list_entries",
            json!({
                "schemeId": ddc_id,
                "parentCode": "500",
                "searchQuery": null
            }),
        )
        .as_array()
        .unwrap()
        .clone();
    assert!(math_entries
        .iter()
        .any(|e| e["code"] == "510" && e["title"] == "Mathematics"));

    // Search classification entries
    let search_res = h
        .ok(
            "classification_list_entries",
            json!({
                "schemeId": ddc_id,
                "parentCode": null,
                "searchQuery": "Physics"
            }),
        )
        .as_array()
        .unwrap()
        .clone();
    assert!(search_res.iter().any(|e| e["code"] == "530"));

    // 3. Custom scheme and custom code creation and deletion
    let custom_scheme_id = h
        .ok(
            "classification_save_scheme",
            json!({
                "input": {
                    "id": null,
                    "code": "special",
                    "name": "Special Collections Scheme",
                    "description": "Internal special collection numbering"
                }
            }),
        )
        .as_i64()
        .unwrap();
    assert!(custom_scheme_id > 0);

    let custom_entry_id = h
        .ok(
            "classification_save_entry",
            json!({
                "input": {
                    "id": null,
                    "schemeId": custom_scheme_id,
                    "code": "SPEC-01",
                    "parentCode": null,
                    "title": "Rare Manuscripts",
                    "description": "Historical palm-leaf and parchment manuscripts"
                }
            }),
        )
        .as_i64()
        .unwrap();
    assert!(custom_entry_id > 0);

    let spec_entries = h
        .ok(
            "classification_list_entries",
            json!({
                "schemeId": custom_scheme_id,
                "parentCode": null,
                "searchQuery": null
            }),
        )
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(spec_entries.len(), 1);
    assert_eq!(spec_entries[0]["code"], "SPEC-01");

    h.ok(
        "classification_delete_entry",
        json!({ "id": custom_entry_id }),
    );
    h.ok(
        "classification_delete_scheme",
        json!({ "id": custom_scheme_id }),
    );

    // 4. Managed Author Authorities
    let author_a_id = h
        .ok(
            "classification_save_author",
            json!({
                "input": {
                    "id": null,
                    "name": "Vaikom Muhammad Basheer",
                    "sortName": "Basheer, Vaikom Muhammad",
                    "kind": "personal",
                    "birthYear": 1908,
                    "deathYear": 1994,
                    "authorityNote": "Renowned Malayalam writer",
                    "variantNames": "Beypore Sultan",
                    "marcTag": "100"
                }
            }),
        )
        .as_i64()
        .unwrap();
    assert!(author_a_id > 0);

    let author_b_id = h
        .ok(
            "classification_save_author",
            json!({
                "input": {
                    "id": null,
                    "name": "V. M. Basheer",
                    "sortName": "Basheer, V. M.",
                    "kind": "personal",
                    "birthYear": 1908,
                    "deathYear": 1994,
                    "authorityNote": "Alternate heading to merge",
                    "variantNames": null,
                    "marcTag": "100"
                }
            }),
        )
        .as_i64()
        .unwrap();
    assert!(author_b_id > 0);

    // 5. Managed Publisher Authorities
    let publisher_id = h
        .ok(
            "classification_save_publisher",
            json!({
                "input": {
                    "id": null,
                    "name": "DC Books",
                    "place": "Kottayam, Kerala",
                    "notes": "Leading Malayalam literary publisher",
                    "marcTag": "264"
                }
            }),
        )
        .as_i64()
        .unwrap();
    assert!(publisher_id > 0);

    // 6. Managed Subject Authorities
    let subject_id = h
        .ok(
            "classification_save_subject",
            json!({
                "input": {
                    "id": null,
                    "heading": "Malayalam fiction",
                    "scheme": "lcsh",
                    "scopeNote": "Fiction written in the Malayalam language",
                    "broaderTerm": "Indic literature",
                    "narrowerTerms": "Malayalam short stories; Malayalam novels",
                    "seeAlso": "Malayalam literature",
                    "marcTag": "650"
                }
            }),
        )
        .as_i64()
        .unwrap();
    assert!(subject_id > 0);

    // 7. Save a book record linked to author_b, publisher, and subject
    let record_id = h
        .ok(
            "catalogue_save_record",
            json!({
                "input": {
                    "id": null,
                    "title": "Pathummayude Aadu",
                    "subtitle": "The Goat of Pathumma",
                    "edition": "1st ed.",
                    "publisher": "DC Books",
                    "publicationPlace": "Kottayam",
                    "publicationYear": 1959,
                    "language": "mal",
                    "physicalDescription": null,
                    "pagination": "96 p.",
                    "series": null,
                    "abstractText": "Humorous and touching autobiographical novella",
                    "notes": null,
                    "isbn": null,
                    "classificationSchemeId": ddc_id,
                    "classificationNumber": "894.8123",
                    "keywords": "goat family humor",
                    "contributors": [{ "name": "V. M. Basheer", "role": "author" }],
                    "subjects": ["Malayalam fiction"]
                }
            }),
        )
        .as_i64()
        .unwrap();
    assert!(record_id > 0);

    // Verify author_b has 1 linked record
    let authors = h
        .ok(
            "classification_list_authors",
            json!({ "searchQuery": "Basheer" }),
        )
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(
        authors.iter().find(|a| a["id"] == author_b_id).unwrap()["recordCount"],
        1
    );

    // Merge author_b into author_a: test linked-record synchronization
    h.ok(
        "classification_merge_authors",
        json!({
            "fromId": author_b_id,
            "toId": author_a_id
        }),
    );

    // author_a now has the linked record
    let updated_authors = h
        .ok(
            "classification_list_authors",
            json!({ "searchQuery": "Basheer" }),
        )
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(
        updated_authors
            .iter()
            .find(|a| a["id"] == author_a_id)
            .unwrap()["recordCount"],
        1
    );
    assert!(!updated_authors.iter().any(|a| a["id"] == author_b_id));

    // 8. Update Publisher Authority and verify linked records reflect update
    h.ok(
        "classification_save_publisher",
        json!({
            "input": {
                "id": publisher_id,
                "name": "DC Books Publications",
                "place": "Kottayam, Kerala",
                "notes": "Updated publisher name",
                "marcTag": "264"
            }
        }),
    );
    let rec = h.ok("catalogue_get_record", json!({ "id": record_id }));
    assert_eq!(rec["publisher"], "DC Books Publications");

    // 9. Controlled MARC Mappings & Parser Testing
    let mappings = h
        .ok("classification_list_marc_mappings", json!({}))
        .as_array()
        .unwrap()
        .clone();
    assert!(mappings.len() >= 15);

    let marc_author_test = h.ok(
        "classification_test_marc_mapping",
        json!({
            "input": {
                "rawMarcLine": "100 1# $a Thakazhi Sivasankara Pillai, $d 1912-1999"
            }
        }),
    );
    assert_eq!(marc_author_test["tag"], "100");
    assert_eq!(marc_author_test["mappedAuthorityType"], "author_personal");
    assert_eq!(
        marc_author_test["targetFields"]["name"],
        "Thakazhi Sivasankara Pillai"
    );
    assert_eq!(marc_author_test["targetFields"]["dates"], "1912-1999");

    let marc_pub_test = h.ok(
        "classification_test_marc_mapping",
        json!({
            "input": {
                "rawMarcLine": "264 #1 $a Thrissur : $b Current Books, $c 1956"
            }
        }),
    );
    assert_eq!(marc_pub_test["tag"], "264");
    assert_eq!(marc_pub_test["mappedAuthorityType"], "publisher");
    assert_eq!(marc_pub_test["targetFields"]["name"], "Current Books");
    assert_eq!(marc_pub_test["targetFields"]["place"], "Thrissur");

    let marc_class_test = h.ok(
        "classification_test_marc_mapping",
        json!({
            "input": {
                "rawMarcLine": "082 04 $a 894.8123"
            }
        }),
    );
    assert_eq!(marc_class_test["tag"], "082");
    assert_eq!(marc_class_test["mappedAuthorityType"], "classification_ddc");
    assert_eq!(
        marc_class_test["targetFields"]["classification_number"],
        "894.8123"
    );
}

#[test]
fn lan_full_workflow_via_ipc() {
    let h = Harness::new();
    h.ok("complete_setup", setup_body());

    // 1. Initial LAN config
    let config = h.ok("lan_get_config", json!({}));
    assert_eq!(config["port"], 4318);
    assert!(!config["enabled"].as_bool().unwrap());
    assert!(!config["serverFingerprint"].as_str().unwrap().is_empty());
    let server_fp = config["serverFingerprint"].as_str().unwrap().to_string();

    // 2. Update LAN config: enable server and configure port
    let updated = h.ok(
        "lan_update_config",
        json!({
            "input": {
                "enabled": true,
                "bindAddress": "127.0.0.1",
                "port": 4318,
                "serverName": "Pusthakam Central Server",
                "opacEnabled": true,
                "browserStaffEnabled": true
            }
        }),
    );
    assert!(updated["enabled"].as_bool().unwrap());
    assert_eq!(updated["serverName"], "Pusthakam Central Server");

    // 3. Generate single-use pairing code
    let pair_code = h.ok("lan_generate_pairing_code", json!({}));
    let code_str = pair_code["code"].as_str().unwrap().to_string();
    assert_eq!(code_str.len(), 6);

    let code_list = h.ok("lan_list_pairing_codes", json!({}));
    assert!(code_list.as_array().unwrap().len() >= 1);

    // 4. Pair a client device
    let pair_res = h.ok(
        "lan_pair_client",
        json!({
            "request": {
                "pairingCode": code_str,
                "clientName": "Circulation Desk 2",
                "deviceKind": "desktop",
                "clientIp": "192.168.1.101"
            }
        }),
    );
    let client_id = pair_res["clientId"].as_str().unwrap().to_string();
    assert!(!client_id.is_empty());
    assert_eq!(pair_res["serverFingerprint"], server_fp);

    // Verify code cannot be reused
    assert_eq!(
        h.code(
            "lan_pair_client",
            json!({
                "request": {
                    "pairingCode": code_str,
                    "clientName": "Attacker Machine"
                }
            })
        ),
        "VALIDATION"
    );

    // 5. List paired clients
    let clients = h.ok("lan_list_paired_clients", json!({}));
    assert_eq!(clients.as_array().unwrap().len(), 1);
    assert_eq!(clients[0]["clientName"], "Circulation Desk 2");

    // 6. Sign in staff member via LAN client
    let login_res = h.ok(
        "lan_login",
        json!({
            "request": {
                "clientId": client_id,
                "serverFingerprint": server_fp,
                "username": "admin",
                "password": "a long passphrase"
            }
        }),
    );
    assert!(!login_res["token"].as_str().unwrap().is_empty());
    assert_eq!(login_res["role"], "administrator");

    // 7. Catalogue book setup & public OPAC search
    let book_id = h.ok(
        "catalogue_save_record",
        json!({
            "input": {
                "title": "Computer Networks: Principles and Practice",
                "contributors": [],
                "subjects": []
            }
        }),
    );

    let opac_search_res = h.ok(
        "lan_opac_search",
        json!({
            "query": "",
            "page": 1,
            "pageSize": 20
        }),
    );
    assert_eq!(opac_search_res["total"], 1);
    assert_eq!(
        opac_search_res["items"][0]["title"],
        "Computer Networks: Principles and Practice"
    );

    let opac_record = h.ok("lan_opac_get_record", json!({ "id": book_id }));
    assert_eq!(
        opac_record["title"],
        "Computer Networks: Principles and Practice"
    );

    // 8. Revoke the client device
    h.ok("lan_revoke_client", json!({ "clientId": client_id }));
    let clients_after = h.ok("lan_list_paired_clients", json!({}));
    assert!(!clients_after[0]["revokedAt"].is_null());

    // Login from revoked client is blocked
    assert_eq!(
        h.code(
            "lan_login",
            json!({
                "request": {
                    "clientId": client_id,
                    "serverFingerprint": server_fp,
                    "username": "admin",
                    "password": "a long passphrase"
                }
            })
        ),
        "PERMISSION_DENIED"
    );
}

#[test]
fn reports_save_export_checks_permission_and_payload_before_dialog() {
    let h = Harness::new();
    let body = json!({"fileName":"report.pdf","dataBase64":"JVBERi0xLjQK"});
    assert_eq!(h.code("reports_save_export", body.clone()), "AUTH_FAILED");
    h.ok("complete_setup", setup_body());
    assert_eq!(
        h.code(
            "reports_save_export",
            json!({"fileName":"../report.pdf","dataBase64":"JVBERi0xLjQK"})
        ),
        "VALIDATION"
    );
    assert_eq!(
        h.code(
            "reports_save_export",
            json!({"fileName":"report.pdf","dataBase64":"invalid"})
        ),
        "VALIDATION"
    );
    {
        let state = h.webview.state::<AppState>();
        state.db.lock().unwrap().execute("DELETE FROM role_permissions WHERE role_id=(SELECT role_id FROM staff_users WHERE username='admin') AND permission_id=(SELECT id FROM permissions WHERE code='reports.read')", []).unwrap();
    }
    assert_eq!(h.code("reports_save_export", body), "PERMISSION_DENIED");
}

#[test]
fn lan_settings_start_and_stop_the_actual_http_listener() {
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    let h = Harness::new();
    assert_eq!(h.code("lan_get_config", json!({})), "AUTH_FAILED");
    h.ok("complete_setup", setup_body());
    let reservation = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = reservation.local_addr().unwrap().port();
    drop(reservation);
    let mut input = json!({"enabled":true,"bindAddress":"127.0.0.1","port":port,"serverName":"Test library","opacEnabled":true,"browserStaffEnabled":true});
    let running = h.ok("lan_update_config", json!({"input":input.clone()}));
    assert_eq!(running["listenerRunning"], true);
    assert!(running["listenerError"].is_null());
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(3)))
        .unwrap();
    stream
        .write_all(b"GET /opac HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    assert!(response.starts_with("HTTP/1.1 200"));
    assert!(response.contains("Public catalogue"));
    assert!(response.contains("id=\"catalogue-title\""));
    assert!(response.contains("/api/opac/search?"));
    assert!(response.contains("data:font/woff2;base64,"));
    for (path, status) in [
        ("/api/opac/search?class=8&available=true&page=2", "200"),
        ("/api/opac/search?class=invalid", "400"),
    ] {
        let mut request = TcpStream::connect(("127.0.0.1", port)).unwrap();
        request
            .set_read_timeout(Some(std::time::Duration::from_secs(3)))
            .unwrap();
        request
            .write_all(
                format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
                    .as_bytes(),
            )
            .unwrap();
        let mut result = String::new();
        request.read_to_string(&mut result).unwrap();
        assert!(result.starts_with(&format!("HTTP/1.1 {status}")));
    }
    input["enabled"] = json!(false);
    let stopped = h.ok("lan_update_config", json!({"input":input}));
    assert_eq!(stopped["listenerRunning"], false);
    assert!(TcpListener::bind(("127.0.0.1", port)).is_ok());
}

#[test]
fn open_loan_directory_enforces_auth_and_permissions() {
    let h = Harness::new();
    let body = json!({"search":"","page":1});
    assert_eq!(
        h.code("circulation_list_open_loans", body.clone()),
        "AUTH_FAILED"
    );
    h.ok("complete_setup", setup_body());
    assert_eq!(
        h.ok("circulation_list_open_loans", body.clone())["total"],
        0
    );
    assert_eq!(
        h.code(
            "circulation_list_open_loans",
            json!({"search":"x".repeat(301),"page":1})
        ),
        "VALIDATION"
    );
    {
        let state = h.webview.state::<AppState>();
        state.db.lock().unwrap().execute("DELETE FROM role_permissions WHERE role_id=(SELECT role_id FROM staff_users WHERE username='admin') AND permission_id=(SELECT id FROM permissions WHERE code='circulation.issue')",[]).unwrap();
    }
    assert_eq!(
        h.code("circulation_list_open_loans", body),
        "PERMISSION_DENIED"
    );
}

#[test]
fn open_loan_directory_paginates_due_order_and_excludes_closed_loans() {
    let h = Harness::new();
    h.ok("complete_setup", setup_body());
    let cfg = h.ok("config_overview", json!({}));
    let category = cfg["policies"][0]["itemCategoryId"].as_i64().unwrap();
    let member=h.ok("members_save",json!({"input":{"name":"Directory reader","memberNumber":"DIR-M","categoryId":cfg["policies"][0]["memberCategoryId"]}})).as_i64().unwrap();
    {
        let state = h.webview.state::<AppState>();
        let conn = state.conn().unwrap();
        let staff: i64 = conn
            .query_row(
                "SELECT id FROM staff_users WHERE username='admin'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            "INSERT INTO bibliographic_records(title) VALUES ('Directory title')",
            [],
        )
        .unwrap();
        let record = conn.last_insert_rowid();
        for index in 0..28 {
            conn.execute("INSERT INTO physical_items(record_id,accession_no,barcode,item_category_id,status) VALUES (?1,?2,?3,?4,'on_loan')",rusqlite::params![record,format!("ACC-{index}"),format!("DIR-{index}"),category]).unwrap();
            let item = conn.last_insert_rowid();
            conn.execute("INSERT INTO loans(item_id,member_id,item_category_id,issued_by,due_on,returned_at,returned_by) VALUES (?1,?2,?3,?4,?5,?6,?7)",rusqlite::params![item,member,category,staff,if index<10 {"2026-01-01"} else {"2099-12-01"},if index==27 {Some("2026-01-02T00:00:00Z")} else {None},if index==27 {Some(staff)} else {None}]).unwrap();
        }
    }
    let first = h.ok("circulation_list_open_loans", json!({"search":"","page":1}));
    assert_eq!(first["total"], 27);
    assert_eq!(first["rows"].as_array().unwrap().len(), 25);
    assert_eq!(first["rows"][0]["dueOn"], "2026-01-01");
    let last = h.ok(
        "circulation_list_open_loans",
        json!({"search":"","page":99}),
    );
    assert_eq!(last["page"], 2);
    assert_eq!(last["rows"].as_array().unwrap().len(), 2);
    let accession = h.ok(
        "circulation_list_open_loans",
        json!({"search":"ACC-26","page":1}),
    );
    assert_eq!(accession["total"], 1);
    assert_eq!(accession["rows"][0]["barcode"], "DIR-26");
    assert_eq!(
        h.ok(
            "circulation_list_open_loans",
            json!({"search":"DIR-27","page":1})
        )["total"],
        0
    );
}
