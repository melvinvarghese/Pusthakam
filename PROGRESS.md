# PROGRESS

_Last updated: 2026-11-18_

## Local progress monitor

For continuation on another computer, read [SESSION_HANDOFF.md](docs/SESSION_HANDOFF.md) and the current task in PROGRESS.json. Uncommitted changes remain local until Git sync.

Pusthakam is **50.8% complete overall**, with 62.1% implemented, 41.8% verified and 10% release-ready. Next product milestone: verify member suggestions and tray behavior in Windows.

Run `npm run progress:monitor` from this directory and open <http://127.0.0.1:4317/>. Keep the terminal open while using the dashboard. Run `npm run progress:validate` to check the task data and percentages.

| Phase | Implemented | Verified |
|---|---:|---:|
| 0 · Discovery and architecture | 100% | 100% |
| 1 · Application foundation | 83.3% | 83.3% |
| 2 · Setup and configuration | 100% | 100% |
| 3 · Catalogue and accession | 92% | 50% |
| 4 · Circulation and members | 84.7% | 36.9% |
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

## Work on 18 November 2026

Local JPG/PNG cover selection and removal compile on Windows. Covers are bounded, decoded and re-encoded as PNG and saved in the database. Bulk archiving and covers await their native walkthrough. All 13 IPC tests and 20 catalogue frontend tests passed, as did typecheck and lint. The native cover executable is ready. Added Load cover by ISBN to records with an ISBN and no saved cover. The explicit Open Library lookup omits credentials and referrer, limits response size and time, converts the image and saves it locally. Existing saved covers take precedence. The combined covers and ISBN Windows executable is ready. Live lookup and offline persistence still need a walkthrough.

## Automatic ISBN covers

ISBN cover lookup now starts automatically when opening a record with an ISBN and no saved cover. It retries on reconnect, preserves local cover priority, and quietly tolerates absent covers. Authorized catalogue editors save the downloaded cover locally. Read-only sessions display the fetched image without writing. The combined Windows build compiled. Live network and reconnect verification remains open.

## Saved search checkpoint

Saved catalogue searches support names, text, filters and sort order, plus update and removal. Saved filters are scoped to the staff account on this computer. The Windows restart walkthrough remains open. All 22 focused tests, production build/typecheck and lint passed. An initial build failure from unsupported test query options was corrected. Update and removal now have persistence tests. The owner confirmed fetched covers work offline. Both Windows native builds passed; `pusthakam-phase3-searches.exe` is ready using the existing verification library.

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

- At application startup, show a loading screen for at least three seconds. Requested by the owner on 18 November 2026; implement later.

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

## Saved search reapplication fix

Owner reported that saving a search did not reflect as a search. The selected dropdown could not rerun its already selected value after filters changed. Added Run saved search and instructions distinguishing the name from catalogue search words. Both regression tests, frontend build/typecheck, lint and both Windows native builds passed. Updated executable: `pusthakam-phase3-searches-fixed.exe`. Interactive verification remains open.

## Saved search execution and label correction

Owner reported Run saved search did not run a query. Applying unchanged filters did not change the query key, so added an explicit run counter to execute a fresh search on every run. Checking catalogue-level regression tests. Single labels now show matching barcode/accession numbers once; both label formats have a heading defaulting to the library name with custom text or blank allowed for each print job. All 23 focused tests, frontend build/typecheck, lint and both Windows native builds passed. Updated executable: `pusthakam-phase3-labels-search.exe`. Interactive saved-search and print checks remain open.

## Saved search no-match investigation

Owner confirms label printing works. Saved Animal Farm search reportedly leaves The Man Who Was Thursday visible. The exact catalogue-level test with simulated responses passes before changes, so the native cause is unresolved. Added visible saved search words, immediate application without typing debounce, and removed previous-result placeholders. Follow-up native observation required; no verification credit added.

No-match checkpoint: all 24 focused tests, frontend build/typecheck, lint and both Windows native builds passed. `pusthakam-phase3-search-results.exe` shows the stored words and applies a saved search immediately without old-result placeholders. Native cause remains unconfirmed. Progress stays 47.1%; no new verification credit.

## Saved search screenshot diagnosis

Owner screenshots show Animal Farm and Thursday entries both saved with empty search text: All titles (no search words saved). This establishes the cause of the observed all-title results; earlier query-rerun fixes did not address it. Save controls now follow the main search box, the name field explains it is a label, empty default searches are rejected, and existing entries can explicitly use their saved name as search words. Filter-only searches remain supported. Regression checks and refreshed native build are in progress.

Empty-query checkpoint: 27 focused tests pass. Build output verification was interrupted by automatic review rejecting a Visual Studio telemetry connection to an unrequested external endpoint. No retry or workaround attempted and no new native executable built. Existing app workaround: select saved entry, enter actual words in main search box, Update saved search. Standalone typecheck and lint passed.

Owner confirmation: after understanding saved names versus catalogue search words and correcting the entry, the owner confirmed saved search works. New control clarity/validation build remains pending, as do remaining restart/update/removal checks.

## Pause handoff: LAN browser and public OPAC idea

Owner requested staff use via same-network phone/computer browsers and anonymous OPAC. Reviewed existing Phase 7b plan: one-host database already planned, but clients assumed desktop installations and OPAC was absent. Added browser/OPAC scope and acceptance checks to the roadmap, specification, decisions and LAN_BROWSER_OPAC.md. LAN-CLIENT/OPS criteria expanded; original weights retained and no implementation credit claimed. LAN remains not started. Host must run and be reachable; server enforces staff permissions and exposes only public catalogue fields to visitors.

Pause requested. Resume previous phase order, preserving saved search and label confirmations and pending refreshed native build. The active phase remains Phase 3. Browser staff access and OPAC are queued for Phase 7b; normal phase order continues and LAN implementation has not started.

## Resumed Phase 3: JSON catalogue import

Building JSON import using the existing staged workflow and transactional commit. Nested copies and supported bibliographic fields will be validated and previewed. MARCXML/XLSX and native acceptance remain open. LAN/OPAC stays in Phase 7b.

JSON import checkpoint: added strict UTF-8 JSON array parsing with bibliographic fields, contributors/subjects and nested copies. Unknown fields, malformed types, numeric identifiers, fractional/negative minor-unit prices and circulation statuses are rejected. UI detects .json files and uses existing preview/transactional commit; format documented in docs/JSON_CATALOGUE_IMPORT.md. All 8 import frontend tests, typecheck and lint passed. Initial frontend run found an outdated three-option assertion; corrected to four including JSON. Rust IPC compilation began but automatic review blocked an unexpected Visual Studio telemetry connection; no retry/workaround attempted. Parser/IPC tests added but unconfirmed, including later guards. Native JSON verification and MARCXML/XLSX remain open. Overall remains 47.1%; partial task earns no full implementation credit.

Owner approved the previously blocked Visual Studio telemetry connection. Resumed JSON import backend verification and preparation of the next Windows test build.

JSON verification after approval: real IPC JSON preview/commit test passed, including no preview writes, exact identifier/money/status preservation, malformed later-record rejection and unauthenticated/viewer denials. All 30 Rust import tests passed, including new strict JSON parsing tests. Production frontend build passed. Windows test build in progress; docs/examples/catalogue-import.json supplies one title and one available copy for the disposable library. MARCXML/XLSX and native walkthrough remain open.

Windows JSON test executable built successfully: pusthakam-phase3-json-import.exe, using the existing disposable library and including saved-search clarity fixes. Sample docs/examples/catalogue-import.json has one Animal Farm title and one available copy (barcode/accession JSON-DEMO-001; price 12550 minor units). Check preview shows one record/one copy without a write, confirm, then inspect title/copy details. The standard repository build also passed. Remaining format work is MARCXML and XLSX; no full format-task completion credit yet.

## Phase 3 continuation
Owner confirmed JSON import worked in the Windows test. Current task: IMP-FORMATS, MARCXML followed by XLSX. LAN browser access and OPAC remain in Phase 7b.


### MARCXML checkpoint

MARCXML now supports a single MARC21 record or collection, namespace prefixes, standard entities, and Koha 952 holdings through the existing MARC mapping. Preview and confirmed commit share the established validation and transaction path. DTDs, custom entities and malformed XML are rejected. Sample and walkthrough: `docs/MARCXML_CATALOGUE_IMPORT.md` and `docs/examples/catalogue-import.xml`.

Passed: 33 Rust import tests, 9 import dialog tests, lint and production build/typecheck. Initial parser API, encoding and source-count test failures were corrected and rerun successfully. JSON Windows success is owner-confirmed. Native MARCXML verification remains open. IMP-FORMATS remains in progress with no additional completion credit because XLSX is not implemented.

XLSX blocker: `cargo add calamine --manifest-path src-tauri/Cargo.toml` failed because crates.io is unreachable and the reader is not cached. Next: native MARCXML walkthrough, obtain the spreadsheet reader, then implement XLSX. LAN browser access and OPAC stay Phase 7b.

All 14 existing Tauri IPC tests passed. Progress recalculation and validation passed at 47.1% overall, 57.4% implemented, 38.9% verified and 10% release-ready. Timestamp precision was normalized after the initial validator failures. `git diff --check` passed.

Windows debug compilation passed: `cargo build --offline --manifest-path src-tauri/Cargo.toml`. A standalone verification executable with bundled assets and the native MARCXML walkthrough remain open.

## Continued MARCXML verification

Current IMP-FORMATS work: real IPC coverage, parser edge cases and standalone Windows verification build using the existing disposable library. Retried XLSX reader download; crates.io remains unreachable. Phase 3 continues; LAN/OPAC stay Phase 7b.

Focused continuation checks passed: `cargo test --offline --manifest-path src-tauri/Cargo.toml marcxml` ran three parser/service tests and one real command test. Verified no preview writes, Koha/titles-only profiles, exact identifier/money/call-number preservation, malformed rejection, and signed-out/viewer denials on preview and commit. Fixed silently ignored misplaced CDATA/entity content with regression coverage. The initial isolated fresh-target build was stopped to reuse the compiled target cache; replacement standalone build is in progress. XLSX dependency retry failed again.

Standalone MARCXML Windows build passed with `tauri/custom-protocol` and embedded production assets. Ready: `C:\Users\Melvin\AppData\Local\Temp\pusthakam-native-verification-cea843fbd8cb4e9bbaf6c2274b98c468\pusthakam-phase3-marcxml-import.exe`. Uses the existing disposable library and a MARCXML test window title. Valid and deliberately malformed samples sit beside the executable. Native launch/walkthrough remains open. Next: preview the valid sample (one record, one copy), confirm and inspect XML-000123 / 020 XML / 125.50; the malformed sample must show an error without adding its title. XLSX reader remains unavailable after retry; no implementation credit added.

## MARCXML Windows confirmation

Owner reported the valid sample was okay and the malformed sample showed an error. Recorded the basic native walkthrough as passed. Detailed field reconciliation and titles-only native profile were not explicitly reported; automated checks cover those behaviors. Current Phase 3 task is XLSX catalogue import (IMP-FORMATS), pending spreadsheet-reader availability. No full format-task credit yet. LAN/OPAC stay Phase 7b.

XLSX dependency retry encountered Windows TLS error `schannel: AcquireCredentialsHandle failed: SEC_E_NO_CREDENTIALS`. Certificate verification remains enabled. Next action: retry `cargo add calamine --manifest-path src-tauri/Cargo.toml` and `cargo fetch --manifest-path src-tauri/Cargo.toml` from interactive PowerShell in D:\Pusthakam. Progress totals remain 47.1% overall, 57.4% implemented, 38.9% verified and 10% release-ready.

## XLSX implementation resumed

Owner successfully installed calamine 0.30.1, compatible with declared Rust 1.80, and fetched all dependencies. The dependency blocker is cleared. Current IMP-FORMATS work: XLSX workbook validation, column mapping and shared preview/commit, automated checks and Windows verification build.

XLSX parser and mapping UI implemented. The importer validates bounded ZIP expansion/cell coordinates before reading one worksheet, preserves text identifiers, rejects formulas and invalid workbook structures, and reuses the CSV mapping/grouping and staged transaction path. Added documented valid and negative fixtures. All 38 Rust import tests and 10 frontend dialog tests passed, with production build/typecheck and lint. Full IPC checks and standalone Windows verification build are in progress.

Full Tauri IPC suite passed all 16 tests, including XLSX mapping, preview without writes, one-title/two-copy commit, exact identifier/money/call-number/status preservation, malformed/formula rejection without new titles, and signed-out/viewer denials. All three formats now meet implementation criteria; IMP-FORMATS is implemented, not verified. Native XLSX walkthrough and explicit remaining MARCXML checks are open.

Standalone Windows XLSX build passed with embedded frontend and the existing disposable library: `C:\Users\Melvin\AppData\Local\Temp\pusthakam-native-verification-cea843fbd8cb4e9bbaf6c2274b98c468\pusthakam-phase3-xlsx-import.exe`. Samples are beside the executable. Native walkthrough: preview catalogue-import.xlsx (one title/two copies), confirm and inspect barcodes/accessions 000XLSX123/000XLSX124, call number 020 XLSX, price 125.50, and damaged second copy. Formula sample must show an error without additions. Overall is 48.2%, implementation 59.2%, verification 38.9%, release readiness 10%; Phase 3 is 82% implemented and 40% verified. Progress validation and whitespace checks passed after aligning currentTask status with implemented. Next is native XLSX acceptance, then Phase 3 catalogue/holdings export. LAN/OPAC remain Phase 7b.

## Import formats verified in Windows

Owner confirmed the XLSX walkthrough worked. With earlier JSON and MARCXML Windows confirmations and recorded parser, service, IPC and UI tests, IMP-FORMATS is verified. Next is EXP-CATALOGUE: catalogue and holdings export with supported-field round-trip checks, in Phase 3. LAN browser access and OPAC remain Phase 7b.

## Catalogue and holdings export

Current Phase 3 task EXP-CATALOGUE: selected-record exports with holdings in JSON, CSV, XLSX and MARC exchange formats, plus supported-field reimport checks. Existing accession CSV remains available. LAN/OPAC stay Phase 7b.

Export checkpoint: all five formats implemented from an explicit selection and one database snapshot. Two real database round-trip service tests and 36 focused frontend tests passed, with build/typecheck and lint. Supported-field tests include UDC, Unicode, exact prices, leading-zero identifiers and title-only records. Initial IPC denial check used an ineffective cached-session mutation; changed the test to revoke database permission and reran the full Rust suite. Native save/open/repreview acceptance remains open.

Catalogue exchange implementation checks passed: all 115 Rust unit tests, 17 IPC tests and 36 focused frontend tests, plus build/typecheck and lint. The corrected permission test revokes catalogue.read in the fixture database and confirms denial through real IPC. EXP-CATALOGUE is implemented; native save/open/repreview checks remain open. Standalone export verification build is running.

Standalone catalogue export Windows build passed: `C:\Users\Melvin\AppData\Local\Temp\pusthakam-native-verification-cea843fbd8cb4e9bbaf6c2274b98c468\pusthakam-phase3-catalogue-export.exe`. Uses the existing disposable library with embedded frontend. Export walkthrough copied beside it. Native check: select the XLSX verification title, prepare/save each format, open the saved files and repreview through Import without committing duplicates. Expect one title/two copies with identifiers, price and damaged status intact; use Koha profile for MARC. EXP-CATALOGUE remains implemented pending this acceptance. Overall 49.8%, implementation 61%, verification 40.7%, release readiness 10%; Phase 3 92% implemented and 50% verified. Progress recalculation/validation and diff whitespace checks passed.

## Export Save As destination

Owner confirmed the export works and requested folder/filename selection when Save is clicked. Current EXP-CATALOGUE work adds Windows Save As, cancellation/error handling and saved-path feedback. Five-format native reconciliation remains separately open.

Windows Save As checkpoint: added a read-permission-checked command that opens a native folder/filename picker, confirms overwrites and writes only the OS-selected destination. Frontend receives no general filesystem permission or arbitrary destination command. Cancel writes nothing and retains the export; success shows the path; errors allow retry. Two Rust service and three UI tests passed, plus build/typecheck and lint. Existing browser/non-Windows download behavior is retained. Native dialog/overwrite/cancel acceptance remains open.

Save As IPC verification passed: the new command rejects signed-out users, revoked catalogue.read, invalid filenames and malformed bytes before a dialog is opened. All 18 IPC tests passed. Progress validation and whitespace checks passed; native Windows Save As/overwrite/cancel walkthrough remains open. The requested startup loading screen (minimum three seconds) is documented as deferred; startup code is unchanged.

Standalone Save As test build passed: `C:\Users\Melvin\AppData\Local\Temp\pusthakam-native-verification-cea843fbd8cb4e9bbaf6c2274b98c468\pusthakam-phase3-save-as.exe`, using the existing disposable library. Native test: prepare/export, click Save file, choose folder/name and confirm Saved to path; cancel another save without writing; test overwrite confirmation. Automated service, IPC, UI, build/typecheck and lint checks passed. Native acceptance remains open; totals stay 49.8% overall, 61% implemented, 40.7% verified and 10% release-ready. The startup loading screen request remains deferred and documentation-only.

## Windows Save As confirmed

Owner reported the Save As build works following the folder/filename, cancellation and overwrite instructions. Recorded the walkthrough as passed. Save As is confirmed; specific five-format save/open/repreview reconciliation was not separately reported and remains the export acceptance check. Automated round trips already passed. Startup loading screen remains deferred; LAN/OPAC stay Phase 7b.

## Phase 4: staff-managed calendar

Phase 3 implementation is in place; real Koha acceptance remains blocked on a representative anonymised export, and recorded native checks remain open. Continue the next actionable task in normal order: CIRC-HOLIDAYS, holiday dates and weekly closed days in Library Settings, applied to new issue and renewal due dates. Existing loan due dates will not be rewritten. Startup delay remains deferred; LAN/OPAC remain Phase 7b.

Calendar checkpoint: migration 17 adds recurring closed weekdays without altering existing holiday data or default loan behavior. Administrators can add/edit/remove audited holidays and configure weekly closures. Issue and renewal advance to an open due date and reject stale confirmations; existing loans and fine rules are not rewritten. Two calendar service tests, date validation, real IPC workflow and 14 settings UI tests passed, as did production build/typecheck and lint. Full Rust suite and Windows build are running.

Full calendar regression suite passed: 119 Rust unit tests and 19 IPC tests, including schema 17 migration/upgrade/reopen and existing circulation/backups. The real calendar workflow confirms stale issue/renewal rejection and unchanged stored due dates after calendar changes. CIRC-HOLIDAYS is implemented, awaiting the native Windows walkthrough.

Standalone calendar Windows build passed: `C:\Users\Melvin\AppData\Local\Temp\pusthakam-native-verification-cea843fbd8cb4e9bbaf6c2274b98c468\pusthakam-phase4-calendar.exe`. Uses the existing disposable library and embedded frontend; schema 17 upgrades at launch. Close older test apps first. `LIBRARY_CALENDAR.md` beside the executable explains holiday CRUD, weekly closure persistence after restart, and issue/renewal date checks. Automated checks passed; native walkthrough remains open.

## Member suggestions and maximized startup

Owner confirmed holidays work; CIRC-HOLIDAYS is verified. Current Phase 4 work: live member-name dropdown suggestions for Circulation and Reservations, plus maximized desktop startup. Existing member-number entry and permissions remain. Startup loading delay is still deferred.

Cross-computer continuation: read `docs/SESSION_HANDOFF.md` together with the progress files and CLAUDE.md. Active work includes member suggestions in both circulation flows, maximized startup and close-to-tray/session-preserving restore. Commit/push/pull is needed to transfer local source/doc changes between computers; user databases and secrets are never Git content.

Live lookup implementation passed 11 focused frontend tests, production build/typecheck and lint. Both Issue and Reservations use the same debounced dropdown with keyboard selection, duplicate-name member numbers, stale-response exclusion, pagination and retry. Maximized startup and close-to-tray/Open/Quit are implemented in source; Rust compile/native checks remain pending. `docs/DESKTOP_AND_MEMBER_LOOKUP.md` documents behavior. Session/deferred decisions and cross-computer Git handoff are in `docs/SESSION_HANDOFF.md`, now referenced by AGENTS.md.

Tray-enabled Rust compile passed (`cargo check --offline --manifest-path src-tauri/Cargo.toml`). Close-to-tray is enabled only after tray creation succeeds; Open restores the existing window/session and explicit Quit ends the process. No password storage was added. Native lifecycle checks remain open. Refreshed Windows build is running.

Standalone member lookup/tray Windows build passed: `C:\Users\Melvin\AppData\Local\Temp\pusthakam-native-verification-cea843fbd8cb4e9bbaf6c2274b98c468\pusthakam-phase4-members-tray.exe`. Starts maximized and uses the existing disposable library. Final member UI rerun passed all 11 tests; prior production build/typecheck/lint and Rust tray compile also passed. Native checks: Issue/Reservations live suggestions, X-to-tray, session-preserving Open, and explicit Quit followed by fresh sign-in. Handoff and feature walkthrough docs are current. These local source/doc changes must be committed, pushed and pulled to transfer development state between computers; no Git push was performed.

## Standing progress and handoff rule

Owner requires **PROGRESS.md and PROGRESS.json**, **docs/SESSION_HANDOFF.md**, and affected feature docs to be updated after every meaningful change, not just at session end or an announced computer switch. Record partial work, decisions, actual verification including failed/incomplete checks, blockers and the exact next step. Check consistency before ending each turn. This is also recorded in AGENTS.md for future work on any computer. Git commit/push/pull is still required to transfer local files. This documentation-only change does not alter implementation credit or the active product milestone.
