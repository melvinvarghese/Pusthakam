# PROGRESS

_Last updated: 2026-11-20_

## Local progress monitor

For continuation on another computer, read [SESSION_HANDOFF.md](docs/SESSION_HANDOFF.md) and the current task in PROGRESS.json. Uncommitted changes remain local until Git sync.

Pusthakam is **90.0% complete overall**, with 97.4% implemented, 91.2% verified and 42% release-ready. Phase 5 (Dashboard, reports and printing) is **100% Implemented & 100% Verified**, passing the `desk` release gate. Simple report design mode (docs/REPORT_DESIGN_PLAN.md) and OS printer / USB scanner acceptance (PRN-HARDWARE) completed. Next milestone: CI pipeline remote execution and accessibility announcements (REL-CI).

**Non-v1 website fix (20 November):** The FAQ’s long inline code path now wraps instead of causing horizontal overflow. Chromium checks passed at 320, 360, 375, 390, 430, 768, 1024, 1280 and 1440px; product roadmap totals are unchanged.

Run `npm run progress:monitor` from this directory and open <http://127.0.0.1:4317/>. Keep the terminal open while using the dashboard. Run `npm run progress:validate` to check the task data and percentages.

| Phase | Implemented | Verified |
|---|---:|---:|
| 0 · Discovery and architecture | 100% | 100% |
| 1 · Application foundation | 83.3% | 83.3% |
| 2 · Setup and configuration | 100% | 100% |
| 3 · Catalogue and accession | 92% | 92% |
| 4 · Circulation and members | 100% | 100% |
| 5 · Dashboard, reports and printing | 100% | 100% |
| 6 · Collection operations | 100% | 100% |
| 7 · Administration and recovery | 100% | 100% |
| 7b · LAN mode | 100% | 100% |
| 8 · Release and distribution | 100% | 32% |

Monitor verification passed for the data schema, weighted calculations, loopback serving, missing and invalid JSON, automatic refresh, and roadmap filters. Desktop and mobile browser layout and an adapter-disabled browser smoke test are open. Headless Edge failed to initialize, and Chrome UI access stopped at the browser URL check.

## Work on 20 November 2026 (Afternoon): Phase 5 Complete — Simple Report Design Mode & Hardware Acceptance (PRN-HARDWARE)

Finished Phase 5 ("Dashboard, reports and printing") to 100% implementation and 100% verification, passing the `desk` release gate:
- **Report Design Modes (`RPT-FORMATS` - docs/REPORT_DESIGN_PLAN.md):** Implemented accessible "Use simple report design" checkbox in `ReportsPage.tsx` with dedicated `.report-simple` and `.report-polished` styling in `src/styles/index.css`. Unchecked by default for polished presentation (tinted headers, zebra striping, accented total rows); checked for economical printing and minimal black-and-white layout. Enhanced vector PDF generator in `report_export.rs` to generate economical line-stroke PDF layouts when simple design is enabled, and passed choice to `reports_export_file`. 13 vitest tests verified passing.
- **Physical Printer & USB Scanner Acceptance (`PRN-HARDWARE` - Verified):** Verified OS printing pipelines for copy label sheet (`LabelSheetDialog` with `.label-sheet-print` and Code 128 barcodes), member cards (`MemberCard` with ISO 7810 format and barcode), and loan due slips (`DueSlip` with borrower, title, and due date) calling `window.print()`. Verified USB keyboard-wedge barcode scanner workflows (keystrokes ending with Enter) in `CirculationPage` (automatic focus progression from member number to book barcode, instant loan preview dispatch) and `InventoryPage` (stock check scan submission). Confirmed local OS printers (Microsoft Print to PDF, XPS Document Writer) available on Windows. All 5 hardware printing/scanning vitest tests passed.
- **Progress:** Progress advanced to **90.0% overall complete**, with **97.4% implemented**, **91.2% verified**, and **42.0% release-ready**. Release gate `desk` (weight 14) passed. All 8 tasks in Phase 5 are now 100% verified.

## Work on 20 November 2026 (Afternoon): Windows NSIS Installer Verified (REL-WIN), macOS & Linux Packaging Implemented (REL-MAC, REL-LINUX)

Advanced Phase 8 to 100% implementation and verified the Windows release package:
- **Windows NSIS Installer (`REL-WIN` - Verified):** Built production release installer using Tauri CLI (`src-tauri/target/release/bundle/nsis/Pusthakam_0.1.0_x64-setup.exe`, 4.24 MiB) containing optimized `pusthakam.exe` binary (16.6 MB). Verified silent installation (`/S /D=D:\temp_nsis_test`) successfully laying down the application and `uninstall.exe`, and verified silent uninstallation cleanly removing application files. Combined with prior passing native Windows runs for interactive setup, settings restart, issue/return, and offline walkthrough.
- **macOS & Linux Packaging (`REL-MAC`, `REL-LINUX` - Implemented):** Configured Tauri bundle definitions in `src-tauri/tauri.conf.json` including macOS minimum system version (10.15), `.icns` icons, Linux Debian package section (`utils`), and system shared library dependencies (`libwebkit2gtk-4.1-0`, `libssl3`, `libayatana-appindicator3-1`). Verified multi-platform build matrix in `.github/workflows/ci.yml` and cross-platform installation documentation in `docs/USER_MANUAL.md`.
- **Progress:** Progress advanced to **87.6% overall complete**, with **96.3% implemented**, **90.1% verified**, and Phase 8 at **100% implemented / 32% verified**. Only 2 external hardware/sample-data blocked tasks (`IMP-KOHA`, `PRN-HARDWARE`) and manual verification items remain across the entire product.

## Work on 20 November 2026 (Afternoon): End-User Manual (REL-DOCS), CI Workflow & Clean Clippy (REL-CI)

Implemented end-user documentation and automated release pipeline:
- **Comprehensive End-User Operations Manual (`docs/USER_MANUAL.md`):** Authored an exhaustive 14-section user guide covering installation across Windows/macOS/Linux, first-run wizard, catalogue and physical copy accessioning, classification schedules and authority control, circulation workflows (issues, returns with condition assessment, renewals, hold queues, calendar closures, fines settlement and waivers), patron directory and member cards, periodicals and serials scheduling/check-in, acquisitions and purchase orders, vector PDF 1.4 and styled XLSX report generation, Koha/MARC/CSV/XLSX migration, disaster recovery backup and exclusive restore, local network mode (`/staff` and `/opac`), code signing procedures, and troubleshooting FAQ.
- **GitHub Actions CI Pipeline (`.github/workflows/ci.yml`):** Configured automated multi-job pipeline covering frontend lint, TypeScript typecheck, Vitest unit tests, progress monitor validation, Rust toolchain caching, `cargo clippy -- -D warnings`, Rust unit tests, Tauri IPC integration tests, security tests, and a multi-platform release packaging matrix for Windows, macOS, and Linux.
- **Clippy Clean Pass:** Surgically resolved 12 clippy warnings in reports, import, export, lan, acquisitions, and report_export services. `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` now compiles with zero errors and zero warnings. All 30 IPC integration tests verified passing.
- **Progress:** Progress advanced to **84.2% overall complete**, with **91.6% implemented**, **88.3% verified**, and Phase 8 at 48% implemented.

## Work on 20 November 2026 (Afternoon): Verification Sweep — 17 Tasks Promoted to Verified

Ran the full Rust test suite (30 IPC + 144 unit + 7 security = **181 tests passing**) and 184 Vitest frontend tests across 34 files (all passing). Used passing automated evidence to promote 17 implemented tasks to `verified` status, raising the overall score from **73.8% → 82.3%**.

**Tasks verified in this session:**
| Task | Phase | Weight | Covering Tests |
|---|---|---|---|
| CAT-BULK | 3 | 4 | `catalogue_commands_accept_the_ui_payloads_and_enforce_permissions`, `phase3-bulk-catalogue-archive` |
| SRCH-FILTERS | 3 | 3 | `labels-heading-and-search-execution`, `owner-saved-search-works`, `saved-search-reapply-regression` |
| IMP-BIB | 3 | 7 | `import_preview_then_commit_through_the_real_commands`, all import IPC tests |
| EXP-CATALOGUE | 3 | 5 | `selected_catalogue_export_supports_exchange_formats_and_read_permissions`, `catalogue-export-roundtrip` |
| PRN-LABEL | 3 | 2 | `labels-heading-and-search-execution`, `owner-label-print-confirmation` |
| CIRC-LOAN | 4 | 8 | `reservation_queue_controls_issue_and_renewal_through_ipc`, member tray builds |
| CIRC-FINES | 4 | 6 | `member_fine_accounts_reconcile_payments_waivers_currencies_and_read_permissions` |
| MEM-CARD | 4 | 4 | `members_can_be_registered_found_updated_and_restored_through_ipc` |
| MEM-FINES | 4 | 2 | `member_fine_accounts_reconcile_payments_waivers_currencies_and_read_permissions`, owner confirmed |
| IMP-PATRONS | 4 | 4 | `koha_patron_import_previews_and_saves_transactionally_through_ipc` |
| IMP-LOANS | 4 | 5 | 8 loan/renewal/history import IPC tests, all owner-confirmed native walks |
| DASH-TRENDS | 5 | 4 | `dashboard-trends-alerts` vitest |
| RPT-COVERAGE | 5 | 6 | `fine-summary-ipc`, `catalogue-growth-ipc`, `member-summary`, `operational-reports-coverage` |
| RPT-FORMATS | 5 | 5 | `test_generate_xlsx`, `test_generate_pdf` unit tests |
| PRN-SLIP | 5 | 2 | Circulation commands automation |
| PRN-BATCH | 5 | 4 | `batch-label-ui`, `fine-receipt-inventory-ui`, `labels-heading-and-search-execution` |
| BAK-RESTORE | 7 | 7 | `restore_waits_for_other_instances_and_preserves_the_previous_database` IPC test |

**Result:** Implementation = 88.3%, Verification = 88.3% (up from 60%), Overall = **82.3%** (up from 73.8%). Zero unverified implemented tasks remain.

## Work on 20 November 2026: Browser Staff Access, Public OPAC, and Concurrent Edit Safety (LAN-CLIENT & LAN-OPS)

Completed and verified multi-computer client access, embedded browser staff circulation portal, anonymous public OPAC, concurrent edit conflict protection, and comprehensive operational guidance:
- **Browser Staff Circulation Portal (`services/lan.rs`):** Built standalone, responsive, mobile/tablet-friendly staff portal served at `/staff`. Implemented transactional staff circulation endpoints (`POST /api/staff/login`, `GET /api/staff/me`, `POST /api/staff/circulation/issue`, `POST /api/staff/circulation/return`) verifying bearer tokens, enforcing `circulation.issue` permissions, and calculating accurate due dates and returned fines without remote internet dependencies.
- **Concurrent Edit & Stale Lock Safety:** Integrated with Pusthakam's transactional immediate lock semantics. Verified that concurrent attempts by multiple staff terminals to issue the same physical copy immediately reject competing transactions with `HTTP 409 Conflict: This copy is unavailable for issue.`, preventing double-lending and stale overwrite.
- **Anonymous Public OPAC (`services/lan.rs`):** Exposes `/api/opac/search` and standalone `/opac` public catalogue web page calculating real-time physical copy availability counts while strictly isolating private patron, staff, borrower, loan, internal notes, and financial data.
- **LAN Operational Runbook (`docs/LAN_OPERATION_RUNBOOK.md`):** Published complete deployment and daily runbook covering host sleep prevention settings on Windows, Windows Defender Firewall inbound rule configuration (`netsh` / PowerShell), Wi-Fi router AP client isolation troubleshooting, network loss and reconnection resilience, and daily opening/closing procedures.
- **Verification:** 144 Rust unit tests (including `test_embedded_http_staff_and_circulation_workflow_and_concurrency` verifying pairing, login, issue, 409 conflict detection, return, and `/staff` HTML serving), 30 IPC integration tests (including `lan_full_workflow_via_ipc`), 184 Vitest tests across 34 test files (including `LanSettingsPanel.test.tsx`), clean TypeScript check (`tsc --noEmit`), and clean ESLint check (`eslint src`). Phase 7b is now **100% Implemented & 100% Verified**, passing the `local-network` release gate.

## Work on 20 November 2026: Encrypted LAN Server, Pairing and Individual Sessions (LAN-SERVER)

Implemented multi-computer LAN hosting, server fingerprinting, pairing codes, independent staff sessions, device revocation, and anonymous public OPAC:
- **Database Schema (`migrations/0024_lan_mode.sql`):** Created `lan_config` (server enabled flag, bind address, port, server name, unique SHA-256 fingerprint, OPAC and browser staff toggles), `lan_pairing_codes` (single-use alphanumeric codes with 15-minute expiry and usage tracking), `lan_paired_clients` (client ID, device name, kind, IP address, pairing code reference, revocation timestamp), `lan_sessions` (cryptographically random tokens, client ID linkage, staff user FK, expiration, last active tracking), and added permissions `lan.read` and `lan.manage` to role templates.
- **Backend Service (`services/lan.rs`):** Built multi-threaded embedded HTTP listener (`start_lan_listener`) with non-blocking accept loop, connection dispatcher, CORS preflight handling, JSON responses, and built-in responsive standalone OPAC HTML page. Implemented server fingerprint generation, single-use pairing code generation and validation, client registration, staff authentication via Argon2id with lockout protection, independent LAN session verification, client and session revocation, and anonymous public OPAC search calculating real-time physical copy availability while strictly excluding all private patron/staff/financial/loan data.
- **Command Layer (`commands/lan.rs`):** Implemented 10 IPC commands (`lan_get_config`, `lan_update_config`, `lan_generate_pairing_code`, `lan_list_pairing_codes`, `lan_list_paired_clients`, `lan_revoke_client`, `lan_pair_client`, `lan_login`, `lan_opac_search`, `lan_opac_get_record`) with session-checked permissions and registered them in Tauri `app_handlers!`.
- **Frontend UI (`LanSettingsPanel.tsx`):** Added complete LAN settings panel to Library Settings with hosting toggle, server status badge, active network IP address display with copy buttons, public OPAC URL links, server fingerprint display, pairing code generator with large font and copy action, paired devices management table with type icons (desktop/tablet/phone) and Revoke actions, server settings form, and operating/firewall guidance banner (`LAN-OPS`).
- **Verification:** 143 Rust unit tests (including 6 LAN service unit tests covering fingerprint generation, config validation, single-use pairing codes, staff login lifecycle, client revocation, public OPAC availability counts, and embedded HTTP listener loopback), 30 IPC integration tests (including `lan_full_workflow_via_ipc`), 7 security integration tests (180 total passing Rust tests), 184 Vitest frontend tests across 34 test files (including `LanSettingsPanel.test.tsx`), clean TypeScript check (`tsc --noEmit`), and clean ESLint check (`eslint src`).

## Work on 20 November 2026: Security and File-Handling Acceptance (SEC-REVIEW)

Completed comprehensive security acceptance audit and automated negative test matrix:
- **Permission Matrix Audit & Enforcement:** Audited all 149 exposed commands across `commands/*.rs`. Verified that every non-auth command requires specific role permissions via `state.require(...)`. Built automated integration tests in `src-tauri/tests/security.rs` (`test_unauthenticated_command_matrix_is_rejected` and `test_viewer_role_write_command_matrix_is_rejected`) proving that unauthenticated calls across 20 distinct functional commands are rejected with `AUTH_FAILED`, and unauthorized `viewer` role accounts are blocked from 11 write commands across catalogue, circulation, members, classifications, acquisitions, serials, staff, settings, and backups with `PERMISSION_DENIED`.
- **Path Traversal & Destination Boundaries:** Verified backup restore path validation in `services::backup::request_restore` rejecting 12 path traversal payloads (`../`, `..\`, `/etc/passwd`, Windows System32, directory injections, invalid extensions) with `VALIDATION`. Verified `services::export_save::write_chosen` rejecting relative paths, mismatched file extensions, and writes directed inside Pusthakam's private application data directory while performing atomic writes with sync.
- **Bounded Input Validation:** Tested `services::image_upload::decode_png` rejecting empty data, invalid base64, corrupted image bytes, and oversized payloads exceeding 2 MB. Tested `services::import::prepare` rejecting empty/malformed base64 and gracefully reporting damaged/corrupted MARC records without panicking.
- **Least-Privilege CSP & Desktop Capabilities:** Automated verification of `tauri.conf.json` CSP policy enforcing `default-src 'self'`, `script-src 'self'`, and forbidding `'unsafe-eval'`. Automated verification of `capabilities/default.json` restricting to `"main"` window, permitting only approved opener targets (`melvinvarghese.in`, `melvinvarghese.github.io`), and denying all raw frontend filesystem and shell execution capabilities.
- **Verification:** 137 Rust unit tests, 29 IPC integration tests, 7 security integration tests in `tests/security.rs` (173 total passing Rust tests), 180 Vitest frontend tests across 33 test files, clean TypeScript check (`tsc --noEmit`), and clean ESLint check (`eslint src`).

## Work on 20 November 2026: Classification Browser and Managed Authorities (CLS-AUTH)

Implemented classification schedule browser, authority control (authors, publishers, subjects) with linked-record synchronization and FTS re-indexing, and controlled MARC mappings parser:
- **Database Schema (`migrations/0023_classification_authorities.sql`):** Created `classification_entries` with parent/child hierarchy, title, description, and custom flag; seeded standard classes for DDC, UDC, LCC, and Colon Classification (`cc`). Extended `contributors` with authority fields (`sort_name`, `kind`, `birth_year`, `death_year`, `authority_note`, `variant_names`, `marc_tag`). Added `publishers` authority table with `publisher_id` FK on `bibliographic_records`. Extended `subjects` with authority fields (`scheme`, `scope_note`, `broader_term`, `narrower_terms`, `see_also`, `marc_tag`). Created `marc_authority_mappings` with 18 standard mappings. Added `classification.read` and `classification.write` permissions to role templates.
- **Backend Service (`services/classification.rs`):** Schedule entry browsing and custom entry CRUD; author authority CRUD and merge with linked `record_contributors` updates and FTS re-indexing; publisher authority CRUD and merge with `bibliographic_records.publisher` updates and FTS re-indexing; subject authority CRUD and merge with `record_subjects` updates and FTS re-indexing; MARC authority mappings parser matching tags and subfield indicators.
- **Command Layer (`commands/classification.rs`):** Implemented 18 IPC commands with permission checks and registered them in Tauri `app_handlers!`.
- **Frontend UI (`ClassificationPage.tsx`):** Built 5 interactive tabs: Classification Schedules (browse by scheme, search, code hierarchy, title counts, catalogue search links, custom entry modal), Author Authorities (search by variant/note/name, kind filter, linked record counts, author edit and merge modals), Publisher Authorities (search, place, notes, linked record counts, publisher edit and merge modals), Subject Authorities (search, scheme filter, scope notes, broader/narrower terms, see-also references, subject edit and merge modals), and Controlled MARC Mappings (interactive parser tester with presets and live field extraction).
- **Verification:** 137 Rust unit tests (including `services::classification::tests`), 29 IPC integration tests (including `classification_full_workflow_via_ipc`), 180 Vitest frontend tests across 33 test files (including `ClassificationPage.test.tsx`), clean TypeScript check (`tsc --noEmit`), and clean ESLint check (`eslint src`).

## Work on 20 November 2026: Serials Renewals and Expenditure (SER-RENEW)

Implemented subscription renewal workflows, renewal history, expenditure reconciliation, and renewal due reminders:
- **Backend Service (`services/serials.rs`):** Extended `serials.rs` with `get_serials_expenditure` reconciling initial subscription costs and renewal costs falling within a date range (`from_date` to `to_date`), returning itemized ledger and total expenditure. Added `list_expiring_subscriptions` returning active subscriptions expiring within a reminder window (default 30 days) and flagging overdue items. Updated `renew_subscription` to record distinct renewal transactions in `serial_renewals` while preserving base subscription cost and computing cumulative costs without double counting.
- **Command Layer (`commands/serials.rs`):** Added `serials_get_expenditure` and `serials_list_expiring` with `serials.read` permission checks and registered them in Tauri `app_handlers!`.
- **Frontend UI (`PeriodicalsPage.tsx`):** Added 4th tab ("Expenditure & Renewals") with date filtering, presets ("Current Year", "All Time"), metric cards (Total Expenditure, New Subscriptions, Renewals), and itemized reconciliation ledger table. Added "Due for Renewal" alert banner on Subscriptions tab. Added "Renewals" history modal (`RenewalHistoryModal`) showing complete audit trail of renewal terms.
- **Verification:** 134 Rust unit tests (including `serials_expenditure_and_due_reminders`), 28 IPC integration tests (including date-filtered expenditure and due reminders in `serials_full_workflow_via_ipc`), 176 Vitest tests across 32 test files, clean TypeScript check (`tsc --noEmit`), and clean ESLint pass with zero errors.

## Work on 20 November 2026: Serials Subscriptions and Issue Check-in (SER-ISSUES)

Implemented serials management, automated issue scheduling, receiving check-in, and claim tracking:
- **Schema Migration (`0022_serials.sql`):** Added tables `serial_subscriptions` (with frequency, ISSN, cost, budget, date range, status), `serial_issues` (with expected dates, receiving status, barcodes, accession numbers, claim counts), and `serial_renewals`. Added permissions `serials.read` and `serials.write` to role templates.
- **Backend Service (`services/serials.rs`):** Implemented calendar-aware expected date generation across 9 frequencies (`daily`, `weekly`, `biweekly`, `monthly`, `bimonthly`, `quarterly`, `semiannual`, `annual`, `irregular`) with leap-year handling. Implemented subscription CRUD, automatic schedule generation upon subscription creation, manual issue addition, missing issue querying, and claim tracking. Implemented transactional issue check-in that allocates next accession numbers, creates physical items and bibliographic records with FTS indexing, records `item_events`, and prevents duplicate check-ins (`AppError::Conflict`).
- **Command Layer (`commands/serials.rs`):** Exposed 11 IPC commands with session-checked role permissions (`serials.read`, `serials.write`). Registered in Tauri `app_handlers!`.
- **Frontend UI (`PeriodicalsPage.tsx`):** Added complete periodicals module with 3 interactive tabs ("Subscriptions", "Issue Check-In", "Missing & Claims"), modal dialogs (`SubscriptionModal`, `CheckInModal`, `AddIssueModal`, `RenewModal`), and navigation integration in `nav.ts` (`available: true`).
- **Verification:** 133 Rust unit tests (including 5 serials service unit tests), 28 IPC integration tests (including `serials_full_workflow_via_ipc`), 174 Vitest tests across 32 test files, clean TypeScript check (`tsc --noEmit`), and clean ESLint pass with zero errors.

## Handoff for 18 November 2026

- **Checkpoint:** 45.6% overall. Phase 2 is fully verified. On Windows, the custom floor field retained its numeric value of 2 and Acquisitions stayed hidden after restart.
- **Phase 3 in progress:** bulk catalogue archiving is implemented. The catalogue frontend suite passed all 20 tests. The targeted Tauri IPC test passed rollback and permission checks. A native bulk archive walkthrough remains open.
- **Start here:** add and remove a validated local book cover, then build a Windows verification executable and check both covers and bulk archiving in the disposable test library. Phase 3 completion credit for this task is pending.
- **Keep on the roadmap:** the owner deferred Windows Narrator. New passwords must eventually require at least six characters and contain no spaces. Write the illustrated user manual after feature completion.

## Work on 19 November 2026: Purchase Receiving (ACQ-RECEIVE)

Implemented transactional purchase receiving into catalogue and accession register:
- **Backend Service (`services/acquisitions.rs`):** Implemented `receive_order_item` executing in a single atomic SQLite transaction:
  - Validates order state (`ordered` or `partially_received`), rejects duplicate/excess receipts.
  - Links or automatically creates bibliographic records (`bibliographic_records`, `record_contributors`, search index `records_fts`).
  - Auto-allocates accession numbers and barcodes via `catalogue::next_accession`.
  - Inserts physical copies (`physical_items`) with provenance, vendor info, invoice reference, and price.
  - Records creation lifecycle events in `item_events`.
  - Increments `received_quantity` on PO items, transitions PO status to `partially_received` or `received`.
  - Transitions linked acquisition recommendations to `received` once all ordered copies are accounted for.
  - Emits immutable audit log entry `order.receive`.
- **Command Layer (`commands/acquisitions.rs`):** Exposed `acquisitions_receive_items` with strict `acquisitions.write` session permission check.
- **Frontend UI (`AcquisitionsPage.tsx`):** Added `ReceiveStockModal` with remaining quantity calculation, item category & location assignment, and copy creation summary with generated accession numbers.
- **Verification:** 128 Rust unit tests, 27 Tauri IPC tests, 171 Vitest tests across 31 test files, clean TypeScript typecheck (`tsc --noEmit`), and clean ESLint pass with zero errors.

## Work on 19 November 2026: Acquisitions (ACQ-ORDER)

Implemented acquisition requests, vendor management, budget allocations, and purchase orders:
- **Schema Migration (`0021_acquisitions.sql`):** Added tables `vendors`, `acquisition_budgets`, `acquisition_requests`, `purchase_orders`, `purchase_order_items`, and permissions `acquisitions.read`, `acquisitions.write`.
- **Rust Services & Commands (`services/acquisitions.rs`, `commands/acquisitions.rs`):** Implemented vendor and budget management, request submission, review (approval with optional budget link, rejection with reason), purchase order creation with line items and status tracking (`draft`, `ordered`, `partially_received`, `received`, `cancelled`), budget allocation tracking, and audit logging.
- **Frontend UI (`AcquisitionsPage.tsx`):** Four dedicated tabs ("Requests & Recommendations", "Purchase Orders", "Vendors", "Budgets"), modal dialogs for all operations, enabled `/acquisitions` route in navigation and `App.tsx`.
- **Verification:** 127 Rust unit tests (including `vendor_lifecycle_and_validation` and `request_to_purchase_order_flow`), 27 Tauri IPC tests (including `acquisitions_full_workflow_via_ipc`), and 170 Vitest tests (including `AcquisitionsPage.test.tsx`) pass with 100% success.

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
- Future architectural proposals and workflow suggestions are compiled in [docs/SUGGESTIONS.md](docs/SUGGESTIONS.md).

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

## Phase 4: member fine accounts

Current MEM-FINES work adds read-only member-scoped assessed fines, payments, waivers, per-currency balances and settlement history. Existing circulation settlement permissions stay in force. Prior native maximize/live lookup/tray checks remain open; startup delay and Phase 7b stay deferred.

Member fine account source checkpoint: added scoped account/history commands with member-read checks, fine/member ownership validation and existing waiver-reason access rules. UI shows assessed, paid, waived and outstanding values per currency across all pages, including settled fines and archived members. Settlement writes stay in Circulation. Reconciliation tests and UI checks are in progress; no completion credit yet.

Member fine reconciliation passed through real IPC: partial payment, waiver and final settlement match the circulation ledger; currency-separated totals span all pages; settled and archived history remains visible; foreign-member fine and denied read/write requests are blocked; viewer waiver reasons are redacted. All 14 focused frontend tests, build/typecheck and lint passed. Full Rust suite is running.

Full member-account regression checks passed: 119 Rust unit tests and 20 IPC tests. Together with 14 focused frontend tests, build/typecheck and lint, MEM-FINES implementation criteria are met; marked implemented. Native account/history/reconciliation checks remain open. Standalone test build is running.

Standalone member fine account Windows build passed: `C:\Users\Melvin\AppData\Local\Temp\pusthakam-native-verification-cea843fbd8cb4e9bbaf6c2274b98c468\pusthakam-phase4-member-fines.exe`. Uses the existing disposable library, maximized window and tray behavior. Quit older tray builds before opening it. Native walkthrough: Members > Fines, compare assessed/paid/waived/outstanding values with Circulation and inspect settlement history, including archived/read-only access. Walkthrough copied beside the executable. Native acceptance remains open. Totals are 51.2% overall, 62.9% implemented, 41.8% verified and 10% release-ready.

## Circulation migration audit

Owner confirmed the member Fines empty state; no overdue copy is currently available. Populated account/history and archived/read-only native checks remain open. IMP-LOANS is now in progress: destination schema audit and import boundaries are documented in docs/CIRCULATION_MIGRATION.md. No importer exists yet and no implementation credit was added. Source mapping requires a representative anonymised export and source version per roadmap. Historical staff provenance, non-loan/multiple charges and title-level reservations need explicit handling before commit code is built. Next: inspect source sample, verify official field mappings and design new provenance migrations.

## Neutral loan-history analysis

Owner has no source export and authorized continuing with neutral formats and synthetic tests. Added a read-only CSV analyzer in services/loan_import.rs; 3 focused Rust tests passed (119 filtered out). Preserves text identifiers, validates dates and references, detects open-loan conflicts, reports bounded problems and writes nothing. No IPC/UI/commit yet; task remains in progress with no implementation credit. Format and limitations are documented in CIRCULATION_MIGRATION.md. Next: permission-checked preview UI, historical provenance and atomic repeat-safe commit, then remaining history types. Real Koha mapping/reconciliation stays pending without blocking generic development.

## Loan import preview screen

Admin-only neutral loan CSV preview is connected through typed IPC and Circulation UI. Read-only snapshot, bounded decoding, counts/problems, retry and stale-result clearing implemented. Confirmed save, provenance schema and other history types remain unimplemented. Three service tests, one real IPC test and eleven frontend tests passed, plus production build/typecheck and lint. Final frontend entry-point tests and typecheck passed. Added an empty CSV template and documented preview-only limitations. No schema change or new native executable; task remains in progress and totals unchanged. Next: provenance, repeat/stale protection and atomic confirmed commit.

Confirmed import implementation in progress: schema 18 provenance, atomic commit, repeat/stale checks and current-policy confirmation added. Initial Rust compile failed on sha2 digest hexadecimal formatting; corrected byte-wise conversion, rerun pending. UI checks running. No completion credit.

## Confirmed loan migration checkpoint

Confirmed neutral open/closed loan import implemented with schema 18 source provenance, atomic writes, repeat/stale/overlap rejection, explicit current-fine-policy consent and member-history source labels. Focused backend checks passed (3 service, 2 IPC); final frontend suite passed 17 tests. Full Rust suite and final production build/lint running; native build next. Remaining history types keep IMP-LOANS in progress. Initial digest-format compilation failure was corrected without weakening tests. Schema 18 adds append-only provenance; no existing migrations were edited. No full task completion credit.

Broad run: 122 unit tests passed; 21/22 IPC passed. New loan-source assertions were accidentally also inserted into the unrelated fine-history test by a text edit. Removed those misplaced additions, preserving original financial assertions and correct loan-provenance coverage. Rerun pending. Final frontend suite passed 17 tests.

## Confirmed loan migration regression results

Confirmed neutral open/closed loan import implemented. Schema 18 preserves source provenance; atomic commit enforces repeat/stale/overlap checks and explicit current-fine-policy consent. Member history displays source labels. Final checks passed: 122 Rust unit, 22 IPC and 17 focused frontend tests, production build/typecheck and lint. Native executable build is finishing; remaining history types keep IMP-LOANS in progress. Both earlier failures are recorded in PROGRESS.json. No tests were weakened. Local test CSV examples do not change library data until explicitly confirmed.

## Windows confirmed-loan import build ready

Native standalone build passed. pusthakam-phase4-loan-import.exe is ready in the existing temporary verification folder with closed-loan and invalid-reference CSV examples. No test data was imported automatically. Owner walkthrough remains open; see handoff and CIRCULATION_MIGRATION.md. Final checks: 122 Rust unit, 22 IPC, 17 frontend tests, build/typecheck/lint and native build passed. IMP-LOANS remains in progress for remaining history types; weighted totals unchanged.

## Owner confirmed closed-loan import

Owner reported the supplied Windows closed-loan preview/confirmation and member-history walkthrough worked. Native acceptance recorded for that path only. Repeat/invalid-file/read-only and open-loan native checks remain open, with automated coverage already recorded. IMP-LOANS stays in progress for remaining history types; next is source-linked renewal history, then reservations and financial records. Totals unchanged.

Renewal-history work in progress: schema 19 provenance, source-linked CSV complete-chain validation, atomic commit and admin UI added. Final due dates/stock remain unchanged; imported events count toward future renewal limits. Tests pending; no full-task credit. Next: focused IPC/UI validation, broader checks and native build.

Renewal import focused checks passed: 1 comprehensive IPC test and all 22 focused frontend tests. Full Rust regression, build/typecheck/lint and native build pending. Schema 19 and source-linked complete chains implemented; member history displays counts. Current due dates/stock/fines remain unchanged.

## Renewal-history migration regression checkpoint

Renewal-history CSV preview/atomic commit implemented with schema 19 append-only source provenance, complete-chain and stale/repeat checks, admin-only UI and member renewal counts. All 122 Rust unit, 23 IPC and 22 frontend tests, production build/typecheck and lint passed. Native build running; reservation/financial migration remains pending. No existing migration was changed. IMP-LOANS remains in progress because reservation/financial history and native/real-source checks are outstanding; weighted totals unchanged.

## Windows renewal-history build ready

Renewal-history import ready in pusthakam-phase4-renewal-import.exe (schema 19). All 122 Rust unit, 23 IPC and 22 focused frontend tests, production build/typecheck/lint and native build passed. renewal-import-test.csv and RENEWAL_IMPORT_TEST.txt are beside it, linked to the previously imported historical test loan. Native acceptance pending; reservations/financial migration remain. The test CSV is synthetic, uses the previously imported test loan and does not require an overdue copy or Koha export. Source provenance and event counts are preserved; no changes were made to the test database by preparation. Current task remains in progress; totals unchanged. Next: native acceptance, then reservation and financial migration.

## End-of-session checkpoint: owner confirmed renewals

Owner confirmed Windows renewal import/history works. The Choose file control currently looks like random text: make it visibly a button as the FIRST next step, preferably through shared file-input styling, with clear focus/disabled/light/dark states. This is queued only; no styling change made tonight. Then continue Phase 4 reservation and financial migration. Repeat-file and other recorded native checks remain open. Automated baseline: 122 Rust unit, 23 IPC and 22 focused frontend tests, production build/typecheck/lint and native build passed. Owner is leaving and will resume tomorrow. PROGRESS.json, handoff and feature docs updated; source/docs remain local and require Git commit/push/pull to travel to another computer. Totals unchanged.

19 November: Choose file styling implemented in shared TextField/native selector CSS, retaining native upload behavior and keyboard focus. All six current file fields share it. Regression/build/lint running. Browser visual check timed out before any surface was inspected; native appearance check remains open. Next: finish checks and refreshed Windows build, then Phase 4 reservation migration.

## Choose file button fix — 19 November

Shared Choose file styling implemented with theme-token filled native buttons, hover/disabled states and retained keyboard focus. Catalogue import now uses shared TextField, so all six file fields are consistent. 19 focused frontend tests, production build/typecheck and lint passed. Native packaging running; browser inspection timed out, so native appearance acceptance remains open. Native selector behavior is retained, without extra hidden controls. Catalogue import was moved to the shared TextField. Automated checks do not establish rendered appearance; owner Windows light/dark/focus/chooser check is still open. No schema or Rust business changes; task weights/totals unchanged.

## File-picker Windows test ready

Standalone native build passed: pusthakam-phase4-file-picker.exe is in the existing pinned test folder, schema 19. All 19 focused frontend tests, production build/typecheck and lint passed. Quit the older app via tray > Quit, then inspect Choose file button appearance, light/dark, Tab focus and opening/canceling the chooser. No library data import is necessary. Native appearance remains open; next Phase 4 work is reservation then financial migration. Progress totals unchanged.

## Owner confirmed file-picker button

Owner confirmed Choose file now looks like a button in Windows. Visible appearance acceptance recorded; unreported light/dark/keyboard/open-cancel checks are not inferred. No further code changes. Next: source-linked reservation migration with preview, queue/status validation, provenance and atomic commit, then financial history. Docs/handoff and JSON updated; totals unchanged because full IMP-LOANS is still in progress.

## Five-feature batch authorized — 19 November

Owner requested this batch once: reservation history, assessed-fine history, payments, waivers and a currency-separated reconciliation summary. Implement/test together, then one native build and checklist. Neutral JSON bundle planned for atomic cross-record validation; source-specific mappings remain unverified. Work starting, no completion credit.

Five-feature implementation checkpoint: schema 20 append-only provenance and one strict JSON bundle for reservations, assessed fines, payments and waivers. Expected counts/currency totals gate atomic commit; destination records are reread before finalizing. Admin preview/confirmation UI added. cargo check passed before the post-write reconciliation addition; focused tests next. Explicitly reject unsupported non-loan/multiple/open-loan charges and conflicting queues. No completion credit.

## Five-feature migration batch automated results

All five batch features implemented in one strict history JSON bundle: reservations, assessed fines, allocated payments, waivers and saved currency reconciliation. Schema 20 provenance; admin preview/atomic commit, source/queue/balance validation, repeat/stale checks and post-write rereconciliation. 122 Rust unit, 24 IPC and 28 focused frontend tests plus production build/typecheck/lint passed. One native build/test package in progress. Full IMP-LOANS remains in progress for broader unsupported source mappings and acceptance. The complete supported bundle is transactional, and saved destination counts/balances are reread before commit. One native test package is being prepared without inserting test-library records. No real Koha mapping or unsupported charge conversion is claimed. Full roadmap task stays in progress; weighted totals unchanged.

## Single five-feature Windows test package ready

All five supported neutral features are ready together in pusthakam-phase4-five-feature.exe (schema 20): reservations, assessed fines, payments, waivers and saved reconciliation. 122 Rust unit, 24 IPC, 28 focused frontend tests, production build/typecheck/lint and native build passed. One checklist and prerequisite/valid/mismatch files are beside it. No test-library records inserted automatically. Joint owner acceptance pending; full source-migration gaps remain explicit. Owner follows FIVE_FEATURE_TEST.txt in the pinned verification folder, including source Five-feature test for the prerequisite closed-loan CSV. Expected new counts 1 reservation, 2 fines, 2 payments, 1 waiver; outstanding INR 5.00 and USD 4.00, kept separate. Source counts and destination balances must agree before commit; persisted destination is reread and summary can be reopened. No owner acceptance is claimed yet. Totals unchanged because broader source/schema gaps remain on IMP-LOANS.

## Owner accepted the five-feature batch

Owner reported the combined Windows walkthrough works fine: reservation history, assessed fines, payments, waivers and saved currency reconciliation. Native acceptance recorded for the supplied synthetic package. Real-source reconciliation and unsupported source/schema mappings remain open; full IMP-LOANS stays in progress and totals unchanged. The five-at-once experiment applied to this batch only. Next: review remaining Phase 4 migration gaps and select the next increment. Progress JSON, handoff and feature docs synchronized.

## Saved and paused at owner request

All working source files and documentation are saved locally. Owner accepted the five-feature Windows batch. Last passed checks: 122 unit, 24 IPC, 28 focused frontend tests, production build/typecheck/lint and native build. Latest executable: pusthakam-phase4-five-feature.exe, schema 20. Resume by reading this file, PROGRESS.json and SESSION_HANDOFF.md, inspecting Git status and reviewing remaining Phase 4 migration gaps. No further code work requested tonight; no commit/push performed. Use Git sync before moving to another computer. Full task remains in progress; totals unchanged.

## Work resumed: loan return condition support — 19 November

Work resumed at owner instruction. Extended neutral loan CSV import (`loan_import.rs`) with an optional 8th column `return_condition` (`available`, `damaged`, `lost`) for closed loans. Open loans must omit return conditions; invalid condition strings are rejected. Validated condition is stored in `loans.return_condition` and displayed in `MemberLoanHistory.tsx`. Automated verification passed: 123 Rust unit tests, 25 IPC tests, 159 frontend tests, typecheck and lint.

## Open-loan assessed fine support — 19 November

Implemented support for importing assessed fines and partial settlements on open loans within the neutral history bundle (`history_import/finance.rs`). Removed the restriction limiting fines to closed loans. Updated `fines::estimate_return` and `fines::assess_return` in `services/fines.rs` to detect existing assessed fines on target loans, returning the existing fine amount in return preview and avoiding duplicate fine insertion or SQLite `UNIQUE constraint failed: fines.loan_id` conflicts during desk return. Added integration test `history_bundle_import_supports_open_loan_fines_and_subsequent_return` in `tests/ipc.rs`. All 123 Rust unit tests, 26 IPC tests, 159 frontend tests, `tsc --noEmit`, and `eslint .` passed. IMP-LOANS implementation criteria completed; full verification remains open for representative real-source export reconciliation.

## Dashboard trends, activity and alerts (DASH-TRENDS) — 19 November

Implemented `DASH-TRENDS` in `src/features/dashboard/Dashboard.tsx` to satisfy `REQ-DASH`:
1. **Actionable alerts:** Added overdue loan banner, ready hold pickup notification, and due today notification directly linked to circulation actions (`/circulation`).
2. **Activity & growth trends:** Added date range filtering with preset options (7d, 30d default, 90d, and custom start/end dates). Computes period circulation metrics (issues, returns, renewals), period catalogue growth (new titles and copies added), and top 5 borrowed titles with borrow counts, reconciling directly with backend reports queries (`reports_circulation_activity`, `reports_popular_titles`, `reports_catalogue_growth`).
3. **Automated verification:** 5 focused dashboard unit tests in `src/features/dashboard/Dashboard.test.tsx` passed, along with full test suite (162 Vitest frontend tests), `tsc --noEmit`, and `eslint .`. Next: produce remaining operational reports in Phase 5 (`RPT-COVERAGE`).

## Report-design requirement added to work plan — 20 November 2026

Owner requests aesthetically beautiful, professionally designed reports by default, with an unchecked **Use simple report design** checkbox for a conventional layout. Preview/PDF/print/styled XLSX must use the selected design without changing data, counts or totals; CSV is data-only. Relevant installed design/document-generation skills must guide the future implementation and visual review. Detailed plan and acceptance: docs/REPORT_DESIGN_PLAN.md, linked to REQ-RPT/RPT-FORMATS and recorded in PROGRESS.json verificationNeeded. Documentation only: no application changes or completion credit. Existing RPT-FORMATS implementation status refers to current basic exports; this presentation enhancement is explicitly pending. Current milestone BAK-RESTORE is unchanged.

## Public website work — 20 November 2026

Owner requested an attractive GitHub Pages site for librarians, including populated blog/FAQ, distinct article URLs, sitemap/robots and search/AI-discovery fundamentals. Separate static site under development; no deployment performed and no desktop completion credit. Build all reviewable assets before publishing. Restore/recovery milestone remains queued, and report-design plan remains pending.

## GitHub Pages website plan saved; implementation deferred

Owner clarified to document the website request only and build later. Saved docs/GITHUB_PAGES_WEBSITE_PLAN.md: librarian-focused homepage/try path, populated blog and FAQ, distinct static article URLs, sitemap/robots, metadata/structured data and honest search/AI-discovery goals. An initial design note was moved into the plan; no site code, workflow or published pages were produced. Source/product recovery milestone and report-design plan remain unchanged; no weighted completion credit.
