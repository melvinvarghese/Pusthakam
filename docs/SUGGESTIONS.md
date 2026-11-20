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

For quick cross-reference, earlier owner-requested changes deferred in `PROGRESS.md`:
* **Startup Presentation:** 3-second startup loading screen.
* **Password Policy:** Minimum 6 characters and no spaces.
* **Documentation:** Illustrated end-user manual covering setup, workflows, migration, and troubleshooting.
* **Staff Profiles:** Optional locally stored profile photos for administrators and library staff.
* **Recovery Redundancy:** Secondary recovery credentials scoped to a recovery contact for single-library emergency restore.

## Immediate next-session owner request

Make Choose file visibly a button; owner says it currently looks like random text in import screens. Inspect the shared file-field styling so the fix is consistent, keyboard accessible and supports light/dark/disabled/focus states. Prioritize this before Phase 4 reservation migration. Recorded 18 November 2026; implemented 19 November in shared TextField/native selector-button styling, including catalogue import. Nineteen focused tests and build/typecheck/lint passed; refreshed native build pusthakam-phase4-file-picker.exe passed; owner confirmed visible button appearance in Windows on 19 November. Renewal import itself was owner-confirmed working.

Owner result, 19 November: Choose file is now visibly a button in Windows. Appearance accepted; unreported theme/keyboard checks are not inferred. Next development task is reservation migration, followed by financial history.

## Owner-approved report presentation requirement — 20 November 2026

Polished, aesthetically beautiful reports are the default; do not settle for plain table dumps. Provide an unchecked **Use simple report design** checkbox for conventional minimal layouts with unchanged data. Use relevant installed design/document/PDF/spreadsheet skills when implementing and reviewing outputs. Applies to preview/PDF/print/styled XLSX; CSV remains data-only. See REPORT_DESIGN_PLAN.md. Planned, not implemented; no progress credit.

## Deferred owner request: GitHub Pages website

Record now, implement later: an attractive librarian-focused homepage, populated Blog and FAQ tabs, individually addressable/crawlable blog posts, sitemap.xml, robots.txt and metadata/structured data for ordinary and AI-assisted search discovery. Relevant design/writing/site skills are available. No implementation or publication now. See GITHUB_PAGES_WEBSITE_PLAN.md for hosting/root-robots constraints, content and acceptance.
