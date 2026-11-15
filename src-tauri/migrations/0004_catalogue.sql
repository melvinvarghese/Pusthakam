-- 0004_catalogue: bibliographic records, contributors, subjects, physical copies, search index.
-- A bibliographic record (the title) is separate from its physical copies. History is never deleted:
-- records are archived, copies change status.

CREATE TABLE bibliographic_records (
    id                     INTEGER PRIMARY KEY,
    title                  TEXT NOT NULL COLLATE NOCASE CHECK (length(trim(title)) > 0),
    subtitle               TEXT,
    edition                TEXT,
    publisher              TEXT,           -- publisher authority records arrive with Phase 6
    publication_place      TEXT,
    publication_year       INTEGER CHECK (publication_year IS NULL OR publication_year BETWEEN 1 AND 2200),
    language               TEXT,
    physical_description   TEXT,
    pagination             TEXT,
    series                 TEXT,
    abstract               TEXT,
    notes                  TEXT,
    cover_path             TEXT,
    isbn10                 TEXT,
    isbn13                 TEXT,
    classification_scheme_id INTEGER REFERENCES classification_schemes(id) ON DELETE RESTRICT,
    classification_number  TEXT,
    keywords               TEXT,
    archived_at            TEXT,
    created_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE INDEX idx_records_isbn13 ON bibliographic_records (isbn13);
CREATE INDEX idx_records_isbn10 ON bibliographic_records (isbn10);
CREATE INDEX idx_records_title  ON bibliographic_records (title);
CREATE INDEX idx_records_class  ON bibliographic_records (classification_number);

CREATE TABLE contributors (
    id         INTEGER PRIMARY KEY,
    name       TEXT NOT NULL COLLATE NOCASE UNIQUE CHECK (length(trim(name)) > 0),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);

CREATE TABLE record_contributors (
    record_id       INTEGER NOT NULL REFERENCES bibliographic_records(id) ON DELETE CASCADE,
    contributor_id  INTEGER NOT NULL REFERENCES contributors(id) ON DELETE RESTRICT,
    role            TEXT NOT NULL CHECK (role IN ('author','additional_author','editor','translator','illustrator','other')),
    position        INTEGER NOT NULL,
    PRIMARY KEY (record_id, contributor_id, role)
);
CREATE INDEX idx_record_contributors_contributor ON record_contributors (contributor_id);

CREATE TABLE subjects (
    id       INTEGER PRIMARY KEY,
    heading  TEXT NOT NULL COLLATE NOCASE UNIQUE CHECK (length(trim(heading)) > 0)
);

CREATE TABLE record_subjects (
    record_id   INTEGER NOT NULL REFERENCES bibliographic_records(id) ON DELETE CASCADE,
    subject_id  INTEGER NOT NULL REFERENCES subjects(id) ON DELETE RESTRICT,
    PRIMARY KEY (record_id, subject_id)
);
CREATE INDEX idx_record_subjects_subject ON record_subjects (subject_id);

-- One row per physical copy.
CREATE TABLE physical_items (
    id                  INTEGER PRIMARY KEY,
    record_id           INTEGER NOT NULL REFERENCES bibliographic_records(id) ON DELETE RESTRICT,
    accession_no        TEXT NOT NULL COLLATE NOCASE UNIQUE CHECK (length(trim(accession_no)) > 0),
    barcode             TEXT NOT NULL COLLATE NOCASE UNIQUE CHECK (length(trim(barcode)) > 0),
    item_category_id    INTEGER NOT NULL REFERENCES item_categories(id) ON DELETE RESTRICT,
    collection_id       INTEGER REFERENCES collections(id) ON DELETE RESTRICT,
    location_id         INTEGER REFERENCES locations(id) ON DELETE RESTRICT,
    shelf_id            INTEGER REFERENCES shelves(id) ON DELETE RESTRICT,
    call_number         TEXT,
    acquisition_date    TEXT CHECK (acquisition_date IS NULL OR acquisition_date GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]'),
    acquisition_source  TEXT NOT NULL DEFAULT 'purchase'
                          CHECK (acquisition_source IN ('purchase','donation','transfer','exchange','other')),
    price_minor         INTEGER CHECK (price_minor IS NULL OR price_minor >= 0),
    vendor              TEXT,
    invoice_ref         TEXT,
    notes               TEXT,
    status              TEXT NOT NULL DEFAULT 'available'
                          CHECK (status IN ('available','on_loan','reserved_hold','in_repair','lost','damaged','withdrawn','in_transit')),
    created_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE INDEX idx_items_record ON physical_items (record_id);
CREATE INDEX idx_items_status ON physical_items (status);
CREATE INDEX idx_items_collection_location ON physical_items (collection_id, location_id);

-- Append-only lifecycle of each copy.
CREATE TABLE item_events (
    id             INTEGER PRIMARY KEY,
    item_id        INTEGER NOT NULL REFERENCES physical_items(id) ON DELETE RESTRICT,
    occurred_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    event_type     TEXT NOT NULL,
    from_status    TEXT,
    to_status      TEXT,
    note           TEXT,
    staff_user_id  INTEGER
);
CREATE INDEX idx_item_events_item ON item_events (item_id);
CREATE TRIGGER item_events_no_update BEFORE UPDATE ON item_events BEGIN SELECT RAISE(ABORT, 'item_events is append-only'); END;
CREATE TRIGGER item_events_no_delete BEFORE DELETE ON item_events BEGIN SELECT RAISE(ABORT, 'item_events is append-only'); END;

-- Full-text search over titles, people, subjects and identifiers. Rows are written by the catalogue
-- service (rowid = record id). `tokenchars` keeps Malayalam vowel signs, virama and joiners inside
-- words so Malayalam titles are searchable; prefix indexes make "type-ahead" searches fast.
CREATE VIRTUAL TABLE catalogue_fts USING fts5(
    title, subtitle, authors, subjects, publisher, keywords, identifiers, classification,
    tokenize = "unicode61 remove_diacritics 2 tokenchars 'ഀഁംഃാിീുൂൃൄെേൈൊോൌ്ൗൢൣ‌‍'",
    prefix = '2 3 4'
);
