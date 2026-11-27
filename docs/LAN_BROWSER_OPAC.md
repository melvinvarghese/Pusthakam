# LAN browser access and OPAC handoff

## Confirmed target — 25 November 2026

Both same-network and internet OPAC/staff addresses; /staff includes the shared desktop feature workspace within the user's role. Library PC hosts by default; the optional pusthakam-server console host uses the same services and a persistent library directory. The authenticated HTTP gateway, browser downloads, isolated sessions and configured HTTPS proxy boundary are implemented. Thirty HTTP workflows, all 14 browser module screens, browser backup controls and an actual host restart/restore have passed local automated checks. See STAFF_BROWSER_PARITY.md, STAFF_BROWSER_COMMAND_INVENTORY.json and WEB_HOSTING.md. Physical other-device connectivity, a public deployment and hardware acceptance remain open. Earlier local-only/circulation-only scope is superseded for v3.


## Historical scope checkpoints

The following entries preserve earlier decisions and evidence; use the confirmed target above and the current progress/handoff records for implementation status.

### Web-address access request - 25 November 2026

Owner requests web-address access for both OPAC and staff portal, comparable to their Koha workflow. Source audit confirms existing local HTTP /opac and /staff routes. OPAC is a public catalogue; the staff browser portal currently covers pairing/sign-in and issue/return, not the complete desktop staff workspace. Asked whether desired addresses are same-Wi-Fi/network only or also reachable over the internet; answer pending. Do not treat local IP access as public internet hosting, or the limited circulation portal as full browser parity. Next exact step: apply owner network-scope answer, then define address/hosting and full staff browser requirements against existing services; internet access requires an appropriate HTTPS deployment/authentication design before exposure. No DNS, router, firewall, server deployment or application change in this clarification turn. Preserve final standalone installer and cover work; cross-device acceptance remains open. No new task weights or product verification credit.


## Wi-Fi access, standalone delivery and book covers — 25 November 2026

Wi-Fi and Ethernet use the same local-network listener. Settings now show each active host IPv4 address and complete OPAC/staff URL, separate loopback addresses, copy actions, and Refresh addresses. Network access can be switched from This computer only to Same Wi-Fi or wired network. No shared URLs are advertised for an inactive or loopback-only listener. Windows adapter discovery uses GetAdaptersAddresses, including active networks without an internet route. No firewall rules or owner database settings were changed automatically. A real second-device acceptance test remains pending.

Plain cargo build is not a standalone delivery command: Tauri development mode loads devUrl at 127.0.0.1:1420. Use npm.cmd run build:standalone-debug or npm.cmd run build:windows (Tauri CLI custom-protocol builds). The old development executable must not be presented as standalone.

Desktop catalogue results now show cover thumbnails, using the same automatic ISBN lookup and editable-record cache as book details. Detail pages show errors/placeholder and Retry ISBN cover instead of silently blank space; a direct image fallback handles download/conversion failure. Public OPAC results/details first request GET /api/opac/cover/{recordId}, which serves only a stored PNG for an active record while OPAC is enabled. Missing local images automatically try https://covers.openlibrary.org/b/isbn/{isbn}-M.jpg?default=false for syntactically valid ISBNs. Only the public ISBN is sent, without a referrer; missing/invalid/offline covers show a neutral placeholder. Existing saved desktop images remain available from the host without internet. Public requests never write images to the library database. Optional online covers do not block catalogue search. No cover-availability guarantee for every ISBN.


## Current implementation — 25 November 2026

Swiss Stacks is selected and integrated into the native public OPAC route. Production assets live in src-tauri/src/services/opac/ and use only public catalogue fields. The masthead shows the configured library name; browse-category/available-copy filters run on the server before pagination. Book details open in a keyboard-accessible dialog. There are no demonstration records or fabricated counts/shelf coordinates in production. Fonts and branding are embedded; optional Open Library ISBN covers are the owner-approved external image lookup. Mobile intrinsic-width clipping is fixed. Prototype live-search reset is fixed. Updated browser checks exercise the current three study views and the production assets; see design/opac/README.md and previews/*review-results.json for evidence and fixture limitations. Earlier concept/prototype descriptions below are historical and do not describe current implementation status. The Windows debug executable was rebuilt successfully; the installer was not rebuilt. Cross-device LAN remains unresolved.

The existing public DTO, anonymous read-only boundary, host lifecycle and staff permissions remain. GET /api/opac/search accepts q, page, category (empty or a positive browse-category id), and available=true. GET /api/opac/categories lists active browse categories that hold visible books, with counts (27 Nov 2026; replaced the earlier `class` digit filter). Page size remains 20 for HTTP; the existing Rust command remains compatible. Search reuses the catalogue FTS expression builder so Malayalam vowel marks are retained. Browse categories replace the earlier DDC main-class buttons; books with no category are reachable under All books. Members never see class numbers in result rows; the detail view shows the location and shelf call number under "Where to find it". Precise shelf coordinates are not part of the public contract.

Historical planning checkpoint follows.


Recorded 18 November 2026. Planning checkpoint only; LAN server, browser client and public OPAC are not implemented.

## Intended use

One designated library PC runs Pusthakam and owns the SQLite database and backups. With LAN hosting enabled, staff open the host's staff address in their browser, sign in, and use functions allowed by their role. Visitors open the OPAC address without signing in to search books and availability. Phones and other clients need no app installation. Internet access is not required. Host must remain running and reachable.

## Current gaps

Existing frontend uses Tauri invoke. AppState has a desktop session rather than independent browser sessions. No LAN listener, browser authentication, public response contract or OPAC page exists. Reuse business services and design tokens, introduce a browser transport and keep database access on the host. Never share the live SQLite file over the network.

## Public boundary

Publish only approved public book fields and availability/location information. Exclude members, borrowers, loan history, staff records, internal notes, financial records and audit/settings data. Public requests cannot mutate library data or reuse the host's signed-in session. Test response contents and denied routes rather than relying on hidden UI controls.

## Implementation sequence within Phase 7b

1. Define public read DTOs and routes, staff session/permission contracts and independent browser transport. Review all desktop-only functionality.
2. Add host lifecycle, encryption and phone-compatible certificate trust workflow; LAN/OPAC controls; staff/OPAC addresses and QR codes.
3. Serve browser staff UI and responsive public OPAC from bundled assets. Add browser file/export/print flows as applicable.
4. Verify live availability, transactions, competing issues and stale edits; session expiry/revocation; privacy and anonymous write denial.
5. Pilot host plus multiple browser devices, including a phone, with internet disconnected. Document firewall, IP changes, guest-network isolation, sleep/restart and reconnect. Measure practical capacity without an application-imposed client cap.

## Resume after the break

Continue the existing development order; this request expands planned Phase 7b rather than skipping unfinished phases. Saved catalogue search confusion is diagnosed: old entries had names but empty words. Owner confirmed the corrected search works, and label printing works. Source UX improvements have 27 focused passing tests plus passing standalone typecheck/lint; refreshed native executable is still pending because automatic review interrupted the build sequence over an unrequested Visual Studio telemetry destination. No blocked action was retried. PROGRESS.json and PROGRESS.md hold the current evidence.

Final delivery checkpoint 2026-11-25T03:08:03.631Z: Tauri CLI standalone debug and Windows NSIS builds passed with the current Wi-Fi and cover changes. NSIS SHA-256 EEAC317C9DE465D5E229CE880466B13E020F2A9568BAC7EE8AE6F469F79AB87F (4594047 bytes). Installer not installed automatically. Same-network second-device and live provider availability remain owner checks.

Owner acceptance 2026-11-25T12:17:30.108Z: both /opac and /staff work from another same-network device. Outside-network access still requires a configured public HTTPS endpoint. Pairing-free settings source changes are not yet delivered in a new installer.
