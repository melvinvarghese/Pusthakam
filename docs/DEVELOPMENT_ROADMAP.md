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
