# Known Issues

| ID | Severity | Issue | Status |
|---|---|---|---|
| KI-001 | Blocker | Toolchain missing and low disk space on `C:`. Installed to `D:\dev` on 2026-11-14 with user approval. | RESOLVED |
| KI-002 | Low | `git` not on PATH. GitHub Desktop's bundled git works (path in CLAUDE.md). | Workaround |
| KI-003 | Info | Node v26 is very new; everything passes with it. | Monitoring |
| KI-004 | Medium | Commands were unauthenticated. Now every data command checks the signed-in user's permission in Rust (verified by IPC tests). Only `setup_status`, `complete_setup` (works once), `login`, `logout`, `current_session` and `app_info` work signed out. | RESOLVED 2026-11-15 |
| KI-005 | Low | App icon is a generated placeholder and does not match the Spine Label brand mark. Needs a final icon before release. | Open |
| KI-006 | Low | `npm audit` reports moderate advisories in dev dependencies; review before release. | Open |
| KI-007 | Low | `C:` has under 10 GB free; build output goes to D: via `CARGO_TARGET_DIR`. | Monitoring |
| KI-008 | Low | Without `?preview`, a plain browser shows "Open Pusthakam as an app" (backend only exists in the desktop app). Intended. | Accepted |
| KI-009 | Low | Vermilion class-8 label band sits near the red overdue colour; re-check when real catalogue data arrives. | Watch |
| KI-010 | Low | `cargo fmt`/`clippy`, Playwright tests and CI are not set up yet. | Open |
| KI-011 | Medium | Staff management and account unlock are implemented. Administrator recovery requires a one-use code generated while signed in. | Implemented |
| KI-012 | Low | Currency amounts assume 2 decimal places (minor units = 1/100). Currencies with other exponents (e.g. JPY) are not handled. | Open |
| KI-013 | Low | Account lockout is per account (5 failures, 5 minutes). There is no separate rate limit for unknown usernames; acceptable for a local desktop app. | Accepted |
| KI-014 | Info | Rust integration tests on Windows need a Common Controls manifest, embedded for test builds only by `src-tauri/build.rs`. | Documented |
| KI-015 | Medium | Koha import follows Koha's default MARC21 framework (952 subfields) as documented, but has not been tried on a real Koha export. A customised framework may use other subfields. The preview always shows what will be imported. | Open: needs a sample file |
| KI-016 | Low | Import supports ISO 2709 MARC (UTF-8) and CSV. MARCXML, JSON, XLSX and MARC-8 encoded files are not supported yet. Non-UTF-8 text is read as Latin-1 and flagged. | Open |
| KI-017 | Low | Publisher and author authority records, subject hierarchies and bulk edit/delete are not built (Phase 6). Subjects are flat headings; publishers are plain text. | Open |
| KI-018 | Low | Import has not been benchmarked on very large files; the whole file is held in memory until confirmed (limit 100 MB). | Open |
| KI-019 | Low | Imported locations, collections and item categories are named after the source codes (for example MAIN, BK); rename them in Library Settings. A code-to-name mapping step is not built. | Open |
| KI-020 | Low | The Accession Register supports batch label sheets on plain paper or full-sheet label stock. Preset die-cut layouts and physical printer and scanner checks remain. | Open |
| KI-021 | Low | Setup sidebar now fills the app viewport; reveal wrapper and wizard height corrected. All three steps checked in real Chrome at 1366x705 and 1024x480, with reachable navigation and no horizontal overflow. | RESOLVED 2026-11-24; Windows owner walkthrough pending |
| KI-022 | Low | PDF and XLSX reports now use native Windows Save As with cancellation, overwrite confirmation and saved-path feedback. Shared catalogue save helper retains permission, payload and private-data safeguards. | Implemented 2026-11-24; automated checks passed, native dialog walkthrough pending |
| KI-023 | Medium | Barcode & Printing now offers book barcode/spine label sheets, member cards and current-loan due-slip reprints, with individual permissions. Existing tools reused; route and print-content regressions pass. | Implemented 2026-11-24; physical printer/scanner checks and die-cut presets remain |
| KI-024 | Medium | Same-Wi-Fi remote OPAC connection remains pending real second-device acceptance. Development executable previously needed Vite at 127.0.0.1:1420; standalone Tauri packaging and explicit host-IP/OPAC URLs now address delivery/address ambiguity. Windows enumerates active adapters. Do not assume firewall failure without confirming current standalone build, host-local/network URLs and exact remote URL. | Open - external-device walkthrough pending |
| KI-025 | High | Windows accepted sockets inherited the nonblocking listener mode, causing immediate 408 or connection reset when headers/body arrived separately. A 100 ms delayed raw request reproduced it. Request streams now explicitly use blocking mode with timeouts; delayed-header/body regression passes. | RESOLVED 2026-11-25; gateway regression and 3 exact-release delayed-packet/restore runs passed |

KI-024 live diagnosis 25 November 2026: host all-interface listener and both portals respond200. Active trusted Wi-Fi is still classified Public; explicit Public TCP Block matches the installed running executable, and its Allow rule is Private only. Administrator launch failed0xc0000142. Set trusted connection Private through Windows Settings and recheck actual second device; not yet resolved.

KI-024 update 2026-11-25T12:14:58.549Z: owner explicitly confirmed the full OPAC URL with port4318 works on an actual second same-network device. OPAC reachability is confirmed by owner; staff login, capacity and outside-network access are not inferred. Earlier firewall/profile diagnoses describe prior observations and must not be treated as current failure evidence.

KI-024 owner confirmation 2026-11-25T12:17:30.108Z: both OPAC and staff now work on another device on the same network. Original same-network reachability issue resolved by owner observation. Public deployment, capacity and exhaustive workflow checks remain separate.

## KI-026: mobile browser usability

Owner reported OPAC and staff were not optimized on phones. The fresh compiled app now passes115 real-server Edge touch-emulation checks at320/390/768px: all14 staff sections/internal tabs, focus restoration, populated records/dialogs, sign-in, public OPAC and backups. Review found and corrected phone autofocus, backdrop focus loss, Classification/Periodicals filters, tab/toolbar overflow and page padding. Full frontend suite252 passed before backdrop correction; subsequent full run had2 loading-deadline failures, both resolved by an unchanged focused27-test rerun including navigation regressions. TypeScript/Vite/debug build and ESLint pass. Fresh NSIS package built; exact release115 mobile/browser and11 headless restore/delayed-packet checks pass. Physical phone, Safari and real soft-keyboard acceptance remain. See MOBILE_BROWSER_ACCEPTANCE.md; the installed application has not been replaced automatically.
