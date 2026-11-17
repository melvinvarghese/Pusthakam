# PROGRESS

_Last updated: 2026-11-17_

## Local progress monitor

Pusthakam is **30.7% complete overall**, with 40.3% implemented, 18.5% verified and 10% release-ready. The current task is Phase 7 database backups. Next product milestone: Verify the Windows circulation workflow.

Run `npm run progress:monitor` from this directory and open <http://127.0.0.1:4317/>. Keep the terminal open while using the dashboard. Run `npm run progress:validate` to check the task data and percentages.

| Phase | Implemented | Verified |
|---|---:|---:|
| 0 · Discovery and architecture | 60% | 60% |
| 1 · Application foundation | 55.5% | 0% |
| 2 · Setup and configuration | 80% | 80% |
| 3 · Catalogue and accession | 58% | 40% |
| 4 · Circulation and members | 78.2% | 19.5% |
| 5 · Dashboard, reports and printing | 43.7% | 0% |
| 6 · Collection operations | 0% | 0% |
| 7 · Administration and recovery | 0% | 0% |
| 7b · LAN mode | 0% | 0% |
| 8 · Release and distribution | 12% | 12% |

Monitor verification passed for the data schema, weighted calculations, loopback serving, missing and invalid JSON, automatic refresh, and roadmap filters. Desktop and mobile browser layout and an adapter-disabled browser smoke test are open. Headless Edge failed to initialize, and Chrome UI access stopped at the browser URL check.

## Current phase
Phase 4 circulation and member workflows are implemented, and the first Phase 5 dashboard and reports are available. The native Windows build succeeds. A disposable WebView2 smoke run applied the database migrations but did not produce a page-load result. An interactive desktop walkthrough, physical print checks, and validation with a real Koha export remain open.

## What exists
- **Members:** permission-checked directory with paginated name and member-number search, local contact details, expiry defaulted from the selected category, and audited profile and status changes.
- **Member loan history:** the Loans action shows current, overdue and returned copies for active and archived members, with open loans first. Staff with read-only member access can view it.
- **Member cards and photos:** active member cards use the configured library name, membership expiry, and a Code 128 member number where encodable. Photos are optional, stored locally as bounded PNGs, and re-encoded to clear metadata.
- **Koha patrons:** preview a UTF-8 CSV, map Koha category codes to active local categories, and import eligible rows together. Leading zeroes in card numbers and valid expiry dates are preserved. Duplicate numbers and invalid rows are reported and skipped.
- **Circulation:** find a member by number and a copy by barcode, review the borrower, title and due date, then confirm issue or return. Renew by scanning a copy and reviewing its current and proposed due dates. Renewal extends the current due date using the configured loan rule, skips holidays, and blocks overdue loans and loans that reached their renewal limit. Renewal history and audit events are saved together with the new due date.
- **Overdue loans:** Circulation lists open loans whose due date is before the local calendar date, oldest due first. Staff can start a return from the list and the returned loan clears after confirmation.
- **Reservations:** reserve a particular copy by barcode. When available, it goes to the earliest eligible member and stays held for the configured number of days. Expired and cancelled holds advance the queue. A waiting member blocks renewal.
- **Fines and copy condition:** preview an overdue fine before return. Issue-time terms govern assessment. Existing open loans use their configured terms at upgrade. Staff can record payments, and authorized staff can waive balances or close loans as lost or damaged. Fine history records individual entries.
- **Dashboard and reports:** live counts for catalogue stock, active members, circulation, ready holds, and outstanding fines. Reports provide date-range circulation activity, most borrowed titles, stock by location, all-time fines by currency, daily catalogue additions and member totals by category, with CSV export and printing.
- **Batch copy labels:** select copies across pages of the Accession Register and preview a two-column sheet for plain paper or full-sheet label stock. Labels include the title, accession number, call number when available, and a Code 128 barcode or text fallback.
- **Receipts and inventory:** Fine History opens printable receipts for individual recorded payments. The Accession Register prints the current filtered page with accession, title, call number, location, status and a column for staff to mark checked copies.
- **Phases 1-2:** Tauri shell, Spine Label design, SQLite + migrations, first-launch wizard, staff sign-in with roles enforced in Rust, Library Settings (categories, loan rules, collections, locations, shelves, accession number format), Online Tools menu.
- **Catalogue:** records with multiple contributors (author, editor, translator…) and subjects, ISBN-10/13 validation and conversion, classification (Dewey/UDC/LC scheme + number), archive/restore (never deleted), duplicate warning (same ISBN, or same title + author + year), edit with audit trail.
- **Copies and accession:** many copies per title, each with its own accession number, barcode, call number (suggested from classification + author), collection/location/shelf, acquisition date/source/price/vendor, status (available, in repair, damaged, lost, withdrawn, in transit; on loan/reserved set by circulation later) and a permanent history. Accession numbers follow a configurable prefix/digits/counter and skip numbers already used. Code 128 barcode and printable label per copy.
- **Search:** full-text (prefix matching, any word order, across title, author, subject, publisher, ISBN, class), including Malayalam; a scanned barcode, accession number or ISBN jumps straight to its record; filters (collection, available now, archived), sorting, server-side pagination. **Accession Register** lists every copy with filters and exports all matching copies as CSV.
- **Import (owner request: migrate from Koha etc.):** MARC (ISO 2709) with a Koha profile (holdings in field 952 become copies with barcode, call number, location, shelf, collection, item type, price, acquisition date, lost/withdrawn/damaged flags) or generic MARC (titles only); CSV with automatic column recognition and manual column mapping, rows of one title grouped into copies. Preview first (counts, sample, every warning, what new locations/collections/categories would be created); nothing is saved until confirmed; one transaction; duplicates by ISBN can be skipped, merged or created anew; barcode clashes skip only that copy; damaged records are reported; summary at the end; audit entry.

## Verification log (2026-11-17)

| Check | Result |
|---|---|
| `npm test -- --maxWorkers 2` | 90 passed, including cross-page label selection, Code 128 print content, payment receipts and filtered inventory sheets. |
| `cargo test --offline --manifest-path src-tauri/Cargo.toml` | 99 unit and 8 Tauri IPC tests passed, including the return fine estimate and recorded amount. |
| `npm run lint`, `npm run typecheck`, `npm run build` | Passed. |
| Windows startup retry | An instrumented build created an isolated database and WebView2 profile. No page-load event arrived in 30 seconds. |
| Member reports, accession CSV and catalogue/circulation checks | 91 frontend, 101 Rust unit and 8 Tauri IPC tests passed. Confirmed archive and renewal rollback, hold handover and UDC/LCC handling. The first export test failed on an extra mock argument and passed on rerun. |
| Catalogue growth | 90 frontend tests, 99 Rust unit tests, 8 Tauri IPC tests, build and lint passed. Counts matched seeded records and empty dates were shown. Native print output remains unverified. |
| Fine summary | 90 frontend tests and 8 Tauri IPC tests passed. Build and lint passed. Exact decimal rendering, assessment and settlement totals, and signed-out denial were checked. |
| Rendered label sheet | Headless Edge failed to initialize its renderer. A4 and Letter pagination, physical printing and scanner readability remain open. |

## Verification log (2026-11-15)
| Check | Result |
|---|---|
| `cargo test` | 103 passed: 98 unit (catalogue, search incl. Malayalam and hostile input, accession numbering, ISBN, MARC reader incl. truncated/garbage input, Koha mapping, CSV, import analysis/commit/duplicates/rollback, plus earlier phases) + 5 command-layer tests through Tauri's real IPC |
| `npm test` | 81 passed (catalogue, record form + duplicate warning, copies, status, labels, accession register, import wizard, plus earlier phases) |
| typecheck, eslint, production build | clean; production bundle contains no dev preview data and no gallery |
| Existing database upgrade | Real app launched on the v3 database: migrated to v4 in place (including the Malayalam search index), earlier data intact, `integrity_check` ok |
| Visual review | Catalogue, record, import preview reviewed in the browser preview with synthetic data |

## NOT yet verified
- The new screens inside the real Tauri window; a real import through the desktop app.
- **Koha mapping against a real Koha export.** It follows Koha's default MARC21 framework as documented; a library can change its framework. Please provide a small anonymised sample export to verify.
- Import speed on very large files (tens of thousands of records) is not benchmarked.
- Network-off run, `cargo fmt`/`clippy`, Playwright, CI, installers.

## Requirements status
| ID | Status | Notes |
|---|---|---|
| REQ-CAT | IN PROGRESS | CRUD, contributors, subjects, duplicates, search, archive done; bulk operations, cover images, export pending |
| REQ-ACC | IN PROGRESS | Copies, numbering, statuses, register done; transfers, stock verification (Phase 6) pending |
| REQ-SRCH | IN PROGRESS | Catalogue and copies search done; member search with Phase 4; saved filters pending |
| REQ-IMP | IN PROGRESS | MARC/Koha and catalogue CSV import, Koha patron CSV import. Loan and fine import, JSON/XLSX/MARCXML, and catalogue export remain. |
| REQ-PRN | IN PROGRESS | Single and batch copy labels, member cards and due slips. Physical print checks and other print formats remain. |
| REQ-CLS | IN PROGRESS | Scheme + number on records; authority control and classification browser (Phase 6) pending |
| REQ-CFG, REQ-ADM, REQ-SEC, REQ-UI, REQ-OFF | IN PROGRESS | see previous entries |
| REQ-NET | NOT STARTED | Added 2026-11-15: several computers on one library (Phase 7b), see roadmap |
| REQ-MEM | IN PROGRESS | Directory, loan history, photos, cards and Koha patron import. |
| REQ-CIRC | IN PROGRESS | Issue, return, renewal, overdue tracking, copy reservations, overdue fines and loss/damage close-outs. Desktop verification remains. |
| REQ-DASH, REQ-RPT | IN PROGRESS | Live dashboard, circulation activity and most borrowed titles with CSV export. Additional reports and formats remain. |
| REQ-ACQ, REQ-SER | NOT STARTED | Phase 6 |
| REQ-BAK | NOT STARTED | Phase 7 |

## Next milestone
Run the circulation and printing walkthrough on an interactive Windows desktop and validate a real Koha export. Continue with additional reports and exports. Open questions: past circulation history in a Koha migration, or only current loans? Which computer will be the library server (Phase 7b)?
