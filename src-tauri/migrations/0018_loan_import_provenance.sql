-- Local staff references on imported loans identify the recording operator.
-- Original source labels and payloads are preserved separately, without creating staff accounts.
CREATE TABLE loan_import_batches (
    id INTEGER PRIMARY KEY,
    source_system TEXT NOT NULL CHECK(length(trim(source_system)) BETWEEN 1 AND 128),
    file_digest TEXT NOT NULL UNIQUE,
    imported_by INTEGER NOT NULL REFERENCES staff_users(id) ON DELETE RESTRICT,
    imported_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    row_count INTEGER NOT NULL CHECK(row_count > 0)
);
CREATE TABLE loan_import_sources (
    loan_id INTEGER PRIMARY KEY REFERENCES loans(id) ON DELETE RESTRICT,
    batch_id INTEGER NOT NULL REFERENCES loan_import_batches(id) ON DELETE RESTRICT,
    source_system TEXT NOT NULL,
    source_id TEXT NOT NULL,
    source_staff TEXT NOT NULL,
    source_payload TEXT NOT NULL,
    terms_basis TEXT NOT NULL CHECK(terms_basis IN ('closed_history','current_policy_confirmed')),
    UNIQUE(source_system, source_id)
);
CREATE TRIGGER loan_import_batches_no_update BEFORE UPDATE ON loan_import_batches
BEGIN SELECT RAISE(ABORT, 'loan import provenance is append-only'); END;
CREATE TRIGGER loan_import_batches_no_delete BEFORE DELETE ON loan_import_batches
BEGIN SELECT RAISE(ABORT, 'loan import provenance is append-only'); END;
CREATE TRIGGER loan_import_sources_no_update BEFORE UPDATE ON loan_import_sources
BEGIN SELECT RAISE(ABORT, 'loan import provenance is append-only'); END;
CREATE TRIGGER loan_import_sources_no_delete BEFORE DELETE ON loan_import_sources
BEGIN SELECT RAISE(ABORT, 'loan import provenance is append-only'); END;
