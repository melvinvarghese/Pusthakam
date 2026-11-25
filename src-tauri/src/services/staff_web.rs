//! Shared staff application gateway. Library resources are shared; logins/previews are not.
use super::{
    auth::Session,
    import::Pending,
    lan_runtime::LanRuntime,
    web_http::{self, Head},
};
use crate::{
    error::{AppError, AppResult},
    paths::AppPaths,
    state::AppState,
};
use rand_core::RngCore;
use rusqlite::Connection;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    io::Write,
    net::{IpAddr, TcpStream},
    sync::{Arc, Mutex, Weak},
    time::{Duration, Instant},
};
use tracing_appender::non_blocking::WorkerGuard;

mod embedded {
    include!(concat!(env!("OUT_DIR"), "/staff_assets.rs"));
}
const PUBLIC_URL_KEY: &str = "web.public_url";
const LOCAL_COOKIE: &str = "pusthakam_staff_local";
const SECURE_COOKIE: &str = "__Host-pusthakam_staff";

struct BrowserSession {
    login: Arc<Mutex<Option<Session>>>,
    pending: Arc<Mutex<Option<Pending>>>,
    csrf: String,
    expires: Instant,
    authenticated: bool,
}
pub struct StaffWeb {
    paths: AppPaths,
    db: Arc<Mutex<Connection>>,
    runtime: Weak<Mutex<LanRuntime>>,
    log: Arc<WorkerGuard>,
    sessions: Mutex<HashMap<String, Arc<BrowserSession>>>,
    attempts: Mutex<HashMap<IpAddr, (Instant, u32)>>,
}

fn random_token() -> String {
    let mut bytes = [0u8; 32];
    rand_core::OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
pub(crate) fn private_peer(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => ip.is_loopback() || ip.is_private() || ip.is_link_local(),
        IpAddr::V6(ip) => {
            ip.is_loopback()
                || (ip.segments()[0] & 0xfe00 == 0xfc00)
                || (ip.segments()[0] & 0xffc0 == 0xfe80)
        }
    }
}

pub fn normalize_public_url(value: &str) -> AppResult<String> {
    if value.trim().is_empty() {
        return Ok(String::new());
    }
    let url = tauri::Url::parse(value.trim())
        .map_err(|_| AppError::Validation("Enter a complete HTTPS website address.".into()))?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
    {
        return Err(AppError::Validation(
            "Use an HTTPS website address without a path, query or login details.".into(),
        ));
    }
    Ok(url.origin().ascii_serialization())
}

impl StaffWeb {
    pub fn clear_sessions(&self) {
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.clear();
        }
    }
    pub fn ready(&self) -> bool {
        let Ok(conn) = self.db.lock() else {
            return false;
        };
        // Explicit false is an operator rollback; normal hosting serves the shared app.
        super::settings::get(&conn, "web.full_staff_enabled")
            .map(|value| value.as_deref() != Some("false"))
            .unwrap_or(false)
    }
    pub fn new(state: &AppState) -> Self {
        Self {
            paths: state.paths.clone(),
            db: Arc::clone(&state.db),
            runtime: Arc::downgrade(&state.lan),
            log: Arc::clone(&state._log_guard),
            sessions: Mutex::new(HashMap::new()),
            attempts: Mutex::new(HashMap::new()),
        }
    }
    fn settings(&self) -> AppResult<(bool, String)> {
        let conn = self
            .db
            .lock()
            .map_err(|_| AppError::Internal("database lock poisoned".into()))?;
        let config = super::lan::get_or_init_config(&conn)?;
        let public = super::settings::get(&conn, PUBLIC_URL_KEY)?.unwrap_or_default();
        Ok((
            config.enabled && config.browser_staff_enabled,
            normalize_public_url(&public)?,
        ))
    }
    fn context(&self, session: &BrowserSession) -> AppResult<AppState> {
        Ok(AppState {
            paths: self.paths.clone(),
            db: Arc::clone(&self.db),
            lan: self
                .runtime
                .upgrade()
                .ok_or_else(|| AppError::Internal("Library host stopped.".into()))?,
            session: Arc::clone(&session.login),
            pending_import: Arc::clone(&session.pending),
            _log_guard: Arc::clone(&self.log),
        })
    }
    fn request_origin(&self, stream: &TcpStream, head: &Head) -> AppResult<(bool, bool)> {
        let host = head
            .header("host")
            .ok_or_else(|| AppError::Permission("Missing host address.".into()))?;
        if host.contains(['@', '/', '\\']) {
            return Err(AppError::Permission("Host address not accepted.".into()));
        }
        let url = tauri::Url::parse(&format!("http://{host}"))
            .map_err(|_| AppError::Permission("Host address not accepted.".into()))?;
        let peer = stream.peer_addr()?.ip();
        let (_, public) = self.settings()?;
        let public_url = tauri::Url::parse(&public).ok();
        let is_public = public_url.as_ref().is_some_and(|p| {
            p.host_str() == url.host_str()
                && p.port_or_known_default()
                    == tauri::Url::parse(&format!("https://{host}"))
                        .ok()
                        .and_then(|u| u.port_or_known_default())
        });
        let secure =
            is_public && peer.is_loopback() && head.header("x-forwarded-proto") == Some("https");
        let name = url.host_str().unwrap_or("");
        let is_loopback = name == "localhost"
            || name
                .trim_matches(['[', ']'])
                .parse::<IpAddr>()
                .is_ok_and(|ip| ip.is_loopback());
        let local_name =
            is_loopback || super::lan::discover_local_ips().iter().any(|ip| ip == name);
        if !secure && (!private_peer(peer) || !local_name || (is_loopback && !peer.is_loopback())) {
            return Err(AppError::Permission(
                "Open the configured HTTPS website or the displayed library network address."
                    .into(),
            ));
        }
        if is_public && !secure {
            return Err(AppError::Permission(
                "Public staff access requires the configured HTTPS proxy.".into(),
            ));
        }
        if let Some(origin) = head.header("origin") {
            let expected = if secure {
                public
            } else {
                format!("http://{host}")
            };
            if origin != expected {
                return Err(AppError::Permission(
                    "Cross-origin staff requests are not accepted.".into(),
                ));
            }
        }
        Ok((secure, peer.is_loopback() && is_loopback && !is_public))
    }
    pub fn public_request(&self, head: &Head) -> bool {
        self.settings()
            .ok()
            .and_then(|(_, url)| tauri::Url::parse(&url).ok())
            .is_some_and(|url| {
                head.header("host")
                    .and_then(|host| tauri::Url::parse(&format!("https://{host}")).ok())
                    .is_some_and(|host| host.origin() == url.origin())
            })
    }
    fn cookie(head: &Head, secure: bool) -> Option<String> {
        let name = if secure { SECURE_COOKIE } else { LOCAL_COOKIE };
        head.header("cookie")?.split(';').find_map(|part| {
            let (key, value) = part.trim().split_once('=')?;
            (key == name && value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit()))
                .then(|| value.to_string())
        })
    }
    fn existing(
        &self,
        head: &Head,
        secure: bool,
    ) -> AppResult<Option<(String, Arc<BrowserSession>)>> {
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| AppError::Internal("Session lock poisoned".into()))?;
        sessions.retain(|_, session| session.expires > Instant::now());
        Ok(Self::cookie(head, secure)
            .and_then(|key| sessions.get(&key).map(|session| (key, Arc::clone(session)))))
    }
    fn create(
        &self,
        old: Option<&str>,
        login: Option<Session>,
        csrf: Option<String>,
    ) -> AppResult<(String, Arc<BrowserSession>)> {
        let authenticated = login.is_some();
        let session = Arc::new(BrowserSession {
            login: Arc::new(Mutex::new(login)),
            pending: Arc::new(Mutex::new(None)),
            csrf: csrf.unwrap_or_else(random_token),
            expires: Instant::now()
                + Duration::from_secs(if authenticated { 24 * 60 * 60 } else { 10 * 60 }),
            authenticated,
        });
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| AppError::Internal("Session lock poisoned".into()))?;
        sessions.retain(|_, value| value.expires > Instant::now());
        if let Some(old) = old {
            sessions.remove(old);
        }
        if !authenticated
            && sessions
                .values()
                .filter(|value| !value.authenticated)
                .count()
                >= 4096
        {
            if let Some(old) = sessions
                .iter()
                .filter(|(_, value)| !value.authenticated)
                .min_by_key(|(_, value)| value.expires)
                .map(|(key, _)| key.clone())
            {
                sessions.remove(&old);
            }
        }
        let key = random_token();
        sessions.insert(key.clone(), Arc::clone(&session));
        Ok((key, session))
    }
    fn response_headers(
        key: &str,
        session: &BrowserSession,
        secure: bool,
    ) -> Vec<(&'static str, String)> {
        let name = if secure { SECURE_COOKIE } else { LOCAL_COOKIE };
        vec![
            (
                "Set-Cookie",
                format!(
                    "{name}={key}; Path=/; HttpOnly; SameSite=Strict{}",
                    if secure { "; Secure" } else { "" }
                ),
            ),
            ("X-CSRF-Token", session.csrf.clone()),
        ]
    }
    pub fn body_limit(&self, stream: &TcpStream, head: &Head) -> usize {
        if head.path() != "/api/staff/commands" {
            return 16 * 1024;
        }
        self.request_origin(stream, head)
            .ok()
            .and_then(|(secure, _)| self.existing(head, secure).ok().flatten())
            .filter(|(_, session)| {
                session.authenticated
                    && self
                        .context(session)
                        .is_ok_and(|state| state.current().is_ok())
            })
            .map_or(16 * 1024, |_| web_http::MAX_BODY)
    }
    fn attempt(&self, peer: IpAddr) -> AppResult<()> {
        let mut attempts = self
            .attempts
            .lock()
            .map_err(|_| AppError::Internal("Rate lock poisoned".into()))?;
        attempts.retain(|_, (at, _)| at.elapsed() < Duration::from_secs(60));
        let entry = attempts.entry(peer).or_insert((Instant::now(), 0));
        entry.1 += 1;
        if entry.1 > 20 {
            return Err(AppError::Permission(
                "Too many sign-in attempts. Wait a minute and try again.".into(),
            ));
        }
        Ok(())
    }
    fn json(
        stream: &mut TcpStream,
        status: u16,
        result: AppResult<Value>,
        headers: &[(&str, String)],
    ) -> std::io::Result<()> {
        let body = match result {
            Ok(result) => json!({"result":result}),
            Err(error) => json!({"error":error}),
        };
        web_http::respond(
            stream,
            status,
            "application/json; charset=utf-8",
            body.to_string().as_bytes(),
            headers,
        )
    }
    pub fn handles(path: &str) -> bool {
        path == "/staff"
            || path == "/staff/"
            || path.starts_with("/assets/")
            || path.starts_with("/api/staff/commands")
            || path.starts_with("/api/staff/backups/")
    }
    pub fn handle(&self, stream: &mut TcpStream, head: &Head, body: &[u8]) -> std::io::Result<()> {
        if head.path().starts_with("/assets/") {
            if head.method != "GET" {
                return web_http::reject(stream, 405);
            }
            return match embedded::asset(head.path()) {
                Some((mime, data)) => web_http::respond(stream, 200, mime, data, &[]),
                None => web_http::reject(stream, 404),
            };
        }
        let (enabled, _) = match self.settings() {
            Ok(config) => config,
            Err(error) => return Self::json(stream, 503, Err(error), &[]),
        };
        if !enabled {
            return Self::json(
                stream,
                403,
                Err(AppError::Permission(
                    "Browser staff access is disabled.".into(),
                )),
                &[],
            );
        }
        let (secure, local_setup) = match self.request_origin(stream, head) {
            Ok(origin) => origin,
            Err(error) => return Self::json(stream, 403, Err(error), &[]),
        };
        if head.path() == "/staff" || head.path() == "/staff/" {
            if head.method != "GET" {
                return web_http::reject(stream, 405);
            }
            let Some((_, html)) = embedded::asset("/index.html") else {
                return Self::json(
                    stream,
                    503,
                    Err(AppError::Internal(
                        "Staff application assets have not been built.".into(),
                    )),
                    &[],
                );
            };
            let existing = self.existing(head, secure).ok().flatten();
            let (key, session) = match existing
                .map(Ok)
                .unwrap_or_else(|| self.create(None, None, None))
            {
                Ok(value) => value,
                Err(error) => return Self::json(stream, 503, Err(error), &[]),
            };
            let markers=format!("<meta name=\"pusthakam-staff-api\" content=\"/api/staff/commands\"><meta name=\"pusthakam-staff-csrf\" content=\"{}\">",session.csrf);
            let html =
                String::from_utf8_lossy(html).replacen("</head>", &format!("{markers}</head>"), 1);
            let mut headers = Self::response_headers(&key, &session, secure);
            headers.push(("Content-Security-Policy","default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob: https://covers.openlibrary.org; font-src 'self' data:; connect-src 'self' https://covers.openlibrary.org; object-src 'none'; base-uri 'none'; frame-ancestors 'none'".into()));
            return web_http::respond(
                stream,
                200,
                "text/html; charset=utf-8",
                html.as_bytes(),
                &headers,
            );
        }
        let (key, session) = match self.existing(head, secure) {
            Ok(Some(value)) => value,
            _ => {
                let fresh = self.create(None, None, None);
                let headers = fresh
                    .as_ref()
                    .map(|(key, session)| Self::response_headers(key, session, secure))
                    .unwrap_or_default();
                return Self::json(
                    stream,
                    401,
                    Err(AppError::Auth("Your session ended. Sign in again.".into())),
                    &headers,
                );
            }
        };
        let context = match self.context(&session) {
            Ok(context) => context,
            Err(error) => return Self::json(stream, 503, Err(error), &[]),
        };
        if let Some(name) = head.path().strip_prefix("/api/staff/backups/") {
            if head.method != "GET" {
                return web_http::reject(stream, 405);
            }
            let download = context
                .require("admin.backup")
                .and_then(|_| super::backup::download_file(&context.paths, name));
            return match download {
                Ok(mut file) => {
                    let size = file.metadata()?.len();
                    let response=format!("HTTP/1.1 200 OK\r\nContent-Type: application/vnd.sqlite3\r\nContent-Length: {size}\r\nContent-Disposition: attachment; filename=\"{name}\"\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\n");
                    stream.write_all(response.as_bytes())?;
                    std::io::copy(&mut file, stream)?;
                    Ok(())
                }
                Err(error) => Self::json(stream, 403, Err(error), &[]),
            };
        }
        if head.path() != "/api/staff/commands" {
            return web_http::reject(stream, 404);
        }
        if head.method != "POST" {
            return web_http::reject(stream, 405);
        }
        if head.header("x-csrf-token") != Some(session.csrf.as_str())
            || head
                .header("content-type")
                .map_or(true, |value| !value.starts_with("application/json"))
        {
            return Self::json(
                stream,
                403,
                Err(AppError::Permission(
                    "Staff request verification failed. Reload this page.".into(),
                )),
                &[],
            );
        }
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Command {
            command: String,
            #[serde(default = "empty_arguments")]
            args: Value,
        }
        fn empty_arguments() -> Value {
            json!({})
        }
        let input: Command = match serde_json::from_slice(body) {
            Ok(input) => input,
            Err(_) => {
                return Self::json(
                    stream,
                    400,
                    Err(AppError::Validation("Invalid staff request.".into())),
                    &[],
                )
            }
        };
        if ["login", "staff_recover", "complete_setup"].contains(&input.command.as_str()) {
            let peer = if secure {
                head.header("x-forwarded-for")
                    .and_then(|value| value.rsplit(',').next())
                    .and_then(|value| value.trim().parse::<IpAddr>().ok())
                    .unwrap_or(stream.peer_addr()?.ip())
            } else {
                stream.peer_addr()?.ip()
            };
            if let Err(error) = self.attempt(peer) {
                return Self::json(stream, 429, Err(error), &[]);
            }
        }
        let result =
            super::staff_dispatch::dispatch(&context, &input.command, input.args, local_setup);
        let mut headers = Vec::new();
        if result.is_ok() && ["login", "logout", "complete_setup"].contains(&input.command.as_str())
        {
            let login = context.current().ok();
            if let Ok((new_key, new_session)) =
                self.create(Some(&key), login, Some(session.csrf.clone()))
            {
                headers = Self::response_headers(&new_key, &new_session, secure);
            }
        }
        let status = match &result {
            Ok(_) => 200,
            Err(AppError::Auth(_)) => 401,
            Err(AppError::Permission(_)) => 403,
            Err(AppError::Validation(_) | AppError::Conflict(_) | AppError::NotFound(_)) => 400,
            Err(_) => 500,
        };
        Self::json(stream, status, result, &headers)
    }
}
