-- 0021_acquisitions: vendors, budgets, requests, purchase orders, order line items.

CREATE TABLE vendors (
    id            INTEGER PRIMARY KEY,
    name          TEXT NOT NULL COLLATE NOCASE UNIQUE CHECK (length(trim(name)) > 0),
    code          TEXT COLLATE NOCASE UNIQUE,
    contact_name  TEXT,
    email         TEXT,
    phone         TEXT,
    address       TEXT,
    notes         TEXT,
    archived_at   TEXT,
    created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);

CREATE TABLE acquisition_budgets (
    id            INTEGER PRIMARY KEY,
    name          TEXT NOT NULL COLLATE NOCASE UNIQUE CHECK (length(trim(name)) > 0),
    code          TEXT COLLATE NOCASE UNIQUE,
    amount_minor  INTEGER NOT NULL CHECK (amount_minor >= 0),
    currency      TEXT NOT NULL CHECK (length(trim(currency)) == 3),
    fiscal_year   TEXT,
    archived_at   TEXT,
    created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);

CREATE TABLE acquisition_requests (
    id                    INTEGER PRIMARY KEY,
    title                 TEXT NOT NULL COLLATE NOCASE CHECK (length(trim(title)) > 0),
    author                TEXT,
    isbn                  TEXT,
    publisher             TEXT,
    publication_year      INTEGER CHECK (publication_year IS NULL OR publication_year BETWEEN 1 AND 2200),
    estimated_price_minor INTEGER CHECK (estimated_price_minor IS NULL OR estimated_price_minor >= 0),
    currency              TEXT NOT NULL DEFAULT 'INR' CHECK (length(trim(currency)) == 3),
    copies                INTEGER NOT NULL DEFAULT 1 CHECK (copies > 0),
    requested_by          TEXT NOT NULL CHECK (length(trim(requested_by)) > 0),
    request_reason        TEXT,
    status                TEXT NOT NULL DEFAULT 'pending'
                            CHECK (status IN ('pending', 'approved', 'rejected', 'ordered', 'received')),
    rejection_reason      TEXT,
    budget_id             INTEGER REFERENCES acquisition_budgets(id) ON DELETE RESTRICT,
    created_by            INTEGER REFERENCES staff_users(id) ON DELETE RESTRICT,
    reviewed_by           INTEGER REFERENCES staff_users(id) ON DELETE RESTRICT,
    reviewed_at           TEXT,
    created_at            TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at            TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE INDEX idx_acq_requests_status ON acquisition_requests (status);

CREATE TABLE purchase_orders (
    id            INTEGER PRIMARY KEY,
    order_number  TEXT NOT NULL COLLATE NOCASE UNIQUE CHECK (length(trim(order_number)) > 0),
    vendor_id     INTEGER NOT NULL REFERENCES vendors(id) ON DELETE RESTRICT,
    budget_id     INTEGER REFERENCES acquisition_budgets(id) ON DELETE RESTRICT,
    status        TEXT NOT NULL DEFAULT 'draft'
                    CHECK (status IN ('draft', 'ordered', 'partially_received', 'received', 'cancelled')),
    invoice_ref   TEXT,
    ordered_at    TEXT,
    expected_at   TEXT,
    notes         TEXT,
    created_by    INTEGER NOT NULL REFERENCES staff_users(id) ON DELETE RESTRICT,
    created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE INDEX idx_purchase_orders_vendor ON purchase_orders (vendor_id);
CREATE INDEX idx_purchase_orders_status ON purchase_orders (status);

CREATE TABLE purchase_order_items (
    id                INTEGER PRIMARY KEY,
    order_id          INTEGER NOT NULL REFERENCES purchase_orders(id) ON DELETE CASCADE,
    request_id        INTEGER REFERENCES acquisition_requests(id) ON DELETE SET NULL,
    title             TEXT NOT NULL COLLATE NOCASE CHECK (length(trim(title)) > 0),
    author            TEXT,
    isbn              TEXT,
    quantity          INTEGER NOT NULL CHECK (quantity > 0),
    unit_price_minor  INTEGER NOT NULL DEFAULT 0 CHECK (unit_price_minor >= 0),
    received_quantity INTEGER NOT NULL DEFAULT 0 CHECK (received_quantity >= 0 AND received_quantity <= quantity),
    notes             TEXT
);
CREATE INDEX idx_po_items_order ON purchase_order_items (order_id);

-- Permissions for acquisitions.
INSERT INTO permissions (code, description) VALUES
    ('acquisitions.read', 'View acquisition requests, vendors and orders'),
    ('acquisitions.write', 'Create and manage acquisition requests, purchase orders and receiving');

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'administrator' AND p.code IN ('acquisitions.read', 'acquisitions.write');

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'librarian' AND p.code IN ('acquisitions.read', 'acquisitions.write');

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'viewer' AND p.code = 'acquisitions.read';
