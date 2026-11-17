CREATE TABLE loans (
    id INTEGER PRIMARY KEY,
    item_id INTEGER NOT NULL REFERENCES physical_items(id) ON DELETE RESTRICT,
    member_id INTEGER NOT NULL REFERENCES members(id) ON DELETE RESTRICT,
    item_category_id INTEGER NOT NULL REFERENCES item_categories(id) ON DELETE RESTRICT,
    issued_by INTEGER NOT NULL REFERENCES staff_users(id) ON DELETE RESTRICT,
    issued_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    due_on TEXT NOT NULL,
    returned_at TEXT,
    returned_by INTEGER REFERENCES staff_users(id) ON DELETE RESTRICT,
    CHECK ((returned_at IS NULL) = (returned_by IS NULL))
);

CREATE UNIQUE INDEX loans_one_open_per_copy ON loans (item_id) WHERE returned_at IS NULL;
CREATE INDEX loans_open_member_category ON loans (member_id, item_category_id) WHERE returned_at IS NULL;
