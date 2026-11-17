CREATE TABLE loan_renewals (
    id INTEGER PRIMARY KEY,
    loan_id INTEGER NOT NULL REFERENCES loans(id) ON DELETE RESTRICT,
    renewed_by INTEGER NOT NULL REFERENCES staff_users(id) ON DELETE RESTRICT,
    renewed_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    previous_due_on TEXT NOT NULL,
    due_on TEXT NOT NULL CHECK (due_on > previous_due_on)
);

CREATE INDEX loan_renewals_by_loan ON loan_renewals (loan_id);

CREATE TRIGGER loan_renewals_no_update BEFORE UPDATE ON loan_renewals
BEGIN SELECT RAISE(ABORT, 'loan_renewals is append-only'); END;

CREATE TRIGGER loan_renewals_no_delete BEFORE DELETE ON loan_renewals
BEGIN SELECT RAISE(ABORT, 'loan_renewals is append-only'); END;
