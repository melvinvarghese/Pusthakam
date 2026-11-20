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

## Current task: Phase 7b Complete & Verified (Local Network Mode: LAN-SERVER, LAN-CLIENT, LAN-OPS)

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
- The startup loading screen must remain visible for at least three seconds when implemented; the owner explicitly deferred it. Do not implement it as part of unrelated work.
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
