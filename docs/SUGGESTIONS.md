# Pusthakam — Suggestions & Future Enhancements

_Recorded on: 18 November 2026_  
_Purpose: Repository of architectural, ergonomic, and operational suggestions to be reviewed and implemented in future development milestones._

---

## 1. Desk Ergonomics & Hardware Interaction

### S-001: Audio Feedback on Barcode Scan (Success vs. Error Chime)
* **Context:** In busy libraries (especially schools and universities), librarians scan continuous stacks of books during peak issue/return rushes. They watch the physical item and barcode label rather than staring at the monitor on every scan.
* **Proposal:** Provide subtle, offline audio feedback:
  * **Success Chime:** A pleasant, crisp high chime (e.g. 880 Hz sine burst) on confirmed issue, return, or renewal.
  * **Alert / Error Tone:** A distinctive low double-buzz (e.g. 220 Hz sawtooth) when a loan is blocked, an item is overdue/has fines, a copy has an active reservation queue, or a barcode is unrecognized.
* **Technical Approach:**
  * Zero external audio files or dependencies.
  * Use the browser's native **Web Audio API** (`AudioContext` oscillator and gain node) for 100% offline, zero-asset synthesis.
  * Add an Audio Feedback toggle (enabled by default) and volume control in Library Settings.

### S-002: Scanner Keystroke "Wedge" Guard
* **Context:** Handheld USB barcode scanners operate in keyboard wedge mode, firing a high-speed burst of keystrokes (20–50 characters in ~100ms) terminated by `Enter`. If staff inadvertently click outside an input field, the scan characters spill into arbitrary UI elements or accidentally trigger hotkeys.
* **Proposal:**
  * Implement a lightweight global keypress monitor on the Circulation and Inventory screens.
  * Detect burst keystrokes (keystrokes arriving with inter-key intervals `< 30ms`).
  * If burst input is detected while no input field has focus, automatically redirect the buffered characters to the active accession/barcode input box and trigger submission.

---

## 2. Cataloguing & Ingestion Workflows

### S-003: ISBN Metadata Autofill (Extending Open Library Engine)
* **Context:** Pusthakam currently supports fetching book cover art from Open Library when an ISBN is present (`book_cover.rs`). However, cataloguing new books still requires typing titles, authors, publishers, and publication years manually.
* **Proposal:**
  * Add a **"Fetch Details from ISBN"** button in the *Add Record* dialog.
  * When online, query the public Open Library Books API (`https://openlibrary.org/api/books?bibkeys=ISBN:...&format=json&jscmd=data`).
  * Autofill available metadata: title, contributors (authors, editors), publisher, publication year, pagination, and subjects.
  * The librarian reviews, adjusts, and confirms the fields before committing to the database.
  * Fully optional, non-blocking, and adheres to the rule that core functions operate completely offline.

---

## 3. Data Safety & Disaster Recovery

### S-004: Dedicated "Backup to USB / External Drive" Action
* **Context:** Database snapshots currently save to the local OS app-data directory (`pusthakam/backups`). If the primary drive suffers hardware failure or ransomware, local backups on that same physical drive are compromised. Many community and school librarians rely on physical USB thumb drives kept securely off-system.
* **Proposal:**
  * Provide an explicit **"Backup to Flash Drive / External Folder"** action in the Backup & Restore screen and/or the main dashboard.
  * Open a native OS folder/file picker (`export_save.rs` pattern).
  * Execute SQLite's online backup API, verify the schema version and integrity check on the target, and save the verified `.db` snapshot directly to the removable media with a timestamped filename.

---

## 4. Architecture & Future-Proofing

### S-005: Multi-Session Abstraction in Rust (Phase 7b Preparation)
* **Context:** `AppState.session` currently holds a single `Option<Session>` in memory for the single desktop operator. In Phase 7b (LAN mode and browser OPAC), multiple staff devices will be logged in concurrently.
* **Proposal:**
  * Refactor `AppState.session` from `Option<Session>` to a `SessionStore` (`HashMap<SessionToken, Session>`) with expiration tracking and thread-safe locking.
  * Have commands accept a session token handle.
  * Completing this abstraction ahead of Phase 7b avoids large-scale breaking changes across all Tauri commands when network multi-client support begins.

---

## 5. Distribution & Packaging

### S-006: Portable Windows "Zero-Install" Zip Distribution
* **Context:** In institutional environments (government institutions, school computer labs), staff accounts frequently lack administrative privileges required to run Windows NSIS/MSI installers.
* **Proposal:**
  * In Phase 8 release builds, provide a standalone **Portable `.zip`** release alongside the standard installer.
  * Operates with user-level privileges with app data stored in standard local user paths (`%LOCALAPPDATA%`), requiring zero system elevation.

---

## 6. Consolidated Deferred Items

For quick cross-reference, status of owner-requested changes:
* **Startup Presentation:** 3-second startup loading screen (*Implemented 20 November 2026: `StartupSplashScreen.tsx` & `Gate.tsx`*).
* **Password Policy:** Minimum 6 characters and no spaces.
* **Documentation:** Illustrated end-user manual covering setup, workflows, migration, and troubleshooting (*Implemented 20 November 2026: `docs/USER_MANUAL.md`*).
* **Staff Profiles:** Optional locally stored profile photos for administrators and library staff.
* **Recovery Redundancy:** Secondary recovery credentials scoped to a recovery contact for single-library emergency restore.

## Immediate next-session owner request

Make Choose file visibly a button; owner says it currently looks like random text in import screens. Inspect the shared file-field styling so the fix is consistent, keyboard accessible and supports light/dark/disabled/focus states. Prioritize this before Phase 4 reservation migration. Recorded 18 November 2026; implemented 19 November in shared TextField/native selector-button styling, including catalogue import. Nineteen focused tests and build/typecheck/lint passed; refreshed native build pusthakam-phase4-file-picker.exe passed; owner confirmed visible button appearance in Windows on 19 November. Renewal import itself was owner-confirmed working.

Owner result, 19 November: Choose file is now visibly a button in Windows. Appearance accepted; unreported theme/keyboard checks are not inferred. Next development task is reservation migration, followed by financial history.

## Owner-approved report presentation requirement — 20 November 2026

Polished, aesthetically beautiful reports are the default; do not settle for plain table dumps. Provide an unchecked **Use simple report design** checkbox for conventional minimal layouts with unchanged data. Use relevant installed design/document/PDF/spreadsheet skills when implementing and reviewing outputs. Applies to preview/PDF/print/styled XLSX; CSV remains data-only. See REPORT_DESIGN_PLAN.md. Planned, not implemented; no progress credit.

## Deferred owner request: GitHub Pages website

Record now, implement later: an attractive librarian-focused homepage, populated Blog and FAQ tabs, individually addressable/crawlable blog posts, sitemap.xml, robots.txt and metadata/structured data for ordinary and AI-assisted search discovery. Relevant design/writing/site skills are available. No implementation or publication now. See GITHUB_PAGES_WEBSITE_PLAN.md for hosting/root-robots constraints, content and acceptance.

## Paused owner request: real quotations and logo refinement

21 November: replace the current splash closing line with a locally bundled set of about 100 real, source-verified book/reading quotations, one per launch, author credit and no immediate repeat. Not implemented; no quotations have been curated/verified yet. Owner likes the book/P concept but is not fully satisfied; retain draft status and refine its flat silhouette/small-size clarity. Work paused until usage reset; see SPLASH_QUOTES_PLAN.md and the latest SESSION_HANDOFF.md checkpoint.

---

## Release-readiness review: 27 November 2026 (Claude)

_Added by a read-only review of D:\Pusthakam on 27 November 2026. Nothing below is implemented; no progress credit. Each item says where the problem was seen. Review, then accept, change or reject each one. Related new known issues: KI-027 to KI-033 in KNOWN_ISSUES.md._

### Now: before any real library uses Pusthakam

### S-R01: Keep more than one day of automatic backups (see KI-027)
* **Seen:** `run_daily` in `src-tauri/src/services/backup.rs` deletes every older `pusthakam-YYYYMMDD-daily.sqlite3` after creating today's. Only one day of history exists, in the same folder and disk as the live database.
* **Proposal:** keep 7 daily, 4 weekly and 12 monthly copies. Let the librarian choose a second backup folder (USB or another drive) that receives a copy automatically when present. Show "last backup copied off this computer" on the dashboard, with a warning after 7 days.
* **Also:** USER_MANUAL section 12 says "rolling daily snapshots"; correct it once retention is fixed.

### S-R02: Protect staff sign-in on Wi-Fi (see KI-028)
* **Seen:** the LAN listener serves `/staff` and `/api/staff/*` over plain HTTP. No TLS crate is in `src-tauri/Cargo.toml`. DECISIONS D-026 promised an encrypted LAN API. Staff passwords and session cookies cross shared Wi-Fi unencrypted.
* **Options (owner decision):** (a) HTTPS on the LAN with a locally generated certificate and a documented one-time trust step on phones; or (b) keep browser staff access off by default, warn in Settings that it is only safe on a trusted private network, and update D-026.

### S-R03: Remove the old LAN staff API (see KI-029)
* **Seen:** `handle_http_connection` in `src-tauri/src/services/lan.rs` still serves `/api/lan/pair`, `/api/staff/login`, `/api/staff/circulation/issue` and `/return` with bearer tokens and `Access-Control-Allow-Origin: *`, alongside the newer `/api/staff/commands` gateway (cookies, CSRF, origin checks). `GET /api/lan/status` answers without sign-in and returns `server_fingerprint`, which the old login checks.
* **Proposal:** delete the old routes and their inline HTML pages once nothing uses them; keep one staff gateway. Add an attempt limit to pairing codes if pairing stays.

### S-R04: Add a licence (see KI-030)
* **Seen:** no LICENSE file; no `license` field in `package.json` or `src-tauri/Cargo.toml`. Without a licence, others may not legally use, copy or change the code, which conflicts with "free" and "contributions welcome" in README.
* **Proposal:** choose one (GPL-3.0 or AGPL-3.0 keep forks open; MIT or Apache-2.0 are permissive), add LICENSE and the field in both manifests, and mention it in README and the installer.

### S-R05: First real-world trial
* Install from the NSIS installer on a clean Windows PC (not the dev laptop), run setup, import a real Koha export (KI-015), issue/return/renew, print labels and a member card on real printers, scan with a real USB scanner (KI-020, KI-023), use staff pages on a real phone including Safari (KI-026), back up, restore, uninstall and reinstall. Record every problem as a known issue.
* Also test: host PC sleep/restart/Wi-Fi loss during an issue; a 10,000+ record import (KI-018); two staff working at once.

### S-R06: Rewrite the user guide for librarians (see KI-031)
* **Seen in `docs/USER_MANUAL.md`:** offers an MSIX package and a "Universal" macOS DMG that are not built; says macOS 12 minimum while `tauri.conf.json` says 10.15; describes restore as an immediate swap with rollback, but the app queues restore for the next start; section 14 shows signing commands with placeholder names.
* **Proposal:** a short, true guide with screenshots: install, first setup, daily desk work, backups to USB, what to do if the host PC dies. Move developer material (signing, CI) to a separate developer doc.

### S-R07: Bring status docs up to date (see KI-032)
* README status table says Acquisitions & serials "Planned"; both are built.
* DEVELOPMENT_ROADMAP marks phases 6, 7, 7b and 8 "NOT STARTED"; much of that work exists.
* KNOWN_ISSUES KI-005 (placeholder icon) is out of date: icons were regenerated from the approved logo on 23 November. KI-010 says CI is not set up; `.github/workflows/ci.yml` exists.
* `PROGRESS.md` (214 KB), `PROGRESS.json` (420 KB) and `docs/SESSION_HANDOFF.md` (142 KB) are too large to read. Archive old entries (e.g. `docs/archive/`) and keep a one-page current status.

### Next: before a public release

### S-R08: Sign the Windows installer
* Unsigned installers trigger SmartScreen ("Run anyway"), which non-technical librarians will distrust. Free signing for open-source projects exists (for example SignPath Foundation). Apple notarisation is needed if macOS is offered.

### S-R09: Versioned releases and update notice
* Still version 0.1.0; CHANGELOG has only "Unreleased"; no GitHub Release exists. Tag versions, publish releases with SHA-256 checksums and notes, add a download page to the website.
* No updater. Add an optional "new version available" notice (respecting the offline rule: check only when online, never required) or Tauri's updater with signed updates.
* Confirm the bundle identifier `app.pusthakam.desktop` is final before the first release (DECISIONS D-015).

### S-R10: Make CI fail when a package is missing (see KI-033)
* **Seen in `.github/workflows/ci.yml`:** the Windows smoke step is wrapped in `if ($installer)` and the macOS step in `if [ -f "$DMG_FILE" ]`, so a missing package passes silently. CI uses Node 20 while development uses Node 26 (KI-003). Add `npm audit` / `cargo audit` steps (KI-006).

### S-R11: Host PC failure plan and service mode
* When the host PC is off, nobody can work. Write a one-page recovery plan: keep a recent USB backup; install Pusthakam on another PC and restore it in under 15 minutes. Recommend a UPS.
* `pusthakam-server` runs only as a console window (WEB_HOSTING.md). Offer a Windows service or start-on-boot option.

### Later: growth and upkeep

### S-R12: Pilot libraries and a support route
* Find 2 or 3 pilot libraries. Add GitHub issue templates and a contact (email or WhatsApp) on the website and in the app's About page.

### S-R13: Demo library and first-week checklist
* An optional sample library so people can try Pusthakam without their own data (kept out of production paths, per CLAUDE.md rule 11). A printable one-page quick start and a first-week checklist: set a USB backup folder, print a first label, add a first member.

### S-R14: Split the largest files gradually
* `src-tauri/src/services/lan.rs` 87 KB (includes old inline HTML), `services/catalogue.rs` 75 KB, `services/serials.rs` 63 KB, `src/features/classification/ClassificationPage.tsx` 61 KB, `acquisitions/AcquisitionsPage.tsx` 57 KB, `serials/PeriodicalsPage.tsx` 53 KB, `src-tauri/tests/ipc.rs` 187 KB. CLAUDE.md asks for small, focused files. Split one file per change, with tests passing before and after.

### S-R15: Make online cover lookup a visible setting
* Cover lookups contact `covers.openlibrary.org` (allowed in both CSPs) and send ISBNs. That is optional and sends no member data, but README says "no network". Add an on/off setting in Library Settings and mention it in README's privacy section.
