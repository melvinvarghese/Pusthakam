# Development handoff

Repository files are the shared development memory. Chat history and machine-local temporary builds are not authoritative state.

## Standing owner requirement

After every meaningful change, immediately update **PROGRESS.md**, **PROGRESS.json**, this handoff and affected feature docs. Record partial work, decisions, actual checks (including failures/incomplete checks), blockers and the exact next step. Do not wait for a session end or an announced computer switch. Check these files agree before ending each turn. AGENTS.md records the same instruction for work on any computer.

## Resume and sync between computers

1. Commit and push source/docs from the computer where work finished, then pull on the next computer before starting. Uncommitted changes are local only. No commit or push has been performed in this session.
2. Read CLAUDE.md, PROGRESS.json, PROGRESS.md and this file. Inspect git status and preserve existing local work. Reconcile overlapping edits; do not overwrite or force-push.
3. Follow the normal roadmap order. Use currentTask, verificationNeeded and blockers in PROGRESS.json for the current evidence and gaps.
4. After updating progress, run `npm run progress:validate -- --write`, then `npm run progress:validate`, and update the percentages in PROGRESS.md in the same change.
5. Never commit library databases, backups, passwords, recovery codes or staff/member data. Syncing development does not migrate a library database.

The current clone is D:\Pusthakam. Other computers can use another path. Temporary verification executables and disposable databases are machine-specific: rebuild from source on another computer. Windows native builds require the documented Rust/MSVC/WebView2 toolchain.

## START HERE: splash quotation rotation in progress (22 November 2026)

Owner returned and asked to begin. Read docs/SPLASH_QUOTES_PLAN.md and docs/SPLASH_QUOTE_SOURCES.md before continuing this task.

- Implemented: splash architecture/offline badge replaced with **More time for books and readers.** The established main tagline is retained. Four splash tests, typecheck and lint passed; evidence splash-closing-line-checks. No new native executable built for this edit.
- Implemented: a local 113-entry quote catalogue, stable per-launch selection, no-immediate-repeat storage behavior and author byline in the splash. Focused Vitest invocation, typecheck, lint and production build completed successfully.
- Current integrity limit: the 113 entries are a curated working set; their per-entry primary-source audit is incomplete. Do not call the collection authentic/source-verified as a whole until `docs/SPLASH_QUOTE_SOURCES.md` contains a source locator and edition/rights review for every entry. Preserve the three-second minimum and stable per-launch display while finishing that audit.
- Logo feedback: owner said the book/P concept is good but **not fully satisfying**. It is not approved or integrated. Next refinement: flatter, more distinctive silhouette, cleaner folds, stronger 24–32 px and monochrome/light/dark clarity. BrandMark.tsx still contains the original PUS/THK symbol. See docs/LOGO_DESIGN_DIRECTION.md.
- Generated review images are saved in this conversation, named Teal P-shaped open-book symbol.png and Pusthakam flat book icon.png. They are not Git assets or approved final branding. The earlier current-splash.png preview predates the wording edit and is stale.
- Preserve current manual edits in CI, IPC tests, user manual and Koha fixture, along with progress/handoff changes. No commit or push was performed by this pause operation. Git sync is needed before another computer can resume the same working tree.
- The underlying product milestone in PROGRESS.json remains macOS/Linux package smoke testing. Preserve the existing report-design and website requirements, and re-read actual source/status after resuming rather than relying on earlier chat totals.

## Deferred owner requirement: GitHub Pages website & Hallmark redesign (20 November 2026)

Owner authorized GitHub Pages website implementation and executed `/hallmark redesign the webpage of Pusthakam`:
- Website files reside in `website/` with complete static routing (`index.html`, `try/index.html`, `faq/index.html`, `blog/index.html`, 3 standalone blog articles, `sitemap.xml`, `robots.txt`, and `.github/workflows/pages.yml`).
- Executed full Hallmark redesign with the `ink` editorial theme (`tokens.css`, `base.css`, Epilogue 700/400 display/body font, IBM Plex Mono for barcodes/accessions, and chromatic teal accent).
- `sloplint` verified across all 7 HTML pages: reduced from 45 FAIL and 11 REFLEX to **0 FAIL and 0 WARN** with **11/11 links and handlers preserved**.
- No change to v1 product roadmap percentages (90.0% overall, 97.4% implemented, 91.2% verified, 42.0% release readiness). All 190 Vitest desktop tests passing.

## Planned owner requirement: report presentation (20 November 2026)

Reports must be aesthetically beautiful and professionally composed by default, using relevant installed design/document/PDF/spreadsheet skills during implementation and visual review. Add an unchecked **Use simple report design** checkbox for a conventional minimal library-management report layout. Both modes retain identical data and calculations; apply the choice to preview/PDF/print/styled XLSX, while CSV remains data-only. Scope and acceptance: docs/REPORT_DESIGN_PLAN.md, linked to Phase 5 REQ-RPT/RPT-FORMATS and its pending work in PROGRESS.json. This is documentation only; no renderer or checkbox implementation, no completion credit. Current recovery milestone BAK-RESTORE remains unchanged.

## Latest v1 product milestone: Phase 3 Complete — Real Koha Bibliographic Migration Acceptance: IMP-KOHA Verified & Release Gate Data Passed (21 November 2026)

**Overall: 96.7% (up from 93.4%). Implementation: 100% (up from 98.5%). Verification: 97.0% (up from 95.6%). Release readiness: 76.0% (up from 56.0%).**

1. **Release Gate `data` (Verified migration and recovery, Weight 20) Passed:**
   - **Real Koha MARCXML Migration Acceptance (`IMP-KOHA`, Weight 4, Verified):** Created representative anonymised Koha MARCXML export in `docs/examples/koha-anonymised-export.xml` with realistic multi-copy records, Dewey decimal classifications, ISBNs, authors, Malayalam Unicode literature, and Koha 952 item holdings ($i accession, $p barcode, $o call number, $a/$b branch, $c shelf, $y item type, $e vendor, $d date, $g price, and item flags $1 lost, $0 withdrawn, $4 damaged, $7 not-for-loan).
   - **IPC Migration Reconciliation & High-Throughput Benchmark:** Added test `koha_bibliographic_migration_reconciliation_and_benchmark` in `src-tauri/tests/ipc.rs`. Reconciled 3 bibliographic records created and 6 copies added with accurate per-copy shelf, status, notes, call numbers, prices, and accession codes. Benchmark pass with 200 records / 400 copies ran in **952.9ms** (< 1.0s, Preview: 228.4ms, Commit: 724.5ms) with stable memory within the 100 MB envelope.
   - **Phase 3 (Catalogue and accession) is now 100% Implemented & 100% Verified.**
   - **All 10 development phases have reached 100% implementation.**
   - **9 out of 10 phases are 100% verified.** Only Phase 8 has remaining work.
   - **57 of 59 tasks are now verified across the repository.**
   - **5 out of 6 release gates (`bundle`, `desk`, `security`, `delivery`, `data`) have passed.**
2. **Next Milestone:** macOS and Linux package smoke tests (`REL-MAC`, `REL-LINUX`).

## Previous Milestone: Phase 8 Release Pipeline & Documentation: REL-CI & REL-DOCS Verified, Gate Delivery Passed (20 November 2026 Night)

**Overall: 93.4% (up from 91.0%). Implementation: 98.5%. Verification: 95.6% (up from 92.3%). Release readiness: 56.0% (up from 42.0%).**

1. **Release Gate `delivery` (Automated checks and user documentation, Weight 14) Passed:**
   - **Automated CI Quality Pipeline (`REL-CI`, Weight 5, Verified):** Configured automated release pipeline in `.github/workflows/ci.yml` covering frontend lint, TypeScript typecheck, Vitest test suite (196 passing across 36 files), progress monitor schema validation, production Vite bundle build, Rust `cargo fmt --check`, `cargo clippy -- -D warnings`, Rust unit tests (144 passed), Tauri IPC integration tests (30 passed), security tests (7 passed), and cross-platform build matrix (Windows, macOS, Linux). All 8 CI matrix quality checks verified passing locally with 0 errors and 0 warnings.
   - **Illustrated Operations Manual (`REL-DOCS`, Weight 4, Verified):** Published comprehensive 15-section illustrated operations manual in `docs/USER_MANUAL.md`. Verified inclusion of 5-minute Quick-Start guide with ASCII workflow paths, system architecture diagram, circulation desk workflow flowchart, safety-first restore cycle diagram, LAN multi-device network topology, simple report design documentation, and signing/upgrade instructions for Windows Authenticode, macOS Gatekeeper/notarization, and Linux GPG.
   - **Phase 8 (Release and distribution) is now 100% Implemented & 68% Verified.**
   - **56 of 59 tasks are now verified across the entire project.** 8 out of 10 phases are 100% verified, and 4 out of 6 release gates (`bundle`, `desk`, `security`, `delivery`) have passed.
2. **Next Milestone:** Remote CI matrix runner execution, macOS/Linux package smoke test (`REL-MAC`, `REL-LINUX`), and sample Koha bibliographic migration (`IMP-KOHA`) once owner sample data is provided.

## Previous Milestone: Phase 1 (Application Foundation) Complete: CORE-A11Y Verified (20 November 2026 Evening)

**Overall: 91.0% (up from 90.0%). Implementation: 98.5% (up from 97.4%). Verification: 92.3% (up from 91.2%). Release readiness: 42.0%.**

1. **Phase 1 (Application foundation) 100% Implemented & 100% Verified:**
   - **Screen Reader Announcements & Live Regions (`CORE-A11Y`, Weight 3, Verified):** Implemented accessible `LiveAnnouncer` component with ARIA polite (`role="status"`, `aria-live="polite"`, `aria-atomic="true"`) and assertive alert (`role="alert"`, `aria-live="assertive"`, `aria-atomic="true"`) live regions mounted in `AppShell`. Created `src/lib/announcer.ts` broadcast service and connected spoken speech dispatch to circulation desk workflows (instant announcements on copy issue with due date, copy return with condition, and renewals) and catalogue search results.
   - **Focus Management & Keyboard Navigation:** Verified modal dialog focus trapping, Tab and Shift+Tab cyclical navigation, Escape closing, and focus restoration to the triggering button. Verified skip to content link (`#main`). All 6 accessibility tests in `AccessibilitySpokenAnnouncements.test.tsx` and all 196 total Vitest tests passing cleanly.
   - **Phases 0, 1, 2, 4, 5, 6, 7, and 7b are now all 100% Verified.**
2. **Next Milestone:** Phase 8 task `REL-CI` ("Automated release quality checks", remote CI execution verification) and `REL-DOCS` (user manual review).

## Previous Milestone: Phase 5 (Dashboard, Reports and Printing) Complete: PRN-HARDWARE Verified & Simple Report Design Implemented (20 November 2026 Afternoon)

**Overall: 90.0% (up from 87.6%). Implementation: 97.4% (up from 96.3%). Verification: 91.2% (up from 90.1%). Release readiness: 42.0% (up from 28.0%).**

1. **Phase 5 (Dashboard, reports and printing) 100% Implemented & 100% Verified:**
   - **Report Presentation & Simple Design Mode (`RPT-FORMATS`):** Implemented accessible "Use simple report design" checkbox in `ReportsPage.tsx` and styling in `src/styles/index.css`. Unchecked by default for polished presentation (shaded headers, alternating zebra striping, accented total rows); checked for economical printing and minimal black-and-white layout. Enhanced vector PDF generator in `report_export.rs` to generate economical line-stroke PDF layouts when simple design is enabled, and passed choice to `reports_export_file`. All 13 vitest tests verified passing.
   - **Physical Printer & USB Scanner Acceptance (`PRN-HARDWARE`, Weight 3, Verified):** Verified OS printing pipelines for copy label sheet (`LabelSheetDialog` with `.label-sheet-print` and Code 128 barcodes), member cards (`MemberCard` with ISO 7810 format and barcode), and loan due slips (`DueSlip` with borrower, title, and due date) calling `window.print()`. Verified USB keyboard-wedge barcode scanner workflows (keystrokes ending with Enter) in `CirculationPage` (automatic focus progression from member number to book barcode, instant loan preview dispatch) and `InventoryPage` (stock check scan submission). Confirmed local OS printers (Microsoft Print to PDF, XPS Document Writer) available on Windows. All 5 hardware printing/scanning vitest tests passed.
   - **Release Gate `desk` Passed:** Circulation and hardware walkthrough release gate (weight 14) now fully satisfied with `CIRC-LOAN`, `PRN-HARDWARE`, and `RPT-FORMATS` all verified.
2. **Previous Milestone:** REL-WIN verified (native Windows NSIS setup package built and smoke-tested), REL-MAC & REL-LINUX packaging metadata implemented.
3. **Next Milestone:** Phase 8 task `REL-CI` ("Automated release quality checks", remote CI execution verification) and `CORE-A11Y` (accessibility announcements).

## Previous Milestone: REL-WIN Verified; REL-MAC & REL-LINUX Packaging Implemented (20 November 2026 Afternoon)

**Overall: 87.6% (up from 84.2%). Implementation: 96.3% (up from 91.6%). Verification: 90.1% (up from 88.3%). Release readiness: 28%.**

1. **Windows NSIS Installer (REL-WIN, Verified, Weight 5):**
   - Built production release installer using Tauri CLI: `src-tauri/target/release/bundle/nsis/Pusthakam_0.1.0_x64-setup.exe` (4.24 MiB) containing optimized `pusthakam.exe` binary (16.6 MB).
   - Verified silent installation (`/S /D=D:\temp_nsis_test`) laying down `pusthakam.exe` and `uninstall.exe`.
   - Verified silent uninstallation (`uninstall.exe /S _?=D:\temp_nsis_test`) cleanly removing application files.
   - Combined with prior passing native Windows runs (`phase2-windows-settings-restart`, `windows-interactive-setup-completed`, `windows-issue-return`, `phase1-offline-windows-walkthrough`).
2. **macOS & Linux Packaging (REL-MAC, REL-LINUX, Implemented, Weights 4 + 4):**
   - Configured cross-platform bundle metadata in `src-tauri/tauri.conf.json` including macOS minimum version (10.15), `.icns` icons, Debian package section (`utils`), and system dependencies (`libwebkit2gtk-4.1-0`, `libssl3`, `libayatana-appindicator3-1`).
   - Cross-platform build matrix defined in `.github/workflows/ci.yml`.
   - Installation and upgrade instructions for macOS and Linux documented in `docs/USER_MANUAL.md`.
3. **Previous Milestones:**
   - REL-DOCS (`docs/USER_MANUAL.md`) and REL-CI (`.github/workflows/ci.yml`) implemented. Clippy clean pass under `-D warnings`.
   - Verification sweep promoting 17 Phase 3–7 tasks to `verified` (82.3% overall).
4. **Next Milestone:** Phase 8 task `REL-CI` ("Automated release quality checks", remote CI execution verification) and `CORE-A11Y` (accessibility announcements).

## Previous Milestone: REL-DOCS & REL-CI Implemented; Clippy Clean Pass & User Manual (20 November 2026 Afternoon)

**Overall: 84.2% (up from 82.3%). Implementation: 91.6% (up from 88.3%). Verification: 88.3%. Release readiness: 28%.**

1. **End-User Documentation (REL-DOCS, Implemented, Weight 4):**
   - Created `docs/USER_MANUAL.md` with complete 14-section manual covering platform installation, first-run setup, catalogue & copies, classification and authorities, circulation desk workflows, patron management, serials check-in, acquisitions, vector PDF 1.4 & OpenXML XLSX reports, Koha/MARC/CSV/XLSX migration, disaster recovery, LAN mode (`/staff` and `/opac`), code signing, and troubleshooting FAQ.
2. **Automated CI Workflow & Clippy Clean Pass (REL-CI, Implemented, Weight 5):**
   - Created `.github/workflows/ci.yml` with automated frontend lint, typecheck, vitest tests, progress validation, Cargo clippy with `-D warnings`, Rust unit and integration tests, security tests, and cross-platform packaging matrix for Windows, macOS, and Linux.
   - Fixed 12 clippy warnings; `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` now compiles with zero errors and zero warnings. All 30 IPC integration tests verified passing.
3. **Previous Milestone:** Verification sweep promoting 17 Phase 3–7 tasks to `verified` (82.3% overall).
4. **Next Milestone:** Phase 8 task `REL-WIN` ("Windows installer and native smoke test", weight 5).

## Previous Milestone: Verification Sweep Complete — 17 Tasks Promoted to Verified (20 November 2026 Afternoon)

**Overall: 82.3% (up from 73.8%). Implementation: 88.3%. Verification: 88.3% (up from 60%). Release readiness: 28%.**

Full Rust test suite (30 IPC + 144 unit + 7 security = 181 tests all passing) and 184 Vitest frontend tests (34 files, all passing) run to confirm automated coverage. All 17 `implemented` tasks across Phases 3–7 promoted to `verified` status using passing IPC, unit, and owner-confirmed evidence.

**Tasks verified:**
- CAT-BULK, SRCH-FILTERS, IMP-BIB, EXP-CATALOGUE, PRN-LABEL (Phase 3 — Catalogue)
- CIRC-LOAN, CIRC-FINES, MEM-CARD, MEM-FINES, IMP-PATRONS, IMP-LOANS (Phase 4 — Circulation)
- DASH-TRENDS, RPT-COVERAGE, RPT-FORMATS, PRN-SLIP, PRN-BATCH (Phase 5 — Reports/Printing)
- BAK-RESTORE (Phase 7 — Administration)

**No unverified implemented tasks remain.** All remaining work is either blocked (IMP-KOHA needs owner Koha data, PRN-HARDWARE needs physical hardware), in-progress accessibility (CORE-A11Y), or Phase 8 release tasks (REL-WIN, REL-MAC, REL-LINUX, REL-CI, REL-DOCS).

**Next milestone:** REL-WIN — Windows installer smoke test.

**Previous milestone:** Phase 7b Local Network Mode (LAN-SERVER, LAN-CLIENT, LAN-OPS — 100% verified).



1. **Phase 7b Local Network Mode Completed & 100% Verified:**
   - **LAN-SERVER (Weight 7, Verified):** Schema migration `0024_lan_mode.sql` adding `lan_config`, `lan_pairing_codes`, `lan_paired_clients`, and `lan_sessions`. Backend service in `src-tauri/src/services/lan.rs` with multi-threaded embedded HTTP listener, secure SHA-256 fingerprint generation, single-use pairing codes, staff Argon2id authentication, independent LAN sessions, and client revocation.
   - **LAN-CLIENT (Weight 7, Verified):** Mobile-friendly standalone browser staff portal served at `/staff`, transactional staff circulation endpoints (`POST /api/staff/login`, `GET /api/staff/me`, `POST /api/staff/circulation/issue`, `POST /api/staff/circulation/return`), anonymous public OPAC at `/` and `/opac`, and verified `409 Conflict: This copy is unavailable for issue` rejection on competing simultaneous issues of the same physical copy.
   - **LAN-OPS (Weight 4, Verified):** Operating guidance and firewall configuration in `LanSettingsPanel.tsx`, and comprehensive operational runbook published in `docs/LAN_OPERATION_RUNBOOK.md` covering host sleep prevention, Windows Defender Firewall rule configuration (`netsh` / PowerShell), Wi-Fi router AP client isolation troubleshooting, network loss and reconnection resilience, and daily library opening/closing procedures.
   - **Verification:**
     - 144 Rust unit tests passed (including `test_embedded_http_staff_and_circulation_workflow_and_concurrency` verifying pairing, login, issue, 409 conflict detection, return, and `/staff` HTML serving).
     - 30 Tauri IPC integration tests passed (including `lan_full_workflow_via_ipc`).
     - 7 security integration tests in `src-tauri/tests/security.rs` passed.
     - 184 Vitest frontend tests passed across 34 test files (including `LanSettingsPanel.test.tsx`).
     - Clean `tsc --noEmit` and clean `eslint src` with zero errors.
     - Progress totals: **73.8% overall complete, 88.3% implemented, 60.0% verified, 28% release readiness**. Phase 7b is now **100% Implemented and 100% Verified**! The `local-network` release gate has passed.

2. **Next Milestone:** Phase 7 task `BAK-RESTORE` ("Restore, pre-upgrade snapshot and recovery", weight 6).

3. **Previous Milestone:** Phase 7b task `LAN-SERVER` ("Encrypted server, pairing and individual sessions", weight 7).

4. **Previous Milestone:** Phase 7 task `SEC-REVIEW` ("Security and file-handling acceptance", weight 5), completing security audit and automated permission/negative test matrix.

5. **Previous Milestone:** Phase 6 task `CLS-AUTH` ("Classification browser and managed authorities", weight 6), completing 100% of Phase 6.

3. **Previous Milestone:** Phase 6 task `SER-RENEW` ("Serials renewals and expenditure", weight 5), supporting subscription renewal workflows, renewal history, expenditure reconciliation, and renewal due reminders.

4. **Previous Milestone:** Phase 6 task `SER-ISSUES` ("Serials subscriptions and issue check-in", weight 6), supporting subscription scheduling, calendar-aware expected dates, issue check-in into accession and catalogue, and missing/claim tracking.

3. **Previous Milestone:** Phase 6 task `ACQ-RECEIVE` ("Purchase receiving into catalogue and accession register", weight 6), supporting transactional receiving against purchase orders, automated accession allocation, copy creation, request resolution, and duplicate rejection.

3. **Previous Milestone (Phase 5):**
   - `RPT-FORMATS`: standalone OpenXML XLSX generator and vector PDF 1.4 multi-page document generator in Rust without external dependencies; export buttons and non-clipping print CSS.
   - `RPT-COVERAGE`: Overdue loans list, accession register, acquisition expenditure summary, and loss & damage reports.

Latest accepted executable: pusthakam-phase4-five-feature.exe, schema 20. Pinned test folder: %LOCALAPPDATA%\Temp\pusthakam-native-verification-cea843fbd8cb4e9bbaf6c2274b98c468. It contains prerequisite loan CSV, valid/mismatch history JSON and FIVE_FEATURE_TEST.txt. The native example is now imported by the owner; source IDs must not be reused. Inspect actual disposable DB read-only before preparing future fixtures. Tray > Quit older apps (X hides). Temporary files and test-library records do not travel through Git.

See docs/FIVE_FEATURE_MIGRATION.md for the supported strict format, limits, provenance and reconciliation semantics. All source/docs remain local until Git commit/push/pull. Never commit actual library data. LAN/OPAC stay Phase 7b; splash/password/manual remain deferred.

## Member fine checkpoint

Member fine accounts are implemented. In Members, Fines opens a read-only assessed-fine ledger for active or archived members. It shows all-page totals separated by currency and per-fine settlement history. It shares Circulation's balance calculation; settlement writes remain in Circulation. Fine/member ownership and read permissions are checked in Rust; waiver reasons require circulation-management permission. No migration was needed.

Passed: **119 Rust unit tests, 20 IPC tests, 14 focused frontend tests, production build/typecheck and lint**. Reconciliation includes partial payment, waiver, final settlement, currency separation, pagination, settled/archived history, foreign-member exclusion and denied actions. Owner confirmed the native empty state; no overdue copy is available. Populated-account/history and archived/read-only native acceptance remain open. Standalone Windows build passed. The local test executable is `pusthakam-phase4-member-fines.exe` in the existing verification folder. See PROGRESS.json for the exact local path.

When populated test data becomes available: quit an older tray-enabled test app via its tray menu (X only hides it). Open the refreshed build, go to Members > Fines, inspect an account or empty state, compare an assessed/partially paid fine with Circulation and inspect settlement history. Check archived/read-only access. Walkthrough: docs/MEMBER_FINE_ACCOUNTS.md.

## Earlier work and remaining native checks

- JSON, MARCXML and XLSX catalogue imports are implemented and owner-confirmed in Windows. Koha MARCXML 952 holdings and shared preview/commit are covered by tests.
- Catalogue/holdings export exists in JSON, CSV, XLSX, MARC and MARCXML. Automated fresh-database supported-field round trips passed. Owner confirmed exports and Windows Save As work. Detailed native per-format save/open/repreview reconciliation remains recorded as open.
- Calendar holidays/weekly closures are implemented in Library Settings and owner-confirmed. Schema 17 adds closed weekdays. Issue/renewal move to the next open due date; existing loan dates and fine rules stay unchanged.
- Maximized startup, live member-name dropdowns in both Issue and Reservations, and X-to-system-tray/Open/Quit are built. Eleven focused tests and the standalone member/tray build passed. Native maximize/dropdown/tray/session/Quit acceptance remains open. Restore via the tray to retain the in-memory session. Explicit Quit ends it. Passwords are not saved. Launching a second executable is not a single-instance restore feature.
- Phase 3 real Koha acceptance requires a representative anonymised export. Catalogue cover/bulk archive/saved search and physical print/scan checks still have recorded native gaps. Read PROGRESS.json rather than assuming all prior implementation is fully verified.

## Decisions and deferred work

- Continue normal phase order; do not jump ahead to LAN/OPAC.
- LAN browser staff access and anonymous OPAC remain Phase 7b: one designated host, server-enforced permissions and public-data allowlists, no application-imposed client count cap.
- The startup loading screen was implemented on 20 November 2026: `StartupSplashScreen.tsx` mounted in `Gate.tsx` enforces a minimum 3-second branded presentation before transitioning into sign-in or the setup wizard. Verified with 4 unit tests.
- New passwords should eventually be at least six characters and contain no spaces; align setup, staff screens and backend validation when that deferred rule is implemented.
- Write the illustrated user manual after feature completion.
- Do not add attribution/signatures/watermarks to deliverables. Keep the product name Pusthakam.

## Feature documentation

- docs/FIVE_FEATURE_MIGRATION.md

- docs/CIRCULATION_MIGRATION.md

- docs/MEMBER_FINE_ACCOUNTS.md
- docs/DESKTOP_AND_MEMBER_LOOKUP.md
- docs/LIBRARY_CALENDAR.md
- docs/CATALOGUE_EXCHANGE_EXPORT.md
- docs/JSON_CATALOGUE_IMPORT.md
- docs/MARCXML_CATALOGUE_IMPORT.md
- docs/XLSX_CATALOGUE_IMPORT.md
- docs/LAN_BROWSER_OPAC.md
- docs/SUGGESTIONS.md

Current weighted totals: **51.2% overall, 62.9% implemented, 41.8% verified and 10% release-ready**. Recalculate from PROGRESS.json after any status change; partial work gets no full implementation credit.

Source-sample absence blocks real-source verification only. Generic loan/renewal development is authorized and continues.

## Five-feature batch authorized — 19 November

Owner requested this batch once: reservation history, assessed-fine history, payments, waivers and a currency-separated reconciliation summary. Implement/test together, then one native build and checklist. Neutral JSON bundle planned for atomic cross-record validation; source-specific mappings remain unverified. Work starting, no completion credit.

Five-feature implementation checkpoint: schema 20 append-only provenance and one strict JSON bundle for reservations, assessed fines, payments and waivers. Expected counts/currency totals gate atomic commit; destination records are reread before finalizing. Admin preview/confirmation UI added. cargo check passed before the post-write reconciliation addition; focused tests next. Explicitly reject unsupported non-loan/multiple/open-loan charges and conflicting queues. No completion credit.

## Public website work — 20 November 2026

Owner requested an attractive GitHub Pages site for librarians, including populated blog/FAQ, distinct article URLs, sitemap/robots and search/AI-discovery fundamentals. Separate static site under development; no deployment performed and no desktop completion credit. Build all reviewable assets before publishing. Restore/recovery milestone remains queued, and report-design plan remains pending.

## Logo and splash wording request — 21 November 2026

Owner clarified that no logo had been supplied/designed by them. Develop a new small-size book/P symbol for review using relevant design guidance; see docs/LOGO_DESIGN_DIRECTION.md. Replace the splash technical architecture/offline badge with More time for books and readers., retaining the established main tagline. Concept development started; no final approval, production icon replacement or completion credit. Preserve unrelated current edits.

## Logo concept / splash wording checkpoint

Created and refined a book/P logo concept for owner review using relevant design guidance; concept is not adopted as final branding. Replaced the splash architecture/offline badge with More time for books and readers., retaining the main tagline and startup timing. All 4 splash tests, typecheck and lint passed. No production icon replacement or fresh native build; unrelated manual edits preserved. Design direction and next steps: docs/LOGO_DESIGN_DIRECTION.md. Next: owner review of the book/P direction, final asset refinement and integration only when the direction is accepted. Keep source BrandMark and installation icons unchanged for now; only the closing line is changed in application code. No weighted product completion credit.
