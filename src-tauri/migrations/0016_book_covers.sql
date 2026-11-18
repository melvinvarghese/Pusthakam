CREATE TABLE book_covers (
 record_id INTEGER PRIMARY KEY REFERENCES bibliographic_records(id) ON DELETE RESTRICT,
 png BLOB NOT NULL CHECK (length(png) BETWEEN 1 AND 2097152)
);
