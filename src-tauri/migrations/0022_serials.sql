-- 0022_serials: subscriptions, expected/received issues, renewals.

CREATE TABLE serial_subscriptions (
    id                      INTEGER PRIMARY KEY,
    title                   TEXT NOT NULL COLLATE NOCASE CHECK (length(trim(title)) > 0),
    issn                    TEXT,
    publisher               TEXT,
    frequency               TEXT NOT NULL CHECK (frequency IN ('daily', 'weekly', 'biweekly', 'monthly', 'bimonthly', 'quarterly', 'semiannual', 'annual', 'irregular')),
    vendor_id               INTEGER REFERENCES vendors(id) ON DELETE SET NULL,
    budget_id               INTEGER REFERENCES acquisition_budgets(id) ON DELETE SET NULL,
    cost_minor              INTEGER NOT NULL DEFAULT 0 CHECK (cost_minor >= 0),
    currency                TEXT NOT NULL DEFAULT 'INR' CHECK (length(trim(currency)) == 3),
    start_date              TEXT NOT NULL CHECK (start_date GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]'),
    end_date                TEXT NOT NULL CHECK (end_date GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]' AND end_date >= start_date),
    status                  TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'expired', 'cancelled')),
    notes                   TEXT,
    archived_at             TEXT,
    created_by              INTEGER REFERENCES staff_users(id) ON DELETE SET NULL,
    created_at              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE INDEX idx_serial_subs_status ON serial_subscriptions (status);
CREATE INDEX idx_serial_subs_title ON serial_subscriptions (title);

CREATE TABLE serial_issues (
    id                      INTEGER PRIMARY KEY,
    subscription_id         INTEGER NOT NULL REFERENCES serial_subscriptions(id) ON DELETE CASCADE,
    issue_number            TEXT NOT NULL CHECK (length(trim(issue_number)) > 0),
    expected_date           TEXT NOT NULL CHECK (expected_date GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]'),
    received_date           TEXT,
    status                  TEXT NOT NULL DEFAULT 'expected' CHECK (status IN ('expected', 'received', 'missing', 'claimed', 'cancelled')),
    barcode                 TEXT,
    accession_no            TEXT,
    physical_item_id        INTEGER REFERENCES physical_items(id) ON DELETE SET NULL,
    claim_count             INTEGER NOT NULL DEFAULT 0 CHECK (claim_count >= 0),
    claim_date              TEXT,
    notes                   TEXT,
    received_by             INTEGER REFERENCES staff_users(id) ON DELETE SET NULL,
    created_at              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    UNIQUE (subscription_id, issue_number)
);
CREATE INDEX idx_serial_issues_sub ON serial_issues (subscription_id);
CREATE INDEX idx_serial_issues_status ON serial_issues (status);
CREATE INDEX idx_serial_issues_expected ON serial_issues (expected_date);

CREATE TABLE serial_renewals (
    id                      INTEGER PRIMARY KEY,
    subscription_id         INTEGER NOT NULL REFERENCES serial_subscriptions(id) ON DELETE CASCADE,
    start_date              TEXT NOT NULL CHECK (start_date GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]'),
    end_date                TEXT NOT NULL CHECK (end_date GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]' AND end_date >= start_date),
    cost_minor              INTEGER NOT NULL DEFAULT 0 CHECK (cost_minor >= 0),
    currency                TEXT NOT NULL DEFAULT 'INR' CHECK (length(trim(currency)) == 3),
    budget_id               INTEGER REFERENCES acquisition_budgets(id) ON DELETE SET NULL,
    notes                   TEXT,
    created_by              INTEGER REFERENCES staff_users(id) ON DELETE SET NULL,
    created_at              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE INDEX idx_serial_renewals_sub ON serial_renewals (subscription_id);

-- Permissions for serials.
INSERT INTO permissions (code, description) VALUES
    ('serials.read', 'View serial subscriptions and issues'),
    ('serials.write', 'Create and manage serial subscriptions, issue prediction, check-in and renewals');

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'administrator' AND p.code IN ('serials.read', 'serials.write');

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'librarian' AND p.code IN ('serials.read', 'serials.write');

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'viewer' AND p.code = 'serials.read';
