# LAN browser access and OPAC handoff

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
