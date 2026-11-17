-- Revoke cached sessions when staff roles, activation or credentials change.
ALTER TABLE staff_users ADD COLUMN session_revision INTEGER NOT NULL DEFAULT 0 CHECK (session_revision >= 0);
