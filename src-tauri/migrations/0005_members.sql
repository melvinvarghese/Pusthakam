CREATE TABLE members (
    id INTEGER PRIMARY KEY,
    member_number TEXT NOT NULL COLLATE NOCASE UNIQUE,
    name TEXT NOT NULL,
    category_id INTEGER NOT NULL REFERENCES member_categories(id) ON DELETE RESTRICT,
    email TEXT,
    phone TEXT,
    expires_on TEXT NOT NULL,
    archived_at TEXT
);

CREATE INDEX members_name_idx ON members (name COLLATE NOCASE, id);
