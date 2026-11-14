# Development Roadmap

Status key: NOT STARTED · IN PROGRESS · IMPLEMENTED · VERIFIED · BLOCKED

| Phase | Scope | Status | Acceptance criteria |
|---|---|---|---|
| 0 | Discovery, docs, architecture | IN PROGRESS | Docs in `docs/`, CLAUDE.md, workspace audit |
| 1 | Tauri shell, React UI shell, SQLite + migrations, nav, design system, theme, errors, logging, settings | BLOCKED (KI-001) | Launches offline; DB created in app-data and persists across restarts |
| 2 | First-launch wizard, library profile, collections/locations/shelves, categories, circulation policies, admin login | NOT STARTED | Admin configures a new library with no code edits |
| 3 | Catalogue, contributors, subjects, classification, copies, accession, barcodes, search, bulk import | NOT STARTED | One title → many independently managed copies |
| 4 | Members, issue/return/renew/reserve, due dates, overdue, fines, history | NOT STARTED | Full borrow/return workflow on real records |
| 5 | Dashboard analytics, reports, PDF/CSV/XLSX, labels, receipts | NOT STARTED | Reports match DB; exports open in Excel/PDF readers |
| 6 | Acquisitions, serials, stock verification, advanced classification, MARC21 | NOT STARTED | Receive PO → records + copies created |
| 7 | Staff/roles, audit log UI, backup/restore, integrity checks | NOT STARTED | Backup → restore round-trip verified |
| 8 | Installers, CI matrix, signing docs, user docs | NOT STARTED | Per-platform smoke tests passed |

## Phase 1 work breakdown
1. Toolchain ready (Rust stable, MSVC Build Tools, WebView2 ✔ present).
2. `npm create tauri-app` style scaffold in place: Vite + React + TS, Tailwind, shadcn/ui init, Lucide, fontsource.
3. Rust: `paths`, `logging`, `error`, `db` (open + PRAGMAs + migration runner + 0001_core), `commands::app` (`app_info`, `get_settings`, `set_setting`).
4. UI shell: sidebar (grouped nav from brief), top bar with search/command palette, theme toggle (light/dark/system), error boundary, toasts, empty-state component. Modules not yet built render an honest "Not yet available in this version" state.
5. Tests: Rust (migration runner, PRAGMAs, settings roundtrip, checksum mismatch), Vitest (theme, nav, api wrapper), Playwright smoke against the Vite dev server with mocked IPC only for UI tests.
6. Verify restart persistence manually and by an integration test that opens/closes the DB twice.

## Requirement tracking
Requirements are tracked in `PROGRESS.md` with IDs (REQ-xxx) mapped to source files, tests and verification results.
