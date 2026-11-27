# Pending Work

_Created 27 November 2026 by a review (Claude). Single checklist of what is still open in Pusthakam, apart from Malayalam translation (see `docs/translations/README.md`). Each line names its source; open that source for full detail before starting. Do not duplicate details here: update the source, then tick the box here._

**How to use this file (people or AI agents):**
- Pick an item, read its source, follow CLAUDE.md rules (tests first for bug fixes, no code without tests, never mark VERIFIED without running them).
- When done, tick the box, add the date and the evidence (test names, build, owner check), and update the source document's status.
- Items marked **[owner]** need Melvin: a decision, physical hardware, or a real library. An agent can prepare checklists for them but cannot complete them.
- Items marked **[unchecked]** come from older suggestions that this review did not verify against the code. Check the code first; they may already be built.

Suggested order: section 1, then the [owner] tests in section 3, then section 4. Sections 2, 5 and 6 can run alongside.

## 1. Safety and security fixes (do first)

- [x] Keep 7 daily automatic backups instead of one day, in a folder chosen at install (another drive or USB suggested), with fallback and a warning when that folder is missing; start with Windows in the tray so the daily backup runs. Done 27 Nov 2026 (owner spec): IPC tests `daily_backups_use_the_chosen_folder_and_fall_back_when_it_is_missing` and the updated `manual_backups_are_permission_checked_complete_and_retained`; Vitest `BackupPage.test.tsx`, `StartupPanel.test.tsx`. Source: SUGGESTIONS S-R01, KNOWN_ISSUES KI-027.
- [ ] **[owner]** Try the new Windows installer once: backup-folder prompt, Pusthakam starting with Windows in the tray (hidden icons), a dated daily file in the chosen folder, and the warning after unplugging the USB drive. Source: KI-027.
- [ ] Optional, if wanted later: weekly and monthly backup copies, and a "last backup copied off this computer" dashboard warning. Source: S-R01.
- [ ] **[owner]** Decide how to protect staff sign-in on Wi-Fi: HTTPS on the LAN, or browser staff access off by default with a warning. Then implement and update DECISIONS D-026. Source: S-R02, KI-028.
- [ ] Remove the old LAN staff API (`/api/lan/pair`, `/api/staff/login`, `/api/staff/circulation/*`, `GET /api/lan/status` fingerprint leak) from `handle_http_connection` in `src-tauri/src/services/lan.rs`; keep only the `/api/staff/commands` gateway. Source: S-R03, KI-029.
- [ ] Add an on/off setting for online cover lookups (contacts `covers.openlibrary.org`, sends ISBNs) and mention it in README's privacy section. Source: S-R15.
- [ ] Review `npm audit` and run `cargo audit` before release. Source: KI-006.

## 2. Features not built yet

- [ ] Polished report designs by default plus an unchecked "Use simple report design" option (preview, PDF, print, styled XLSX; CSV stays data-only). Source: DEVELOPMENT_ROADMAP "Planned report presentation enhancement", REPORT_DESIGN_PLAN.md.
- [ ] Author and publisher authority records, subject hierarchies, bulk edit and bulk delete (archive, never hard delete). Source: KI-017.
- [x] OPAC browse categories instead of DDC class buttons (owner decision 27 Nov 2026, DECISIONS "OPAC browse categories"). Done 27 Nov 2026 on branch `opac-browse-categories`: migration 0025, Rust tests `services::browse_categories::tests::*`, `services::lan::tests` OPAC filter test, IPC tests `browse_categories_drive_record_categories_and_need_permission` and the `/api/opac/categories` route check; Vitest `browseCategories.test.ts`.
- [ ] **[owner]** Check the new OPAC category buttons and the default category list on a real catalogue; rename, add or archive categories to suit the library. Source: DECISIONS "OPAC browse categories".
- [ ] Optional: a parent level for browse categories (Fiction → Malayalam Fiction), Malayalam display names for categories, and bulk "add category to selected books" in the catalogue. Source: DECISIONS "OPAC browse categories".
- [ ] Import step that maps source codes (e.g. MAIN, BK) to proper location, collection and item-category names. Source: KI-019.
- [ ] Preset layouts for common die-cut label sheets. Source: KI-020, KI-023.
- [ ] MARC-8 encoded MARC import (UTF-8 MARC, MARCXML, JSON and XLSX already exist; update KI-016 text). Source: KI-016.
- [ ] Currencies with other than 2 decimal places (e.g. JPY). Source: KI-012.
- [ ] **[unchecked]** Audio feedback on barcode scan. Source: SUGGESTIONS S-001.
- [ ] **[unchecked]** Redirect barcode-scanner bursts typed outside an input. Source: S-002.
- [x] **[unchecked]** Fill in book details from ISBN (optional, online). Source: S-003. _(Done 2026-11-27: user-triggered `Fill from ISBN` in `RecordFormDialog.tsx` via `src/lib/isbnLookup.ts` using Open Library / Google Books; verified with `isbnLookup.test.ts` (4 tests) and `RecordFormDialog.isbn.test.tsx` (1 test).)_
- [ ] **[unchecked]** Portable Windows zip version. Source: S-006.
- [ ] Finish open source checks for splash quotations. Source: SPLASH_QUOTE_SOURCES.md.

## 3. Real-world testing

- [ ] **[owner]** Install from the NSIS installer on a clean Windows PC; click through setup, catalogue, circulation, fines, reports; uninstall and reinstall and confirm data survives. Source: DEVELOPMENT_ROADMAP phases 2-4, S-R05.
- [ ] **[owner]** Import a real Koha export and reconcile record counts. Source: KI-015.
- [ ] **[owner]** Real USB barcode scanner, label printer, A4 label sheets, member cards, due slips. Source: KI-020, KI-023.
- [ ] **[owner]** Native Save As dialogs for PDF/XLSX reports and catalogue exports. Source: KI-022.
- [ ] **[owner]** Staff pages and OPAC on real phones, including Safari and the on-screen keyboard. Source: KI-026, MOBILE_BROWSER_ACCEPTANCE.md.
- [ ] **[owner]** Host PC sleep, restart and Wi-Fi loss during an issue; two or more staff working at once. Source: TESTING_STRATEGY "Staff HTTP and console-host verification".
- [ ] Import 10,000+ records and time import and search. Source: KI-018.
- [ ] Run the app with the network adapter disabled. Source: DEVELOPMENT_ROADMAP phase 1, TESTING_STRATEGY "Offline".
- [ ] Add Playwright UI tests and run them in CI. Source: KI-010, TESTING_STRATEGY.
- [ ] **[owner]** First run of macOS and Linux builds by a person, or state "Windows first" in README. Source: TESTING_STRATEGY "Linux package acceptance", S-R05.
- [ ] Recheck the class-8 label colour against overdue red with real data. Source: KI-009.

## 4. Release and distribution

- [ ] **[owner]** Choose and add a licence (LICENSE file plus `license` in `package.json` and `src-tauri/Cargo.toml`). Source: S-R04, KI-030.
- [ ] **[owner]** Confirm bundle identifier `app.pusthakam.desktop` is final. Source: DECISIONS D-015.
- [ ] Sign the Windows installer (e.g. SignPath Foundation for open source). Source: S-R08.
- [ ] Versioned GitHub Releases with checksums and notes; website download page; optional "new version available" notice. Source: S-R09.
- [ ] Make CI fail when the Windows or macOS package is missing; align CI Node version. Source: S-R10, KI-033, KI-003.
- [ ] Host-failure recovery guide; run `pusthakam-server` as a Windows service or at start-up. Source: S-R11, WEB_HOSTING.md.

### Commercial trial release readiness

- [ ] **[owner/product]** Decide the 30-day trial start trigger and offline clock policy.
- [ ] **[owner/product]** Decide the exact post-expiry access policy; preserve all library data and never make it inaccessible through destructive expiry handling.
- [ ] **[owner/product]** Set the one-time licence price, licence scope/duration, update entitlement and support entitlement.
- [ ] **[owner/product]** Decide activation, online/offline validation and device-transfer rules.
- [ ] Implement and test trial countdown/expiry, purchase and licence activation only after the owner decisions above are recorded.
- [ ] Complete CI, security, installer and supported-platform acceptance; run a controlled library beta/pilot before the public official release.
- [ ] At official release, publish clear 30-day trial terms and purchase/activation instructions.

Source: owner decision recorded 27 November 2026 in PROGRESS.md, PROGRESS.json, DEVELOPMENT_ROADMAP.md and SESSION_HANDOFF.md.

## 5. Documentation

- [ ] Rewrite `docs/USER_MANUAL.md` as a short, accurate librarian guide with screenshots; move developer material elsewhere. Source: S-R06, KI-031.
- [ ] Update README status table, DEVELOPMENT_ROADMAP phase table (6, 7, 7b, 8 are marked NOT STARTED but largely built), and close or update KI-005 and KI-010. Source: S-R07, KI-032.
- [ ] Archive old entries from `PROGRESS.md`, `PROGRESS.json` and `docs/SESSION_HANDOFF.md`; keep a one-page current status. Source: S-R07.
- [ ] Printable quick start, first-week checklist and demo library. Source: S-R13.

## 6. Code health and growth

- [ ] Split the largest files one at a time, tests green before and after: `services/lan.rs`, `services/catalogue.rs`, `services/serials.rs`, `ClassificationPage.tsx`, `AcquisitionsPage.tsx`, `PeriodicalsPage.tsx`, `tests/ipc.rs`. Source: S-R14.
- [ ] **[owner]** Find 2-3 pilot libraries; add GitHub issue templates and a support contact. Source: S-R12.
