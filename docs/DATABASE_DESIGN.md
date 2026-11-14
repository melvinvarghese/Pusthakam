# Database Design (SQLite)

Conventions: see DECISIONS D-003, D-007, D-008, D-009. All tables have `id INTEGER PRIMARY KEY`, `created_at`, `updated_at` unless append-only. FKs are `ON DELETE RESTRICT` unless noted. `archived_at` marks soft deletion.

## Migration plan
| Migration | Contents | Phase |
|---|---|---|
| 0001_core | `schema_migrations`, `libraries`, `application_settings`, `audit_events` | 1 |
| 0002_config | `collections`, `locations`, `shelves`, `item_categories`, `member_categories`, `circulation_policies`, `holidays`, `custom_fields`, `classification_schemes` | 2 |
| 0003_auth | `staff_users`, `roles`, `permissions`, `role_permissions` | 2 (login needed from first-launch wizard) |
| 0004_catalogue | `bibliographic_records`, `contributors`, `record_contributors`, `subjects`, `record_subjects`, `classifications`, `physical_items`, `item_events`, `catalogue_fts` | 3 |
| 0005_circulation | `members`, `member_fts`, `loans`, `renewals`, `reservations`, `fines`, `payments` | 4 |
| 0006_acquisitions | `vendors`, `acquisition_requests`, `purchase_orders`, `purchase_order_items`, `invoices`, `budgets` | 6 |
| 0007_serials | `serial_subscriptions`, `serial_issues` | 6 |
| 0008_backup | `backup_history`, `stock_verifications`, `stock_verification_items` | 6–7 |

## Core tables (key columns and constraints)

**libraries** — single row: `name`, `profile` (school/college/university/public/special/research/institutional/private), `address`, `contact`, `currency`, `locale`.

**application_settings** — `key UNIQUE`, `value` (JSON text). Holds optional-module flags, theme default, accession format, backup schedule.

**audit_events** (append-only) — `occurred_at`, `staff_user_id NULL`, `action`, `entity_type`, `entity_id`, `summary`, `details` (JSON; no passwords/PII beyond IDs). Index `(entity_type, entity_id)`, `(occurred_at)`.

**bibliographic_records** — `title NOT NULL`, `subtitle`, `edition`, `publisher_id NULL`, `publication_place`, `publication_year`, `language`, `physical_description`, `pagination`, `series`, `abstract`, `notes`, `cover_path`, `isbn10`, `isbn13`, `classification_scheme_id`, `classification_number`, `keywords`, `custom` (JSON). Indexes on `isbn13`, `isbn10`, `title`, `classification_number`. Duplicate detection = same ISBN or normalised (title + main author + year), surfaced as a warning, not a hard block.

**contributors** — `name`, `sort_name`, `authority_note`; UNIQUE(`sort_name`). **record_contributors** — `(record_id, contributor_id, role)` where role ∈ author/additional_author/editor/translator/illustrator/other, `position`. PK on (record_id, contributor_id, role).

**subjects / record_subjects** — controlled vocabulary with optional `scheme`; many-to-many.

**physical_items** (one per copy) — `record_id NOT NULL`, `accession_no NOT NULL UNIQUE`, `barcode NOT NULL UNIQUE`, `item_category_id`, `collection_id`, `location_id`, `shelf_id`, `acquisition_date`, `acquisition_source` (purchase/donation/transfer/exchange/other), `price_minor`, `vendor_id NULL`, `invoice_ref`, `status` CHECK IN (`available`,`on_loan`,`reserved_hold`,`in_repair`,`lost`,`damaged`,`withdrawn`,`in_transit`), `archived_at`. Indexes: `record_id`, `status`, `(collection_id, location_id)`.

**item_events** (append-only) — copy lifecycle: acquired, status_changed, transferred, withdrawn, verified.

**members** — `member_no UNIQUE`, `barcode UNIQUE`, `name`, `email`, `phone`, `address`, `department`, `class_name`, `designation`, `category_id`, `joined_on`, `expires_on`, `status` CHECK IN (`active`,`suspended`,`expired`,`archived`), `photo_path`.

**member_categories / item_categories** — `name UNIQUE`, `code`. **circulation_policies** — `(member_category_id, item_category_id)` UNIQUE → `loan_days`, `max_loans`, `max_renewals`, `renewal_days`, `fine_per_day_minor`, `fine_grace_days`, `fine_cap_minor NULL`, `reservation_hold_days`. **holidays** — `date UNIQUE`, `name`, weekly closed days stored in settings.

**loans** (never deleted) — `item_id`, `member_id`, `issued_at`, `due_on`, `returned_at NULL`, `issued_by`, `returned_by`, `renewal_count`, `policy_snapshot` (JSON of the rules applied). **Partial UNIQUE index `loans(item_id) WHERE returned_at IS NULL`** guarantees an item cannot be on loan twice. Index `(member_id, returned_at)`, `due_on` where open.

**renewals** — `loan_id`, `renewed_at`, `previous_due_on`, `new_due_on`, `staff_user_id`.

**reservations** — `record_id` (title-level) or `item_id`, `member_id`, `status` (pending/ready/fulfilled/cancelled/expired), `queue_position`, `placed_at`, `expires_at`.

**fines / payments** — `fines`: `loan_id`, `member_id`, `reason` (overdue/lost/damaged), `amount_minor`, `waived_minor`, `status`. `payments`: `fine_id`, `amount_minor`, `paid_at`, `method`, `staff_user_id`. Outstanding = fine − waived − Σ payments (computed, not stored).

**staff_users** — `username UNIQUE`, `display_name`, `password_hash` (argon2id PHC string), `role_id`, `active`, `failed_attempts`, `locked_until`. **roles** (Administrator, Librarian, Circulation Clerk, Viewer) + **permissions** (`catalogue.write`, `circulation.issue`, `members.write`, `admin.settings`, `admin.backup`, …) via `role_permissions`. Built-in roles are seeded by migration and can't be deleted; at least one active Administrator is always enforced.

**vendors, acquisition_requests, purchase_orders(+items), invoices, budgets** — see roadmap Phase 6; receiving a PO line creates `bibliographic_records` + `physical_items` in one transaction.

**serial_subscriptions / serial_issues** — ISSN, frequency, period; expected vs received issue rows with `status` (expected/received/missing/claimed).

**backup_history** — `created_at`, `path`, `size_bytes`, `schema_version`, `integrity_ok`, `kind` (manual/scheduled/pre_restore/pre_migration), `verified_at`.

## Integrity rules enforced in the database
- FK enforcement ON; RESTRICT deletes for referenced rows.
- UNIQUE: accession number, barcode, member number, ISBN-less duplicates only warned.
- Partial UNIQUE index preventing double active loans.
- CHECK constraints on status enums and non-negative money/counts.
- Triggers keep FTS tables in sync.

## Seed data
Production seeds: built-in roles/permissions and profile-based defaults chosen in the wizard. Development sample data lives in `src-tauri/src/dev_seed.rs`, compiled only with the `dev-seed` cargo feature.
