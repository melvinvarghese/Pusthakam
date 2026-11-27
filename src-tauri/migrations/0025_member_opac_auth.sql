ALTER TABLE members ADD COLUMN opac_username TEXT COLLATE NOCASE;
ALTER TABLE members ADD COLUMN opac_password_hash TEXT;
CREATE UNIQUE INDEX members_opac_username_idx ON members (opac_username) WHERE opac_username IS NOT NULL;
