# OPAC design and browser checks

## Automatic covers

OPAC results and details now try the host's saved PNG at /api/opac/cover/{id}, then an automatic Open Library ISBN image. Neutral placeholders keep the catalogue usable when offline or no cover exists. The production browser review now includes saved covers, provider-failure fallback and invalid ISBN; provider requests are blocked in the harness. Its external request list may contain these expected blocked ISBN image attempts; no provider is actually contacted. No other external assets are allowed. Earlier no-external-request evidence predates the owner-authorized cover lookup.


The owner selected **The Swiss Stacks** on 25 November 2026. Its public masthead, horizontal classification ribbon and compact holdings layout are integrated into the embedded OPAC served at / and /opac by the desktop LAN listener.

## Prototype study

Run node design/opac/server.mjs from the repository root, then open http://127.0.0.1:5193/design/opac/?view=swiss-stacks. The other study views are ?view=reading-room and ?view=sovereign-folio. Stop the server with Ctrl+C. These views contain demonstration holdings and never connect to a library database.

## Production implementation

Source: src-tauri/src/services/opac/ (HTML, CSS, browser JavaScript), assembled by src-tauri/src/services/opac_page.rs. Data comes from GET /api/opac/search. The host's configured server name becomes the page title. Assets, approved logo and fonts are embedded in the native binary; there are no CDN dependencies. A native rebuild is required to update an installed OPAC. The old installer is not refreshed by a source change.

Public fields stay within the existing DTO. There are no demonstration titles/counts, precise shelf coordinates, patron sign-in or reservations in the production page. Classification and availability filters apply on the server before counting and pagination. Search is explicitly submitted; use search or reload to refresh availability after circulation changes.

## Repeatable checks

Run node design/opac/review.mjs for the three study views: 15 layout checks (1366, 1024, 768, 390, 320px) and 21 interactions. Run node design/opac/review-production.mjs for the exact production assets against public DTO contract fixtures: six widths (including 640px), pagination, filtering, author search, no matches, HTML injection, keyboard dialog/focus, disabled access, errors/retry, stale responses and reduced motion. Real database search and HTTP wiring are checked independently by Rust service/IPC tests.

Each review launches and closes its own isolated headless Chromium browser and temporary profile. Edge is detected on Windows; set OPAC_REVIEW_BROWSER to another installed Chromium executable if needed. Test-only sandbox/GPU/extension flags address this host's headless launch constraints. No existing user browser is attached. Screenshots and JSON evidence are in previews/. Production screenshots use test fixtures, not user data.

The previous review-results.json targeting shelf/index/table was superseded. The original mobile clipping and live-search reset bug are fixed and checked. Cross-device LAN reachability remains a separate unresolved acceptance check.

Final delivery checkpoint 2026-11-25T03:08:03.631Z: Tauri CLI standalone debug and Windows NSIS builds passed with the current Wi-Fi and cover changes. NSIS SHA-256 EEAC317C9DE465D5E229CE880466B13E020F2A9568BAC7EE8AE6F469F79AB87F (4594047 bytes). Installer not installed automatically. Same-network second-device and live provider availability remain owner checks.
