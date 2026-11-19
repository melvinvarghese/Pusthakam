CREATE TABLE history_import_batches (
    id INTEGER PRIMARY KEY,
    source_system TEXT NOT NULL,
    file_digest TEXT NOT NULL UNIQUE,
    imported_by INTEGER NOT NULL REFERENCES staff_users(id) ON DELETE RESTRICT,
    imported_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    summary_json TEXT NOT NULL
);
CREATE TABLE history_import_sources (
    id INTEGER PRIMARY KEY,
    batch_id INTEGER NOT NULL REFERENCES history_import_batches(id) ON DELETE RESTRICT,
    source_system TEXT NOT NULL,
    kind TEXT NOT NULL CHECK(kind IN ('reservation','fine','settlement')),
    source_id TEXT NOT NULL,
    local_id INTEGER NOT NULL,
    source_staff TEXT NOT NULL,
    source_payload TEXT NOT NULL,
    UNIQUE(source_system,kind,source_id),
    UNIQUE(kind,local_id)
);
CREATE TRIGGER history_import_batches_no_update BEFORE UPDATE ON history_import_batches
BEGIN SELECT RAISE(ABORT,'history import provenance is append-only'); END;
CREATE TRIGGER history_import_batches_no_delete BEFORE DELETE ON history_import_batches
BEGIN SELECT RAISE(ABORT,'history import provenance is append-only'); END;
CREATE TRIGGER history_import_sources_no_update BEFORE UPDATE ON history_import_sources
BEGIN SELECT RAISE(ABORT,'history import provenance is append-only'); END;
CREATE TRIGGER history_import_sources_no_delete BEFORE DELETE ON history_import_sources
BEGIN SELECT RAISE(ABORT,'history import provenance is append-only'); END;
