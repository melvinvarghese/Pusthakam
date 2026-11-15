-- 0002_config: library configuration (categories, collections, locations, shelves, policies).
-- Names are unique case-insensitively. Rows are archived, never deleted, once anything may reference them.

CREATE TABLE classification_schemes (
    id    INTEGER PRIMARY KEY,
    code  TEXT NOT NULL UNIQUE CHECK (length(code) BETWEEN 1 AND 16),
    name  TEXT NOT NULL UNIQUE
);
-- Names only. Pusthakam ships no licensed classification data.
INSERT INTO classification_schemes (code, name) VALUES
    ('ddc', 'Dewey Decimal Classification'),
    ('udc', 'Universal Decimal Classification'),
    ('lcc', 'Library of Congress Classification');

CREATE TABLE collections (
    id           INTEGER PRIMARY KEY,
    name         TEXT NOT NULL COLLATE NOCASE UNIQUE CHECK (length(trim(name)) > 0),
    description  TEXT,
    archived_at  TEXT,
    created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);

CREATE TABLE locations (
    id           INTEGER PRIMARY KEY,
    name         TEXT NOT NULL COLLATE NOCASE UNIQUE CHECK (length(trim(name)) > 0),
    description  TEXT,
    archived_at  TEXT,
    created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);

CREATE TABLE shelves (
    id           INTEGER PRIMARY KEY,
    location_id  INTEGER NOT NULL REFERENCES locations(id) ON DELETE RESTRICT,
    code         TEXT NOT NULL COLLATE NOCASE CHECK (length(trim(code)) > 0),
    description  TEXT,
    archived_at  TEXT,
    created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    UNIQUE (location_id, code)
);

CREATE TABLE item_categories (
    id           INTEGER PRIMARY KEY,
    name         TEXT NOT NULL COLLATE NOCASE UNIQUE CHECK (length(trim(name)) > 0),
    description  TEXT,
    archived_at  TEXT,
    created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);

CREATE TABLE member_categories (
    id               INTEGER PRIMARY KEY,
    name             TEXT NOT NULL COLLATE NOCASE UNIQUE CHECK (length(trim(name)) > 0),
    description      TEXT,
    membership_days  INTEGER NOT NULL DEFAULT 365 CHECK (membership_days BETWEEN 1 AND 36500),
    archived_at      TEXT,
    created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);

-- One row per (member category, item category). No row = that member category cannot borrow that item category.
CREATE TABLE circulation_policies (
    id                     INTEGER PRIMARY KEY,
    member_category_id     INTEGER NOT NULL REFERENCES member_categories(id) ON DELETE RESTRICT,
    item_category_id       INTEGER NOT NULL REFERENCES item_categories(id) ON DELETE RESTRICT,
    loan_days              INTEGER NOT NULL CHECK (loan_days BETWEEN 1 AND 365),
    max_loans              INTEGER NOT NULL CHECK (max_loans BETWEEN 1 AND 500),
    max_renewals           INTEGER NOT NULL DEFAULT 0 CHECK (max_renewals BETWEEN 0 AND 50),
    renewal_days           INTEGER NOT NULL DEFAULT 0 CHECK (renewal_days BETWEEN 0 AND 365),
    fine_per_day_minor     INTEGER NOT NULL DEFAULT 0 CHECK (fine_per_day_minor >= 0),
    fine_grace_days        INTEGER NOT NULL DEFAULT 0 CHECK (fine_grace_days BETWEEN 0 AND 365),
    fine_cap_minor         INTEGER CHECK (fine_cap_minor IS NULL OR fine_cap_minor >= 0),
    reservation_hold_days  INTEGER NOT NULL DEFAULT 3 CHECK (reservation_hold_days BETWEEN 1 AND 60),
    created_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    UNIQUE (member_category_id, item_category_id),
    CHECK (max_renewals = 0 OR renewal_days > 0)
);

CREATE TABLE holidays (
    id    INTEGER PRIMARY KEY,
    date  TEXT NOT NULL UNIQUE CHECK (date GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]'),
    name  TEXT NOT NULL CHECK (length(trim(name)) > 0)
);
