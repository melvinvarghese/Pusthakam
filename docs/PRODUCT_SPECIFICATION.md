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
| REQ-IMP | Import/export | CSV/XLSX/JSON and MARC21/ISO 2709/MARCXML; migration profiles for other systems starting with Koha (catalogue + holdings, members, current loans, fines); staged wizard with mapping, validation, duplicate detection, summary; transactional. See roadmap "Migration from other systems" | 3, 4 |
| REQ-BAK | Backup/restore | Manual + scheduled backups, history, integrity checks, validation, pre-restore snapshot | 7 |
| REQ-NET | Multi-computer (LAN) | One Library Server computer holds the data; other computers connect over the local network, each user signs in with their own account and everyone sees the same live data; no internet; encrypted, pairing-code based, server-side permissions and backups. See roadmap Phase 7b | 7b |
| REQ-SEC | Security | Argon2id, RBAC, restricted Tauri capabilities, CSP, safe file handling, audit trail | 1–7 |
| REQ-OFF | Offline | No network dependency; bundled assets/fonts | all |
| REQ-UI | Design system | Tokens, light/dark, keyboard shortcuts, accessible, consistent tables/dialogs/states | 1+ |

## UX principles
Search-first; keyboard-friendly (Ctrl/Cmd+K palette, scanner-ready fields); dense but calm layouts; honest empty/loading/error states; destructive actions confirmed; no decorative motion. Palette: deep charcoal, warm ivory, muted sand, sage green, soft grey.

## Out of scope for v1
Cloud sync, online ISBN/metadata lookup, multi-site libraries, custom cryptography, mobile apps.

## Definition of done (release candidate)
See the master brief §19: installs, starts, configures, catalogues, circulates, reports, searches, imports/exports, backs up/restores, enforces permissions, passes automated tests, verified offline, packaged for each claimed platform, with user documentation.
