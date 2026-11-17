CREATE TABLE loan_fine_terms (
    loan_id INTEGER PRIMARY KEY REFERENCES loans(id) ON DELETE RESTRICT,
    rate_minor INTEGER NOT NULL CHECK (rate_minor >= 0),
    grace_days INTEGER NOT NULL CHECK (grace_days >= 0),
    cap_minor INTEGER CHECK (cap_minor >= 0),
    currency TEXT NOT NULL CHECK (length(currency) = 3)
);

-- Existing open loans have no historical snapshot. Use the configured rule at upgrade.
INSERT INTO loan_fine_terms (loan_id, rate_minor, grace_days, cap_minor, currency)
SELECT l.id, p.fine_per_day_minor, p.fine_grace_days, p.fine_cap_minor, lib.currency
FROM loans l JOIN members m ON m.id = l.member_id
JOIN circulation_policies p ON p.member_category_id = m.category_id AND p.item_category_id = l.item_category_id
JOIN libraries lib ON lib.id = 1
WHERE l.returned_at IS NULL;

CREATE TABLE fines (
    id INTEGER PRIMARY KEY,
    loan_id INTEGER NOT NULL UNIQUE REFERENCES loans(id) ON DELETE RESTRICT,
    amount_minor INTEGER NOT NULL CHECK (amount_minor BETWEEN 1 AND 9007199254740991),
    assessed_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    currency TEXT NOT NULL CHECK (length(currency) = 3)
);

CREATE TABLE fine_settlements (
    id INTEGER PRIMARY KEY,
    fine_id INTEGER NOT NULL REFERENCES fines(id) ON DELETE RESTRICT,
    staff_user_id INTEGER NOT NULL REFERENCES staff_users(id) ON DELETE RESTRICT,
    kind TEXT NOT NULL CHECK (kind IN ('payment','waiver')),
    amount_minor INTEGER NOT NULL CHECK (amount_minor BETWEEN 1 AND 9007199254740991),
    method TEXT CHECK (method IN ('cash','card','bank_transfer','other')),
    reason TEXT,
    recorded_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    CHECK ((kind = 'payment' AND method IS NOT NULL AND reason IS NULL)
        OR (kind = 'waiver' AND method IS NULL AND reason IS NOT NULL AND length(trim(reason)) > 0))
);

CREATE INDEX fine_settlements_by_fine ON fine_settlements (fine_id);

CREATE TRIGGER fine_settlements_no_update BEFORE UPDATE ON fine_settlements
BEGIN SELECT RAISE(ABORT, 'fine_settlements is append-only'); END;

CREATE TRIGGER fine_settlements_no_delete BEFORE DELETE ON fine_settlements
BEGIN SELECT RAISE(ABORT, 'fine_settlements is append-only'); END;
