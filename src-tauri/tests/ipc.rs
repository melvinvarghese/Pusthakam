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
fn members_can_be_registered_found_updated_and_restored_through_ipc() {
    let h = Harness::new();
    let list = json!({ "search": "M%01", "page": 1, "archived": false });
    assert_eq!(h.code("members_list", list.clone()), "AUTH_FAILED");
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

    input["id"] = id.clone();
    input["name"] = json!("Asha Ravi");
    h.ok("members_save", json!({ "input": input.clone() }));
    h.ok("members_archive", json!({ "id": id, "archived": true }));
    assert_eq!(h.ok("members_list", list.clone())["total"], 0);
    let archived = h.ok("members_list", json!({ "search": "Ravi", "page": 1, "archived": true }));
    assert_eq!(archived["rows"][0]["id"], id);
    h.ok("members_archive", json!({ "id": id, "archived": false }));
    assert_eq!(h.ok("members_list", list.clone())["total"], 1);

    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        auth::create_user(&conn, "viewer", "Visitor", "visitor passphrase", "viewer").unwrap();
    }
    h.ok("login", json!({ "username": "viewer", "password": "visitor passphrase" }));
    assert_eq!(h.ok("members_list", list)["total"], 1);
    assert_eq!(h.code("members_save", json!({ "input": input })), "PERMISSION_DENIED");
    assert_eq!(h.code("members_archive", json!({ "id": id, "archived": true })), "PERMISSION_DENIED");
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
    assert_eq!(h.code("catalogue_set_copy_status", json!({ "id": copy_id, "status": "on_loan", "note": null })), "VALIDATION");
    assert_eq!(h.code("catalogue_add_copies", json!({ "recordId": id, "input": {
        "count": 1, "accessionNumbers": ["LIB-00100"], "barcodes": null, "itemCategoryId": category, "collectionId": null,
        "locationId": null, "shelfId": null, "callNumber": null, "acquisitionDate": null, "acquisitionSource": "purchase",
        "priceMinor": null, "vendor": null, "invoiceRef": null, "notes": null }})), "CONFLICT");

    // A clerk can read the catalogue but not change anything.
    {
        let state = h.webview.state::<AppState>();
        let conn = state.db.lock().unwrap();
        auth::create_user(&conn, "clerk", "Front Desk", "clerk passphrase!", "circulation").unwrap();
    }
    h.ok("logout", json!({}));
    h.ok("login", json!({ "username": "clerk", "password": "clerk passphrase!" }));
    assert_eq!(h.ok("catalogue_search", json!({ "query": { "text": "balya", "page": 1, "pageSize": 25 } }))["total"], 1);
    h.ok("catalogue_get_record", json!({ "id": id }));
    assert_eq!(h.code("catalogue_save_record", record), "PERMISSION_DENIED");
    assert_eq!(h.code("catalogue_archive_record", json!({ "id": id, "archived": true })), "PERMISSION_DENIED");
    assert_eq!(h.code("catalogue_set_copy_status", json!({ "id": copy_id, "status": "available", "note": null })), "PERMISSION_DENIED");
    assert_eq!(h.code("accession_format_set", json!({ "format": { "prefix": "", "padding": 6, "next": 1 } })), "PERMISSION_DENIED");
    // Signed-out callers get nothing.
    h.ok("logout", json!({}));
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
fn app_info_works_before_sign_in_and_reports_the_current_schema() {
    let h = Harness::new();
    let info = h.ok("app_info", json!({}));
    assert_eq!(info["schemaVersion"], 5);
    assert!(info["databasePath"].as_str().unwrap().ends_with("pusthakam.db"));
}
