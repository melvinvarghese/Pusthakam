-- 0023_classification_authorities.sql
-- Classification schedules (DDC, UDC, LCC, CC, custom), managed authorities (authors, publishers, subjects), and MARC authority mappings.

-- 1. Extend classification_schemes and seed Colon Classification.
ALTER TABLE classification_schemes ADD COLUMN description TEXT;
ALTER TABLE classification_schemes ADD COLUMN is_custom INTEGER NOT NULL DEFAULT 0;
ALTER TABLE classification_schemes ADD COLUMN created_at TEXT;
UPDATE classification_schemes SET created_at = strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE created_at IS NULL;

UPDATE classification_schemes SET description = 'Dewey Decimal Classification system by Melvil Dewey' WHERE code = 'ddc';
UPDATE classification_schemes SET description = 'Universal Decimal Classification bibliographic index' WHERE code = 'udc';
UPDATE classification_schemes SET description = 'Library of Congress Classification system' WHERE code = 'lcc';

INSERT OR IGNORE INTO classification_schemes (code, name, description, is_custom, created_at)
VALUES ('cc', 'Colon Classification', 'Colon classification faceted scheme developed by S. R. Ranganathan', 0, strftime('%Y-%m-%dT%H:%M:%SZ','now'));

-- 2. Classification schedule entries.
CREATE TABLE classification_entries (
    id          INTEGER PRIMARY KEY,
    scheme_id   INTEGER NOT NULL REFERENCES classification_schemes(id) ON DELETE CASCADE,
    code        TEXT NOT NULL,
    parent_code TEXT,
    title       TEXT NOT NULL,
    description TEXT,
    is_custom   INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    UNIQUE (scheme_id, code)
);
CREATE INDEX idx_classification_entries_scheme_code ON classification_entries(scheme_id, code);
CREATE INDEX idx_classification_entries_parent ON classification_entries(scheme_id, parent_code);

-- Seed DDC Schedule: 10 Main Classes and standard 2nd-level divisions.
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '000', NULL, 'Computer science, information & general works', 'Generalities, computing, systems and information', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '010', '000', 'Bibliography', 'Bibliographies and catalogues', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '020', '000', 'Library & information sciences', 'Libraries, archives and information science', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '030', '000', 'General encyclopedic works', 'Encyclopedias and general knowledge collections', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '050', '000', 'General serial publications', 'Periodicals and journals', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '070', '000', 'News media, journalism & publishing', 'Newspapers, publishing and journalism', 0 FROM classification_schemes WHERE code = 'ddc';

INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '100', NULL, 'Philosophy & psychology', 'Philosophy, ethics and psychological sciences', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '110', '100', 'Metaphysics', 'Ontology, nature of reality', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '120', '100', 'Epistemology', 'Knowledge, causation and humankind', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '150', '100', 'Psychology', 'Cognition, perception, behavioral sciences', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '160', '100', 'Logic', 'Inductive and deductive reasoning', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '170', '100', 'Ethics (Moral philosophy)', 'Normative and applied ethics', 0 FROM classification_schemes WHERE code = 'ddc';

INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '200', NULL, 'Religion', 'Theology, sacred books and spiritual traditions', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '220', '200', 'The Bible', 'Old and New Testaments', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '290', '200', 'Other religions', 'Comparative religion, Hinduism, Islam, Buddhism, Jainism, etc.', 0 FROM classification_schemes WHERE code = 'ddc';

INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '300', NULL, 'Social sciences', 'Sociology, anthropology and social institutions', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '310', '300', 'Collections of general statistics', 'Statistical data and census studies', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '320', '300', 'Political science', 'Government, politics and statecraft', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '330', '300', 'Economics', 'Labor, financial economics and trade', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '340', '300', 'Law', 'Jurisprudence, constitutional, civil and criminal law', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '350', '300', 'Public administration & military science', 'Public service and administrative governance', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '360', '300', 'Social problems & services; associations', 'Welfare, institutional aid and social policy', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '370', '300', 'Education', 'Schools, curricula and higher education', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '380', '300', 'Commerce, communications & transportation', 'Trade, media networks and transit', 0 FROM classification_schemes WHERE code = 'ddc';

INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '400', NULL, 'Language', 'Linguistics and global languages', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '410', '400', 'Linguistics', 'Structural and theoretical linguistics', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '420', '400', 'English & Old English', 'English grammar and vocabulary', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '490', '400', 'Other languages', 'Dravidian, Indo-Aryan and Asian languages', 0 FROM classification_schemes WHERE code = 'ddc';

INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '500', NULL, 'Science', 'Pure natural sciences and mathematics', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '510', '500', 'Mathematics', 'Arithmetic, algebra, calculus and statistics', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '520', '500', 'Astronomy & allied sciences', 'Celestial bodies, cosmology and astrophysics', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '530', '500', 'Physics', 'Mechanics, thermodynamics, quantum physics and optics', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '540', '500', 'Chemistry & allied sciences', 'Inorganic, organic and physical chemistry', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '550', '500', 'Earth sciences', 'Geology, meteorology and oceanography', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '570', '500', 'Life sciences; biology', 'Ecology, evolution, genetics and microbiology', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '580', '500', 'Plants (Botany)', 'Plant anatomy and taxonomy', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '590', '500', 'Animals (Zoology)', 'Animal physiology and taxonomy', 0 FROM classification_schemes WHERE code = 'ddc';

INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '600', NULL, 'Technology (Applied sciences)', 'Applied engineering and practical arts', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '610', '600', 'Medicine & health', 'Human anatomy, pharmacology and clinical health', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '620', '600', 'Engineering & allied operations', 'Electrical, civil, mechanical and computer engineering', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '630', '600', 'Agriculture & related technologies', 'Crop science, forestry and animal husbandry', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '650', '600', 'Management & public relations', 'Business enterprise, finance and administration', 0 FROM classification_schemes WHERE code = 'ddc';

INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '700', NULL, 'Arts & recreation', 'Fine and decorative arts, games and entertainment', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '720', '700', 'Architecture', 'Design and historic buildings', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '740', '700', 'Graphic arts & decorative arts', 'Drawing, illustration and craft', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '780', '700', 'Music', 'Musical traditions, instruments and scores', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '790', '700', 'Sports, games & entertainment', 'Athletics, performing arts and cinema', 0 FROM classification_schemes WHERE code = 'ddc';

INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '800', NULL, 'Literature & rhetoric', 'Poetry, drama, fiction and criticism', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '810', '800', 'American literature in English', 'American prose, fiction and verse', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '820', '800', 'English & Old English literatures', 'British literature and Shakespearean drama', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '894', '800', 'Dravidian literatures (including Malayalam)', 'Malayalam, Tamil, Kannada and Telugu literary works', 0 FROM classification_schemes WHERE code = 'ddc';

INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '900', NULL, 'History & geography', 'World history, geography, travel and biography', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '910', '900', 'Geography & travel', 'Atlases, travelogues and expedition accounts', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '920', '900', 'Biography, genealogy & insignia', 'Biographies and historical memoirs', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '950', '900', 'History of Asia; Far East', 'Asian history, kingdoms and modern nations', 0 FROM classification_schemes WHERE code = 'ddc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '954', '950', 'History of India', 'Indian history, freedom struggle and modern states (Kerala)', 0 FROM classification_schemes WHERE code = 'ddc';

-- Seed UDC Main Classes.
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '0', NULL, 'Science and Knowledge. Documentation. Computer science.', 'Prolegomena, fundamentals of knowledge', 0 FROM classification_schemes WHERE code = 'udc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '1', NULL, 'Philosophy. Psychology.', 'Human thought, metaphysics and mind', 0 FROM classification_schemes WHERE code = 'udc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '2', NULL, 'Religion. Theology.', 'Faiths, scriptures and religious traditions', 0 FROM classification_schemes WHERE code = 'udc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '3', NULL, 'Social Sciences.', 'Sociology, politics, law and economics', 0 FROM classification_schemes WHERE code = 'udc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '5', NULL, 'Mathematics. Natural Sciences.', 'Pure mathematics, physics, biology and chemistry', 0 FROM classification_schemes WHERE code = 'udc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '6', NULL, 'Applied Sciences. Medicine. Technology.', 'Engineering, medicine, industry and craft', 0 FROM classification_schemes WHERE code = 'udc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '7', NULL, 'The Arts. Recreation. Entertainment. Sport.', 'Visual arts, architecture, games and cinema', 0 FROM classification_schemes WHERE code = 'udc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '8', NULL, 'Language. Linguistics. Literature.', 'Linguistic studies, poetry, prose and drama', 0 FROM classification_schemes WHERE code = 'udc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '9', NULL, 'Geography. Biography. History.', 'Regional geography, memoirs and chronicles', 0 FROM classification_schemes WHERE code = 'udc';

-- Seed LCC Main Classes.
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, 'A', NULL, 'General Works', 'Encyclopedias, periodicals and general reference', 0 FROM classification_schemes WHERE code = 'lcc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, 'B', NULL, 'Philosophy, Psychology, Religion', 'Philosophical systems and religious texts', 0 FROM classification_schemes WHERE code = 'lcc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, 'D', NULL, 'World History', 'History of Europe, Asia, Africa and Australia', 0 FROM classification_schemes WHERE code = 'lcc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, 'H', NULL, 'Social Sciences', 'Economics, commerce, finance and sociology', 0 FROM classification_schemes WHERE code = 'lcc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, 'K', NULL, 'Law', 'Statutes, jurisprudence and common law', 0 FROM classification_schemes WHERE code = 'lcc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, 'L', NULL, 'Education', 'Educational theory, universities and training', 0 FROM classification_schemes WHERE code = 'lcc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, 'P', NULL, 'Language and Literature', 'Philology, classical and modern literatures', 0 FROM classification_schemes WHERE code = 'lcc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, 'Q', NULL, 'Science', 'Mathematics (QA), Physics (QC), Chemistry (QD), Biology (QH)', 0 FROM classification_schemes WHERE code = 'lcc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, 'R', NULL, 'Medicine', 'Internal medicine, surgery, pharmacology and nursing', 0 FROM classification_schemes WHERE code = 'lcc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, 'T', NULL, 'Technology', 'Engineering, chemical technology and construction', 0 FROM classification_schemes WHERE code = 'lcc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, 'Z', NULL, 'Bibliography, Library Science, Information Resources', 'Libraries, archival collections and bibliographies', 0 FROM classification_schemes WHERE code = 'lcc';

-- Seed Colon Classification (CC) Main Classes.
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, '2', NULL, 'Library Science', 'Librarianship, classification and library management', 0 FROM classification_schemes WHERE code = 'cc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, 'B', NULL, 'Mathematics', 'Arithmetic, algebra, analysis and astronomy', 0 FROM classification_schemes WHERE code = 'cc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, 'C', NULL, 'Physics', 'Matter, energy and mechanics', 0 FROM classification_schemes WHERE code = 'cc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, 'D', NULL, 'Engineering', 'Civil, electrical and mechanical engineering', 0 FROM classification_schemes WHERE code = 'cc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, 'L', NULL, 'Medicine', 'Medical sciences and therapeutics', 0 FROM classification_schemes WHERE code = 'cc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, 'O', NULL, 'Literature', 'Poetry, drama, fiction and literary arts', 0 FROM classification_schemes WHERE code = 'cc';
INSERT INTO classification_entries (scheme_id, code, parent_code, title, description, is_custom)
SELECT id, 'V', NULL, 'History', 'Chronicles, regional history and civilization', 0 FROM classification_schemes WHERE code = 'cc';

-- 3. Extend contributors table with authority management fields.
ALTER TABLE contributors ADD COLUMN sort_name TEXT;
ALTER TABLE contributors ADD COLUMN kind TEXT NOT NULL DEFAULT 'personal';
ALTER TABLE contributors ADD COLUMN birth_year INTEGER;
ALTER TABLE contributors ADD COLUMN death_year INTEGER;
ALTER TABLE contributors ADD COLUMN authority_note TEXT;
ALTER TABLE contributors ADD COLUMN variant_names TEXT;
ALTER TABLE contributors ADD COLUMN marc_tag TEXT NOT NULL DEFAULT '100';
ALTER TABLE contributors ADD COLUMN updated_at TEXT;
UPDATE contributors SET updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE updated_at IS NULL;

UPDATE contributors SET sort_name = name WHERE sort_name IS NULL;

-- 4. Publisher authorities table and record linkage.
CREATE TABLE publishers (
    id         INTEGER PRIMARY KEY,
    name       TEXT NOT NULL COLLATE NOCASE UNIQUE CHECK (length(trim(name)) > 0),
    place      TEXT,
    notes      TEXT,
    marc_tag   TEXT NOT NULL DEFAULT '264',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);

ALTER TABLE bibliographic_records ADD COLUMN publisher_id INTEGER REFERENCES publishers(id) ON DELETE SET NULL;
CREATE INDEX idx_records_publisher_id ON bibliographic_records(publisher_id);

-- Link existing publishers from bibliographic_records.
INSERT OR IGNORE INTO publishers (name, place)
SELECT DISTINCT trim(publisher), trim(publication_place)
FROM bibliographic_records
WHERE publisher IS NOT NULL AND length(trim(publisher)) > 0;

UPDATE bibliographic_records
SET publisher_id = (SELECT p.id FROM publishers p WHERE p.name = trim(bibliographic_records.publisher) COLLATE NOCASE)
WHERE publisher IS NOT NULL AND length(trim(publisher)) > 0;

-- 5. Extend subjects table with authority management fields.
ALTER TABLE subjects ADD COLUMN scheme TEXT NOT NULL DEFAULT 'local';
ALTER TABLE subjects ADD COLUMN scope_note TEXT;
ALTER TABLE subjects ADD COLUMN broader_term TEXT;
ALTER TABLE subjects ADD COLUMN narrower_terms TEXT;
ALTER TABLE subjects ADD COLUMN see_also TEXT;
ALTER TABLE subjects ADD COLUMN marc_tag TEXT NOT NULL DEFAULT '650';
ALTER TABLE subjects ADD COLUMN updated_at TEXT;
UPDATE subjects SET updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE updated_at IS NULL;

-- 6. Controlled MARC authority mappings table.
CREATE TABLE marc_authority_mappings (
    id             INTEGER PRIMARY KEY,
    authority_type TEXT NOT NULL,
    marc_tag       TEXT NOT NULL,
    subfield_code  TEXT NOT NULL,
    description    TEXT NOT NULL,
    target_field   TEXT NOT NULL,
    is_default     INTEGER NOT NULL DEFAULT 1
);

INSERT INTO marc_authority_mappings (authority_type, marc_tag, subfield_code, description, target_field, is_default) VALUES
    ('author_personal', '100', 'a', 'Personal author heading / primary name', 'name', 1),
    ('author_personal', '100', 'd', 'Personal author dates of birth/death', 'dates', 1),
    ('author_corporate', '110', 'a', 'Corporate body author name', 'name', 1),
    ('author_meeting', '111', 'a', 'Meeting or conference name', 'name', 1),
    ('author_added', '700', 'a', 'Personal added entry / co-author', 'name', 1),
    ('author_added_corporate', '710', 'a', 'Corporate added entry', 'name', 1),
    ('publisher', '264', 'b', 'Publisher name (MARC 264)', 'name', 1),
    ('publisher', '264', 'a', 'Publisher place (MARC 264)', 'place', 1),
    ('publisher', '260', 'b', 'Publisher name (legacy MARC 260)', 'name', 1),
    ('publisher', '260', 'a', 'Publisher place (legacy MARC 260)', 'place', 1),
    ('subject_topical', '650', 'a', 'Topical subject heading', 'heading', 1),
    ('subject_geographic', '651', 'a', 'Geographic subject heading', 'heading', 1),
    ('subject_personal', '600', 'a', 'Personal name as subject', 'heading', 1),
    ('subject_genre', '655', 'a', 'Genre / form heading', 'heading', 1),
    ('classification_ddc', '082', 'a', 'Dewey Decimal Classification number', 'classification_number', 1),
    ('classification_udc', '080', 'a', 'Universal Decimal Classification number', 'classification_number', 1),
    ('classification_lcc', '050', 'a', 'Library of Congress Classification number', 'classification_number', 1),
    ('classification_custom', '084', 'a', 'Other classification number', 'classification_number', 1),
    ('classification_custom', '084', '2', 'Classification source scheme code', 'scheme_code', 1);

-- 7. Permissions for classification and authorities.
INSERT INTO permissions (code, description) VALUES
    ('classification.read', 'Browse classification schedules and view authority files'),
    ('classification.write', 'Manage classification schemes, codes, authorities, and MARC mappings');

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'administrator' AND p.code IN ('classification.read', 'classification.write');

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'librarian' AND p.code IN ('classification.read', 'classification.write');

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'viewer' AND p.code = 'classification.read';
