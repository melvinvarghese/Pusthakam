# Changelog

## [Unreleased]
### Added
- Live member-name suggestions in Circulation and Reservations, maximized startup, and close-to-tray with explicit Open/Quit actions.
- Library calendar settings for audited holiday dates and recurring closed weekdays, shared by issue and renewal due-date calculation.
- Windows Save As destination selection for catalogue exports, with cancellation, overwrite confirmation and saved-path feedback.
- Selected-title catalogue and holdings export in JSON, CSV, XLSX, MARC and MARCXML, with format warnings, permission checks and supported-field round-trip tests.
- Classification scheme codes in JSON/tabular imports, UDC/LC MARC mapping and Koha 952 inventory/accession-number import.
- Phase 1 foundation: Tauri 2 shell, Rust backend (SQLite WAL, migrations, settings, errors, logging), React UI shell, 17 Rust and 21 frontend tests.
- Visual world "Spine Label": ink rail and top bar, call-number label component, light/dark themes, Ctrl+K navigation, bundled fonts.
- `PRODUCT.md`, `DESIGN.md`, `.impeccable/` design records.
- Phase 2: first-launch setup wizard, local staff sign-in (argon2id, lockout, roles and permissions enforced in Rust), Library Settings page (library, member/item categories, loan rules, collections, locations, shelves), audit entries for changes, command-layer IPC tests.
- Phase 3: catalogue (records, contributors, subjects, ISBN, duplicates, archive), copies with accession numbers/barcodes/statuses and history, Malayalam-aware full-text search with scanner shortcuts, Accession Register, accession number format setting, printable barcode labels.
- Import: MARC (ISO 2709) with a Koha profile and CSV with column mapping; preview-then-confirm wizard, duplicate policies, one transaction, full report.
- Roadmap: LAN mode (several computers, one library) added as Phase 7b.
- "Online Tools" menu (DDC Number List, Library Toolkit) opening approved links in the system browser.
- Roadmap: MARC21/Koha migration moved up to Phase 3.
### Changed
- Removed the permanent "Offline" message from the UI at the owner's request.
- Replaced the original ivory/sage Inter UI (rejected as generic) with Spine Label. Removed the Malayalam word from the UI at the owner's request.
- Phase 0 documentation: product specification, architecture, database design, roadmap, testing strategy, decisions, known issues.
- `CLAUDE.md` persistent project instructions; `PROGRESS.md` status tracking.
