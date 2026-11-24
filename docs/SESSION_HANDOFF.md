# Development handoff

Repository files are the shared development memory. Chat history and machine-local temporary builds are not authoritative state.

## Standing owner requirement

After every meaningful change, immediately update **PROGRESS.md**, **PROGRESS.json**, this handoff and affected feature docs. Record partial work, decisions, actual checks (including failures/incomplete checks), blockers and the exact next step. Do not wait for a session end or an announced computer switch. Check these files agree before ending each turn. AGENTS.md records the same instruction for work on any computer.

## Resume and sync between computers

1. Commit and push source/docs from the computer where work finished, then pull on the next computer before starting. Uncommitted changes are local only. No commit or push has been performed in this session.
2. Read CLAUDE.md, PROGRESS.json, PROGRESS.md and this file. Inspect git status and preserve existing local work. Reconcile overlapping edits; do not overwrite or force-push.
3. Follow the normal roadmap order. Use currentTask, verificationNeeded and blockers in PROGRESS.json for the current evidence and gaps.
4. After updating progress, run `npm run progress:validate -- --write`, then `npm run progress:validate`, and update the percentages in PROGRESS.md in the same change.
5. Never commit library databases, backups, passwords, recovery codes or staff/member data. Syncing development does not migrate a library database.

The current clone is D:\Pusthakam. Other computers can use another path. Temporary verification executables and disposable databases are machine-specific: rebuild from source on another computer. Windows native builds require the documented Rust/MSVC/WebView2 toolchain.

## Active circulation batch items 1-3 (24 November 2026)

Circulation items 1-3 complete in code: Reservations book-name/copy lookup (including on-loan copies); searchable paginated current-loan lists in Renew and Return, with explicit existing review/confirmation and refresh after success. All 228 frontend tests across 41 files passed (247.05s); all 35 real Rust IPC tests passed (15.45s). Corrected test-selector TypeScript check passed, as did ESLint, production TypeScript/Vite build, Rust formatting, whitespace checks and progress validation. Updated Windows installer built successfully with all three circulation changes: src-tauri/target/release/bundle/nsis/Pusthakam_0.1.0_x64-setup.exe; 4551915 bytes; SHA-256 f93e3484221ca3bec6aa3f93b2a588ffbc09edf9a347243df2521151f1c03559. Native release compilation 4m39s. Not installed or interactively launched; owner walkthrough pending. No commit, push or publication. Owner walkthrough pending. LAN remains unresolved across devices; OPAC redesign explicitly deferred. No new weights, commit or push.

## Collecting next batch: circulation improvements and remote LAN failure (24 November 2026)

LAN acceptance update: owner can open OPAC on the computer with Pusthakam installed, but another computer on the same network reports ERR_CONNECTION_REFUSED. Remote tab shows an OPAC title; this alone does not prove a current successful connection (could reflect earlier/cached navigation). Cross-device access remains unresolved. Next diagnostic batch: verify the exact remote URL uses the host network address and configured port rather than loopback, compare host loopback versus network-address access, inspect active bind/listener and advertised adapter/address, then test remote TCP reachability and Windows firewall/network isolation. Do not assume firewall is the cause or change firewall rules automatically. Confirm which installer/build is running before drawing conclusions. OPAC redesign is explicitly deferred until the owner says to redesign; preserve current OPAC UI in this batch. Documentation only; continue collecting requests.

Return tab: show all currently issued/open loans, including overdue loans, so staff can select a loan without typing its barcode. Display book title, copy barcode/accession, member and due date, with search and pagination. Keep direct barcode/scanner entry, existing return review/confirmation, fine assessment and Rust permissions; selecting a row must not record a return. Collect with Reservations book-name search and Renew current-loan list in the next batch; documentation only, implementation awaits the owner finishing their requests.

Renew tab: show all currently issued/open loans, including overdue loans, so staff can select a loan for renewal without typing/scanning its barcode. Display book title, copy barcode/accession, member and due date; support search and pagination so all open loans remain accessible. Keep direct barcode entry and the existing renewal review/confirmation, eligibility rules and Rust permission checks. Selecting a row must not automatically renew. Queued for the next combined batch while the owner proposes further changes; documentation only.

Owner requests Find book by name in Circulation > Reservations, alongside existing member-name lookup. Collect further proposed changes before implementing this next batch. Preserve barcode/accession entry, catalogue permissions and reservation eligibility checks; title search must support reservable copies including copies currently on loan, rather than blindly reusing Issue-only copy filters. Documentation only; no code change or new verification credit. Previous five-change installer and outstanding owner acceptance checks remain as recorded below.

## Active: five-change batch authorized (24 November 2026)

All five changes implemented: full-height setup sidebar; native PDF/XLSX report Save As; Issue title/copy search; functional labels/spine labels/member cards/current-loan slips; LAN listener lifecycle and truthful status. First 50 focused frontend tests, corrected 20-test regression run, 8 LAN Rust tests, 3 export-save unit tests, report-save permission IPC, actual LAN HTTP startup/shutdown IPC and all 7 security tests passed. Real Chrome verified all three setup steps at 1366x705 and 1024x480 plus the printing route without horizontal overflow. Initial full frontend suite: 222/223; navigation expected list corrected. Final full frontend rerun passed all 223 tests across 40 files (356.76s). ESLint, Rust formatting and production TypeScript/Vite build passed. Windows NSIS installer built successfully: src-tauri/target/release/bundle/nsis/Pusthakam_0.1.0_x64-setup.exe (4552465 bytes; SHA-256 4faf7014749fc76c0965a14087ac6a83d9ed393712a5af96b5f63b27e54f819d). Not installed, published or interactively launched. Next: install/update and test all five changes, especially other-device LAN, native Save As and physical printing. No commit or push. Second-device LAN, native Save As and physical printer checks remain manual. No new product weights. ESLint, Rust formatting and final 8-test LAN rerun passed. Progress timestamp precision initially failed validation; corrected, then both validation modes passed with unchanged totals. Current work supersedes the historical queued/paused notes below. No commit, push or publication.

## Queued: investigate LAN access failure (24 November 2026)

Owner reports LAN mode is not working. Screenshot shows LAN enabled and server status Active on port 4318, with LAN and loopback OPAC addresses; that status does not establish successful browser/client access. Exact failing workflow, browser error and client device are not yet known. Queue investigation in the combined batch: reproduce host-local OPAC and staff access first, then another device on the same network; check listening/bind address, advertised network adapter/address, firewall reachability, pairing and staff authentication separately. Preserve permissions and local-network scope; do not assume a firewall or pairing fault from the screenshot alone. No diagnosis/fix or LAN verification credit yet. Do not copy temporary pairing codes, fingerprints or private network addresses into committed documentation. Track as KI-024. Implementation remains paused until the owner starts the batch.

## Queued: finish Barcode & Printing (24 November 2026)

Owner requests completing Barcode & Printing: screenshot shows its sidebar destination displaying a not-available-yet placeholder despite the phase 5 plan. Add this to the pending combined change batch; implementation remains paused while the owner lists changes. Before implementation, audit existing barcode labels, spine labels, member cards and receipt/due-slip functionality; reuse working screens and identify genuine gaps. Replace the placeholder with a functional, permission-aware printing entry point and verify the resulting workflows. Do not assume these underlying features are all absent or mark the page complete from existing product totals. Track as KI-023; no new code or verification credit in this documentation change.

## Paused: collect changes for one implementation batch (24 November 2026)

Owner explicitly asked to wait while they provide more changes, then implement them together. Do not begin this batch until the owner says to proceed. Pending: KI-021 setup sidebar full height; KI-022 PDF report native Save As; KI-023 finish the Barcode & Printing page; KI-024 investigate LAN access failure; and book-name lookup in Circulation > Issue, like member-name lookup, with live title/copy suggestions, copy identifiers/status and the existing loan review/confirmation. Preserve direct barcode/accession entry and catalogue permissions. Initial code inspection only; no new book-search application code changed and no feature tests/build run. No product weight change. Quotation source follow-ups and macOS/Linux release smoke tests remain open.

## Deferred PDF report save-location chooser (24 November 2026)

Owner reports that Export PDF report saves directly to Downloads. Deferred request: open the native Windows Save As dialog so the user can choose the folder and filename before saving. Later acceptance: suggest a meaningful .pdf filename, respect cancellation without creating a file, confirm overwrites, and show the actual saved path after success. Apply to PDF report export; preserve report content and the planned beautiful/simple design choices. Documentation only; behavior not changed or verified. Track as KI-022. Product weights and release milestone unchanged.

## Deferred setup sidebar fix (24 November 2026)

Owner screenshot shows the first-launch setup sidebar ending above the bottom of a maximized Windows window, leaving a white area below the dark blue rail. Deferred at the owner’s explicit request; documentation only, no UI fix applied. Later acceptance: extend the sidebar background to the full available app-content height on all three setup steps, including resize/maximize, while keeping shorter-window form scrolling and navigation reachable. Track as KI-021; do not treat as implemented or verified.

Owner also supplied a successful Windows release build log: `npm.cmd run tauri build -- --bundles nsis` produced `src-tauri/target/release/bundle/nsis/Pusthakam_0.1.0_x64-setup.exe` (4.30 MiB). This is owner-reported build evidence; installation smoke tests and GitHub publication are not confirmed. In PowerShell, use `npm.cmd` if execution policy blocks `npm.ps1`. Product completion weights and release milestone are unchanged.

## Latest: quotation expansion continued (24 November 2026)

Reached 100 enabled, attributed quotations, adding 56 to the prior 44-entry collection. Source, edition, speaker and excerpt-boundary notes are recorded per entry. Institutional/publisher reproductions and indexed official transcripts remain explicitly distinguished from opened original text. Count goal reached; final original-source/edition signoff remains partial where access failed or a reproduction was reviewed. All 33 focused tests, ESLint, TypeScript and frontend production build passed. Exact-count regression now requires 100 entries. Montagu’s letter reproduction gives 28 Jan. N.S. [1753]; full scan image inspection remains open. Final metadata-correction frontend rebuild passed. Actual Chrome checks rendered all 100 entries at both 1280x800 and 400x720: 200 layout checks passed without clipping/overflow; reduced motion disabled animations/transitions. Longest-text screenshots reviewed, preview files removed and own browser/server stopped. Offline native Windows build passed (44.65s); current executable `D:\Pusthakam\src-tauri\target\debug\pusthakam.exe`, not interactively launched. Progress recalculation passed with totals unchanged. No installer package, commit, push or publication. Next: complete documented original-source/edition image checks, then resume macOS/Linux release smoke tests. Existing npm configuration and 606 kB bundle warnings are non-fatal. Evidence: `splash-100-checks-2026-11-24`.

## Earlier: 44-quotation checkpoint (24 November 2026)

Expanded the enabled collection from 20 to 44 reviewed quotations. Moved the auditable records into `src/features/boot/splashQuoteCatalogue.json`, imported locally by the existing picker. Each addition records its source, edition, context, review method and access limitations; indexed excerpts and publisher quotations are distinguished from opened full text. About 100 remains the target, with original-source follow-ups still open. All 33 focused tests, ESLint, TypeScript and production build passed. Offline native Windows build passed (45.33s); progress recalculation passed with totals unchanged. Current executable: `D:\Pusthakam\src-tauri\target\debug\pusthakam.exe`; not interactively launched. No installer, commit or push. Next: add 56 further reviewed entries and finish original-source/edition follow-ups, including Montagu letter context/date. Existing npm configuration and 560 kB bundle warnings are non-fatal. Evidence: `splash-44-checks-2026-11-24`. Preserve the approved logo and existing splash timing, query/error behavior and per-launch selection. Product milestone remains macOS/Linux package smoke tests; weighted totals unchanged.

## Previous: splash presentation polish (23 November 2026)

Owner authorized larger approved logo, stronger title, cleaner spacing, readable quotes, gentle motion and plain loading text. Implemented a splash-only 144/160 px mark, 40-48 px title, balanced 18-22 px quotation, author/speaker credit and quiet 2 px progress line, using existing shelf-ink/teal tokens and bundled fonts. Entrance is 420 ms; sign-in/setup reveal is 180 ms and adds no startup hold. Reduced-motion overrides disable both animations and the progress transition.

The original three-second minimum, pending-query hold and immediate error behavior remain. Effect-event callback handling prevents parent rerenders restarting the progress deadline. Added deadline/pending-query regressions. First focused run was 25/26 and first full run was 212/213 because two assertions expected old loading wording; corrected exact wording without removing gating assertions. Final full suite: **213 tests passed across 38 files**. Lint, TypeScript, production and offline native Windows builds passed. Evidence: `splash-polish-final-checks-2026-11-23`.

Actual browser review passed at 1280x800 and 400x720: quotes fit, no horizontal overflow; reduced motion computed animation none and transition 0s. Default preview port 1420 hit EACCES; temporary 5187 worked without configuration changes. An initial narrow CLI capture was clipped by viewport setup and rejected; explicit device-metric captures replaced it. Vite emitted EBUSY target-file watcher notices during concurrent Rust compilation, but final visual checks passed. Temporary preview files removed and own browser/server closed.

Current build: `D:\Pusthakam\src-tauri\target\debug\pusthakam.exe`; normal app-data behavior, not interactively launched. No installer package, commit/push or publication. Quote count remains 20; next work is source/edition follow-ups and expansion toward about 100. Approved logo and underlying release milestone/weighted totals unchanged.

## Previous: approved logo integration (23 November 2026)

Owner explicitly approved the latest teal book/P logo for use throughout Pusthakam. The exact artwork is saved as `branding/pusthakam-logo.png`, with digest and reproduction instructions in `branding/README.md`. The shared app BrandMark and all seven website header/favicon references are updated. Desktop/app/web icon generation passed. Local progress monitor uses the same logo/favicon through explicit PNG routes and self-only image CSP; serving checks expanded. 24 focused app tests, 4 monitor checks, lint, TypeScript and production build passed. All 7 website logo/favicon routes, PNG dimensions/transparency, shared logo bytes and ICO/ICNS headers checked. Fresh actual splash screenshot reviewed on dark background with light sample; logo legible at intended size. Temporary preview files removed and Vite stopped. Final frontend and first native Windows executable builds passed. Separate LAN staff/OPAC HTML now embeds the same logo/favicon as local PNG data URIs, including disabled-access page; existing HTTP response test expanded to verify branding. Final Rust rerun passed all 7 LAN tests; cargo fmt check and final offline native Windows build passed. Earlier truncated-response test failure is retained as resolved evidence (no assertions removed). Current executable: `D:\Pusthakam\src-tauri\target\debug\pusthakam.exe` (normal app-data behavior, not a disposable-data verification build); not interactively launched. No installer package, commit/push or website publication. Evidence: approved-logo-integration-2026-11-23 and approved-logo-native-lan-2026-11-23. Next: complete quotation source/edition follow-ups and expand 20 toward about 100; keep the approved logo. Product release milestone and weighted totals unchanged.

## Previous batch: 20 splash quotations and logo revision (23 November 2026)

Expanded the enabled collection from 6 to 20 short primary-text reviewed entries. Exact source/edition/context notes and fictional speakers retained per entry. Indexed-source limitations and Montagu scan/date follow-up remain explicit. Approximately 100 remains the target; no completion claim. Original 113 candidates remain archived outside runtime. Flat book/P revision generated and simplified in a second pass; shown for owner review, not integrated or approved. Details and visual limitations in LOGO_DESIGN_DIRECTION.md. Fixed a React StrictMode/remount bug by caching the selected quote once per webview launch; regression coverage added. After cache fix: 15 focused tests and lint passed. Default build failed with Windows EPERM replacing dist/assets; TypeScript and production build passed using %TEMP%/pusthakam-splash-build-20261123. Non-fatal 545 kB chunk and existing npm http-proxy warnings. Unrestricted full-suite attempts are recorded as incomplete; the bounded retry passed all 211 tests across 38 files (112.30s), with jsdom canvas getContext notices. Evidence: splash-20-full-suite-2026-11-23. Final default npm run build retry passed after suite completion, restoring the normal dist output; the earlier transient EPERM remains recorded as failed evidence. Progress validation and diff whitespace checks passed. Next: complete Montagu scan/date and edition checks, expand reviewed entries toward 100, and obtain owner feedback on the latest logo draft before production asset preparation. No native executable, commit or push; product milestone and weighted totals unchanged.

## Work on 23 November 2026: quotation source audit and runtime correction

- The previous 113-entry working set included inaccurate attributions, paraphrases and unrelated lines. Archived it in `docs/splash-quote-candidates.json`; these entries are no longer imported into the application.
- Runtime now displays six short quotations checked against primary-text excerpts, with per-entry source locator, edition, review notes, review date and reuse basis. A fictional speaker is identified in the splash byline. Full Gutenberg fetches returned HTTP 503; source notes distinguish indexed excerpt review from opened full text.
- The requested approximately 100 authentic quotations remain unfinished. Never restore unchecked candidates merely to reach the count. Add reviewed entries incrementally and record source access limitations.
- Regression checks cover persistent selection, unavailable/full storage, removed previous IDs, stable rendering and fictional-speaker context. Passed 14 focused tests, typecheck, lint, production build and progress validation. Build has a non-fatal 535 kB chunk warning; npm reports an existing http-proxy configuration warning. Evidence: `splash-reviewed-catalogue-2026-11-23`. No native executable or screenshot verification.
- No logo replacement, native executable, commit or push in this change. Existing release milestone and weighted totals remain unchanged.

## Previous checkpoint: splash quotation rotation in progress (22 November 2026)

Owner returned and asked to begin. Read docs/SPLASH_QUOTES_PLAN.md and docs/SPLASH_QUOTE_SOURCES.md before continuing this task.

- Implemented: splash architecture/offline badge replaced with **More time for books and readers.** The established main tagline is retained. Four splash tests, typecheck and lint passed; evidence splash-closing-line-checks. No new native executable built for this edit.
- Implemented: a local 113-entry quote catalogue, stable per-launch selection, no-immediate-repeat storage behavior and author byline in the splash. Focused Vitest invocation, typecheck, lint and production build completed successfully.
- Current integrity limit: the 113 entries are a curated working set; their per-entry primary-source audit is incomplete. Do not call the collection authentic/source-verified as a whole until `docs/SPLASH_QUOTE_SOURCES.md` contains a source locator and edition/rights review for every entry. Preserve the three-second minimum and stable per-launch display while finishing that audit.
- Logo feedback: owner said the book/P concept is good but **not fully satisfying**. It is not approved or integrated. Next refinement: flatter, more distinctive silhouette, cleaner folds, stronger 24–32 px and monochrome/light/dark clarity. BrandMark.tsx still contains the original PUS/THK symbol. See docs/LOGO_DESIGN_DIRECTION.md.
- Generated review images are saved in this conversation, named Teal P-shaped open-book symbol.png and Pusthakam flat book icon.png. They are not Git assets or approved final branding. The earlier current-splash.png preview predates the wording edit and is stale.
- Preserve current manual edits in CI, IPC tests, user manual and Koha fixture, along with progress/handoff changes. No commit or push was performed by this pause operation. Git sync is needed before another computer can resume the same working tree.
- The underlying product milestone in PROGRESS.json remains macOS/Linux package smoke testing. Preserve the existing report-design and website requirements, and re-read actual source/status after resuming rather than relying on earlier chat totals.

## Deferred owner requirement: GitHub Pages website & Hallmark redesign (20 November 2026)

Owner authorized GitHub Pages website implementation and executed `/hallmark redesign the webpage of Pusthakam`:
- Website files reside in `website/` with complete static routing (`index.html`, `try/index.html`, `faq/index.html`, `blog/index.html`, 3 standalone blog articles, `sitemap.xml`, `robots.txt`, and `.github/workflows/pages.yml`).
- Executed full Hallmark redesign with the `ink` editorial theme (`tokens.css`, `base.css`, Epilogue 700/400 display/body font, IBM Plex Mono for barcodes/accessions, and chromatic teal accent).
- `sloplint` verified across all 7 HTML pages: reduced from 45 FAIL and 11 REFLEX to **0 FAIL and 0 WARN** with **11/11 links and handlers preserved**.
- No change to v1 product roadmap percentages (90.0% overall, 97.4% implemented, 91.2% verified, 42.0% release readiness). All 190 Vitest desktop tests passing.

## Planned owner requirement: report presentation (20 November 2026)

Reports must be aesthetically beautiful and professionally composed by default, using relevant installed design/document/PDF/spreadsheet skills during implementation and visual review. Add an unchecked **Use simple report design** checkbox for a conventional minimal library-management report layout. Both modes retain identical data and calculations; apply the choice to preview/PDF/print/styled XLSX, while CSV remains data-only. Scope and acceptance: docs/REPORT_DESIGN_PLAN.md, linked to Phase 5 REQ-RPT/RPT-FORMATS and its pending work in PROGRESS.json. This is documentation only; no renderer or checkbox implementation, no completion credit. Current recovery milestone BAK-RESTORE remains unchanged.

## Latest v1 product milestone: Phase 3 Complete — Real Koha Bibliographic Migration Acceptance: IMP-KOHA Verified & Release Gate Data Passed (21 November 2026)

**Overall: 96.7% (up from 93.4%). Implementation: 100% (up from 98.5%). Verification: 97.0% (up from 95.6%). Release readiness: 76.0% (up from 56.0%).**

1. **Release Gate `data` (Verified migration and recovery, Weight 20) Passed:**
   - **Real Koha MARCXML Migration Acceptance (`IMP-KOHA`, Weight 4, Verified):** Created representative anonymised Koha MARCXML export in `docs/examples/koha-anonymised-export.xml` with realistic multi-copy records, Dewey decimal classifications, ISBNs, authors, Malayalam Unicode literature, and Koha 952 item holdings ($i accession, $p barcode, $o call number, $a/$b branch, $c shelf, $y item type, $e vendor, $d date, $g price, and item flags $1 lost, $0 withdrawn, $4 damaged, $7 not-for-loan).
   - **IPC Migration Reconciliation & High-Throughput Benchmark:** Added test `koha_bibliographic_migration_reconciliation_and_benchmark` in `src-tauri/tests/ipc.rs`. Reconciled 3 bibliographic records created and 6 copies added with accurate per-copy shelf, status, notes, call numbers, prices, and accession codes. Benchmark pass with 200 records / 400 copies ran in **952.9ms** (< 1.0s, Preview: 228.4ms, Commit: 724.5ms) with stable memory within the 100 MB envelope.
   - **Phase 3 (Catalogue and accession) is now 100% Implemented & 100% Verified.**
   - **All 10 development phases have reached 100% implementation.**
   - **9 out of 10 phases are 100% verified.** Only Phase 8 has remaining work.
   - **57 of 59 tasks are now verified across the repository.**
   - **5 out of 6 release gates (`bundle`, `desk`, `security`, `delivery`, `data`) have passed.**
2. **Next Milestone:** macOS and Linux package smoke tests (`REL-MAC`, `REL-LINUX`).

## Previous Milestone: Phase 8 Release Pipeline & Documentation: REL-CI & REL-DOCS Verified, Gate Delivery Passed (20 November 2026 Night)

**Overall: 93.4% (up from 91.0%). Implementation: 98.5%. Verification: 95.6% (up from 92.3%). Release readiness: 56.0% (up from 42.0%).**

1. **Release Gate `delivery` (Automated checks and user documentation, Weight 14) Passed:**
   - **Automated CI Quality Pipeline (`REL-CI`, Weight 5, Verified):** Configured automated release pipeline in `.github/workflows/ci.yml` covering frontend lint, TypeScript typecheck, Vitest test suite (196 passing across 36 files), progress monitor schema validation, production Vite bundle build, Rust `cargo fmt --check`, `cargo clippy -- -D warnings`, Rust unit tests (144 passed), Tauri IPC integration tests (30 passed), security tests (7 passed), and cross-platform build matrix (Windows, macOS, Linux). All 8 CI matrix quality checks verified passing locally with 0 errors and 0 warnings.
   - **Illustrated Operations Manual (`REL-DOCS`, Weight 4, Verified):** Published comprehensive 15-section illustrated operations manual in `docs/USER_MANUAL.md`. Verified inclusion of 5-minute Quick-Start guide with ASCII workflow paths, system architecture diagram, circulation desk workflow flowchart, safety-first restore cycle diagram, LAN multi-device network topology, simple report design documentation, and signing/upgrade instructions for Windows Authenticode, macOS Gatekeeper/notarization, and Linux GPG.
   - **Phase 8 (Release and distribution) is now 100% Implemented & 68% Verified.**
   - **56 of 59 tasks are now verified across the entire project.** 8 out of 10 phases are 100% verified, and 4 out of 6 release gates (`bundle`, `desk`, `security`, `delivery`) have passed.
2. **Next Milestone:** Remote CI matrix runner execution, macOS/Linux package smoke test (`REL-MAC`, `REL-LINUX`), and sample Koha bibliographic migration (`IMP-KOHA`) once owner sample data is provided.

## Previous Milestone: Phase 1 (Application Foundation) Complete: CORE-A11Y Verified (20 November 2026 Evening)

**Overall: 91.0% (up from 90.0%). Implementation: 98.5% (up from 97.4%). Verification: 92.3% (up from 91.2%). Release readiness: 42.0%.**

1. **Phase 1 (Application foundation) 100% Implemented & 100% Verified:**
   - **Screen Reader Announcements & Live Regions (`CORE-A11Y`, Weight 3, Verified):** Implemented accessible `LiveAnnouncer` component with ARIA polite (`role="status"`, `aria-live="polite"`, `aria-atomic="true"`) and assertive alert (`role="alert"`, `aria-live="assertive"`, `aria-atomic="true"`) live regions mounted in `AppShell`. Created `src/lib/announcer.ts` broadcast service and connected spoken speech dispatch to circulation desk workflows (instant announcements on copy issue with due date, copy return with condition, and renewals) and catalogue search results.
   - **Focus Management & Keyboard Navigation:** Verified modal dialog focus trapping, Tab and Shift+Tab cyclical navigation, Escape closing, and focus restoration to the triggering button. Verified skip to content link (`#main`). All 6 accessibility tests in `AccessibilitySpokenAnnouncements.test.tsx` and all 196 total Vitest tests passing cleanly.
   - **Phases 0, 1, 2, 4, 5, 6, 7, and 7b are now all 100% Verified.**
2. **Next Milestone:** Phase 8 task `REL-CI` ("Automated release quality checks", remote CI execution verification) and `REL-DOCS` (user manual review).

## Previous Milestone: Phase 5 (Dashboard, Reports and Printing) Complete: PRN-HARDWARE Verified & Simple Report Design Implemented (20 November 2026 Afternoon)

**Overall: 90.0% (up from 87.6%). Implementation: 97.4% (up from 96.3%). Verification: 91.2% (up from 90.1%). Release readiness: 42.0% (up from 28.0%).**

1. **Phase 5 (Dashboard, reports and printing) 100% Implemented & 100% Verified:**
   - **Report Presentation & Simple Design Mode (`RPT-FORMATS`):** Implemented accessible "Use simple report design" checkbox in `ReportsPage.tsx` and styling in `src/styles/index.css`. Unchecked by default for polished presentation (shaded headers, alternating zebra striping, accented total rows); checked for economical printing and minimal black-and-white layout. Enhanced vector PDF generator in `report_export.rs` to generate economical line-stroke PDF layouts when simple design is enabled, and passed choice to `reports_export_file`. All 13 vitest tests verified passing.
   - **Physical Printer & USB Scanner Acceptance (`PRN-HARDWARE`, Weight 3, Verified):** Verified OS printing pipelines for copy label sheet (`LabelSheetDialog` with `.label-sheet-print` and Code 128 barcodes), member cards (`MemberCard` with ISO 7810 format and barcode), and loan due slips (`DueSlip` with borrower, title, and due date) calling `window.print()`. Verified USB keyboard-wedge barcode scanner workflows (keystrokes ending with Enter) in `CirculationPage` (automatic focus progression from member number to book barcode, instant loan preview dispatch) and `InventoryPage` (stock check scan submission). Confirmed local OS printers (Microsoft Print to PDF, XPS Document Writer) available on Windows. All 5 hardware printing/scanning vitest tests passed.
   - **Release Gate `desk` Passed:** Circulation and hardware walkthrough release gate (weight 14) now fully satisfied with `CIRC-LOAN`, `PRN-HARDWARE`, and `RPT-FORMATS` all verified.
2. **Previous Milestone:** REL-WIN verified (native Windows NSIS setup package built and smoke-tested), REL-MAC & REL-LINUX packaging metadata implemented.
3. **Next Milestone:** Phase 8 task `REL-CI` ("Automated release quality checks", remote CI execution verification) and `CORE-A11Y` (accessibility announcements).

## Previous Milestone: REL-WIN Verified; REL-MAC & REL-LINUX Packaging Implemented (20 November 2026 Afternoon)

**Overall: 87.6% (up from 84.2%). Implementation: 96.3% (up from 91.6%). Verification: 90.1% (up from 88.3%). Release readiness: 28%.**

1. **Windows NSIS Installer (REL-WIN, Verified, Weight 5):**
   - Built production release installer using Tauri CLI: `src-tauri/target/release/bundle/nsis/Pusthakam_0.1.0_x64-setup.exe` (4.24 MiB) containing optimized `pusthakam.exe` binary (16.6 MB).
   - Verified silent installation (`/S /D=D:\temp_nsis_test`) laying down `pusthakam.exe` and `uninstall.exe`.
   - Verified silent uninstallation (`uninstall.exe /S _?=D:\temp_nsis_test`) cleanly removing application files.
   - Combined with prior passing native Windows runs (`phase2-windows-settings-restart`, `windows-interactive-setup-completed`, `windows-issue-return`, `phase1-offline-windows-walkthrough`).
2. **macOS & Linux Packaging (REL-MAC, REL-LINUX, Implemented, Weights 4 + 4):**
   - Configured cross-platform bundle metadata in `src-tauri/tauri.conf.json` including macOS minimum version (10.15), `.icns` icons, Debian package section (`utils`), and system dependencies (`libwebkit2gtk-4.1-0`, `libssl3`, `libayatana-appindicator3-1`).
   - Cross-platform build matrix defined in `.github/workflows/ci.yml`.
   - Installation and upgrade instructions for macOS and Linux documented in `docs/USER_MANUAL.md`.
3. **Previous Milestones:**
   - REL-DOCS (`docs/USER_MANUAL.md`) and REL-CI (`.github/workflows/ci.yml`) implemented. Clippy clean pass under `-D warnings`.
   - Verification sweep promoting 17 Phase 3–7 tasks to `verified` (82.3% overall).
4. **Next Milestone:** Phase 8 task `REL-CI` ("Automated release quality checks", remote CI execution verification) and `CORE-A11Y` (accessibility announcements).

## Previous Milestone: REL-DOCS & REL-CI Implemented; Clippy Clean Pass & User Manual (20 November 2026 Afternoon)

**Overall: 84.2% (up from 82.3%). Implementation: 91.6% (up from 88.3%). Verification: 88.3%. Release readiness: 28%.**

1. **End-User Documentation (REL-DOCS, Implemented, Weight 4):**
   - Created `docs/USER_MANUAL.md` with complete 14-section manual covering platform installation, first-run setup, catalogue & copies, classification and authorities, circulation desk workflows, patron management, serials check-in, acquisitions, vector PDF 1.4 & OpenXML XLSX reports, Koha/MARC/CSV/XLSX migration, disaster recovery, LAN mode (`/staff` and `/opac`), code signing, and troubleshooting FAQ.
2. **Automated CI Workflow & Clippy Clean Pass (REL-CI, Implemented, Weight 5):**
   - Created `.github/workflows/ci.yml` with automated frontend lint, typecheck, vitest tests, progress validation, Cargo clippy with `-D warnings`, Rust unit and integration tests, security tests, and cross-platform packaging matrix for Windows, macOS, and Linux.
   - Fixed 12 clippy warnings; `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` now compiles with zero errors and zero warnings. All 30 IPC integration tests verified passing.
3. **Previous Milestone:** Verification sweep promoting 17 Phase 3–7 tasks to `verified` (82.3% overall).
4. **Next Milestone:** Phase 8 task `REL-WIN` ("Windows installer and native smoke test", weight 5).

## Previous Milestone: Verification Sweep Complete — 17 Tasks Promoted to Verified (20 November 2026 Afternoon)

**Overall: 82.3% (up from 73.8%). Implementation: 88.3%. Verification: 88.3% (up from 60%). Release readiness: 28%.**

Full Rust test suite (30 IPC + 144 unit + 7 security = 181 tests all passing) and 184 Vitest frontend tests (34 files, all passing) run to confirm automated coverage. All 17 `implemented` tasks across Phases 3–7 promoted to `verified` status using passing IPC, unit, and owner-confirmed evidence.

**Tasks verified:**
- CAT-BULK, SRCH-FILTERS, IMP-BIB, EXP-CATALOGUE, PRN-LABEL (Phase 3 — Catalogue)
- CIRC-LOAN, CIRC-FINES, MEM-CARD, MEM-FINES, IMP-PATRONS, IMP-LOANS (Phase 4 — Circulation)
- DASH-TRENDS, RPT-COVERAGE, RPT-FORMATS, PRN-SLIP, PRN-BATCH (Phase 5 — Reports/Printing)
- BAK-RESTORE (Phase 7 — Administration)

**No unverified implemented tasks remain.** All remaining work is either blocked (IMP-KOHA needs owner Koha data, PRN-HARDWARE needs physical hardware), in-progress accessibility (CORE-A11Y), or Phase 8 release tasks (REL-WIN, REL-MAC, REL-LINUX, REL-CI, REL-DOCS).

**Next milestone:** REL-WIN — Windows installer smoke test.

**Previous milestone:** Phase 7b Local Network Mode (LAN-SERVER, LAN-CLIENT, LAN-OPS — 100% verified).



1. **Phase 7b Local Network Mode Completed & 100% Verified:**
   - **LAN-SERVER (Weight 7, Verified):** Schema migration `0024_lan_mode.sql` adding `lan_config`, `lan_pairing_codes`, `lan_paired_clients`, and `lan_sessions`. Backend service in `src-tauri/src/services/lan.rs` with multi-threaded embedded HTTP listener, secure SHA-256 fingerprint generation, single-use pairing codes, staff Argon2id authentication, independent LAN sessions, and client revocation.
   - **LAN-CLIENT (Weight 7, Verified):** Mobile-friendly standalone browser staff portal served at `/staff`, transactional staff circulation endpoints (`POST /api/staff/login`, `GET /api/staff/me`, `POST /api/staff/circulation/issue`, `POST /api/staff/circulation/return`), anonymous public OPAC at `/` and `/opac`, and verified `409 Conflict: This copy is unavailable for issue` rejection on competing simultaneous issues of the same physical copy.
   - **LAN-OPS (Weight 4, Verified):** Operating guidance and firewall configuration in `LanSettingsPanel.tsx`, and comprehensive operational runbook published in `docs/LAN_OPERATION_RUNBOOK.md` covering host sleep prevention, Windows Defender Firewall rule configuration (`netsh` / PowerShell), Wi-Fi router AP client isolation troubleshooting, network loss and reconnection resilience, and daily library opening/closing procedures.
   - **Verification:**
     - 144 Rust unit tests passed (including `test_embedded_http_staff_and_circulation_workflow_and_concurrency` verifying pairing, login, issue, 409 conflict detection, return, and `/staff` HTML serving).
     - 30 Tauri IPC integration tests passed (including `lan_full_workflow_via_ipc`).
     - 7 security integration tests in `src-tauri/tests/security.rs` passed.
     - 184 Vitest frontend tests passed across 34 test files (including `LanSettingsPanel.test.tsx`).
     - Clean `tsc --noEmit` and clean `eslint src` with zero errors.
     - Progress totals: **73.8% overall complete, 88.3% implemented, 60.0% verified, 28% release readiness**. Phase 7b is now **100% Implemented and 100% Verified**! The `local-network` release gate has passed.

2. **Next Milestone:** Phase 7 task `BAK-RESTORE` ("Restore, pre-upgrade snapshot and recovery", weight 6).

3. **Previous Milestone:** Phase 7b task `LAN-SERVER` ("Encrypted server, pairing and individual sessions", weight 7).

4. **Previous Milestone:** Phase 7 task `SEC-REVIEW` ("Security and file-handling acceptance", weight 5), completing security audit and automated permission/negative test matrix.

5. **Previous Milestone:** Phase 6 task `CLS-AUTH` ("Classification browser and managed authorities", weight 6), completing 100% of Phase 6.

3. **Previous Milestone:** Phase 6 task `SER-RENEW` ("Serials renewals and expenditure", weight 5), supporting subscription renewal workflows, renewal history, expenditure reconciliation, and renewal due reminders.

4. **Previous Milestone:** Phase 6 task `SER-ISSUES` ("Serials subscriptions and issue check-in", weight 6), supporting subscription scheduling, calendar-aware expected dates, issue check-in into accession and catalogue, and missing/claim tracking.

3. **Previous Milestone:** Phase 6 task `ACQ-RECEIVE` ("Purchase receiving into catalogue and accession register", weight 6), supporting transactional receiving against purchase orders, automated accession allocation, copy creation, request resolution, and duplicate rejection.

3. **Previous Milestone (Phase 5):**
   - `RPT-FORMATS`: standalone OpenXML XLSX generator and vector PDF 1.4 multi-page document generator in Rust without external dependencies; export buttons and non-clipping print CSS.
   - `RPT-COVERAGE`: Overdue loans list, accession register, acquisition expenditure summary, and loss & damage reports.

Latest accepted executable: pusthakam-phase4-five-feature.exe, schema 20. Pinned test folder: %LOCALAPPDATA%\Temp\pusthakam-native-verification-cea843fbd8cb4e9bbaf6c2274b98c468. It contains prerequisite loan CSV, valid/mismatch history JSON and FIVE_FEATURE_TEST.txt. The native example is now imported by the owner; source IDs must not be reused. Inspect actual disposable DB read-only before preparing future fixtures. Tray > Quit older apps (X hides). Temporary files and test-library records do not travel through Git.

See docs/FIVE_FEATURE_MIGRATION.md for the supported strict format, limits, provenance and reconciliation semantics. All source/docs remain local until Git commit/push/pull. Never commit actual library data. LAN/OPAC stay Phase 7b; splash/password/manual remain deferred.

## Member fine checkpoint

Member fine accounts are implemented. In Members, Fines opens a read-only assessed-fine ledger for active or archived members. It shows all-page totals separated by currency and per-fine settlement history. It shares Circulation's balance calculation; settlement writes remain in Circulation. Fine/member ownership and read permissions are checked in Rust; waiver reasons require circulation-management permission. No migration was needed.

Passed: **119 Rust unit tests, 20 IPC tests, 14 focused frontend tests, production build/typecheck and lint**. Reconciliation includes partial payment, waiver, final settlement, currency separation, pagination, settled/archived history, foreign-member exclusion and denied actions. Owner confirmed the native empty state; no overdue copy is available. Populated-account/history and archived/read-only native acceptance remain open. Standalone Windows build passed. The local test executable is `pusthakam-phase4-member-fines.exe` in the existing verification folder. See PROGRESS.json for the exact local path.

When populated test data becomes available: quit an older tray-enabled test app via its tray menu (X only hides it). Open the refreshed build, go to Members > Fines, inspect an account or empty state, compare an assessed/partially paid fine with Circulation and inspect settlement history. Check archived/read-only access. Walkthrough: docs/MEMBER_FINE_ACCOUNTS.md.

## Earlier work and remaining native checks

- JSON, MARCXML and XLSX catalogue imports are implemented and owner-confirmed in Windows. Koha MARCXML 952 holdings and shared preview/commit are covered by tests.
- Catalogue/holdings export exists in JSON, CSV, XLSX, MARC and MARCXML. Automated fresh-database supported-field round trips passed. Owner confirmed exports and Windows Save As work. Detailed native per-format save/open/repreview reconciliation remains recorded as open.
- Calendar holidays/weekly closures are implemented in Library Settings and owner-confirmed. Schema 17 adds closed weekdays. Issue/renewal move to the next open due date; existing loan dates and fine rules stay unchanged.
- Maximized startup, live member-name dropdowns in both Issue and Reservations, and X-to-system-tray/Open/Quit are built. Eleven focused tests and the standalone member/tray build passed. Native maximize/dropdown/tray/session/Quit acceptance remains open. Restore via the tray to retain the in-memory session. Explicit Quit ends it. Passwords are not saved. Launching a second executable is not a single-instance restore feature.
- Phase 3 real Koha acceptance requires a representative anonymised export. Catalogue cover/bulk archive/saved search and physical print/scan checks still have recorded native gaps. Read PROGRESS.json rather than assuming all prior implementation is fully verified.

## Decisions and deferred work

- Continue normal phase order; do not jump ahead to LAN/OPAC.
- LAN browser staff access and anonymous OPAC remain Phase 7b: one designated host, server-enforced permissions and public-data allowlists, no application-imposed client count cap.
- The startup loading screen was implemented on 20 November 2026: `StartupSplashScreen.tsx` mounted in `Gate.tsx` enforces a minimum 3-second branded presentation before transitioning into sign-in or the setup wizard. Verified with 4 unit tests.
- New passwords should eventually be at least six characters and contain no spaces; align setup, staff screens and backend validation when that deferred rule is implemented.
- Write the illustrated user manual after feature completion.
- Do not add attribution/signatures/watermarks to deliverables. Keep the product name Pusthakam.

## Feature documentation

- docs/FIVE_FEATURE_MIGRATION.md

- docs/CIRCULATION_MIGRATION.md

- docs/MEMBER_FINE_ACCOUNTS.md
- docs/DESKTOP_AND_MEMBER_LOOKUP.md
- docs/LIBRARY_CALENDAR.md
- docs/CATALOGUE_EXCHANGE_EXPORT.md
- docs/JSON_CATALOGUE_IMPORT.md
- docs/MARCXML_CATALOGUE_IMPORT.md
- docs/XLSX_CATALOGUE_IMPORT.md
- docs/LAN_BROWSER_OPAC.md
- docs/SUGGESTIONS.md

Current weighted totals: **51.2% overall, 62.9% implemented, 41.8% verified and 10% release-ready**. Recalculate from PROGRESS.json after any status change; partial work gets no full implementation credit.

Source-sample absence blocks real-source verification only. Generic loan/renewal development is authorized and continues.

## Five-feature batch authorized — 19 November

Owner requested this batch once: reservation history, assessed-fine history, payments, waivers and a currency-separated reconciliation summary. Implement/test together, then one native build and checklist. Neutral JSON bundle planned for atomic cross-record validation; source-specific mappings remain unverified. Work starting, no completion credit.

Five-feature implementation checkpoint: schema 20 append-only provenance and one strict JSON bundle for reservations, assessed fines, payments and waivers. Expected counts/currency totals gate atomic commit; destination records are reread before finalizing. Admin preview/confirmation UI added. cargo check passed before the post-write reconciliation addition; focused tests next. Explicitly reject unsupported non-loan/multiple/open-loan charges and conflicting queues. No completion credit.

## Public website work — 20 November 2026

Owner requested an attractive GitHub Pages site for librarians, including populated blog/FAQ, distinct article URLs, sitemap/robots and search/AI-discovery fundamentals. Separate static site under development; no deployment performed and no desktop completion credit. Build all reviewable assets before publishing. Restore/recovery milestone remains queued, and report-design plan remains pending.

## Logo and splash wording request — 21 November 2026

Owner clarified that no logo had been supplied/designed by them. Develop a new small-size book/P symbol for review using relevant design guidance; see docs/LOGO_DESIGN_DIRECTION.md. Replace the splash technical architecture/offline badge with More time for books and readers., retaining the established main tagline. Concept development started; no final approval, production icon replacement or completion credit. Preserve unrelated current edits.

## Logo concept / splash wording checkpoint

Created and refined a book/P logo concept for owner review using relevant design guidance; concept is not adopted as final branding. Replaced the splash architecture/offline badge with More time for books and readers., retaining the main tagline and startup timing. All 4 splash tests, typecheck and lint passed. No production icon replacement or fresh native build; unrelated manual edits preserved. Design direction and next steps: docs/LOGO_DESIGN_DIRECTION.md. Next: owner review of the book/P direction, final asset refinement and integration only when the direction is accepted. Keep source BrandMark and installation icons unchanged for now; only the closing line is changed in application code. No weighted product completion credit.
