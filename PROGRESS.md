# PROGRESS

_Last updated: 2026-11-17_

## Local progress monitor

Pusthakam is **45.6% complete overall**, with 54.9% implemented, 38.9% verified and 10% release-ready. Next product milestone: Implement Phase 3 bulk catalogue operations and covers.

Run `npm run progress:monitor` from this directory and open <http://127.0.0.1:4317/>. Keep the terminal open while using the dashboard. Run `npm run progress:validate` to check the task data and percentages.

| Phase | Implemented | Verified |
|---|---:|---:|
| 0 · Discovery and architecture | 100% | 100% |
| 1 · Application foundation | 83.3% | 83.3% |
| 2 · Setup and configuration | 100% | 100% |
| 3 · Catalogue and accession | 58% | 40% |
| 4 · Circulation and members | 78.2% | 30.4% |
| 5 · Dashboard, reports and printing | 43.7% | 25% |
| 6 · Collection operations | 18.1% | 18.1% |
| 7 · Administration and recovery | 82.1% | 57.1% |
| 7b · LAN mode | 0% | 0% |
| 8 · Release and distribution | 12% | 12% |

Monitor verification passed for the data schema, weighted calculations, loopback serving, missing and invalid JSON, automatic refresh, and roadmap filters. Desktop and mobile browser layout and an adapter-disabled browser smoke test are open. Headless Edge failed to initialize, and Chrome UI access stopped at the browser URL check.

## Handoff for 18 November 2026

- **Checkpoint:** 45.6% overall. Phase 2 is fully verified. On Windows, the custom floor field retained its numeric value of 2 and Acquisitions stayed hidden after restart.
- **Phase 3 in progress:** bulk catalogue archiving is implemented. The catalogue frontend suite passed all 20 tests. The targeted Tauri IPC test passed rollback and permission checks. A native bulk archive walkthrough remains open.
- **Start here:** add and remove a validated local book cover, then build a Windows verification executable and check both covers and bulk archiving in the disposable test library. Phase 3 completion credit for this task is pending.
- **Keep on the roadmap:** the owner deferred Windows Narrator. New passwords must eventually require at least six characters and contain no spaces. Write the illustrated user manual after feature completion.

## Current phase
Write-authorized staff can confirm and archive multiple titles from a catalogue results page. Updates and audit entries share one transaction. Local cover selection and the native walkthrough remain open.

Library Settings can show or hide Acquisitions and Periodicals in the sidebar, dashboard and command palette. The setting uses the existing admin-only settings command. Administrators can add and edit text, number and yes/no fields for the library profile. Rust validates labels and value types. The stored values survived a database reopen, and a fresh Windows settings build is ready. The owner confirmed the Windows settings restart walkthrough worked. Read-only inspection of the fixture database confirmed that the numeric floor field stored 2 and that Acquisitions was set to hidden.

Added arrow-key navigation and a single Tab stop to the theme selector. Let the shared modal manage initial focus in member lookup and catalogue editing so it can restore the caller. The 40 focused tests passed with typecheck and lint. The owner reported that all four Windows keyboard walkthrough steps worked: setup, catalogue and circulation, Escape focus restoration, and arrow-key theme selection. Audited common text and UI colors in light and dark themes. Increased shared field border contrast against its surface from 1.64 to 3.20 in light mode and from 1.88 to 3.64 in dark mode. The development browser preview exposed named circulation inputs, an invalid-identifier error and the Confirm issue dialog. The issue review showed a synthetic borrower, title and due date in dark mode. Narrator is deferred. Browser setup and permission-error messages were checked in the accessibility tree.

The owner confirmed that Ctrl+K opened Members through keyboard navigation and that dark mode stayed active after restarting the Windows verification app. The owner reported completing the disconnected Windows retry. The fresh fixture contains one configured library, administrator, title, copy, member and returned loan. The startup log shows two launches.

The owner launched and reopened the current Windows build against a disposable database. The live database reached schema 15 and kept the seeded catalogue record. Exactly one pre-upgrade snapshot remained at schema 14 with the same record. Both passed integrity checking.

The owner approved full migration of available data and history from Koha and other library systems, plus a single designated LAN host with no application-imposed client cap.

A verified new daily snapshot rotates older daily files. Manual backups and backups queued for restore are retained. Administrators can copy a saved backup to an existing folder outside Pusthakam app data. The separate SQLite file passes integrity checking before publication. Automatic off-device copies and restore from a file on a fresh installation remain open.

The owner confirmed issue and return in the native Windows verification library. Issue, return, renewal and reservations accept a book barcode or accession number. Codes that match two copies require a unique identifier. The native overdue-list and name lookup walkthroughs remain open. Native printing, restore and a real Koha export remain open.

## Deferred changes

- New passwords must have at least six characters and contain no spaces. Update setup, staff password screens and backend validation when implementing the rule.
- Publish an illustrated Pusthakam user manual after feature completion, covering setup, daily workflows, migration, backups and troubleshooting.

- Verify existing member photo upload and removal from the member card in the native app.
- Add optional locally stored profile photos for administrators and library staff.
- Add local cover selection. When a record has an ISBN and no local cover, allow lookup from a public image source when online, without an API key or service account. Keep downloaded covers available offline.
- Require new administrators to save and type back a generated, single-use recovery code during setup. Offer a .txt export and explain where to store it securely. Text export is now available in the existing administrator recovery dialog.
- For loss of the primary recovery code, evaluate a recovery credential scoped to a single library and held by its authorized recovery contact.

- Add a configurable off-device backup destination and restore from a saved file on a fresh installation. Keep the previous copy until the replacement passes integrity checks.

## What exists
- **Inventory:** save a location or shelf check, scan barcodes and review matched, missing and unexpected copies. Send, receive and cancel tracked copy transfers between locations or shelves.
- **Staff and recovery:** manage roles, activation, unlocks and password changes. Administrators can generate a one-use recovery code before losing access.
- **Database Backups:** verified manual snapshots and daily snapshots enabled by default on first launch each day or when the date changes while the app is open. Check saved files and their schema version, or queue a restore. Startup keeps a recovery copy before restoring and a snapshot before upgrading an existing database.
- **Audit Log:** filter append-only events by local date, staff name and action code.
- **Members:** permission-checked directory with paginated name and member-number search, local contact details, expiry defaulted from the selected category, and audited profile and status changes.
- **Member loan history:** the Loans action shows current, overdue and returned copies for active and archived members, with open loans first. Staff with read-only member access can view it.
- **Member cards and photos:** active member cards use the configured library name, membership expiry, and a Code 128 member number where encodable. Photos are optional, stored locally as bounded PNGs, and re-encoded to clear metadata.
- **Koha patrons:** preview a UTF-8 CSV, map Koha category codes to active local categories, and import eligible rows together. Leading zeroes in card numbers and valid expiry dates are preserved. Duplicate numbers and invalid rows are reported and skipped.
- **Circulation:** find a member by number or name and a copy by barcode or accession number, review the borrower, title and due date, then confirm issue or return. Renew by scanning a copy and reviewing its current and proposed due dates. Renewal extends the current due date using the configured loan rule, skips holidays, and blocks overdue loans and loans that reached their renewal limit. Renewal history and audit events are saved together with the new due date.
- **Overdue loans:** Circulation lists open loans whose due date is before the local calendar date, oldest due first. Staff can start a return from the list and the returned loan clears after confirmation.
- **Reservations:** reserve a particular copy by barcode. When available, it goes to the earliest eligible member and stays held for the configured number of days. Expired and cancelled holds advance the queue. A waiting member blocks renewal.
- **Fines and copy condition:** preview an overdue fine before return. Issue-time terms govern assessment. Existing open loans use their configured terms at upgrade. Staff can record payments, and authorized staff can waive balances or close loans as lost or damaged. Fine history records individual entries.
- **Dashboard and reports:** live counts for catalogue stock, active members, circulation, ready holds, and outstanding fines. Reports provide date-range circulation activity, most borrowed titles, stock by location, all-time fines by currency, daily catalogue additions and member totals by category, with CSV export and printing. CSV exports include the totals shown in each report.
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
| Phase 2 settings after restart | Owner confirmed the walkthrough. The numeric floor value was 2 and Acquisitions was saved as hidden. |
| Disconnected Windows walkthrough | Owner reported completing the retry. Verified the configured fixture, returned loan and two startup entries. |
| Offline production asset and CSP audit | Passed. HTML and CSS assets are local, and the CSP restricts content to local origins. A disconnected native walkthrough remains open. |
| Native Windows keyboard and theme | The owner confirmed keyboard navigation to Members and dark mode persisted after restart. |
| Phase 1 shell tests | 17 tests, typecheck and lint passed on rerun after correcting the loading text assertion. Native walkthrough is open. |
| Interactive Windows upgrade and restart | Owner launched twice. Schema 15 and the schema 14 snapshot retained the test record and passed integrity checks. |
| Phase 1 database upgrade and reopen | All 13 passed on rerun after correcting the new integrity query. The native build compiled, but its first launch returned WebView2 error 0x800700AA. |
| Copy saved backup to a folder | 13 IPC tests, the backup screen test, typecheck and lint passed. A native removable drive remains unverified. |
| Daily backup rotation | All 13 Tauri IPC tests passed, along with the backup page test and typecheck. Verified rotation and retention of manual and queued-restore files. |
| Default daily backup | Backup IPC and frontend tests, typecheck and lint passed. Automatic off-device copies and daily rotation remain open. |
| Recovery text export | Typecheck, lint and eight related tests passed. Saving the recovery file in native Windows remains unverified. |
| Barcode and accession lookup | 13 IPC and five circulation frontend tests passed, with typecheck and lint. The new input still needs a native Windows check. |
| Windows borrower lookup build | Frontend and isolated native build passed. Interactive name lookup and overdue return remain open. |
| Windows issue and return | The owner confirmed both actions worked in the verification library. |
| Borrower lookup by name | Five circulation tests, typecheck and lint passed. Native lookup remains open. |
| Interactive Windows launch | The native setup wizard opened, confirmed by the owner screenshot and the WebView page-load marker. Setup and circulation still need a walkthrough. |
| Windows startup callback probe | The isolated build compiled. Launch returned OS error 5 before the setup entry marker or disposable database directory appeared. |
| Report refresh handling | Seven report tests, typecheck and lint passed. Verified export and print recovery after retry. |
| Isolated Windows startup recheck | Process exited with code 101 and reported OS error 5 during Tauri setup. Native circulation verification remains open. |
| Report CSV exports | Six report tests, 13 Tauri IPC tests, typecheck, lint and build passed. Checked displayed totals, quoted labels and currency amounts. |
| `npm test -- --maxWorkers 2` after CSV changes | The first run had 101 passes and a circulation payment timeout. All 102 passed on rerun. |
| `npm test -- --maxWorkers 2` | 90 passed, including cross-page label selection, Code 128 print content, payment receipts and filtered inventory sheets. |
| `cargo test --offline --manifest-path src-tauri/Cargo.toml` | 99 unit and 8 Tauri IPC tests passed, including the return fine estimate and recorded amount. |
| `npm run lint`, `npm run typecheck`, `npm run build` | Passed. |
| Windows startup retry | An instrumented build created an isolated database and WebView2 profile. No page-load event arrived in 30 seconds. |
| Staff, inventory and dashboard | 102 Rust unit, 13 Tauri IPC and 97 frontend tests passed, along with build and lint. |
| Isolated native Windows build | Compiled. Launch returned Access is denied (OS error 5) before any WebView page load. |
| Backup, restore and audit log | 102 Rust unit, 10 Tauri IPC and 92 frontend tests passed, as did build and lint. Native Windows restore remains unverified. |
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
| REQ-DASH, REQ-RPT | IN PROGRESS | Live dashboard and six reports with verified CSV exports. Broader report coverage and more export formats remain. |
| REQ-ACQ, REQ-SER | NOT STARTED | Phase 6 |
| REQ-BAK | NOT STARTED | Phase 7 |

## Next milestone
Run the circulation and printing walkthrough on an interactive Windows desktop and validate a real Koha export. Continue with additional reports and exports. Open questions: past circulation history in a Koha migration, or only current loans? Which computer will be the library server (Phase 7b)?
