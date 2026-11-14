-- 0001_core: library identity, application settings, audit trail.
-- Timestamps are UTC ISO-8601 TEXT. Dates are YYYY-MM-DD TEXT. Money is INTEGER minor units.

CREATE TABLE libraries (
    id          INTEGER PRIMARY KEY CHECK (id = 1),           -- one library per database (DECISIONS D-003)
    name        TEXT NOT NULL CHECK (length(trim(name)) > 0),
    profile     TEXT NOT NULL CHECK (profile IN
                  ('school','college','university','public','special','research','institutional','private')),
    address     TEXT,
    contact     TEXT,
    currency    TEXT NOT NULL CHECK (length(currency) = 3),
    locale      TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);

CREATE TABLE application_settings (
    key         TEXT PRIMARY KEY CHECK (length(key) BETWEEN 1 AND 64),
    value       TEXT NOT NULL,                                 -- JSON text for structured values
    updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);

-- Append-only audit trail. staff_user_id has no FK on purpose: the actor name is
-- snapshotted so history survives staff account changes.
CREATE TABLE audit_events (
    id             INTEGER PRIMARY KEY,
    occurred_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    staff_user_id  INTEGER,
    actor_name     TEXT,
    action         TEXT NOT NULL,
    entity_type    TEXT NOT NULL,
    entity_id      INTEGER,
    summary        TEXT NOT NULL,
    details        TEXT
);
CREATE INDEX idx_audit_entity ON audit_events (entity_type, entity_id);
CREATE INDEX idx_audit_time   ON audit_events (occurred_at);

CREATE TRIGGER audit_events_no_update BEFORE UPDATE ON audit_events
BEGIN SELECT RAISE(ABORT, 'audit_events is append-only'); END;

CREATE TRIGGER audit_events_no_delete BEFORE DELETE ON audit_events
BEGIN SELECT RAISE(ABORT, 'audit_events is append-only'); END;
