# Architecture

## 1. Overview
```
┌────────────────────────── Tauri 2 shell (WebView2 / WKWebView / WebKitGTK) ──────────────────────────┐
│  React + TS UI  ──invoke(typed)──▶  Tauri commands (Rust) ──▶ services ──▶ db (rusqlite) ──▶ SQLite  │
│  (no DB access)  ◀──── result / AppError ────                                   (WAL, FTS5, FK ON)   │
└───────────────────────────────────────────────────────────────────────────────────────────────────────┘
          OS app-data dir:  pusthakam.db · covers/ · documents/ · backups/ · exports/ · config.json · logs/
```

## 2. Rust backend (`src-tauri/src`)
| Module | Responsibility |
|---|---|
| `commands/` | Thin `#[tauri::command]` fns. Resolve session, check permission, validate DTOs, call a service, map errors. |
| `services/` | Business rules and transaction boundaries (circulation, accession, fines, backup, import). Pure logic (due dates, fines, accession formats) in `domain/` with no I/O so it is unit-testable. |
| `domain/` | Value types, policy calculations, validators. |
| `db/` | Connection setup (PRAGMAs), migration runner, repositories (SQL lives here only). |
| `auth/` | argon2id hashing, in-memory session store, RBAC permission table. |
| `error.rs` | `AppError` enum (`Validation`, `NotFound`, `Conflict`, `Permission`, `Database`, `Io`, …) → serialised `{code, message, details?}`. |
| `paths.rs` | App-data directory resolution; directory creation. |
| `logging.rs` | File logging (`tracing`), PII-free. |

**Concurrency:** one write connection behind a `Mutex`, plus short-lived read connections (WAL permits concurrent readers). Every multi-statement operation runs in `BEGIN IMMEDIATE` transactions.

**PRAGMAs on open:** `foreign_keys=ON`, `journal_mode=WAL`, `synchronous=NORMAL`, `busy_timeout=5000`.

**Migrations:** numbered SQL files embedded with `include_str!`, tracked in `schema_migrations(version, name, applied_at, checksum)`. Each migration is applied in its own transaction (a failure rolls back completely). Checksum mismatch on an applied migration aborts startup, and a database from a *newer* app version is refused. **Planned (Phase 7, with the backup module):** automatic pre-migration backup when upgrading an existing database — not yet implemented.

## 3. Frontend (`src`)
- `app/` shell: sidebar navigation, command palette (Ctrl/Cmd+K), global shortcuts, theme provider, error boundary.
- `features/<module>/` — pages, forms, hooks per module (dashboard, catalogue, circulation, members, …).
- `components/ui/` — shadcn/ui primitives (copied source, themed with our tokens). `components/` — shared composites (DataTable, FilterBar, ConfirmDialog, EmptyState, FormField).
- `lib/api/` — the **only** place that calls `invoke`. One typed function per command; DTO types in `lib/api/types.ts`.
- Server-state: TanStack Query (cache/invalidations around invoke). UI state: React state / Zustand where needed.
- Forms: react-hook-form + zod (UI-level validation for UX; Rust re-validates everything).
- Tables paginate and sort **server-side**; the catalogue is never loaded wholesale into memory.

## 4. Typed contract
Hand-written DTO types mirrored in Rust (`serde`) and TS, covered by a contract test that round-trips fixtures. Evaluate `tauri-specta` in Phase 1 for generated bindings (see DECISIONS D-004).

## 5. Security
- Tauri capabilities: least privilege; custom commands only; no shell/fs/http plugin exposure to the webview beyond dialogs for file pick/save.
- CSP: `default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: asset:; connect-src ipc: http://ipc.localhost; font-src 'self'` — no remote origins.
- Auth: local staff accounts, argon2id, session token held in Rust memory (never persisted); lockout/backoff after repeated failures; roles → permission sets enforced in `commands/`.
- File handling: all user-supplied paths go through dialog-picked paths or app-data-relative names; reject traversal; limit import sizes; validate image types for covers.
- Audit: `audit_events` written in the same transaction as the change.

## 6. Search
SQLite FTS5 virtual tables (`catalogue_fts`, `member_fts`) kept in sync by triggers/service code. Exact lookups (ISBN, accession, barcode, member ID) hit indexed columns first; free text falls back to FTS prefix queries. Results are paginated.

## 7. Backup / restore
Backup: `rusqlite::backup` (online backup API) to a temp file → `PRAGMA integrity_check` + schema-version check + row-count sanity → atomic rename into the backups dir with timestamped name → row in `backup_history`. Restore: validate candidate → snapshot current DB → swap on app restart/reopen → integrity check; roll back on failure. Encryption is reserved as a future layer (no custom crypto).

## 8. Reports, printing, barcodes
- CSV: `csv` crate. XLSX: `rust_xlsxwriter`. PDF: decision deferred to Phase 5 (D-006) — candidate is rendering print-styled HTML in the OS print pipeline plus a Rust PDF writer for batch labels.
- Barcodes: Code 128 SVG generated in Rust (testable, offline). USB scanners act as keyboard input; scan fields are plain inputs with Enter-to-submit.

## 9. Packaging
Tauri bundler: NSIS/MSI (Windows), `.app`/DMG (macOS), AppImage/deb/rpm (Linux). GitHub Actions matrix builds per OS. A platform is "supported" only after it is built and smoke-tested on that OS.

## 10. Modularity of library types
One core. Library **profile** (school, college, …) seeds defaults (member categories, policies, fields) at setup; everything remains editable configuration. Optional modules (acquisitions, serials) are toggled by `application_settings` and hidden from navigation when off.

## V3 browser/server extension

The shared React staff frontend uses desktop IPC or server-advertised same-origin HTTP transport. The 162 common typed command handlers preserve permissions/services; two native save commands have browser download adapters. Backup copying becomes an authenticated client download. The explicit HTTP dispatcher covers all 164 registered commands without granting arbitrary host file access.

AppState shares database, listener and logging ownership through Arc. Each browser owns independent login and pending-import handles. StaffWeb retains weak listener ownership, avoiding a runtime/session cycle. The HTTP boundary uses bounded parsing/uploads, random HttpOnly session cookies, CSRF and origin checks, session rotation and expiry, and current role/revocation checks. Public staff access accepts only a configured HTTPS origin through a loopback proxy; public legacy pairing/login routes are denied.

The desktop library PC is the default host. The optional pusthakam-server console binary uses the same database migrations, pre-upgrade snapshots, startup restore locks and scheduled backups with an explicit persistent directory. Normal stop drains requests outside the listener mutex. Staff HTML/assets are compiled from dist through build-owned snapshots in OUT_DIR; build the frontend first. Both hosts use one authoritative database. Full acceptance matrix and deployment boundaries: STAFF_BROWSER_PARITY.md and WEB_HOSTING.md. Local automated evidence does not establish other-device connectivity, a public deployment or hardware printing acceptance.
