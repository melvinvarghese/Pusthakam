-- Migration 0024: LAN multi-computer server, pairing codes, paired clients, and independent sessions

CREATE TABLE IF NOT EXISTS lan_config (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    enabled INTEGER NOT NULL DEFAULT 0,
    bind_address TEXT NOT NULL DEFAULT '0.0.0.0',
    port INTEGER NOT NULL DEFAULT 4318,
    server_name TEXT NOT NULL DEFAULT 'Pusthakam Library Server',
    server_fingerprint TEXT NOT NULL,
    opac_enabled INTEGER NOT NULL DEFAULT 1,
    browser_staff_enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

INSERT OR IGNORE INTO lan_config (id, enabled, bind_address, port, server_name, server_fingerprint, opac_enabled, browser_staff_enabled)
VALUES (1, 0, '0.0.0.0', 4318, 'Pusthakam Library Server', 'initial-unconfigured-fingerprint', 1, 1);

CREATE TABLE IF NOT EXISTS lan_pairing_codes (
    code TEXT PRIMARY KEY,
    created_by_staff_id INTEGER REFERENCES staff_users(id),
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    used_at TEXT,
    used_by_client_id TEXT
);

CREATE TABLE IF NOT EXISTS lan_paired_clients (
    client_id TEXT PRIMARY KEY,
    client_name TEXT NOT NULL,
    device_kind TEXT NOT NULL DEFAULT 'desktop',
    ip_address TEXT,
    pairing_code_used TEXT,
    paired_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    last_seen_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    revoked_at TEXT
);

CREATE TABLE IF NOT EXISTS lan_sessions (
    token TEXT PRIMARY KEY,
    client_id TEXT REFERENCES lan_paired_clients(client_id) ON DELETE CASCADE,
    staff_user_id INTEGER REFERENCES staff_users(id),
    is_opac INTEGER NOT NULL DEFAULT 0,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    last_active_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    revoked_at TEXT
);

INSERT INTO permissions (code, description) VALUES
    ('lan.read', 'View LAN server status and connected devices'),
    ('lan.manage', 'Configure LAN server, generate pairing codes, and manage paired devices');

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'administrator' AND p.code IN ('lan.read', 'lan.manage');

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'librarian' AND p.code = 'lan.read';
