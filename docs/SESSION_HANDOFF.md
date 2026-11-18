# Development handoff

Use the repository files as the shared development memory between computers. Chat history is not the authoritative project state.

## Resume on any computer

1. Commit and push the source changes and documentation from the computer where work finished. Pull those changes on the next computer before starting. Uncommitted files are local only.
2. Read `CLAUDE.md`, `PROGRESS.json`, `PROGRESS.md` and this file. Follow `docs/DEVELOPMENT_ROADMAP.md` and the product specification.
3. Check `git status` and preserve any local work. If two computers changed the same files, reconcile them before continuing; do not overwrite or force-push.
4. Standing owner requirement: after every meaningful change, immediately update both `PROGRESS.md` and `PROGRESS.json`, this handoff and affected feature docs, including partial work and failed/incomplete checks. Do not wait for a computer switch or session end. Ensure these records agree before ending each turn. Run `npm run progress:validate -- --write`, then `npm run progress:validate`.
5. Never put library databases, backups, passwords, recovery codes or staff/member data in Git. Moving development work does not move the library database.

The current working clone is `D:\Pusthakam` on the owner's Windows computer. Other computers may use another clone path. Temporary verification executables and their disposable database are machine-specific; rebuild from source rather than relying on those paths.

## Current milestone — 18 November 2026

Phase 4 is active. The owner confirmed the new holiday calendar works in Windows. Automated calendar checks passed: 119 Rust unit tests, 19 IPC tests, 14 settings UI tests, production build/typecheck and lint. Calendar verification is recorded in PROGRESS.json.

Current requested changes (implementation added; automated/native verification status is in PROGRESS.json):

- Start the desktop app maximized so it fills the working screen.
- Clicking Find member by name in Circulation must show matching names as staff type, without a separate Search click. Use a bounded, debounced dropdown; show member numbers to distinguish identical names; support keyboard selection and prevent stale results from selecting the wrong member.
- Offer the same member-name lookup in Reservations. Keep member-number entry and backend permission/eligibility checks.
- Clicking the window X should hide the app in the notification-area/system tray and keep the signed-in session in the running process. Add Open Pusthakam and Quit tray actions. A real quit must end the session; do not save passwords to bypass sign-in. The session resumes through the tray icon. Reopening via a second process is a separate behavior unless explicitly implemented and tested.

The live lookup has 11 passing focused frontend tests, with production build/typecheck/lint also passed. Tray-enabled Rust check and the standalone Windows build passed. The native test executable is `pusthakam-phase4-members-tray.exe` in the existing machine-specific verification folder. Maximize/live dropdown/tray/session/Quit acceptance remains open; do not infer native success from compilation. Current source and PROGRESS.json determine what has actually been tested.

## Decisions and deferred work

- Normal phase order remains in force. Phase 3 implementation exists, but representative Koha acceptance and some native catalogue/printing checks remain open.
- LAN browser staff access and anonymous OPAC remain Phase 7b. One designated host holds the database. No application-imposed client count cap.
- A startup loading screen visible for at least three seconds was requested and explicitly deferred. It is documented in the roadmap and progress notes; do not implement it as part of the current work.
- New passwords should eventually be at least six characters and contain no spaces. Keep this deferred until aligned across setup, staff screens and backend validation.
- Write the illustrated user manual after feature completion.
- No attribution/signatures/watermarks in deliverables. Keep the name Pusthakam.

## Completed work in this session

- JSON, MARCXML and XLSX staged catalogue import, including Koha MARCXML holdings; owner Windows confirmations recorded.
- Catalogue/holdings export in JSON, CSV, XLSX, MARC and MARCXML. Supported-field fresh-database round trips passed. Export limitations are documented; exchange files are not database backups.
- Windows Save As for catalogue exports, with chosen folder/filename, overwrite confirmation, cancellation, saved-path feedback and atomic file publication. Owner confirmed it works.
- Library calendar with holiday CRUD, recurring weekly closed days, schema 17, audit transactions and shared issue/renewal next-open-date calculation. Existing loan due dates and fine rules are unchanged. Owner confirmed holidays work.

Feature docs: `DESKTOP_AND_MEMBER_LOOKUP.md`, `JSON_CATALOGUE_IMPORT.md`, `MARCXML_CATALOGUE_IMPORT.md`, `XLSX_CATALOGUE_IMPORT.md`, `CATALOGUE_EXCHANGE_EXPORT.md`, `LIBRARY_CALENDAR.md` and `LAN_BROWSER_OPAC.md`.

Read `verificationNeeded` and blockers in PROGRESS.json before granting completion credit. Real anonymised Koha data and physical print/scan checks remain external acceptance requirements.

Latest checkpoint: owner confirmed the holiday calendar. Current weighted totals are 50.8% overall, 62.1% implemented, 41.8% verified and 10% release-ready. No source/documentation commit or push was performed during this session; sync the current working tree before changing computers.
