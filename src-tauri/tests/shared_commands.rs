//! Request contexts reuse the desktop command implementation without a WebView or shared login.
use pusthakam_lib::{commands, db, logging, paths::AppPaths, services::auth, state::AppState};
use serde_json::json;
use std::sync::{Arc, Mutex};

fn host() -> (tempfile::TempDir, AppState, auth::Session, auth::Session) {
    let dir = tempfile::tempdir().unwrap();
    let paths = AppPaths::ensure(dir.path()).unwrap();
    let conn = db::open(&paths.database).unwrap();
    auth::create_user(&conn, "owner", "Owner", "test-password", "administrator").unwrap();
    auth::create_user(&conn, "reader", "Reader", "test-password", "viewer").unwrap();
    let owner = auth::login(&conn, "owner", "test-password").unwrap();
    let reader = auth::login(&conn, "reader", "test-password").unwrap();
    let state = AppState {
        _log_guard: Arc::new(logging::init(&paths.logs)),
        paths,
        db: Arc::new(Mutex::new(conn)),
        lan: Arc::new(Mutex::new(Default::default())),
        session: Arc::new(Mutex::new(Some(owner.clone()))),
        pending_import: std::sync::Arc::new(Mutex::new(None)),
    };
    (dir, state, owner, reader)
}

#[test]
fn shared_handler_keeps_permissions_and_data_contract_independent_of_desktop() {
    let (_dir, host, owner, reader) = host();
    let viewer = host.independent_session(Some(reader));
    let writer = host.independent_session(Some(owner));
    let args = json!({"key":"browser.test", "value":"shared-data"});
    assert_eq!(
        commands::app::set_setting::dispatch(&viewer, args.clone())
            .unwrap_err()
            .code(),
        "PERMISSION_DENIED"
    );
    commands::app::set_setting::dispatch(&writer, args).unwrap();
    assert_eq!(
        commands::app::get_setting::dispatch(&viewer, json!({"key":"browser.test"})).unwrap(),
        json!("shared-data")
    );
    commands::auth::logout::dispatch(&viewer, json!({})).unwrap();
    assert_eq!(
        commands::app::get_setting::dispatch(&viewer, json!({"key":"browser.test"}))
            .unwrap_err()
            .code(),
        "AUTH_FAILED"
    );
    assert!(host.current().is_ok());
    assert!(writer.current().is_ok());
}

#[test]
fn shared_decoder_uses_camel_case_and_rejects_unknown_arguments() {
    let (_dir, host, _owner, _reader) = host();
    assert_eq!(
        commands::app::get_setting::dispatch(
            &host,
            json!({"key":"browser.test", "unexpected":true})
        )
        .unwrap_err()
        .code(),
        "VALIDATION"
    );
    assert_eq!(
        commands::catalogue::catalogue_set_cover::dispatch(
            &host,
            json!({"id":999999,"pngBase64":null})
        )
        .unwrap_err()
        .code(),
        "NOT_FOUND"
    );
    assert_eq!(
        commands::catalogue::catalogue_set_cover::dispatch(
            &host,
            json!({"id":999999,"png_base64":null})
        )
        .unwrap_err()
        .code(),
        "VALIDATION"
    );
}

#[test]
fn shared_import_cancellation_cannot_clear_another_browser_preview() {
    let (_dir, host, owner, _reader) = host();
    let first = host.independent_session(Some(owner.clone()));
    let second = host.independent_session(Some(owner));
    *first.pending_import.lock().unwrap() = Some(pusthakam_lib::services::import::Pending {
        token: "first-browser".into(),
        source: "test".into(),
        records: vec![],
    });
    commands::import::import_cancel::dispatch(&second, json!({})).unwrap();
    assert_eq!(
        first.pending_import.lock().unwrap().as_ref().unwrap().token,
        "first-browser"
    );
    assert!(host.pending_import.lock().unwrap().is_none());
}
