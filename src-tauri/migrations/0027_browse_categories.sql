-- 0027_browse_categories: member-facing browse categories for the public catalogue (OPAC).
-- Categories are plain names such as "Malayalam Fiction" or "Physics", managed by staff and kept
-- separate from catalogue subject headings and class numbers. Each category may carry DDC prefix
-- rules (comma-separated, e.g. '530' or '894.8123') so records are placed automatically from their
-- class number; staff can also add or remove a category on any record by hand.
-- Categories are never hard-deleted once used: they are archived (archived_at) instead.

CREATE TABLE browse_categories (
    id          INTEGER PRIMARY KEY,
    name        TEXT NOT NULL COLLATE NOCASE UNIQUE CHECK (length(trim(name)) > 0),
    ddc_rules   TEXT,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    is_default  INTEGER NOT NULL DEFAULT 0,
    archived_at TEXT,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);

-- source: 'rule'     = placed automatically by a DDC rule (recomputed when rules or class numbers change)
--         'manual'   = added by staff
--         'excluded' = staff removed a category that a rule would add; rules must not add it back
CREATE TABLE record_categories (
    record_id   INTEGER NOT NULL REFERENCES bibliographic_records(id) ON DELETE CASCADE,
    category_id INTEGER NOT NULL REFERENCES browse_categories(id) ON DELETE CASCADE,
    source      TEXT NOT NULL CHECK (source IN ('rule','manual','excluded')),
    PRIMARY KEY (record_id, category_id)
);
CREATE INDEX idx_record_categories_category ON record_categories (category_id, source);

INSERT INTO browse_categories (name, ddc_rules, sort_order, is_default) VALUES
    ('Malayalam Fiction',            '894.8123',                         10, 1),
    ('Malayalam Poetry',             '894.8121',                         20, 1),
    ('Malayalam Drama',              '894.8122',                         30, 1),
    ('Malayalam Essays & Criticism', '894.8124,894.8128,894.81209',      40, 1),
    ('English Fiction',              '823,813',                          50, 1),
    ('English Poetry',               '821,811',                          60, 1),
    ('Children''s Books',            NULL,                               70, 1),
    ('Biography & Memoir',           '920',                              80, 1),
    ('History',                      '90,93,94,95,96,97,98,99',          90, 1),
    ('Geography & Travel',           '91',                              100, 1),
    ('Religion & Spirituality',      '2',                               110, 1),
    ('Philosophy & Psychology',      '1',                               120, 1),
    ('Politics & Law',               '32,34,35',                        130, 1),
    ('Economics',                    '33',                              140, 1),
    ('Management & Business',        '65',                              150, 1),
    ('Education',                    '37',                              160, 1),
    ('Language & Grammar',           '4',                               170, 1),
    ('Mathematics',                  '51',                              180, 1),
    ('Physics',                      '53',                              190, 1),
    ('Chemistry',                    '54',                              200, 1),
    ('Biology & Nature',             '57,58,59',                        210, 1),
    ('Earth Science & Environment',  '55',                              220, 1),
    ('Computers & IT',               '004,005,006',                     230, 1),
    ('Medicine & Health',            '61',                              240, 1),
    ('Engineering & Technology',     '62',                              250, 1),
    ('Agriculture',                  '63',                              260, 1),
    ('Cookery & Home',               '64',                              270, 1),
    ('Arts & Music',                 '70,71,72,73,74,75,76,77,78',      280, 1),
    ('Sports & Hobbies',             '79',                              290, 1),
    ('Reference',                    '03',                              300, 1),
    ('Competitive Exams',            NULL,                              310, 1);

-- One row per (category, DDC prefix), split from the comma-separated ddc_rules of active categories.
CREATE VIEW browse_category_rules AS
WITH RECURSIVE split(category_id, rule, rest) AS (
    SELECT id, '', replace(COALESCE(ddc_rules, ''), ' ', '') || ','
      FROM browse_categories WHERE archived_at IS NULL
    UNION ALL
    SELECT category_id, substr(rest, 1, instr(rest, ',') - 1), substr(rest, instr(rest, ',') + 1)
      FROM split WHERE rest <> ''
)
SELECT category_id, rule AS prefix FROM split WHERE rule <> '';

-- Place the existing catalogue using the default rules. Only DDC (or unspecified-scheme) numbers match.
INSERT OR IGNORE INTO record_categories (record_id, category_id, source)
SELECT DISTINCT r.id, br.category_id, 'rule'
  FROM bibliographic_records r
  JOIN browse_category_rules br ON trim(r.classification_number) LIKE br.prefix || '%'
 WHERE r.classification_number IS NOT NULL
   AND (r.classification_scheme_id IS NULL
        OR r.classification_scheme_id = (SELECT id FROM classification_schemes WHERE code = 'ddc'));
