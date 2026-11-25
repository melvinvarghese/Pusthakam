//! Real TCP boundary checks against an isolated library, never the owner's data.
use pusthakam_lib::{
    db, logging,
    paths::AppPaths,
    services::{auth, lan, staff_web::StaffWeb},
    state::AppState,
};
use serde_json::{json, Value};
use std::{
    io::{Read, Write},
    net::TcpStream,
    sync::{Arc, Mutex},
    time::Duration,
};

struct Host {
    _dir: tempfile::TempDir,
    state: AppState,
    server: lan::LanServerHandle,
}
impl Host {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::ensure(dir.path()).unwrap();
        let conn = db::open(&paths.database).unwrap();
        auth::create_user(&conn, "owner", "Owner", "test-password", "administrator").unwrap();
        auth::create_user(&conn, "reader", "Reader", "test-password", "viewer").unwrap();
        lan::get_or_init_config(&conn).unwrap();
        conn.execute(
            "UPDATE lan_config SET enabled=1, browser_staff_enabled=1",
            [],
        )
        .unwrap();
        pusthakam_lib::services::settings::set(&conn, "web.full_staff_enabled", "true").unwrap();
        let state = AppState {
            _log_guard: Arc::new(logging::init(&paths.logs)),
            paths,
            db: Arc::new(Mutex::new(conn)),
            lan: Arc::new(Mutex::new(Default::default())),
            session: Arc::new(Mutex::new(None)),
            pending_import: Arc::new(Mutex::new(None)),
        };
        let web = Arc::new(StaffWeb::new(&state));
        let server = lan::start_lan_listener_with_staff(
            state.paths.database.clone(),
            "127.0.0.1",
            0,
            Some(web),
        )
        .unwrap();
        Self {
            _dir: dir,
            state,
            server,
        }
    }
    fn request(&self, method: &str, path: &str, headers: &str, body: &str) -> String {
        self.request_host(
            &format!("127.0.0.1:{}", self.server.port),
            method,
            path,
            headers,
            body,
        )
    }
    fn request_host(
        &self,
        host: &str,
        method: &str,
        path: &str,
        headers: &str,
        body: &str,
    ) -> String {
        let mut stream = TcpStream::connect(("127.0.0.1", self.server.port)).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        write!(
            stream,
            "{method} {path} HTTP/1.1\r\nHost: {host}\r\nContent-Length: {}\r\n{headers}\r\n{body}",
            body.len()
        )
        .unwrap();
        let mut response = Vec::new();
        stream.read_to_end(&mut response).unwrap();
        String::from_utf8_lossy(&response).into_owned()
    }
    fn page(&self) -> (String, String) {
        let response = self.request("GET", "/staff", "", "");
        assert!(response.starts_with("HTTP/1.1 200"), "{response}");
        let cookie = header(&response, "set-cookie")
            .split(';')
            .next()
            .unwrap()
            .to_owned();
        let csrf = response
            .split("name=\"pusthakam-staff-csrf\" content=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap()
            .to_owned();
        assert!(header(&response, "set-cookie").contains("HttpOnly"));
        assert!(header(&response, "set-cookie").contains("SameSite=Strict"));
        (cookie, csrf)
    }
    fn command(&self, cookie: &str, csrf: &str, command: &str, args: Value) -> String {
        self.request("POST", "/api/staff/commands", &format!("Cookie: {cookie}\r\nX-CSRF-Token: {csrf}\r\nOrigin: http://127.0.0.1:{}\r\nContent-Type: application/json\r\n",self.server.port), &json!({"command":command,"args":args}).to_string())
    }
    fn login(&self, username: &str) -> (String, String) {
        let (cookie, csrf) = self.page();
        let response = self.command(
            &cookie,
            &csrf,
            "login",
            json!({"username":username,"password":"test-password"}),
        );
        assert!(response.starts_with("HTTP/1.1 200"), "{response}");
        let rotated = header(&response, "set-cookie")
            .split(';')
            .next()
            .unwrap()
            .to_owned();
        assert_ne!(cookie, rotated);
        let old = self.command(&cookie, &csrf, "current_session", json!({}));
        assert!(old.starts_with("HTTP/1.1 401"));
        (rotated, csrf)
    }
}

#[test]
fn public_staff_requires_configured_https_proxy_and_blocks_legacy_api() {
    let host = Host::new();
    host.state
        .with_conn(|conn| {
            pusthakam_lib::services::settings::set(
                conn,
                "web.public_url",
                "https://catalogue.example.org",
            )
        })
        .unwrap();
    assert!(host
        .request_host("catalogue.example.org", "GET", "/staff", "", "")
        .starts_with("HTTP/1.1 403"));
    assert!(host
        .request_host(
            "evil.example.org",
            "GET",
            "/staff",
            "X-Forwarded-Proto: https\r\n",
            ""
        )
        .starts_with("HTTP/1.1 403"));
    let page = host.request_host(
        "catalogue.example.org",
        "GET",
        "/staff",
        "X-Forwarded-Proto: https\r\n",
        "",
    );
    assert!(page.starts_with("HTTP/1.1 200"), "{page}");
    let cookie = header(&page, "set-cookie");
    assert!(cookie.starts_with("__Host-pusthakam_staff="));
    assert!(cookie.contains("; Secure"));
    assert!(!cookie.contains("Domain="));
    let csrf = page
        .split("name=\"pusthakam-staff-csrf\" content=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap();
    let headers = format!("X-Forwarded-Proto: https\r\nOrigin: https://catalogue.example.org\r\nCookie: {}\r\nContent-Type: application/json\r\nX-CSRF-Token: {csrf}\r\n", cookie.split(';').next().unwrap());
    let setup = host.request_host(
        "catalogue.example.org",
        "POST",
        "/api/staff/commands",
        &headers,
        &json!({"command":"complete_setup","args":{}}).to_string(),
    );
    assert_eq!(body(&setup)["error"]["code"], "PERMISSION_DENIED");
    assert!(host
        .request_host(
            "catalogue.example.org",
            "GET",
            "/api/lan/status",
            "X-Forwarded-Proto: https\r\n",
            ""
        )
        .starts_with("HTTP/1.1 403"));
}

#[test]
fn backup_downloads_require_admin_and_cannot_escape_saved_backup_directory() {
    let host = Host::new();
    let saved = host
        .state
        .with_conn(|conn| pusthakam_lib::services::backup::create(conn, &host.state.paths.backups))
        .unwrap();
    let name = serde_json::to_value(saved).unwrap()["name"]
        .as_str()
        .unwrap()
        .to_owned();
    let (owner, _) = host.login("owner");
    let (reader, _) = host.login("reader");
    let url = format!("/api/staff/backups/{name}");
    assert!(host
        .request("GET", &url, "", "")
        .starts_with("HTTP/1.1 401"));
    assert!(host
        .request("GET", &url, &format!("Cookie: {reader}\r\n"), "")
        .starts_with("HTTP/1.1 403"));
    let download = host.request("GET", &url, &format!("Cookie: {owner}\r\n"), "");
    assert!(download.starts_with("HTTP/1.1 200"), "{download}");
    assert!(download.contains("SQLite format 3"));
    assert!(header(&download, "content-disposition").contains(&name));
    assert!(host
        .request(
            "GET",
            "/api/staff/backups/../../library.sqlite3",
            &format!("Cookie: {owner}\r\n"),
            ""
        )
        .starts_with("HTTP/1.1 403"));
}

#[test]
fn public_address_updates_are_atomic_and_older_clients_preserve_the_address() {
    let host = Host::new();
    host.state
        .with_conn(|conn| {
            let owner = auth::login(conn, "owner", "test-password")?;
            let mut input: lan::UpdateLanConfigInput = serde_json::from_value(json!({
                "enabled":true,"bindAddress":"127.0.0.1","port":host.server.port,
                "serverName":"Web library","opacEnabled":true,"browserStaffEnabled":true,
                "publicBaseUrl":"https://catalogue.example.org/"
            }))
            .unwrap();
            let saved = lan::update_config(conn, &owner, &input)?;
            assert_eq!(
                saved.public_base_url.as_deref(),
                Some("https://catalogue.example.org")
            );
            input.public_base_url = None;
            assert_eq!(
                lan::update_config(conn, &owner, &input)?.public_base_url,
                saved.public_base_url
            );
            input.server_name = "Must not persist".into();
            input.public_base_url = Some("http://catalogue.example.org/staff".into());
            assert!(lan::update_config(conn, &owner, &input).is_err());
            assert_eq!(lan::get_or_init_config(conn)?.server_name, "Web library");
            Ok(())
        })
        .unwrap();
}

#[test]
fn accepted_requests_wait_for_separately_arriving_headers_and_body() {
    let host = Host::new();
    let (cookie, csrf) = host.page();
    let mut stream = TcpStream::connect(("127.0.0.1", host.server.port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let body = json!({"command":"current_session","args":{}}).to_string();
    std::thread::sleep(Duration::from_millis(100));
    write!(stream, "POST /api/staff/commands HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nCookie: {cookie}\r\nX-CSRF-Token: {csrf}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n", host.server.port, body.len()).unwrap();
    std::thread::sleep(Duration::from_millis(100));
    stream.write_all(body.as_bytes()).unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    assert!(body_value(&response)["result"].is_null());
}

fn body_value(response: &str) -> Value {
    body(response)
}
fn header<'a>(response: &'a str, name: &str) -> &'a str {
    response
        .split("\r\n")
        .find_map(|line| {
            line.split_once(':')
                .filter(|(key, _)| key.eq_ignore_ascii_case(name))
                .map(|(_, value)| value.trim())
        })
        .unwrap()
}
fn body(response: &str) -> Value {
    serde_json::from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap()
}

#[test]
fn browser_logins_rotate_cookies_and_keep_permissions_and_logout_separate() {
    let host = Host::new();
    let (owner, owner_csrf) = host.login("owner");
    let (reader, reader_csrf) = host.login("reader");
    let denied = host.command(
        &reader,
        &reader_csrf,
        "set_setting",
        json!({"key":"web.test","value":"no"}),
    );
    assert_eq!(body(&denied)["error"]["code"], "PERMISSION_DENIED");
    let saved = host.command(
        &owner,
        &owner_csrf,
        "set_setting",
        json!({"key":"web.test","value":"yes"}),
    );
    assert!(saved.starts_with("HTTP/1.1 200"), "{saved}");
    let read = host.command(
        &reader,
        &reader_csrf,
        "get_setting",
        json!({"key":"web.test"}),
    );
    assert_eq!(body(&read)["result"], "yes");
    host.command(&reader, &reader_csrf, "logout", json!({}));
    assert!(host
        .command(&reader, &reader_csrf, "current_session", json!({}))
        .starts_with("HTTP/1.1 401"));
    assert_eq!(
        body(&host.command(&owner, &owner_csrf, "current_session", json!({})))["result"]
            ["username"],
        "owner"
    );
    assert!(host.state.current().is_err());
}

#[test]
fn browser_gateway_rejects_missing_csrf_foreign_origins_and_anonymous_commands() {
    let host = Host::new();
    let (cookie, csrf) = host.page();
    assert!(host
        .command(&cookie, "wrong", "current_session", json!({}))
        .starts_with("HTTP/1.1 403"));
    assert!(host
        .command(&cookie, &csrf, "get_setting", json!({"key":"web.test"}))
        .starts_with("HTTP/1.1 401"));
    let foreign=host.request("POST","/api/staff/commands",&format!("Cookie: {cookie}\r\nX-CSRF-Token: {csrf}\r\nOrigin: https://untrusted.example\r\nContent-Type: application/json\r\n"),"{}");
    assert!(foreign.starts_with("HTTP/1.1 403"));
    assert!(host
        .request(
            "POST",
            "/api/staff/commands",
            "Content-Type: application/json\r\n",
            "{}"
        )
        .starts_with("HTTP/1.1 401"));
    let (owner, csrf) = host.login("owner");
    assert_eq!(
        body(&host.command(&owner, &csrf, "not_a_command", json!({})))["error"]["code"],
        "VALIDATION"
    );
}
