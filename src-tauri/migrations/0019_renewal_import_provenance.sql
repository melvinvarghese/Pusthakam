CREATE TABLE renewal_import_batches (
    id INTEGER PRIMARY KEY,
    source_system TEXT NOT NULL,
    file_digest TEXT NOT NULL UNIQUE,
    imported_by INTEGER NOT NULL REFERENCES staff_users(id) ON DELETE RESTRICT,
    imported_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    row_count INTEGER NOT NULL CHECK(row_count > 0)
);
CREATE TABLE renewal_import_sources (
    renewal_id INTEGER PRIMARY KEY REFERENCES loan_renewals(id) ON DELETE RESTRICT,
    batch_id INTEGER NOT NULL REFERENCES renewal_import_batches(id) ON DELETE RESTRICT,
    source_system TEXT NOT NULL,
    source_id TEXT NOT NULL,
    source_staff TEXT NOT NULL,
    source_payload TEXT NOT NULL,
    UNIQUE(source_system, source_id)
);
CREATE TRIGGER renewal_import_batches_no_update BEFORE UPDATE ON renewal_import_batches
BEGIN SELECT RAISE(ABORT, 'renewal import provenance is append-only'); END;
CREATE TRIGGER renewal_import_batches_no_delete BEFORE DELETE ON renewal_import_batches
BEGIN SELECT RAISE(ABORT, 'renewal import provenance is append-only'); END;
CREATE TRIGGER renewal_import_sources_no_update BEFORE UPDATE ON renewal_import_sources
BEGIN SELECT RAISE(ABORT, 'renewal import provenance is append-only'); END;
CREATE TRIGGER renewal_import_sources_no_delete BEFORE DELETE ON renewal_import_sources
BEGIN SELECT RAISE(ABORT, 'renewal import provenance is append-only'); END;
