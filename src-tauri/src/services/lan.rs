//! LAN multi-computer server, pairing codes, client management, and public OPAC services.

use crate::error::{AppError, AppResult};
use crate::services::auth::{self, Session};
use base64::Engine;
use rand_core::RngCore;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
#[cfg(test)]
use std::io::Read;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanConfig {
    pub enabled: bool,
    pub bind_address: String,
    pub port: u16,
    pub server_name: String,
    pub server_fingerprint: String,
    pub opac_enabled: bool,
    pub browser_staff_enabled: bool,
    #[serde(default)]
    pub public_base_url: Option<String>,
    pub active_interfaces: Vec<String>,
    #[serde(default)]
    pub listener_running: bool,
    #[serde(default)]
    pub listener_error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLanConfigInput {
    pub enabled: bool,
    pub bind_address: String,
    pub port: u16,
    pub server_name: String,
    pub opac_enabled: bool,
    pub browser_staff_enabled: bool,
    #[serde(default)]
    pub public_base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanPairingCode {
    pub code: String,
    pub expires_at: String,
    pub created_at: String,
    pub is_valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairedClient {
    pub client_id: String,
    pub client_name: String,
    pub device_kind: String,
    pub ip_address: Option<String>,
    pub paired_at: String,
    pub last_seen_at: String,
    pub revoked_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairRequest {
    pub pairing_code: String,
    pub client_name: String,
    pub device_kind: Option<String>,
    pub client_ip: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PairResponse {
    pub client_id: String,
    pub server_name: String,
    pub server_fingerprint: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanLoginRequest {
    pub client_id: String,
    pub server_fingerprint: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanLoginResponse {
    pub token: String,
    pub expires_at: String,
    pub user_id: i64,
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicOpacItem {
    pub id: i64,
    pub title: String,
    pub subtitle: Option<String>,
    pub authors: Vec<String>,
    pub publisher: Option<String>,
    pub year: Option<i64>,
    pub isbn: Option<String>,
    pub categories: Vec<String>,
    pub subjects: Vec<String>,
    /// Shelf call numbers of the copies, shown only in the detail view as "Where to find it".
    pub call_numbers: Vec<String>,
    pub available_copies: i64,
    pub total_copies: i64,
    pub locations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicOpacSearchResult {
    pub items: Vec<PublicOpacItem>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffIssueRequest {
    pub member_number: String,
    pub barcode: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffIssueResponse {
    pub success: bool,
    pub member_name: String,
    pub title: String,
    pub due_on: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffReturnRequest {
    pub barcode: String,
    pub condition: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffReturnResponse {
    pub success: bool,
    pub member_name: String,
    pub title: String,
    pub due_on: String,
    pub has_fine: bool,
}

// ============================================================================
// Network IP Discovery
// ============================================================================

pub fn discover_local_ips() -> Vec<String> {
    let mut ips = crate::services::network_addresses::local_ipv4_addresses();
    if !ips.contains(&"127.0.0.1".to_string()) {
        ips.push("127.0.0.1".to_string());
    }
    ips
}

// ============================================================================
// Configuration & Lifecycle
// ============================================================================

pub fn get_or_init_config(conn: &Connection) -> AppResult<LanConfig> {
    let mut row = conn.query_row(
        "SELECT enabled, bind_address, port, server_name, server_fingerprint, opac_enabled, browser_staff_enabled
         FROM lan_config WHERE id = 1",
        [],
        |r| {
            Ok((
                r.get::<_, i64>(0)? != 0,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)? as u16,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, i64>(5)? != 0,
                r.get::<_, i64>(6)? != 0,
            ))
        },
    )?;

    if row.4 == "initial-unconfigured-fingerprint" {
        let mut random_bytes = [0u8; 32];
        rand_core::OsRng.fill_bytes(&mut random_bytes);
        let mut hasher = Sha256::new();
        hasher.update(random_bytes);
        hasher.update(row.3.as_bytes());
        let hash = hasher.finalize();
        let fingerprint = hash
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<Vec<_>>()
            .join("");
        let formatted_fingerprint = fingerprint[..32].to_uppercase();

        conn.execute(
            "UPDATE lan_config SET server_fingerprint = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = 1",
            [&formatted_fingerprint],
        )?;
        row.4 = formatted_fingerprint;
    }

    Ok(LanConfig {
        enabled: row.0,
        bind_address: row.1,
        port: row.2,
        server_name: row.3,
        server_fingerprint: row.4,
        opac_enabled: row.5,
        browser_staff_enabled: row.6,
        public_base_url: crate::services::settings::get(conn, "web.public_url")?
            .filter(|value| !value.is_empty()),
        active_interfaces: discover_local_ips(),
        listener_running: false,
        listener_error: None,
    })
}

pub fn update_config(
    conn: &Connection,
    session: &Session,
    input: &UpdateLanConfigInput,
) -> AppResult<LanConfig> {
    session.require("lan.manage")?;

    if input.port < 1024 {
        return Err(AppError::Validation("Port must be at least 1024.".into()));
    }
    if input.server_name.trim().is_empty() {
        return Err(AppError::Validation("Server name cannot be empty.".into()));
    }

    let public_url = input
        .public_base_url
        .as_deref()
        .map(crate::services::staff_web::normalize_public_url)
        .transpose()?;
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE lan_config SET
            enabled = ?1,
            bind_address = ?2,
            port = ?3,
            server_name = ?4,
            opac_enabled = ?5,
            browser_staff_enabled = ?6,
            updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE id = 1",
        params![
            if input.enabled { 1 } else { 0 },
            input.bind_address.trim(),
            input.port as i64,
            input.server_name.trim(),
            if input.opac_enabled { 1 } else { 0 },
            if input.browser_staff_enabled { 1 } else { 0 },
        ],
    )?;

    if let Some(url) = public_url {
        crate::services::settings::set(&tx, "web.public_url", &url)?;
    }
    tx.commit()?;
    get_or_init_config(conn)
}

// ============================================================================
// Pairing Codes
// ============================================================================

pub fn generate_pairing_code(conn: &Connection, session: &Session) -> AppResult<LanPairingCode> {
    session.require("lan.manage")?;

    let mut random_bytes = [0u8; 6];
    rand_core::OsRng.fill_bytes(&mut random_bytes);
    const CHARSET: &[u8] = b"23456789ABCDEFGHJKLMNPQRSTUVWXYZ";
    let code: String = random_bytes
        .iter()
        .map(|&b| CHARSET[(b as usize) % CHARSET.len()] as char)
        .collect();

    let expires_at: String = conn.query_row(
        "SELECT strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '+15 minutes')",
        [],
        |r| r.get(0),
    )?;

    let created_at: String =
        conn.query_row("SELECT strftime('%Y-%m-%dT%H:%M:%SZ', 'now')", [], |r| {
            r.get(0)
        })?;

    conn.execute(
        "INSERT INTO lan_pairing_codes (code, created_by_staff_id, expires_at, created_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![code, session.user_id, expires_at, created_at],
    )?;

    Ok(LanPairingCode {
        code,
        expires_at,
        created_at,
        is_valid: true,
    })
}

pub fn list_pairing_codes(conn: &Connection, session: &Session) -> AppResult<Vec<LanPairingCode>> {
    session.require("lan.read")?;
    let mut stmt = conn.prepare(
        "SELECT code, expires_at, created_at, (used_at IS NULL AND expires_at > strftime('%Y-%m-%dT%H:%M:%SZ', 'now')) as is_valid
         FROM lan_pairing_codes
         ORDER BY created_at DESC LIMIT 20"
    )?;

    let rows = stmt.query_map([], |r| {
        Ok(LanPairingCode {
            code: r.get(0)?,
            expires_at: r.get(1)?,
            created_at: r.get(2)?,
            is_valid: r.get::<_, i64>(3)? != 0,
        })
    })?;

    let mut codes = Vec::new();
    for row in rows {
        codes.push(row?);
    }
    Ok(codes)
}

// ============================================================================
// Client Pairing & Management
// ============================================================================

pub fn pair_client(conn: &Connection, req: &PairRequest) -> AppResult<PairResponse> {
    let config = get_or_init_config(conn)?;
    if !config.enabled {
        return Err(AppError::Conflict(
            "LAN hosting is currently disabled on this server.".into(),
        ));
    }

    let code = req.pairing_code.trim().to_uppercase();
    let valid_code: Option<String> = conn.query_row(
        "SELECT code FROM lan_pairing_codes
         WHERE code = ?1 AND used_at IS NULL AND expires_at > strftime('%Y-%m-%dT%H:%M:%SZ', 'now')",
        [&code],
        |r| r.get(0),
    ).ok();

    if valid_code.is_none() {
        return Err(AppError::Validation(
            "Invalid, expired or already used pairing code.".into(),
        ));
    }

    let mut id_bytes = [0u8; 16];
    rand_core::OsRng.fill_bytes(&mut id_bytes);
    let client_id = id_bytes
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join("");

    conn.execute(
        "UPDATE lan_pairing_codes SET used_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), used_by_client_id = ?1
         WHERE code = ?2",
        params![client_id, code],
    )?;

    let name = if req.client_name.trim().is_empty() {
        "Connected Device".to_string()
    } else {
        req.client_name.trim().to_string()
    };

    let kind = req.device_kind.as_deref().unwrap_or("desktop");

    conn.execute(
        "INSERT INTO lan_paired_clients (client_id, client_name, device_kind, ip_address, pairing_code_used)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![client_id, name, kind, req.client_ip, code],
    )?;

    Ok(PairResponse {
        client_id,
        server_name: config.server_name,
        server_fingerprint: config.server_fingerprint,
    })
}

pub fn list_paired_clients(conn: &Connection, session: &Session) -> AppResult<Vec<PairedClient>> {
    session.require("lan.read")?;
    let mut stmt = conn.prepare(
        "SELECT client_id, client_name, device_kind, ip_address, paired_at, last_seen_at, revoked_at
         FROM lan_paired_clients
         ORDER BY paired_at DESC"
    )?;

    let rows = stmt.query_map([], |r| {
        Ok(PairedClient {
            client_id: r.get(0)?,
            client_name: r.get(1)?,
            device_kind: r.get(2)?,
            ip_address: r.get(3)?,
            paired_at: r.get(4)?,
            last_seen_at: r.get(5)?,
            revoked_at: r.get(6)?,
        })
    })?;

    let mut clients = Vec::new();
    for row in rows {
        clients.push(row?);
    }
    Ok(clients)
}

pub fn revoke_client(conn: &Connection, session: &Session, client_id: &str) -> AppResult<()> {
    session.require("lan.manage")?;

    conn.execute(
        "UPDATE lan_paired_clients SET revoked_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE client_id = ?1",
        [client_id],
    )?;

    conn.execute(
        "UPDATE lan_sessions SET revoked_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE client_id = ?1",
        [client_id],
    )?;

    Ok(())
}

// ============================================================================
// LAN Staff Sessions
// ============================================================================

pub fn lan_login(conn: &Connection, req: &LanLoginRequest) -> AppResult<LanLoginResponse> {
    let config = get_or_init_config(conn)?;
    if !config.enabled {
        return Err(AppError::Conflict(
            "LAN hosting is currently disabled on this server.".into(),
        ));
    }

    if req.server_fingerprint != config.server_fingerprint {
        return Err(AppError::Validation(
            "Server fingerprint mismatch. Please re-pair this device.".into(),
        ));
    }

    let client_status: Result<(Option<String>,), _> = conn.query_row(
        "SELECT revoked_at FROM lan_paired_clients WHERE client_id = ?1",
        [&req.client_id],
        |r| Ok((r.get(0)?,)),
    );

    match client_status {
        Ok((Some(_),)) => {
            return Err(AppError::Permission(
                "This device has been revoked by the library administrator.".into(),
            ));
        }
        Err(_) => {
            return Err(AppError::Permission(
                "Device is not paired with this server.".into(),
            ));
        }
        _ => {}
    }

    let session = auth::login(conn, &req.username, &req.password)?;

    let mut token_bytes = [0u8; 32];
    rand_core::OsRng.fill_bytes(&mut token_bytes);
    let token = token_bytes
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join("");

    let expires_at: String = conn.query_row(
        "SELECT strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '+24 hours')",
        [],
        |r| r.get(0),
    )?;

    conn.execute(
        "INSERT INTO lan_sessions (token, client_id, staff_user_id, is_opac, expires_at)
         VALUES (?1, ?2, ?3, 0, ?4)",
        params![token, req.client_id, session.user_id, expires_at],
    )?;

    conn.execute(
        "UPDATE lan_paired_clients SET last_seen_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE client_id = ?1",
        [&req.client_id],
    )?;

    let mut perms: Vec<String> = session.permissions.into_iter().collect();
    perms.sort();

    Ok(LanLoginResponse {
        token,
        expires_at,
        user_id: session.user_id,
        username: session.username,
        display_name: session.display_name,
        role: session.role,
        permissions: perms,
    })
}

pub fn verify_lan_session(conn: &Connection, token: &str) -> AppResult<(Session, String)> {
    let session_row: Result<(i64, String, Option<String>, Option<String>), _> = conn.query_row(
        "SELECT s.staff_user_id, s.client_id, s.revoked_at, c.revoked_at
         FROM lan_sessions s
         JOIN lan_paired_clients c ON c.client_id = s.client_id
         WHERE s.token = ?1 AND s.expires_at > strftime('%Y-%m-%dT%H:%M:%SZ', 'now')",
        [token],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
    );

    let (user_id, client_id, session_revoked, client_revoked) = match session_row {
        Ok(row) => row,
        Err(_) => {
            return Err(AppError::Auth(
                "Invalid or expired session. Please sign in again.".into(),
            ))
        }
    };

    if session_revoked.is_some() || client_revoked.is_some() {
        return Err(AppError::Permission(
            "This session or device has been revoked.".into(),
        ));
    }

    conn.execute(
        "UPDATE lan_sessions SET last_active_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE token = ?1",
        [token],
    )?;

    let session = auth::load_session(conn, user_id)?;
    Ok((session, client_id))
}

pub fn revoke_session(conn: &Connection, token: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE lan_sessions SET revoked_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE token = ?1",
        [token],
    )?;
    Ok(())
}

// ============================================================================
// Public Anonymous OPAC
// ============================================================================

pub fn opac_cover(conn: &Connection, id: i64) -> AppResult<Option<Vec<u8>>> {
    use rusqlite::OptionalExtension;
    let config = get_or_init_config(conn)?;
    if !config.enabled || !config.opac_enabled {
        return Err(AppError::Permission(
            "Public OPAC access is currently disabled.".into(),
        ));
    }
    Ok(conn.query_row(
        "SELECT c.png FROM book_covers c JOIN bibliographic_records r ON r.id = c.record_id WHERE r.id = ?1 AND r.archived_at IS NULL",
        [id], |row| row.get(0),
    ).optional()?)
}

pub fn opac_search(
    conn: &Connection,
    query_text: &str,
    page: i64,
    page_size: i64,
) -> AppResult<PublicOpacSearchResult> {
    opac_search_filtered(conn, query_text, page, page_size, None, false)
}

pub fn opac_search_filtered(
    conn: &Connection,
    query_text: &str,
    page: i64,
    page_size: i64,
    category: Option<i64>,
    available_only: bool,
) -> AppResult<PublicOpacSearchResult> {
    let config = get_or_init_config(conn)?;
    if !config.enabled || !config.opac_enabled {
        return Err(AppError::Permission(
            "Public OPAC access is currently disabled.".into(),
        ));
    }
    let category = category.unwrap_or(0);
    if category < 0 {
        return Err(AppError::Validation("Choose a valid category.".into()));
    }
    let p = page.max(1);
    let ps = page_size.clamp(1, 50);
    let offset = (p - 1).saturating_mul(ps);
    let query = query_text.trim();
    let limited: String = query.chars().take(300).collect();
    let fts = crate::services::catalogue::fts_expression(&limited).unwrap_or_default();
    if !query.is_empty() && fts.is_empty() {
        return Ok(PublicOpacSearchResult {
            items: vec![],
            total: 0,
            page: p,
            page_size: ps,
        });
    }
    // Apply filters before counting and pagination, never to just the visible page.
    let predicate = "r.archived_at IS NULL
        AND (?1 = '' OR r.id IN (SELECT rowid FROM catalogue_fts WHERE catalogue_fts MATCH ?1))
        AND (?2 = 0 OR r.id IN (SELECT rc.record_id FROM record_categories rc
                 JOIN browse_categories c ON c.id = rc.category_id AND c.archived_at IS NULL
                 WHERE rc.category_id = ?2 AND rc.source IN ('rule','manual')))
        AND (?3 = 0 OR EXISTS (SELECT 1 FROM physical_items pi WHERE pi.record_id = r.id AND pi.status = 'available'))";
    // Empty MATCH expressions are invalid in SQLite even inside an OR. Omit the FTS subquery entirely.
    let predicate = if fts.is_empty() {
        predicate.replace(
            "(?1 = '' OR r.id IN (SELECT rowid FROM catalogue_fts WHERE catalogue_fts MATCH ?1))",
            "?1 = ''",
        )
    } else {
        predicate.to_string()
    };
    let total: i64 = conn.query_row(
        &format!("SELECT COUNT(*) FROM bibliographic_records r WHERE {predicate}"),
        params![fts, category, available_only],
        |r| r.get(0),
    )?;
    let mut stmt = conn.prepare(&format!(
        "SELECT r.id FROM bibliographic_records r WHERE {predicate} ORDER BY r.title COLLATE NOCASE ASC, r.id ASC LIMIT ?4 OFFSET ?5"
    ))?;
    let rows = stmt.query_map(params![fts, category, available_only, ps, offset], |r| {
        r.get::<_, i64>(0)
    })?;
    let mut items = Vec::new();
    for id in rows {
        if let Some(item) = opac_get_record(conn, id?)? {
            items.push(item);
        }
    }
    Ok(PublicOpacSearchResult {
        items,
        total,
        page: p,
        page_size: ps,
    })
}

#[allow(clippy::type_complexity)]
pub fn opac_get_record(conn: &Connection, id: i64) -> AppResult<Option<PublicOpacItem>> {
    let row: Result<(String, Option<String>, Option<String>, Option<i64>, Option<String>, Option<String>), _> = conn.query_row(
        "SELECT title, subtitle, publisher, publication_year, COALESCE(isbn13, isbn10) as isbn, classification_number
         FROM bibliographic_records WHERE id = ?1 AND archived_at IS NULL",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)),
    );

    let (title, subtitle, publisher, year, isbn, class_number) = match row {
        Ok(r) => r,
        Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
        Err(e) => return Err(AppError::Database(e)),
    };

    // Contributors
    let mut contrib_stmt = conn.prepare(
        "SELECT c.name FROM record_contributors rc
         JOIN contributors c ON c.id = rc.contributor_id
         WHERE rc.record_id = ?1 ORDER BY rc.position ASC",
    )?;
    let author_rows = contrib_stmt.query_map([id], |r| r.get::<_, String>(0))?;
    let mut authors = Vec::new();
    for a in author_rows {
        authors.push(a?);
    }

    // Subjects
    let mut subj_stmt = conn.prepare(
        "SELECT s.heading FROM record_subjects rs
         JOIN subjects s ON s.id = rs.subject_id
         WHERE rs.record_id = ?1 ORDER BY s.heading ASC",
    )?;
    let subj_rows = subj_stmt.query_map([id], |r| r.get::<_, String>(0))?;
    let mut subjects = Vec::new();
    for s in subj_rows {
        subjects.push(s?);
    }

    // Physical copies availability (only public counts)
    let (total_copies, available_copies): (i64, i64) = conn
        .query_row(
            "SELECT
            COUNT(*),
            COALESCE(SUM(CASE WHEN status = 'available' THEN 1 ELSE 0 END), 0)
         FROM physical_items
         WHERE record_id = ?1 AND status NOT IN ('withdrawn', 'deleted')",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap_or((0, 0));

    // Shelving locations
    let mut loc_stmt = conn.prepare(
        "SELECT DISTINCT l.name FROM physical_items pi
         JOIN locations l ON l.id = pi.location_id
         WHERE pi.record_id = ?1 ORDER BY l.name ASC",
    )?;
    let loc_rows = loc_stmt.query_map([id], |r| r.get::<_, String>(0))?;
    let mut locations = Vec::new();
    for l in loc_rows {
        locations.push(l?);
    }

    let categories = crate::services::browse_categories::record_categories(conn, id)?
        .into_iter()
        .map(|c| c.name)
        .collect();

    // Call numbers members can use on the shelf. Falls back to the record's class number.
    let mut cn_stmt = conn.prepare(
        "SELECT DISTINCT trim(call_number) FROM physical_items
         WHERE record_id = ?1 AND call_number IS NOT NULL AND trim(call_number) <> ''
           AND status NOT IN ('withdrawn', 'lost')
         ORDER BY 1",
    )?;
    let mut call_numbers = cn_stmt
        .query_map([id], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    if call_numbers.is_empty() {
        if let Some(n) = class_number
            .map(|n| n.trim().to_string())
            .filter(|n| !n.is_empty())
        {
            call_numbers.push(n);
        }
    }

    Ok(Some(PublicOpacItem {
        id,
        title,
        subtitle,
        authors,
        publisher,
        year,
        isbn,
        categories,
        subjects,
        call_numbers,
        available_copies,
        total_copies,
        locations,
    }))
}

// ============================================================================
// Embedded HTTP Server
// ============================================================================

pub struct LanServerHandle {
    shutdown: Arc<AtomicBool>,
    active: Arc<std::sync::atomic::AtomicUsize>,
    thread: Option<thread::JoinHandle<()>>,
    pub port: u16,
}

impl LanServerHandle {
    /// Only the host shutdown path waits for workers; never call from a request worker.
    pub fn drain(&mut self) {
        self.stop();
        while self.active.load(Ordering::SeqCst) != 0 {
            thread::sleep(Duration::from_millis(50));
        }
    }
    pub fn stop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for LanServerHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

pub fn start_lan_listener(
    db_path: PathBuf,
    bind_address: &str,
    port: u16,
) -> AppResult<LanServerHandle> {
    start_lan_listener_with_staff(db_path, bind_address, port, None)
}

pub fn start_lan_listener_with_staff(
    db_path: PathBuf,
    bind_address: &str,
    port: u16,
    staff: Option<Arc<crate::services::staff_web::StaffWeb>>,
) -> AppResult<LanServerHandle> {
    let addr = format!("{bind_address}:{port}");
    let listener = TcpListener::bind((bind_address, port))
        .map_err(|e| AppError::Internal(format!("Failed to bind LAN listener to {addr}: {e}")))?;

    let port = listener.local_addr().map_err(AppError::Io)?.port();
    listener.set_nonblocking(true).map_err(AppError::Io)?;

    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_thread = shutdown.clone();
    let active = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let active_handle = Arc::clone(&active);

    let thread = thread::spawn(move || {
        while !shutdown_thread.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((mut stream, _peer_addr)) => {
                    if active.fetch_add(1, Ordering::SeqCst) >= 64 {
                        active.fetch_sub(1, Ordering::SeqCst);
                        let _ = crate::services::web_http::reject(&mut stream, 503);
                        continue;
                    }
                    let db_path_clone = db_path.clone();
                    let staff = staff.clone();
                    let active = Arc::clone(&active);
                    thread::spawn(move || {
                        struct Active(Arc<std::sync::atomic::AtomicUsize>);
                        impl Drop for Active {
                            fn drop(&mut self) {
                                self.0.fetch_sub(1, Ordering::SeqCst);
                            }
                        }
                        let _active = Active(active);
                        let _ = handle_http_connection(stream, &db_path_clone, staff);
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(50));
                }
                Err(_) => {
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }
    });

    Ok(LanServerHandle {
        shutdown,
        active: active_handle,
        thread: Some(thread),
        port,
    })
}

fn brand_browser_page(html: String) -> String {
    let logo = base64::engine::general_purpose::STANDARD
        .encode(include_bytes!("../../../src/assets/pusthakam-logo.png"));
    let favicon = base64::engine::general_purpose::STANDARD
        .encode(include_bytes!("../../../public/favicon.png"));
    let icon_link = format!(
        "<link rel=\"icon\" type=\"image/png\" href=\"data:image/png;base64,{favicon}\"></head>"
    );
    let mark = format!(
        "<body><img src=\"data:image/png;base64,{logo}\" width=\"40\" height=\"40\" alt=\"Pusthakam\" style=\"display:block;object-fit:contain;margin:0 auto 16px\">"
    );
    html.replacen("</head>", &icon_link, 1)
        .replacen("<body>", &mark, 1)
}

fn handle_http_connection(
    mut stream: TcpStream,
    db_path: &PathBuf,
    staff: Option<Arc<crate::services::staff_web::StaffWeb>>,
) -> std::io::Result<()> {
    use crate::services::web_http;
    // Windows accepted sockets can inherit the nonblocking listener mode. Request reads
    // must wait for separately arriving headers/body instead of treating WouldBlock as timeout.
    stream.set_nonblocking(false)?;
    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));
    let head = match web_http::read_head(&mut stream) {
        Ok(head) => head,
        Err(status) => return web_http::reject(&mut stream, status),
    };
    static LARGE_UPLOADS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    struct UploadPermit(bool);
    impl Drop for UploadPermit {
        fn drop(&mut self) {
            if self.0 {
                LARGE_UPLOADS.fetch_sub(1, Ordering::SeqCst);
            }
        }
    }
    let large = head.length > 1024 * 1024;
    if large && LARGE_UPLOADS.fetch_add(1, Ordering::SeqCst) >= 2 {
        LARGE_UPLOADS.fetch_sub(1, Ordering::SeqCst);
        return web_http::reject(&mut stream, 503);
    }
    let _upload = UploadPermit(large);
    let limit = staff
        .as_ref()
        .map_or(64 * 1024, |web| web.body_limit(&stream, &head));
    let body = match web_http::read_body(&mut stream, &head, limit) {
        Ok(body) => body,
        Err(status) => return web_http::reject(&mut stream, status),
    };
    if let Some(web) = &staff {
        if web.ready() && crate::services::staff_web::StaffWeb::handles(head.path()) {
            return web.handle(&mut stream, &head, &body);
        }
        let public = web.public_request(&head)
            || !crate::services::staff_web::private_peer(stream.peer_addr()?.ip());
        if public && !matches!(head.path(), "/" | "/opac") && !head.path().starts_with("/api/opac/")
        {
            return web_http::reject(&mut stream, 403);
        }
    }
    let body = match std::str::from_utf8(&body) {
        Ok(body) => body,
        Err(_) => return web_http::reject(&mut stream, 400),
    };
    let request_str = format!("{}\r\n\r\n{}", head.raw, body);
    let method = head.method.as_str();
    let uri = head.target.as_str();

    let mut conn = match Connection::open(db_path) {
        Ok(c) => {
            let _ = c.busy_timeout(Duration::from_secs(5));
            let _ = c.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;");
            c
        }
        Err(_) => {
            let resp = "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 21\r\n\r\nDatabase unavailable";
            return stream.write_all(resp.as_bytes());
        }
    };

    // Public covers expose only an active record's stored image, never a filesystem path.
    if method == "GET" && uri.starts_with("/api/opac/cover/") {
        let id = uri
            .trim_start_matches("/api/opac/cover/")
            .parse::<i64>()
            .ok()
            .filter(|id| *id > 0);
        if let Some(id) = id {
            match opac_cover(&conn, id) {
                Ok(Some(png)) => {
                    let headers = format!("HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", png.len());
                    stream.write_all(headers.as_bytes())?;
                    return stream.write_all(&png);
                }
                Err(AppError::Permission(_)) => {
                    return send_json(
                        &mut stream,
                        403,
                        "{\"error\":\"Public catalogue disabled\"}",
                    )
                }
                Err(_) => return send_json(&mut stream, 500, "{\"error\":\"Cover unavailable\"}"),
                Ok(None) => {}
            }
        }
        return send_json(&mut stream, 404, "{\"error\":\"Cover not found\"}");
    }

    // Patron OPAC account routes (isolated from staff sessions; only active when OPAC is enabled)
    if uri.starts_with("/api/opac/patron/") {
        let opac_on = get_or_init_config(&conn)
            .map(|c| c.enabled && c.opac_enabled)
            .unwrap_or(false);
        if !opac_on {
            return send_patron_json(
                &mut stream,
                403,
                r#"{"error":"The public catalogue is currently disabled."}"#,
                None,
            );
        }

        if method == "GET" && uri == "/api/opac/patron/session" {
            if let Some((_, member_id, csrf)) = crate::services::opac_patron::current_session(&head)
            {
                match crate::services::opac_patron::account_summary(&mut conn, member_id, csrf) {
                    Ok(acct) => {
                        return send_patron_json(
                            &mut stream,
                            200,
                            &serde_json::to_string(&acct).unwrap_or_default(),
                            None,
                        );
                    }
                    Err(e) => {
                        let json = serde_json::json!({ "error": e.to_string() }).to_string();
                        return send_patron_json(&mut stream, 404, &json, None);
                    }
                }
            }
            return send_patron_json(
                &mut stream,
                200,
                r#"{"signedIn":false,"unpaidFineMinor":0,"loans":[],"reservations":[]}"#,
                None,
            );
        }

        if method == "POST" {
            if !crate::services::opac_patron::same_origin_ok(&head) {
                return send_patron_json(
                    &mut stream,
                    403,
                    r#"{"error":"Cross-origin request rejected."}"#,
                    None,
                );
            }
            let body_str = extract_body(&request_str);

            if uri == "/api/opac/patron/login" {
                let input: crate::services::opac_patron::PatronLoginInput =
                    match serde_json::from_str(body_str) {
                        Ok(v) => v,
                        Err(_) => {
                            return send_patron_json(
                                &mut stream,
                                400,
                                r#"{"error":"Invalid login payload."}"#,
                                None,
                            );
                        }
                    };
                let member_id = match crate::services::opac_patron::authenticate(&conn, &input) {
                    Ok(id) => id,
                    Err(e) => {
                        let status = if matches!(e, AppError::Validation(_)) {
                            400
                        } else {
                            401
                        };
                        let json = serde_json::json!({ "error": e.to_string() }).to_string();
                        return send_patron_json(&mut stream, status, &json, None);
                    }
                };
                let secure = crate::services::opac_patron::is_secure_request(&head);
                let (cookie, csrf, _) =
                    match crate::services::opac_patron::create_session(member_id, secure) {
                        Ok(v) => v,
                        Err(e) => {
                            let json = serde_json::json!({ "error": e.to_string() }).to_string();
                            return send_patron_json(&mut stream, 500, &json, None);
                        }
                    };
                return match crate::services::opac_patron::account_summary(
                    &mut conn, member_id, csrf,
                ) {
                    Ok(acct) => send_patron_json(
                        &mut stream,
                        200,
                        &serde_json::to_string(&acct).unwrap_or_default(),
                        Some(&cookie),
                    ),
                    Err(e) => {
                        let json = serde_json::json!({ "error": e.to_string() }).to_string();
                        send_patron_json(&mut stream, 500, &json, None)
                    }
                };
            }

            if uri == "/api/opac/patron/logout" {
                let cleared = crate::services::opac_patron::clear_session(&head);
                return send_patron_json(
                    &mut stream,
                    200,
                    r#"{"signedIn":false,"unpaidFineMinor":0,"loans":[],"reservations":[]}"#,
                    Some(&cleared),
                );
            }

            let Some((_, member_id, csrf)) = crate::services::opac_patron::current_session(&head)
            else {
                return send_patron_json(
                    &mut stream,
                    401,
                    r#"{"error":"Please sign in with your library card first."}"#,
                    None,
                );
            };
            let sent_csrf = head.header("x-csrf-token").unwrap_or("");
            if sent_csrf != csrf {
                return send_patron_json(
                    &mut stream,
                    403,
                    r#"{"error":"Security token mismatch. Refresh and try again."}"#,
                    None,
                );
            }

            if uri == "/api/opac/patron/hold" {
                let input: crate::services::opac_patron::PatronHoldInput =
                    match serde_json::from_str(body_str) {
                        Ok(v) => v,
                        Err(_) => {
                            return send_patron_json(
                                &mut stream,
                                400,
                                r#"{"error":"Invalid hold request."}"#,
                                None,
                            );
                        }
                    };
                if let Err(e) = crate::services::reservations::place_for_patron_record(
                    &mut conn,
                    member_id,
                    input.record_id,
                ) {
                    let status = match &e {
                        AppError::Conflict(_) => 409,
                        AppError::Validation(_) => 400,
                        _ => 500,
                    };
                    let json = serde_json::json!({ "error": e.to_string() }).to_string();
                    return send_patron_json(&mut stream, status, &json, None);
                }
                return match crate::services::opac_patron::account_summary(
                    &mut conn, member_id, csrf,
                ) {
                    Ok(acct) => send_patron_json(
                        &mut stream,
                        200,
                        &serde_json::to_string(&acct).unwrap_or_default(),
                        None,
                    ),
                    Err(e) => {
                        let json = serde_json::json!({ "error": e.to_string() }).to_string();
                        send_patron_json(&mut stream, 500, &json, None)
                    }
                };
            }

            if uri == "/api/opac/patron/cancel-hold" {
                let input: crate::services::opac_patron::PatronCancelHoldInput =
                    match serde_json::from_str(body_str) {
                        Ok(v) => v,
                        Err(_) => {
                            return send_patron_json(
                                &mut stream,
                                400,
                                r#"{"error":"Invalid cancel-hold request."}"#,
                                None,
                            );
                        }
                    };
                if let Err(e) = crate::services::reservations::cancel_for_patron(
                    &mut conn,
                    member_id,
                    input.reservation_id,
                ) {
                    let status = match &e {
                        AppError::Conflict(_) => 409,
                        _ => 500,
                    };
                    let json = serde_json::json!({ "error": e.to_string() }).to_string();
                    return send_patron_json(&mut stream, status, &json, None);
                }
                return match crate::services::opac_patron::account_summary(
                    &mut conn, member_id, csrf,
                ) {
                    Ok(acct) => send_patron_json(
                        &mut stream,
                        200,
                        &serde_json::to_string(&acct).unwrap_or_default(),
                        None,
                    ),
                    Err(e) => {
                        let json = serde_json::json!({ "error": e.to_string() }).to_string();
                        send_patron_json(&mut stream, 500, &json, None)
                    }
                };
            }
        }
    }

    // Route: GET /api/opac/categories -> browse categories that hold visible books
    if method == "GET" && uri.split('?').next() == Some("/api/opac/categories") {
        let opac_on = get_or_init_config(&conn)
            .map(|c| c.enabled && c.opac_enabled)
            .unwrap_or(false);
        if !opac_on {
            let json = serde_json::json!({ "error": "Public OPAC access is currently disabled." })
                .to_string();
            return send_json(&mut stream, 403, &json);
        }
        return match crate::services::browse_categories::opac_categories(&conn) {
            Ok(categories) => {
                let json = serde_json::to_string(&categories).unwrap_or_default();
                send_json(&mut stream, 200, &json)
            }
            Err(e) => {
                let json = serde_json::json!({ "error": e.to_string() }).to_string();
                send_json(&mut stream, 500, &json)
            }
        };
    }

    // Route: GET /api/opac/search?q=...
    if method == "GET" && uri.split('?').next() == Some("/api/opac/search") {
        let q = extract_query_param(uri, "q").unwrap_or_default();
        let page = extract_query_param(uri, "page")
            .and_then(|p| p.parse().ok())
            .unwrap_or(1);
        let category = match extract_query_param(uri, "category").filter(|c| !c.is_empty()) {
            None => Some(0),
            Some(c) => c.parse::<i64>().ok().filter(|id| *id > 0),
        };
        let available = extract_query_param(uri, "available").as_deref() == Some("true");
        let result = match category {
            Some(category) => opac_search_filtered(&conn, &q, page, 20, Some(category), available),
            None => Err(AppError::Validation("Choose a valid category.".into())),
        };
        match result {
            Ok(res) => {
                let json = serde_json::to_string(&res).unwrap_or_default();
                return send_json(&mut stream, 200, &json);
            }
            Err(e) => {
                let status = match &e {
                    AppError::Permission(_) => 403,
                    AppError::Validation(_) => 400,
                    _ => 500,
                };
                let json = serde_json::json!({ "error": e.to_string() }).to_string();
                return send_json(&mut stream, status, &json);
            }
        }
    }

    // Route: GET / or /opac -> Swiss Stacks public catalogue
    if method == "GET" && (uri == "/" || uri == "/opac") {
        let config = get_or_init_config(&conn).ok();
        let server_name = config
            .as_ref()
            .map(|c| c.server_name.as_str())
            .unwrap_or("Pusthakam OPAC");
        let html = crate::services::opac_page::render(server_name);
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
            html.len(),
            html
        );
        return stream.write_all(resp.as_bytes());
    }

    // Default 404
    let not_found = "HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\n\r\nNot Found";
    stream.write_all(not_found.as_bytes())
}

fn send_json(stream: &mut TcpStream, status: u16, json: &str) -> std::io::Result<()> {
    let status_line = match status {
        200 => "200 OK",
        400 => "400 Bad Request",
        401 => "401 Unauthorized",
        403 => "403 Forbidden",
        404 => "404 Not Found",
        409 => "409 Conflict",
        _ => "500 Internal Server Error",
    };
    let resp = format!(
        "HTTP/1.1 {}\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        json.len(),
        json
    );
    stream.write_all(resp.as_bytes())
}

fn send_patron_json(
    stream: &mut TcpStream,
    status: u16,
    json: &str,
    set_cookie: Option<&str>,
) -> std::io::Result<()> {
    let status_line = match status {
        200 => "200 OK",
        400 => "400 Bad Request",
        401 => "401 Unauthorized",
        403 => "403 Forbidden",
        404 => "404 Not Found",
        409 => "409 Conflict",
        _ => "500 Internal Server Error",
    };
    let cookie_header = set_cookie
        .map(|c| format!("Set-Cookie: {c}\r\n"))
        .unwrap_or_default();
    let resp = format!(
        "HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\n{cookie_header}Content-Length: {}\r\n\r\n{json}",
        json.len()
    );
    stream.write_all(resp.as_bytes())
}

fn extract_body(request_str: &str) -> &str {
    if let Some(pos) = request_str.find("\r\n\r\n") {
        &request_str[pos + 4..]
    } else if let Some(pos) = request_str.find("\n\n") {
        &request_str[pos + 2..]
    } else {
        ""
    }
}

fn extract_query_param(uri: &str, key: &str) -> Option<String> {
    let query_start = uri.find('?')?;
    let query = &uri[query_start + 1..];
    for pair in query.split('&') {
        let mut parts = pair.split('=');
        if let Some(k) = parts.next() {
            if k == key {
                return parts.next().map(|v| v.replace('+', " "));
            }
        }
    }
    None
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn test_db() -> (tempfile::TempDir, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.sqlite3");
        let conn = db::open(&db_path).unwrap();
        auth::create_user(&conn, "admin", "Admin", "admin_pass_123", "administrator").unwrap();
        (dir, conn)
    }

    fn admin_session() -> Session {
        Session {
            user_id: 1,
            username: "admin".into(),
            display_name: "Admin".into(),
            role: "administrator".into(),
            permissions: [
                "lan.read".into(),
                "lan.manage".into(),
                "catalogue.write".into(),
            ]
            .into_iter()
            .collect(),
            revision: 1,
        }
    }

    fn librarian_session() -> Session {
        Session {
            user_id: 2,
            username: "librarian".into(),
            display_name: "Librarian".into(),
            role: "librarian".into(),
            permissions: ["lan.read".into(), "catalogue.read".into()]
                .into_iter()
                .collect(),
            revision: 1,
        }
    }

    #[test]
    fn test_lan_config_initialization_and_fingerprint() {
        let (_dir, conn) = test_db();
        let config = get_or_init_config(&conn).unwrap();
        assert_eq!(config.port, 4318);
        assert!(!config.enabled);
        assert!(!config.server_fingerprint.is_empty());
        assert_ne!(
            config.server_fingerprint,
            "initial-unconfigured-fingerprint"
        );

        // Re-reading config retains the same fingerprint
        let config2 = get_or_init_config(&conn).unwrap();
        assert_eq!(config.server_fingerprint, config2.server_fingerprint);
    }

    #[test]
    fn test_lan_config_update_permission_and_validation() {
        let (_dir, conn) = test_db();
        let admin = admin_session();
        let librarian = librarian_session();

        let update_input = UpdateLanConfigInput {
            enabled: true,
            bind_address: "127.0.0.1".into(),
            port: 8080,
            server_name: "City Library Central".into(),
            opac_enabled: true,
            browser_staff_enabled: true,
            public_base_url: None,
        };

        // Librarian cannot manage LAN
        assert!(update_config(&conn, &librarian, &update_input).is_err());

        // Admin can update config
        let updated = update_config(&conn, &admin, &update_input).unwrap();
        assert!(updated.enabled);
        assert_eq!(updated.port, 8080);
        assert_eq!(updated.server_name, "City Library Central");

        // Invalid port (< 1024) rejected
        let invalid_port = UpdateLanConfigInput {
            port: 80,
            ..update_input
        };
        assert!(update_config(&conn, &admin, &invalid_port).is_err());
    }

    #[test]
    fn test_pairing_code_single_use_and_revocation() {
        let (_dir, conn) = test_db();
        let admin = admin_session();

        // Enable LAN first
        update_config(
            &conn,
            &admin,
            &UpdateLanConfigInput {
                enabled: true,
                bind_address: "127.0.0.1".into(),
                port: 4318,
                server_name: "Test Lib".into(),
                opac_enabled: true,
                browser_staff_enabled: true,
                public_base_url: None,
            },
        )
        .unwrap();

        // Generate pairing code
        let code_obj = generate_pairing_code(&conn, &admin).unwrap();
        assert_eq!(code_obj.code.len(), 6);
        assert!(code_obj.is_valid);

        // Pair client with valid code
        let pair_req = PairRequest {
            pairing_code: code_obj.code.clone(),
            client_name: "Desk Terminal 1".into(),
            device_kind: Some("desktop".into()),
            client_ip: Some("192.168.1.50".into()),
        };
        let pair_res = pair_client(&conn, &pair_req).unwrap();
        assert!(!pair_res.client_id.is_empty());
        assert_eq!(pair_res.server_name, "Test Lib");

        // Attempting to reuse pairing code fails
        assert!(pair_client(&conn, &pair_req).is_err());

        // List paired clients shows the new client
        let clients = list_paired_clients(&conn, &admin).unwrap();
        assert_eq!(clients.len(), 1);
        assert_eq!(clients[0].client_id, pair_res.client_id);
        assert_eq!(clients[0].client_name, "Desk Terminal 1");
        assert!(clients[0].revoked_at.is_none());

        // Revoke client
        revoke_client(&conn, &admin, &pair_res.client_id).unwrap();
        let clients_after = list_paired_clients(&conn, &admin).unwrap();
        assert!(clients_after[0].revoked_at.is_some());
    }

    #[test]
    fn test_lan_login_session_lifecycle_and_revocation() {
        let (_dir, conn) = test_db();
        let admin = admin_session();

        // Enable LAN
        let config = update_config(
            &conn,
            &admin,
            &UpdateLanConfigInput {
                enabled: true,
                bind_address: "127.0.0.1".into(),
                port: 4318,
                server_name: "Test Lib".into(),
                opac_enabled: true,
                browser_staff_enabled: true,
                public_base_url: None,
            },
        )
        .unwrap();

        // Pair a client
        let code = generate_pairing_code(&conn, &admin).unwrap();
        let paired = pair_client(
            &conn,
            &PairRequest {
                pairing_code: code.code,
                client_name: "Librarian Tablet".into(),
                device_kind: Some("tablet".into()),
                client_ip: Some("192.168.1.102".into()),
            },
        )
        .unwrap();

        // Create a staff user
        auth::create_user(
            &conn,
            "staff_user",
            "Staff Member",
            "secret_passphrase_123",
            "librarian",
        )
        .unwrap();

        // 1. Login with fingerprint mismatch fails
        let bad_fp_req = LanLoginRequest {
            client_id: paired.client_id.clone(),
            server_fingerprint: "WRONG_FINGERPRINT_123456789".into(),
            username: "staff_user".into(),
            password: "secret_passphrase_123".into(),
        };
        assert!(lan_login(&conn, &bad_fp_req).is_err());

        // 2. Login with wrong password fails
        let bad_pw_req = LanLoginRequest {
            client_id: paired.client_id.clone(),
            server_fingerprint: config.server_fingerprint.clone(),
            username: "staff_user".into(),
            password: "wrong_password".into(),
        };
        assert!(lan_login(&conn, &bad_pw_req).is_err());

        // 3. Valid login succeeds and returns token
        let login_req = LanLoginRequest {
            client_id: paired.client_id.clone(),
            server_fingerprint: config.server_fingerprint.clone(),
            username: "staff_user".into(),
            password: "secret_passphrase_123".into(),
        };
        let login_res = lan_login(&conn, &login_req).unwrap();
        assert!(!login_res.token.is_empty());
        assert_eq!(login_res.role, "librarian");

        // 4. Verify LAN session
        let (verified_session, cid) = verify_lan_session(&conn, &login_res.token).unwrap();
        assert_eq!(cid, paired.client_id);
        assert_eq!(verified_session.username, "staff_user");

        // 5. Revoking client blocks the session
        revoke_client(&conn, &admin, &paired.client_id).unwrap();
        assert!(verify_lan_session(&conn, &login_res.token).is_err());

        // Login from revoked client is blocked
        assert!(lan_login(&conn, &login_req).is_err());
    }

    #[test]
    fn test_public_opac_search_and_availability_isolation() {
        let (_dir, conn) = test_db();
        let admin = admin_session();

        // Enable LAN & OPAC
        update_config(
            &conn,
            &admin,
            &UpdateLanConfigInput {
                enabled: true,
                bind_address: "127.0.0.1".into(),
                port: 4318,
                server_name: "Central Library".into(),
                opac_enabled: true,
                browser_staff_enabled: true,
                public_base_url: None,
            },
        )
        .unwrap();

        // Insert a catalogue record with physical copies
        conn.execute(
            "INSERT INTO bibliographic_records (id, title, subtitle, publisher, publication_year, isbn13, classification_number)
             VALUES (101, 'Introduction to Algorithms', 'CLRS Fourth Edition', 'MIT Press', 2022, '9780262046305', '005.1')",
            [],
        ).unwrap();

        // Insert contributors
        conn.execute(
            "INSERT INTO contributors (id, name) VALUES (201, 'Thomas H. Cormen')",
            [],
        )
        .unwrap();
        conn.execute("INSERT INTO record_contributors (record_id, contributor_id, role, position) VALUES (101, 201, 'author', 1)", []).unwrap();

        // Insert shelving location
        conn.execute(
            "INSERT INTO locations (id, name) VALUES (301, 'Computer Science Stack')",
            [],
        )
        .unwrap();

        // Insert item category
        conn.execute(
            "INSERT INTO item_categories (id, name) VALUES (401, 'General Book')",
            [],
        )
        .unwrap();

        // Insert physical items: 2 available, 1 issued
        conn.execute("INSERT INTO physical_items (record_id, location_id, item_category_id, barcode, accession_no, status) VALUES (101, 301, 401, 'ALG-001', 'ACC-1', 'available')", []).unwrap();
        conn.execute("INSERT INTO physical_items (record_id, location_id, item_category_id, barcode, accession_no, status) VALUES (101, 301, 401, 'ALG-002', 'ACC-2', 'available')", []).unwrap();
        conn.execute("INSERT INTO physical_items (record_id, location_id, item_category_id, barcode, accession_no, status) VALUES (101, 301, 401, 'ALG-003', 'ACC-3', 'on_loan')", []).unwrap();

        // Search OPAC
        let opac_res = opac_search(&conn, "", 1, 10).unwrap();
        assert_eq!(opac_res.total, 1);
        let item = &opac_res.items[0];
        assert_eq!(item.title, "Introduction to Algorithms");
        assert_eq!(item.authors, vec!["Thomas H. Cormen"]);
        assert_eq!(item.total_copies, 3);
        assert_eq!(item.available_copies, 2);
        assert_eq!(item.locations, vec!["Computer Science Stack"]);

        crate::services::catalogue::reindex(&conn, 101).unwrap();
        // Filters must count and paginate the entire matching catalogue, not the first page.
        for id in 500..525 {
            conn.execute("INSERT INTO bibliographic_records (id, title, classification_number) VALUES (?1, ?2, '823')", params![id, format!("A fiction {id}")]).unwrap();
        }
        conn.execute("INSERT INTO bibliographic_records (id, title, classification_number) VALUES (600, 'Z Computing', '005.2')", []).unwrap();
        conn.execute("INSERT INTO physical_items (record_id, location_id, item_category_id, barcode, accession_no, status) VALUES (600, 301, 401, 'Z-600', 'Z-600', 'available')", []).unwrap();
        conn.execute("INSERT INTO bibliographic_records (id, title, classification_number, archived_at) VALUES (601, 'Archived', '005', '2026-01-01')", []).unwrap();
        crate::services::browse_categories::apply_all_rules(&conn).unwrap();
        let category = |name: &str| -> i64 {
            conn.query_row(
                "SELECT id FROM browse_categories WHERE name = ?1",
                [name],
                |r| r.get(0),
            )
            .unwrap()
        };
        let computers = category("Computers & IT");
        let english_fiction = category("English Fiction");
        let malayalam_fiction = category("Malayalam Fiction");
        let filtered = opac_search_filtered(&conn, "", 2, 1, Some(computers), true).unwrap();
        assert_eq!(filtered.total, 2);
        assert_eq!(filtered.items.len(), 1);
        assert_eq!(filtered.items[0].id, 600);
        assert_eq!(
            opac_search_filtered(&conn, "", 1, 20, Some(english_fiction), true)
                .unwrap()
                .total,
            0
        );
        assert_eq!(
            opac_search_filtered(&conn, "Algorithms", 1, 20, Some(computers), true)
                .unwrap()
                .total,
            1
        );
        assert_eq!(
            opac_search_filtered(&conn, "Cormen", 1, 20, Some(computers), true)
                .unwrap()
                .total,
            1
        );
        assert_eq!(
            opac_search_filtered(&conn, "9780262046305", 1, 20, None, false)
                .unwrap()
                .total,
            1
        );
        assert_eq!(
            opac_search_filtered(&conn, "!!!", 1, 20, None, false)
                .unwrap()
                .total,
            0
        );
        assert!(opac_search_filtered(&conn, "", 1, 20, Some(-1), false).is_err());
        // Members see category names, never class numbers, in search results.
        assert_eq!(filtered.items[0].categories, vec!["Computers & IT"]);
        assert!(opac_search_filtered(&conn, "", i64::MAX, 20, None, false)
            .unwrap()
            .items
            .is_empty());
        let payload = serde_json::to_string(&filtered).unwrap();
        for private_field in [
            "memberNumber",
            "barcode",
            "accessionNo",
            "borrower",
            "token",
        ] {
            assert!(!payload.contains(private_field));
        }

        // Reuse catalogue token handling so Malayalam vowel marks are retained.
        conn.execute("INSERT INTO bibliographic_records (id, title, classification_number) VALUES (602, 'ബാല്യകാലസഖി', '894.8123')", []).unwrap();
        crate::services::catalogue::reindex(&conn, 602).unwrap();
        crate::services::browse_categories::apply_rules_for_record(&conn, 602).unwrap();
        assert_eq!(
            opac_search_filtered(&conn, "ബാല്യകാലസഖി", 1, 20, Some(malayalam_fiction), false)
                .unwrap()
                .total,
            1
        );

        let png = include_bytes!("../../../src/assets/pusthakam-logo.png").to_vec();
        conn.execute(
            "INSERT INTO book_covers (record_id, png) VALUES (101, ?1), (601, ?1)",
            [&png],
        )
        .unwrap();
        assert_eq!(opac_cover(&conn, 101).unwrap(), Some(png));
        assert!(opac_cover(&conn, 601).unwrap().is_none());
        assert!(opac_cover(&conn, 999999).unwrap().is_none());

        // Turn off OPAC and assert access is denied
        update_config(
            &conn,
            &admin,
            &UpdateLanConfigInput {
                enabled: true,
                bind_address: "127.0.0.1".into(),
                port: 4318,
                server_name: "Central Library".into(),
                opac_enabled: false,
                browser_staff_enabled: true,
                public_base_url: None,
            },
        )
        .unwrap();

        assert!(opac_search(&conn, "", 1, 10).is_err());
        assert!(opac_cover(&conn, 101).is_err());
    }

    #[test]
    fn test_embedded_http_server_loopback() {
        let (dir, conn) = test_db();
        let db_path = dir.path().join("test.sqlite3");
        let admin = admin_session();

        // Enable LAN & OPAC
        update_config(
            &conn,
            &admin,
            &UpdateLanConfigInput {
                enabled: true,
                bind_address: "127.0.0.1".into(),
                port: 4399,
                server_name: "Test HTTP Server".into(),
                opac_enabled: true,
                browser_staff_enabled: true,
                public_base_url: None,
            },
        )
        .unwrap();

        conn.execute(
            "INSERT INTO bibliographic_records (id, title) VALUES (888, 'Cover test')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO book_covers (record_id, png) VALUES (888, ?1)",
            [include_bytes!("../../../src/assets/pusthakam-logo.png").as_slice()],
        )
        .unwrap();

        // Start server on free loopback port
        let mut handle = start_lan_listener(db_path, "127.0.0.1", 4399).unwrap();

        // Test OPAC HTML page response
        let mut client_html = TcpStream::connect("127.0.0.1:4399").unwrap();
        client_html
            .write_all(b"GET /opac HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .unwrap();
        let mut html_buf = Vec::new();
        client_html.read_to_end(&mut html_buf).unwrap();
        let html_resp = String::from_utf8_lossy(&html_buf);
        assert!(html_resp.starts_with("HTTP/1.1 200 OK"));
        assert!(html_resp.contains("<!DOCTYPE html>"));
        assert!(html_resp.contains("Test HTTP Server"));
        assert!(html_resp.contains("data:image/png;base64,"));
        assert!(html_resp.contains("alt=\"Pusthakam\""));

        let mut cover_request = TcpStream::connect("127.0.0.1:4399").unwrap();
        cover_request
            .write_all(b"GET /api/opac/cover/888 HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .unwrap();
        let mut cover_response = Vec::new();
        cover_request.read_to_end(&mut cover_response).unwrap();
        assert!(cover_response.starts_with(b"HTTP/1.1 200 OK"));
        assert!(String::from_utf8_lossy(&cover_response).contains("Content-Type: image/png"));
        assert!(cover_response.ends_with(include_bytes!("../../../src/assets/pusthakam-logo.png")));

        // Clean shutdown
        handle.stop();
    }

    #[test]
    fn test_embedded_http_server_serves_no_legacy_staff_api() {
        let (dir, conn) = test_db();
        let db_path = dir.path().join("test.sqlite3");
        let admin = admin_session();
        update_config(
            &conn,
            &admin,
            &UpdateLanConfigInput {
                enabled: true,
                bind_address: "127.0.0.1".into(),
                port: 4488,
                server_name: "Circulation Server".into(),
                opac_enabled: true,
                browser_staff_enabled: true,
                public_base_url: None,
            },
        )
        .unwrap();

        let mut handle = start_lan_listener(db_path, "127.0.0.1", 4488).unwrap();
        let send_req = |req: &str| -> String {
            let mut stream = TcpStream::connect("127.0.0.1:4488").unwrap();
            let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
            let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));
            stream.write_all(req.as_bytes()).unwrap();
            let mut resp_buf = Vec::new();
            stream.read_to_end(&mut resp_buf).unwrap();
            String::from_utf8_lossy(&resp_buf).to_string()
        };

        // KI-029 regression: the old bearer-token staff API and pairing routes are gone.
        for (method, path) in [
            ("GET", "/api/lan/status"),
            ("POST", "/api/lan/pair"),
            ("POST", "/api/staff/login"),
            ("GET", "/api/staff/me"),
            ("POST", "/api/staff/circulation/issue"),
            ("POST", "/api/staff/circulation/return"),
        ] {
            let resp = send_req(&format!(
                "{method} {path} HTTP/1.1\r\nHost: localhost\r\nContent-Length: 2\r\n\r\n{{}}"
            ));
            assert!(
                resp.starts_with("HTTP/1.1 404 Not Found"),
                "{method} {path} should be gone, got: {resp}"
            );
            assert!(!resp.contains("serverFingerprint"));
        }

        // No wildcard cross-origin access on the embedded server.
        let options = send_req("OPTIONS /api/staff/login HTTP/1.1\r\nHost: localhost\r\n\r\n");
        assert!(!options.contains("Access-Control-Allow-Origin"));

        handle.stop();
    }
}
