-- Only the Argon2 hash of a one-use administrator recovery code is retained.
CREATE TABLE staff_recovery_codes (
    staff_user_id INTEGER PRIMARY KEY REFERENCES staff_users(id) ON DELETE RESTRICT,
    code_hash TEXT NOT NULL
);
