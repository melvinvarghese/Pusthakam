# Development Roadmap

Status key: NOT STARTED · IN PROGRESS · IMPLEMENTED · VERIFIED · BLOCKED

| Phase | Scope | Status | Acceptance criteria |
|---|---|---|---|
| 0 | Discovery, docs, architecture | VERIFIED | Product scope and architecture documented. Migration and LAN decisions recorded. |
| 1 | Tauri shell, React UI shell, SQLite + migrations, nav, design system, theme, errors, logging, settings | IMPLEMENTED (persistence verified; offline-with-network-off, fmt/clippy, Playwright, CI still open: see PROGRESS.md) | Launches offline; DB created in app-data and persists across restarts |
| 2 | First-launch wizard, library profile, collections/locations/shelves, categories, circulation policies, admin login | IMPLEMENTED (50 Rust + 54 frontend tests pass; not yet clicked through inside the Tauri window) | Admin configures a new library with no code edits |
| 3 | Catalogue, contributors, subjects, classification, copies, accession, barcodes, search, bulk import (CSV + MARC/Koha) | IMPLEMENTED (103 Rust + 81 frontend tests pass; Koha mapping unverified against a real export; not clicked through in the Tauri window) | One title → many independently managed copies |
| 4 | Members, issue/return/renew/reserve, due dates, overdue, fines, history | IN PROGRESS (core circulation, cards, photos and Koha patron CSV workflow implemented; desktop verification pending) | Full borrow/return workflow on real records |
| 5 | Dashboard analytics, reports, PDF/CSV/XLSX, labels, receipts | IN PROGRESS (live dashboard, circulation reports, CSV and due slips available) | Reports match DB; exports open in Excel/PDF readers |
| 6 | Acquisitions, serials, stock verification, advanced classification, MARC21 | NOT STARTED | Receive PO → records + copies created |
| 7 | Staff/roles, audit log UI, backup/restore, integrity checks | NOT STARTED | Backup → restore round-trip verified |
| 7b | **Several computers, one library (LAN mode)**: owner request 2026-11-15 | NOT STARTED | Staff on multiple same-LAN computers see and change the same data using individual accounts and one host, with no internet required |
| 8 | Installers, CI matrix, signing docs, user docs | NOT STARTED | Per-platform smoke tests passed |

## Several computers on one network (LAN mode), owner request 2026-11-15
Was **not** in the original plan (the brief describes one local database per installation). Added as Phase 7b.
- **Design:** one computer is the *Library Server*: it holds the only copy of the database and runs a small server inside Pusthakam. The other computers run the same app as *Clients* and connect to it by address plus a one-time pairing code. All rules and permission checks stay in the server, so every librarian sees exactly the same data and each signs in to their own account. Works on a local network with **no internet**.
- **Why not just share the database file on the network:** SQLite files on network shares (SMB/NFS) are not safe for several writers; file locking is unreliable and it can corrupt the library. We will not do this.
- **Needed:** (1) per-connection sessions instead of the single in-memory session; (2) the command layer callable without Tauri (the services already hold all logic, commands stay thin); (3) an authenticated, encrypted LAN endpoint (self-signed certificate, fingerprint confirmed by the pairing code), listening only when the owner turns LAN mode on; (4) live refresh (polling or server events) so one librarian's change appears on the others' screens; (5) edit-conflict protection (a record changed by someone else while you edit is detected, not silently overwritten); (6) server-side backups; (7) settings: turn LAN mode on/off, show address and pairing code, list and revoke connected computers; (8) installer guidance for the Windows firewall.
- **Trade-offs to accept:** the Server computer must be on during opening hours; if it is off, other computers cannot work. A later option (not planned) would be a spare "backup server".
- **To avoid rework, from now on:** new commands stay thin and put logic in `services/`; no new global state that assumes a single user.
- **Hosting and capacity:** each library designates one PC to host its database. Other computers on the same LAN connect as clients. There is no fixed client-count limit in Pusthakam. Measure concurrent capacity on representative host hardware and networks during Phase 7b.

## Browser staff access and public OPAC (owner request 2026-11-18)

Extend Phase 7b beyond desktop clients. When the owner enables LAN hosting and Pusthakam is running on the designated main PC, serve locally bundled browser pages from that PC. Staff use a phone, tablet or computer browser without installing Pusthakam; sign-in and permissions are enforced on the server. Keep desktop-client support planned, with a shared service layer and transport adapter rather than browser calls to Tauri IPC.

Provide a separate public OPAC entry page without staff login. Visitors on a reachable local network can search titles, authors, subjects and ISBN, view public bibliographic details, covers, call number/location and current availability. Public responses must use an allowlist and exclude member details, borrower identities, staff details, internal notes, finance, audit logs and administrative data. The public endpoint exposes no catalogue or circulation writes. Public access must not inherit a host desktop session or unlock staff functions.

Host settings should expose the staff and OPAC addresses and QR codes, LAN/OPAC enable controls, and connection/revocation status. Host-on availability, IP changes, firewall rules and Wi-Fi client isolation must be documented. Same Wi-Fi does not by itself guarantee that devices can reach each other. Closing or sleeping the host makes browser access unavailable; no cloud or router port forwarding is required.

Browser staff sign-in needs per-session authentication, expiry/revocation, transport encryption with a documented certificate trust workflow, and request forgery/origin protections. Do not assume the existing desktop session or desktop pairing/certificate UI can be reused unchanged in phone browsers. Native OS functions such as file dialogs, printing and exports need browser-compatible paths. Measure capacity; keep no application-imposed client count cap.

Acceptance: test staff sign-in and allowed/denied actions from a phone browser; anonymous OPAC search and no-match results; availability after issue/return; privacy of every public response; blocked unauthenticated writes; concurrency and stale edits; session revocation; network loss, host restart and reconnect. Run the pilot with internet disconnected. See LAN_BROWSER_OPAC.md for the handoff. This is planned work, not implemented.

## Migration from other systems (e.g. Koha), owner request 2026-11-15
Libraries must be able to move their existing data into Pusthakam. Plan:
- **Moved earlier:** MARC21 / ISO 2709 / MARCXML import is built together with the catalogue in **Phase 3** (it was Phase 6), because Koha and most library systems export catalogues in MARC.
- **Koha profile:** a ready-made mapping for Koha exports: bibliographic records (MARC) with holdings in field 952 mapped to physical copies (barcode, call number, branch/shelving location, collection, item type, acquisition date, price, copy number); patron exports (CSV) to members and member categories; current checkouts, closed loan history, and outstanding and historical fines so a library can go live mid-stream. Field mappings will be checked against Koha's documentation and a real sample export before building; nothing is assumed.
- **Phase 4:** migrate members and all available circulation and fine history, including current and closed loans, renewals, reservations, and fine settlements.
- **Always:** staged wizard (preview, map fields, validate, duplicates, errors and warnings, confirm, transactional import, summary), nothing silently discarded, nothing imported without the user's confirmation.
- **Full migration scope:** carry over all data available from Koha or other source systems, including catalogue and holdings, patrons, circulation and financial history, and other exported records and settings. Validate supported mappings per source and preserve or explicitly report data without a matching Pusthakam field. Reconcile source and destination counts before cutover.

## Phase 1 work breakdown
1. Toolchain ready (Rust stable, MSVC Build Tools, WebView2 ✔ present).
2. `npm create tauri-app` style scaffold in place: Vite + React + TS, Tailwind, shadcn/ui init, Lucide, fontsource.
3. Rust: `paths`, `logging`, `error`, `db` (open + PRAGMAs + migration runner + 0001_core), `commands::app` (`app_info`, `get_settings`, `set_setting`).
4. UI shell: sidebar (grouped nav from brief), top bar with search/command palette, theme toggle (light/dark/system), error boundary, toasts, empty-state component. Modules not yet built render an honest "Not yet available in this version" state.
5. Tests: Rust (migration runner, PRAGMAs, settings roundtrip, checksum mismatch), Vitest (theme, nav, api wrapper), Playwright smoke against the Vite dev server with mocked IPC only for UI tests.
6. Verify restart persistence manually and by an integration test that opens/closes the DB twice.

## Requirement tracking
Requirements are tracked in `PROGRESS.md` with IDs (REQ-xxx) mapped to source files, tests and verification results.

## Startup presentation (implemented 20 November 2026)

Owner request (18 November 2026): when Pusthakam opens, display a startup loading screen for at least three seconds before showing the application. Implemented in `src/features/boot/StartupSplashScreen.tsx` and mounted in `src/features/boot/Gate.tsx`. Enforces a 3,000 ms branded splash presentation (brand mark, typography, progress bar, accessibility live region, and local sovereign badge) before transitioning into sign-in or the setup wizard. Verified with 4 unit tests in `StartupSplashScreen.test.tsx` (total 200 Vitest tests passing).

## Phase 4 calendar checkpoint

Holiday dates and recurring weekly closures are implemented in Library Settings. Issue and renewal use the same next-open-date rule. Schema 17, validation, permissions, audit rollback, persisted configuration and stale confirmations passed automated checks. Native Windows acceptance remains open; see LIBRARY_CALENDAR.md. Phase 3 real Koha acceptance remains blocked on a representative export, with native catalogue/printing checks recorded in PROGRESS.json.

## Planned report presentation enhancement — owner request, 20 November 2026

Phase 5 / REQ-RPT / RPT-FORMATS: reports must be polished and aesthetically beautiful by default, using relevant installed design and document-generation skills during development. Add an unchecked **Use simple report design** checkbox for a conventional library-management layout. Apply the choice to preview/PDF/print/styled XLSX, preserving identical data and totals; CSV remains data-only. See [REPORT_DESIGN_PLAN.md](REPORT_DESIGN_PLAN.md) for scope and visual acceptance. Planned only; no implementation credit. This does not reorder the active recovery milestone.

## Deferred public GitHub Pages website — owner request, 20 November 2026

Build later: attractive librarian-focused homepage with a clear path to trying Pusthakam, populated Blog and FAQ tabs, and a separate static/crawlable URL for every article. Include sitemap.xml, robots.txt (with correct host-root deployment), unique metadata/canonical URLs and appropriate structured data. Aim for useful search/AI-search discovery without guaranteeing rankings or citations. Use relevant installed design/writing/site skills. Authoritative scope: [GITHUB_PAGES_WEBSITE_PLAN.md](GITHUB_PAGES_WEBSITE_PLAN.md). Documentation only; no site or publishing implemented, no desktop weight/credit. This public project website is separate from Phase 7b OPAC and does not reorder the current recovery milestone.

## Splash quotations and logo refinement — owner request, 21 November 2026

The minimum-three-second splash and local quotation rotation are implemented. Twenty primary-text reviewed short entries are enabled, with remaining source/edition checks explicit in SPLASH_QUOTE_SOURCES.md; the target of approximately 100 authentic quotations remains incomplete. Owner approved the latest teal book/P on 23 November 2026 for use throughout. Exact master and reproducible icon tooling are in branding/; app, native icon family, all seven website pages, progress monitor and LAN views now use it. See SPLASH_QUOTES_PLAN.md, LOGO_DESIGN_DIRECTION.md and the latest handoff. No new roadmap weight/credit or change to the product release milestone.

## Owner-approved usability batch - 24 November 2026

Implemented setup full-height sidebar, native Windows PDF/XLSX report Save As, title and physical-copy selection in Circulation Issue, and the functional Barcode & Printing entry point (barcode/spine labels, member cards, current-loan slips). Fixed Phase 7b LAN listener ownership at desktop startup and settings changes; runtime status now reflects actual binding. Automated and browser evidence is in PROGRESS.json. Remote-device LAN access, native save-dialog walkthrough and physical printing remain owner acceptance checks. These fixes add no task weights and do not close macOS/Linux packaging smoke tests. Historical queued/paused notes are superseded by the current task and handoff.

## Next usability batch - collecting requests, 24 November 2026

Owner requests Find book by name in Circulation > Reservations, alongside existing member-name lookup. Collect further proposed changes before implementing this next batch. Preserve barcode/accession entry, catalogue permissions and reservation eligibility checks; title search must support reservable copies including copies currently on loan, rather than blindly reusing Issue-only copy filters. Documentation only; no code change or new verification credit. Previous five-change installer and outstanding owner acceptance checks remain as recorded below.

### Renew: select from current loans

Renew tab: show all currently issued/open loans, including overdue loans, so staff can select a loan for renewal without typing/scanning its barcode. Display book title, copy barcode/accession, member and due date; support search and pagination so all open loans remain accessible. Keep direct barcode entry and the existing renewal review/confirmation, eligibility rules and Rust permission checks. Selecting a row must not automatically renew. Queued for the next combined batch while the owner proposes further changes; documentation only.

### Return: select from current loans

Return tab: show all currently issued/open loans, including overdue loans, so staff can select a loan without typing its barcode. Display book title, copy barcode/accession, member and due date, with search and pagination. Keep direct barcode/scanner entry, existing return review/confirmation, fine assessment and Rust permissions; selecting a row must not record a return. Collect with Reservations book-name search and Renew current-loan list in the next batch; documentation only, implementation awaits the owner finishing their requests.

### LAN acceptance and deferred OPAC redesign - 24 November 2026

LAN acceptance update: owner can open OPAC on the computer with Pusthakam installed, but another computer on the same network reports ERR_CONNECTION_REFUSED. Remote tab shows an OPAC title; this alone does not prove a current successful connection (could reflect earlier/cached navigation). Cross-device access remains unresolved. Next diagnostic batch: verify the exact remote URL uses the host network address and configured port rather than loopback, compare host loopback versus network-address access, inspect active bind/listener and advertised adapter/address, then test remote TCP reachability and Windows firewall/network isolation. Do not assume firewall is the cause or change firewall rules automatically. Confirm which installer/build is running before drawing conclusions. OPAC redesign is explicitly deferred until the owner says to redesign; preserve current OPAC UI in this batch. Documentation only; continue collecting requests.

### Circulation items 1-3 authorized - 24 November 2026

Reservations title/copy search and searchable paginated Renew/Return open-loan lists are implemented and built: 228 frontend tests and 35 real IPC tests passed; owner walkthrough remains pending. This supersedes their earlier collecting/queued status. LAN diagnosis and OPAC redesign were not authorized in this batch. Product weights and platform release gates unchanged.

## Linux release tooling synchronized locally - 24 November 2026

Strict Debian package/asset/startup/restart/uninstall checks and regression tests are now in D:\Pusthakam, replacing the earlier cloud-only checkpoint. REL-LINUX remains implemented but unverified until target Ubuntu packaging and interactive setup/sign-in/restart acceptance pass. macOS, physical printing and cross-device Windows LAN checks remain separate. OPAC redesign remains deferred.

## V3 browser parity and internet hosting — owner-authorized expansion (25 November 2026)

Library PC by default plus optional always-on server; local and internet OPAC/full staff access. Sequence: command inventory and shared transport; independent request/session/import ownership; permission-preserving handlers and browser file adapters; shared /staff UI; headless host and HTTPS deployment profiles; full feature/role matrix, backup/restore host migration and real-device/internet acceptance. Track partial evidence in currentTask without inflating v1 weights. STAFF_BROWSER_PARITY.md is authoritative for this expansion.


## Deferred Malayalam version — owner request, 26 November 2026

Owner requests a Malayalam version/interface of Pusthakam and explicitly says NOT to start implementation now. Record only. Status: PLANNED / NOT STARTED. Begin only after a later explicit instruction. No application source, translations, language switch, dependency, build or installer changes have been made.

When authorized, agree localization coverage (desktop, browser staff, public OPAC and printed/exported material as applicable), language-selection behavior and Malayalam terminology. Preserve library records and reconcile the older English-only CLAUDE.md convention with this newer request. Existing English functionality and confirmed public-access test are unchanged. This future requirement adds no v1 weight or implementation/verification credit.


## Malayalam implementation resumed — 26 November 2026

Owner explicitly authorized Malayalam implementation, starting with an English-default Language / Malayalam dropdown on first-run setup. Implemented persisted per-device language choice, document lang, immediate translated setup labels/profile descriptions/validation/steps, preservation of entered data, and selected locale in existing setup API. No migration, provider, network requirement or catalogue-data translation. Focused SetupWizard tests 8/8 passed, including Malayalam submission and remount persistence; initial run 7/8 exposed untranslated Continue, fixed. Initial typecheck exposed optional step index, fixed; corrected typecheck and ESLint passed. First increment only: sign-in, application modules, public OPAC, backend errors and reports remain untranslated; bundled Malayalam font, rendered shaping/mobile checks, full translation coverage and native installer rebuild remain pending. Current installed/release exe has no Malayalam changes. Next: extend translation resources and selectors to sign-in/shared app navigation/settings, then feature screens and independent public OPAC; verify font/rendering offline. Owner also confirmed temporary internet-test launcher already quit; domain cleanup complete, do not resume DNS work. V1 weights unchanged.


### Owner translation workflow

Owner chose to provide Malayalam translations through Excel. Created docs/translations/Pusthakam_Malayalam_Translation.xlsx with 1641 unique source-derived frontend/public OPAC candidate phrases, stable phrase IDs, editable blank Malayalam/notes/status columns, screen context and source references; 47 existing setup drafts are separate for owner review. Instructions include one Malayalam example, preserve brace placeholders, translate in batches, note context-dependent meanings and assembled sentence fragments. Source inventory is docs/translations/english-review.json; reproducible scripts tools/export-translation-review.mjs and tools/create-translation-workbook.py. Workbook validated by reopening: row count, unique IDs, blank owner input cells, zero formulas; extractor ESLint passed. Initial openpyxl installs to user profile and D:/workspace were denied; successfully installed only in Windows temp and generated workbook. Saved downloadable workbook successfully. This is a candidate review inventory, not a claim of exhaustive runtime/backend/report coverage. Existing setup implementation unchanged. Next: accept owner-completed workbook, match stable IDs against snapshot/current source, review placeholders/ambiguous terms, then integrate approved translations; do not treat setup drafts as owner-approved. V1 scores unchanged; no native build, database edits, commit or push.


### Approved Malayalam terminology and simpler review

Owner approved all 20 proposed common English/Malayalam terms (reply: perfect). Saved docs/translations/approved-glossary.json as the terminology authority. Use everyday Malayalam and familiar borrowed terms; avoid obscure/formal jargon. Workflow now supersedes the large blank workbook assignment: assistant drafts translations using approved terms; owner reviews small contextual batches or screens and flags awkward wording. Do not require owner to translate all 1641 candidates. Approval applies to these 20 terms only, not all existing setup drafts or future translations. Next: draft remaining UI phrases consistently, handle context and placeholders, integrate incrementally with tests and rendering checks. First-run English default remains. No application code, workbook, database or installer changed in this approval checkpoint; v1 weights unchanged.


### Malayalam shared UI implementation checkpoint

Malayalam shared UI increment implemented: reusable language selector on sign-in/recovery and Library Settings; translated authentication/recovery text, desktop/mobile navigation, bilingual command search, theme labels, startup status/tagline, online-tools shell and settings title. Setup uses same translator, with owner-approved 20 terms overriding drafts. Credentials, records, query caches, routing and backend permission/command values preserved. 3 new translation tests passed after untranslated login submit fix; 39 affected setup/recovery/AppShell/splash/settings tests passed in 5 files. TypeScript/Vite build and full ESLint passed; git diff --check passed. Initial focused run 31/32 and initial lint BOM issue were corrected. Browser source preview tool added; both attempts failed before inspecting screens (Runtime.enable timeout then Page.navigate timeout). No screenshots/layout acceptance earned. Native installer unchanged. Remaining work: rendered Malayalam font/shaping and phone header/navigation checks; translate dashboard/catalogue/circulation/members and other modules, independent public OPAC, generated reports and backend errors. Existing setup drafts/new wording beyond approved glossary remain reviewable drafts. Keep English default and simple everyday wording; owner reviews small batches, never require all 1641 workbook rows. No new v1 weights/credit, owner-data changes, external actions, commit or push.


### Malayalam core screen checkpoint

Implemented the core Malayalam screen increment with 319 draft phrases, keeping approved 20-term glossary authoritative. Dashboard counts/alerts/date controls, catalogue search/results/details/record form/saved searches, circulation issue/return/renew plus member/book lookups/current/overdue lists, member list/registration/history, shared copy-status chips and pagination are localized. Named interpolation preserves all placeholders and treats inserted user data literally. Completion messages retain structured values and re-render in the selected language without repeating transactions. Tests: initial 67/67 across 9 affected files passed; after five new workflow/interpolation tests, 58/58 across 5 files passed; final member/translation subset 10/10 passed including added placeholder parity check. Initial final-build attempt found two new test-only unsupported exact options; removed, then TypeScript/Vite production build and full ESLint passed. jsdom emits its existing canvas-not-implemented notice; no canvas dependency added. git diff --check passed. No claim of complete module translation: copy dialogs, covers, fines/reservations, imports/exports/printing, remaining settings/modules, public OPAC and unmatched backend errors still need work. No browser layout/font/native checks in this increment; previous browser-debugger timeouts remain unresolved. Added docs/translations/README.md with coverage/boundaries. Next: remaining core dialogs and fine/reservation screens, then other modules/OPAC; bundle/test Malayalam font and phone layouts before a new installer. English stays default, user-entered data and backend API payloads unchanged. Progress totals unchanged; no commit/push, installer or owner-data changes.


### Owner pause: 319-draft review workbook delivered

2026-11-26T15:12:32.816Z

OWNER PAUSE: owner will return tomorrow and requested all necessary documents updated. Stop application implementation and design until explicit resume. Created and delivered docs/translations/Pusthakam_Malayalam_Drafts_319.xlsx, a separate correction-only workbook for exactly 319 current core draft resource entries, not the earlier 1641-candidate workbook. English/Malayalam pairs prefilled; editable correction/status/notes columns, filterable inferred screen context, hidden stable phrase IDs, instructions/example, separate 20-term approved glossary. Not reviewed/OK/Changed/Question statuses prevent blank corrections from implying approval. Partial reviews are welcome. Snapshot docs/translations/core-draft-review.json and reproducible tools/create-core-draft-review.mjs + tools/create-core-draft-workbook.py saved. Reopened XLSX and verified 319 exact pairs, unique IDs, 20 approved terms, empty correction cells, no formulas; placeholder parity and exporter ESLint passed. New workbook saved successfully for download; old workbook/user work preserved. No app source edits, new translations, native build, owner-data or external-domain changes in this pause operation. Existing core screen tests/build passed in previous checkpoint; full localization and rendering/font/mobile/native acceptance remain incomplete. Next on explicit resume: accept partially or fully reviewed XLSX, reconcile IDs against snapshot/current English, validate placeholders, review questions and apply only approved corrections. Do not make owner translate all phrases or infer unreviewed rows approved. Then continue remaining core dialogs/fines/reservations, other modules/OPAC, font/mobile rendering and installer. English default, everyday Malayalam, 20 approved terms and unchanged records/API contracts remain requirements. V1 totals unchanged; no commit/push performed.
