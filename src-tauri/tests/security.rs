//! Security Acceptance and Negative Matrix Tests (SEC-REVIEW)
//!
//! Verifies:
//! 1. Permission checks across all command domains (unauthenticated and unauthorized roles).
//! 2. Path traversal rejection in backups and export saving.
//! 3. Bounded input handling (oversized payloads, corrupted images, invalid base64).
//! 4. Least-privilege CSP and capabilities configuration in tauri.conf.json and default.json.

use pusthakam_lib::db;
use pusthakam_lib::logging;
use pusthakam_lib::paths::AppPaths;
use pusthakam_lib::services::{auth, export_save, image_upload, import};
use pusthakam_lib::state::AppState;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{get_ipc_response, mock_builder, mock_context, noop_assets, INVOKE_KEY};
use tauri::webview::InvokeRequest;
use tauri::{Manager, WebviewWindow};

type Res = Result<Value, Value>;

struct Harness {
    webview: WebviewWindow<tauri::test::MockRuntime>,
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
                db: Mutex::new(conn),
                session: Mutex::new(None),
                pending_import: Mutex::new(None),
                _log_guard: guard,
            })
            .build(mock_context(noop_assets()))
            .unwrap();
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        Harness {
            webview,
            _dir: dir,
            _app: app,
        }
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
    json!({
        "input": {
            "library": {
                "name": "Security Test Library",
                "profile": "school",
                "address": null,
                "contact": null,
                "currency": "INR",
                "locale": "en"
            },
            "admin": {
                "username": "admin",
                "displayName": "Security Admin",
                "password": "a very secure passphrase 123"
            },
            "applyProfileDefaults": true,
            "finePerDayMinor": 100
        }
    })
}

#[test]
fn test_unauthenticated_command_matrix_is_rejected() {
    let h = Harness::new();
    h.ok("complete_setup", setup_body());
    h.ok("logout", json!({}));

    // Commands across different functional domains called without a session
    let commands_to_test = [
        (
            "catalogue_search",
            json!({"query": {"page": 1, "pageSize": 25}}),
        ),
        (
            "catalogue_save_record",
            json!({"input": {"title": "Forbidden", "contributors": [], "subjects": []}}),
        ),
        ("catalogue_archive_records", json!({"ids": [1]})),
        ("circulation_prepare_return", json!({"barcode": "0001"})),
        (
            "circulation_return",
            json!({"loanId": 1, "condition": null}),
        ),
        (
            "members_list",
            json!({"search": "", "page": 1, "archived": false}),
        ),
        (
            "members_save",
            json!({"input": {"memberNumber": "M-2", "name": "Eve", "categoryId": 1}}),
        ),
        ("classification_list_schemes", json!({})),
        (
            "classification_save_entry",
            json!({"input": {"schemeId": 1, "code": "001", "title": "Test"}}),
        ),
        (
            "acquisitions_list_vendors",
            json!({"includeArchived": false}),
        ),
        (
            "acquisitions_create_order",
            json!({"input": {"orderNumber": "ORD-1", "vendorId": 1, "items": []}}),
        ),
        ("serials_list_subscriptions", json!({"filter": {}})),
        (
            "serials_save_subscription",
            json!({"input": {"title": "Periodical", "frequency": "monthly", "startDate": "2026-01-01", "endDate": "2026-12-31"}}),
        ),
        ("reports_stock_by_location", json!({})),
        ("backup_list", json!({})),
        ("backup_create", json!({})),
        ("staff_list", json!({})),
        (
            "staff_create",
            json!({"input": {"username": "eve", "displayName": "Eve", "password": "pass", "role": "viewer"}}),
        ),
        ("get_setting", json!({"key": "library.name"})),
        (
            "set_setting",
            json!({"key": "library.name", "value": "Hacked"}),
        ),
        (
            "audit_search",
            json!({"query": {"page": 1, "pageSize": 25}}),
        ),
    ];

    for (cmd, payload) in commands_to_test {
        let err_code = h.code(cmd, payload);
        assert_eq!(
            err_code, "AUTH_FAILED",
            "Command '{cmd}' must reject unauthenticated calls with AUTH_FAILED, got {err_code}"
        );
    }
}

#[test]
fn test_viewer_role_write_command_matrix_is_rejected() {
    let h = Harness::new();
    h.ok("complete_setup", setup_body());

    // Create a viewer user
    {
        let state = h.webview.state::<AppState>();
        let conn = state.conn().unwrap();
        auth::create_user(
            &conn,
            "viewer_user",
            "Viewer Staff",
            "viewer password 123",
            "viewer",
        )
        .unwrap();
    }

    h.ok("logout", json!({}));
    h.ok(
        "login",
        json!({"username": "viewer_user", "password": "viewer password 123"}),
    );

    // Viewer should be able to read catalogue
    let search = h.ok(
        "catalogue_search",
        json!({"query": {"page": 1, "pageSize": 25}}),
    );
    assert!(search.is_object());

    // Viewer must NOT be permitted to perform write operations
    let write_commands = [
        (
            "catalogue_save_record",
            json!({"input": {"title": "Unauthorized Book", "contributors": [], "subjects": []}}),
        ),
        ("catalogue_archive_records", json!({"ids": [1]})),
        (
            "circulation_return",
            json!({"loanId": 1, "condition": null}),
        ),
        (
            "members_save",
            json!({"input": {"memberNumber": "M-99", "name": "Unauthorized Member", "categoryId": 1}}),
        ),
        (
            "classification_save_entry",
            json!({"input": {"schemeId": 1, "code": "999", "title": "Unauthorized"}}),
        ),
        (
            "acquisitions_create_order",
            json!({"input": {"orderNumber": "ORD-1", "vendorId": 1, "items": []}}),
        ),
        (
            "serials_save_subscription",
            json!({"input": {"title": "Unauthorized Serial", "frequency": "monthly", "startDate": "2026-01-01", "endDate": "2026-12-31"}}),
        ),
        ("backup_create", json!({})),
        (
            "backup_request_restore",
            json!({"name": "pusthakam-backup.sqlite3"}),
        ),
        (
            "staff_create",
            json!({"input": {"username": "intruder", "displayName": "Intruder", "password": "pass", "role": "admin"}}),
        ),
        (
            "set_setting",
            json!({"key": "library.name", "value": "Defaced Name"}),
        ),
    ];

    for (cmd, payload) in write_commands {
        let err_code = h.code(cmd, payload);
        assert_eq!(
            err_code, "PERMISSION_DENIED",
            "Viewer role must be denied write command '{cmd}', got {err_code}"
        );
    }
}

#[test]
fn test_path_traversal_in_backup_operations_is_rejected() {
    let h = Harness::new();
    h.ok("complete_setup", setup_body());

    // Path traversal payloads targeting backup_request_restore
    let traversal_names = [
        "../backup.sqlite3",
        "..\\backup.sqlite3",
        "../../etc/passwd",
        "..\\..\\Windows\\System32\\cmd.exe",
        "pusthakam/subfolder.sqlite3",
        "pusthakam\\subfolder.sqlite3",
        "pusthakam-test;rm.sqlite3",
        "pusthakam-test.exe",
        "random-name.sqlite3",
        "pusthakam-valid.sql",
        "C:\\pusthakam-root.sqlite3",
        "/var/pusthakam-root.sqlite3",
    ];

    for bad_name in traversal_names {
        let code = h.code("backup_request_restore", json!({"name": bad_name}));
        assert_eq!(
            code, "VALIDATION",
            "Path traversal/malformed backup name '{bad_name}' must be rejected with VALIDATION, got {code}"
        );
    }
}

#[test]
fn test_export_save_restricts_arbitrary_paths() {
    let temp_dir = tempfile::tempdir().unwrap();
    let private_dir = temp_dir.path().join("private_app_data");
    std::fs::create_dir_all(&private_dir).unwrap();

    let outside_dir = temp_dir.path().join("user_documents");
    std::fs::create_dir_all(&outside_dir).unwrap();

    let payload = export_save::Payload {
        file_name: "pusthakam-catalogue.csv".into(),
        extension: "csv".into(),
        bytes: b"Title,Author\nBook1,Author1\n".to_vec(),
    };

    // 1. Relative paths are rejected
    let rel_res =
        export_save::write_chosen(Some(PathBuf::from("relative.csv")), &payload, &private_dir);
    assert!(rel_res.is_err());

    // 2. Mismatched extensions are rejected
    let wrong_ext =
        export_save::write_chosen(Some(outside_dir.join("export.txt")), &payload, &private_dir);
    assert!(wrong_ext.is_err());

    // 3. Writing inside private app directory is rejected
    let inside_private =
        export_save::write_chosen(Some(private_dir.join("export.csv")), &payload, &private_dir);
    assert!(inside_private.is_err());

    // 4. Writing to safe external directory succeeds
    let valid_dest = outside_dir.join("export.csv");
    let ok_res = export_save::write_chosen(Some(valid_dest.clone()), &payload, &private_dir);
    assert!(ok_res.is_ok());
    assert!(valid_dest.exists());
    let written_content = std::fs::read_to_string(&valid_dest).unwrap();
    assert_eq!(written_content, "Title,Author\nBook1,Author1\n");
}

#[test]
fn test_image_upload_decoder_rejects_malformed_and_oversized() {
    // 1. Empty string
    assert!(image_upload::decode_png("").is_err());

    // 2. Corrupted base64
    assert!(image_upload::decode_png("not_valid_base64!!!").is_err());

    // 3. Valid base64 but corrupted image bytes
    use base64::{engine::general_purpose::STANDARD, Engine};
    let garbage_bytes = STANDARD.encode(b"This is definitely not a PNG file header");
    assert!(image_upload::decode_png(&garbage_bytes).is_err());

    // 4. Oversized payload (greater than 2MB limit)
    let huge_data = vec![b'A'; 3 * 1024 * 1024];
    let huge_base64 = STANDARD.encode(&huge_data);
    assert!(image_upload::decode_png(&huge_base64).is_err());
}

#[test]
fn test_import_prepare_rejects_malformed_and_empty() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.sqlite3");
    let conn = db::open(&db_path).unwrap();

    // 1. Empty data
    let empty_req = import::PreviewRequest {
        file_name: "test.csv".into(),
        data_base64: "".into(),
        format: "csv".into(),
        profile: "generic".into(),
        mapping: None,
    };
    assert!(import::prepare(&conn, &empty_req).is_err());

    // 2. Malformed base64
    let bad_b64_req = import::PreviewRequest {
        file_name: "test.csv".into(),
        data_base64: "bad&&b64!!".into(),
        format: "csv".into(),
        profile: "generic".into(),
        mapping: None,
    };
    assert!(import::prepare(&conn, &bad_b64_req).is_err());

    // 3. Garbage MARC data returns parse errors gracefully without panics
    use base64::{engine::general_purpose::STANDARD, Engine};
    let garbage_marc = import::PreviewRequest {
        file_name: "test.mrc".into(),
        data_base64: STANDARD.encode(b"00123nam  2200049 a 4500garbage content not valid marc"),
        format: "marc".into(),
        profile: "generic".into(),
        mapping: None,
    };
    let res = import::prepare(&conn, &garbage_marc);
    if let Ok((_pending, preview)) = res {
        assert!(preview.sample.is_empty() || !preview.parse_errors.is_empty());
    }
}

#[test]
fn test_csp_and_capabilities_least_privilege() {
    // Read and verify tauri.conf.json security CSP
    let tauri_conf_str = std::fs::read_to_string("tauri.conf.json")
        .or_else(|_| std::fs::read_to_string("src-tauri/tauri.conf.json"))
        .expect("Failed to read tauri.conf.json");
    let tauri_conf: Value = serde_json::from_str(&tauri_conf_str).unwrap();

    let csp = tauri_conf["app"]["security"]["csp"]
        .as_str()
        .expect("CSP must be present in tauri.conf.json");
    assert!(
        csp.contains("default-src 'self'"),
        "CSP must enforce default-src 'self'"
    );
    assert!(
        csp.contains("script-src 'self'"),
        "CSP must enforce script-src 'self'"
    );
    assert!(
        !csp.contains("'unsafe-eval'"),
        "CSP must NOT allow 'unsafe-eval'"
    );
    assert!(
        !csp.contains("default-src *"),
        "CSP must NOT allow wildcard default-src"
    );

    // Read and verify capabilities/default.json
    let cap_str = std::fs::read_to_string("capabilities/default.json")
        .or_else(|_| std::fs::read_to_string("src-tauri/capabilities/default.json"))
        .expect("Failed to read capabilities/default.json");
    let cap: Value = serde_json::from_str(&cap_str).unwrap();

    let windows = cap["windows"]
        .as_array()
        .expect("Capabilities must specify windows");
    assert_eq!(
        windows,
        &[Value::String("main".into())],
        "Capabilities must only apply to main window"
    );

    // Permissions must not grant shell:default or fs:default
    let perms_str = serde_json::to_string(&cap["permissions"]).unwrap();
    assert!(
        !perms_str.contains("shell:default"),
        "Must NOT grant shell:default capability"
    );
    assert!(
        !perms_str.contains("fs:default"),
        "Must NOT grant fs:default capability"
    );
    assert!(
        !perms_str.contains("fs:allow-"),
        "Must NOT grant unrestricted filesystem access"
    );
}
