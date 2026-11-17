CREATE TABLE stock_checks (
    id INTEGER PRIMARY KEY,
    location_id INTEGER NOT NULL REFERENCES locations(id) ON DELETE RESTRICT,
    shelf_id INTEGER REFERENCES shelves(id) ON DELETE RESTRICT,
    location_name TEXT NOT NULL,
    shelf_code TEXT,
    started_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    started_by INTEGER NOT NULL REFERENCES staff_users(id) ON DELETE RESTRICT,
    closed_at TEXT,
    closed_by INTEGER REFERENCES staff_users(id) ON DELETE RESTRICT,
    CHECK ((closed_at IS NULL) = (closed_by IS NULL))
);

CREATE TABLE stock_check_expected (
    check_id INTEGER NOT NULL REFERENCES stock_checks(id) ON DELETE RESTRICT,
    item_id INTEGER NOT NULL REFERENCES physical_items(id) ON DELETE RESTRICT,
    barcode TEXT NOT NULL COLLATE NOCASE,
    accession_no TEXT NOT NULL,
    title TEXT NOT NULL,
    PRIMARY KEY (check_id, barcode)
);

CREATE TABLE stock_check_scans (
    check_id INTEGER NOT NULL REFERENCES stock_checks(id) ON DELETE RESTRICT,
    barcode TEXT NOT NULL COLLATE NOCASE,
    accession_no TEXT,
    title TEXT,
    matched INTEGER NOT NULL CHECK (matched IN (0, 1)),
    scanned_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    scanned_by INTEGER NOT NULL REFERENCES staff_users(id) ON DELETE RESTRICT,
    PRIMARY KEY (check_id, barcode)
);
