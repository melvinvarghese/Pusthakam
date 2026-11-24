# Known Issues

| ID | Severity | Issue | Status |
|---|---|---|---|
| KI-001 | Blocker | Toolchain missing and low disk space on `C:`. Installed to `D:\dev` on 2026-11-14 with user approval. | RESOLVED |
| KI-002 | Low | `git` not on PATH. GitHub Desktop's bundled git works (path in CLAUDE.md). | Workaround |
| KI-003 | Info | Node v26 is very new; everything passes with it. | Monitoring |
| KI-004 | Medium | Commands were unauthenticated. Now every data command checks the signed-in user's permission in Rust (verified by IPC tests). Only `setup_status`, `complete_setup` (works once), `login`, `logout`, `current_session` and `app_info` work signed out. | RESOLVED 2026-11-15 |
| KI-005 | Low | App icon is a generated placeholder and does not match the Spine Label brand mark. Needs a final icon before release. | Open |
| KI-006 | Low | `npm audit` reports moderate advisories in dev dependencies; review before release. | Open |
| KI-007 | Low | `C:` has under 10 GB free; build output goes to D: via `CARGO_TARGET_DIR`. | Monitoring |
| KI-008 | Low | Without `?preview`, a plain browser shows "Open Pusthakam as an app" (backend only exists in the desktop app). Intended. | Accepted |
| KI-009 | Low | Vermilion class-8 label band sits near the red overdue colour; re-check when real catalogue data arrives. | Watch |
| KI-010 | Low | `cargo fmt`/`clippy`, Playwright tests and CI are not set up yet. | Open |
| KI-011 | Medium | Staff management and account unlock are implemented. Administrator recovery requires a one-use code generated while signed in. | Implemented |
| KI-012 | Low | Currency amounts assume 2 decimal places (minor units = 1/100). Currencies with other exponents (e.g. JPY) are not handled. | Open |
| KI-013 | Low | Account lockout is per account (5 failures, 5 minutes). There is no separate rate limit for unknown usernames; acceptable for a local desktop app. | Accepted |
| KI-014 | Info | Rust integration tests on Windows need a Common Controls manifest, embedded for test builds only by `src-tauri/build.rs`. | Documented |
| KI-015 | Medium | Koha import follows Koha's default MARC21 framework (952 subfields) as documented, but has not been tried on a real Koha export. A customised framework may use other subfields. The preview always shows what will be imported. | Open: needs a sample file |
| KI-016 | Low | Import supports ISO 2709 MARC (UTF-8) and CSV. MARCXML, JSON, XLSX and MARC-8 encoded files are not supported yet. Non-UTF-8 text is read as Latin-1 and flagged. | Open |
| KI-017 | Low | Publisher and author authority records, subject hierarchies and bulk edit/delete are not built (Phase 6). Subjects are flat headings; publishers are plain text. | Open |
| KI-018 | Low | Import has not been benchmarked on very large files; the whole file is held in memory until confirmed (limit 100 MB). | Open |
| KI-019 | Low | Imported locations, collections and item categories are named after the source codes (for example MAIN, BK); rename them in Library Settings. A code-to-name mapping step is not built. | Open |
| KI-020 | Low | The Accession Register supports batch label sheets on plain paper or full-sheet label stock. Preset die-cut layouts and physical printer and scanner checks remain. | Open |
| KI-021 | Low | Setup wizard dark blue sidebar stops above the bottom of a maximized Windows window (owner screenshot, 24 November 2026). Extend its background through the full available app-content height across all setup steps; preserve form scrolling/navigation when resized. | Open — explicitly deferred by owner; documentation only |
| KI-022 | Low | PDF report export saves directly to Downloads (owner report, 24 November 2026). Add native Save As folder/filename selection, cancellation, overwrite confirmation and saved-path feedback. Preserve report content/design choices. | Open — explicitly deferred by owner; documentation only |
| KI-023 | Medium | Barcode & Printing sidebar destination still shows a not-available-yet placeholder (owner screenshot, 24 November 2026). Audit/reuse existing label/card/receipt tools and finish the functional printing entry point; reconcile completion evidence after implementation. | Open — queued for owner’s combined batch; implementation paused |
| KI-024 | Medium | Owner reports LAN access failing despite server showing Active on port 4318 (24 November 2026). Exact browser error/client workflow unknown; reproduce host and remote OPAC/staff access, then isolate binding/address, reachability, pairing or authentication. Do not infer a cause or mark fixed from status alone. | Open — queued for combined batch; implementation paused |
