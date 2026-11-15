-- 0003_auth: staff accounts, roles and permissions. Passwords are stored only as argon2id PHC strings.

CREATE TABLE roles (
    id           INTEGER PRIMARY KEY,
    code         TEXT NOT NULL UNIQUE,
    name         TEXT NOT NULL UNIQUE,
    description  TEXT NOT NULL
);

CREATE TABLE permissions (
    id           INTEGER PRIMARY KEY,
    code         TEXT NOT NULL UNIQUE,
    description  TEXT NOT NULL
);

CREATE TABLE role_permissions (
    role_id        INTEGER NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id  INTEGER NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

CREATE TABLE staff_users (
    id               INTEGER PRIMARY KEY,
    username         TEXT NOT NULL COLLATE NOCASE UNIQUE CHECK (length(trim(username)) BETWEEN 3 AND 40),
    display_name     TEXT NOT NULL CHECK (length(trim(display_name)) > 0),
    password_hash    TEXT NOT NULL,
    role_id          INTEGER NOT NULL REFERENCES roles(id) ON DELETE RESTRICT,
    active           INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
    failed_attempts  INTEGER NOT NULL DEFAULT 0,
    locked_until     TEXT,
    last_login_at    TEXT,
    created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);

-- Built-in roles (cannot be deleted by the application).
INSERT INTO roles (code, name, description) VALUES
    ('administrator',   'Administrator',   'Full access, including settings, staff and backups.'),
    ('librarian',       'Librarian',       'Catalogue, circulation, members, acquisitions and reports.'),
    ('circulation',     'Circulation Clerk','Issue, return and renew; look up members and the catalogue.'),
    ('viewer',          'Viewer',          'Read-only access.');

INSERT INTO permissions (code, description) VALUES
    ('catalogue.read',    'View the catalogue and copies'),
    ('catalogue.write',   'Add and edit catalogue records and copies'),
    ('circulation.issue', 'Issue, return and renew items'),
    ('circulation.manage','Waive fines, handle lost and damaged items'),
    ('members.read',      'View members'),
    ('members.write',     'Add and edit members'),
    ('reports.read',      'View and export reports'),
    ('admin.settings',    'Change library settings and policies'),
    ('admin.staff',       'Manage staff accounts and roles'),
    ('admin.backup',      'Create and restore backups'),
    ('audit.read',        'View the audit log');

-- Administrator: everything.
INSERT INTO role_permissions SELECT r.id, p.id FROM roles r, permissions p WHERE r.code = 'administrator';
-- Librarian: everything except admin.*
INSERT INTO role_permissions SELECT r.id, p.id FROM roles r, permissions p
    WHERE r.code = 'librarian' AND p.code NOT LIKE 'admin.%';
-- Circulation clerk.
INSERT INTO role_permissions SELECT r.id, p.id FROM roles r, permissions p
    WHERE r.code = 'circulation' AND p.code IN ('catalogue.read','circulation.issue','members.read');
-- Viewer: read only.
INSERT INTO role_permissions SELECT r.id, p.id FROM roles r, permissions p
    WHERE r.code = 'viewer' AND p.code IN ('catalogue.read','members.read','reports.read');
