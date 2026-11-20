# Product Specification — Pusthakam (പുസ്തകം)

*A Better Way to Manage Libraries.* Pusthakam is the working name; no trademark/domain clearance is claimed.

## Vision
A premium, fully offline desktop library management system (Windows, macOS, Linux) that a librarian can install, configure and use daily — for school, college, university, public, special, research, institutional and private libraries — from a single configurable core.

## Users and roles
Administrator · Librarian · Circulation Clerk · Viewer (built-in, permission-based; custom roles later).

## Non-negotiables
Offline operation (nothing requires network) · local SQLite in OS app-data · data integrity (transactions, FKs, no history deletion) · verified backups · local auth with argon2id · RBAC enforced in the backend · premium, calm, productivity-focused UI.

## Modules and requirement IDs
Each requirement is tracked in `PROGRESS.md`.

| ID | Module | Summary | Phase |
|---|---|---|---|
| REQ-DASH | Dashboard | Real-data KPIs (records, copies, available, issued, members, overdue, reservations), activity, growth, popular titles, alerts, date filters | 5 |
| REQ-CAT | Catalogue | Full bibliographic CRUD, multiple contributors/subjects, duplicate detection, advanced search, bulk ops, import/export | 3 |
| REQ-ACC | Accession & inventory | Per-copy accession/barcode/location/status; auto/manual numbering; donations, transfers, withdrawals, lost/damaged; stock verification | 3, 6 |
| REQ-CIRC | Circulation | Issue, return, renew, reserve, overdue, fines, waivers, lost/damaged, history; configurable policies by member × item category, holidays | 4 |
| REQ-MEM | Members | Categories, profile, photo, cards, barcodes, history, current loans, fines | 4 |
| REQ-ACQ | Acquisitions | Recommendations, requests, approvals, vendors, POs, invoices, budgets, receiving → records + copies | 6 |
| REQ-SER | Serials | Journals/magazines/newspapers, ISSN, frequency, expected/received/missing, renewals, spend | 6 |
| REQ-CLS | Classification & authority | DDC/UDC/LCC/custom schemes, subject headings, authors, publishers, controlled vocab; MARC21-ready | 3, 6 |
| REQ-RPT | Reports | Catalogue, accession register, available/issued/overdue, members, circulation stats, most borrowed, dept-wise, growth, expenditure, lost/damaged, stock verification, fines; PDF/CSV/XLSX; printable | 5 |
| REQ-PRN | Barcode & printing | Barcodes, labels, member cards, receipts, due slips, inventory sheets; USB scanner (keyboard wedge); OS printing | 3, 5 |
| REQ-ADM | Administration | Staff, roles, local auth, library config, policies, custom fields, audit log, backup/restore, import/export, preferences | 2, 7 |
| REQ-CFG | Multi-library configuration | Profiles, categories, collections, sections, locations, shelves, schemes, accession formats, policies, custom fields, optional modules | 2 |
| REQ-SRCH | Search | FTS5 + exact lookups across titles, authors, ISBN, accession, barcode, publisher, subject, classification, member ID/name; pagination, saved filters | 3, 4 |
| REQ-IMP | Import/export | CSV/XLSX/JSON and MARC21/ISO 2709/MARCXML; migration profiles for other systems starting with Koha (full available data and history, including catalogue + holdings, members, open and closed loans, reservations, and fines and settlements); staged wizard with mapping, validation, duplicate detection, summary; transactional. See roadmap "Migration from other systems" | 3, 4 |
| REQ-BAK | Backup/restore | Manual + scheduled backups, history, integrity checks, validation, pre-restore snapshot | 7 |
| REQ-NET | Multi-computer (LAN) | One Library Server computer holds the data; other computers connect over the local network with no fixed application client cap, each user signs in with their own account and everyone sees the same live data; no internet; encrypted, pairing-code based, server-side permissions and backups. See roadmap Phase 7b | 7b |
| REQ-SEC | Security | Argon2id, RBAC, restricted Tauri capabilities, CSP, safe file handling, audit trail | 1–7 |
| REQ-OFF | Offline | No network dependency; bundled assets/fonts | all |
| REQ-UI | Design system | Tokens, light/dark, keyboard shortcuts, accessible, consistent tables/dialogs/states | 1+ |

## UX principles
Search-first; keyboard-friendly (Ctrl/Cmd+K palette, scanner-ready fields); dense but calm layouts; honest empty/loading/error states; destructive actions confirmed; no decorative motion. Palette: deep charcoal, warm ivory, muted sand, sage green, soft grey.

## LAN browser extension (owner request 2026-11-18)

REQ-NET includes staff browser access from same-network phones, tablets and computers while the designated host PC runs Pusthakam. REQ-OPAC adds anonymous public catalogue search, public book details and availability on the same reachable LAN. Staff functions require server-validated staff sessions and permissions. OPAC exposes no member, borrower, internal-note, financial or administrative data and no write actions. See DEVELOPMENT_ROADMAP Phase 7b and LAN_BROWSER_OPAC.md. Browser access is planned; native mobile apps remain out of scope.

## Out of scope for v1
Cloud sync, online ISBN/metadata lookup, multi-site libraries, custom cryptography, mobile apps.

## Definition of done (release candidate)
See the master brief §19: installs, starts, configures, catalogues, circulates, reports, searches, imports/exports, backs up/restores, enforces permissions, passes automated tests, verified offline, packaged for each claimed platform, with user documentation.

## Report presentation requirement — 20 November 2026

REQ-RPT: generated reports should be professionally designed and aesthetically beautiful by default. An unchecked **Use simple report design** checkbox allows a conventional, minimal library-management report appearance. Preview and PDF/print/styled XLSX should honor the selected design, with identical records/calculations/totals in either mode. CSV has no visual layout. Relevant installed design/document/PDF/spreadsheet skills should guide implementation and visual review. This is a planned enhancement; see REPORT_DESIGN_PLAN.md for acceptance.

## Deferred public project website — 20 November 2026

Owner requests a GitHub Pages marketing/information site for librarians, separate from the library OPAC. Required: attractive homepage, populated blog and FAQ, individual static article URLs, clear release/try information, sitemap/robots and search/AI-discovery fundamentals. All public claims must reflect actual release status; no guaranteed indexing/rankings. Implementation and publication are deferred. See GITHUB_PAGES_WEBSITE_PLAN.md; no desktop completion credit.
