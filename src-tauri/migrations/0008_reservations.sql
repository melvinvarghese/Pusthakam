CREATE TABLE reservations (
    id INTEGER PRIMARY KEY,
    item_id INTEGER NOT NULL REFERENCES physical_items(id) ON DELETE RESTRICT,
    member_id INTEGER NOT NULL REFERENCES members(id) ON DELETE RESTRICT,
    placed_by INTEGER NOT NULL REFERENCES staff_users(id) ON DELETE RESTRICT,
    placed_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','ready','fulfilled','cancelled','expired')),
    expires_at TEXT,
    closed_at TEXT,
    CHECK (status <> 'ready' OR expires_at IS NOT NULL),
    CHECK ((status IN ('pending','ready')) = (closed_at IS NULL))
);

CREATE UNIQUE INDEX reservations_active_pair ON reservations (item_id, member_id) WHERE closed_at IS NULL;
CREATE UNIQUE INDEX reservations_ready_copy ON reservations (item_id) WHERE status = 'ready';
CREATE INDEX reservations_open_queue ON reservations (item_id, id) WHERE closed_at IS NULL;
