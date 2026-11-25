# Internet access: next-session checkpoint

Saved 2026-11-25T15:57:48.149Z. Work is paused at the owner's request. Resume only on explicit instruction.

## Current state

- Both OPAC and staff work on another same-network device, confirmed by the owner.
- Fresh Windows installer: src-tauri/target/release/bundle/nsis/Pusthakam_0.1.0_x64-setup.exe. SHA256: bcf0d9a0d8910e1e294d88de4f8e27ff6983224f4944acba0f6ccb0d844a3d04.
- Exact-release mobile browser checks: 115 passed, then repeated 115 passed; zero exceptions. Exact-release headless restore/delayed-packet checks: 11 passed. See WEB_RELEASE_VERIFICATION.json.
- Internet access remains unconfigured and untested. No tunnel, public URL, domain record or DNS change exists.
- melvinvarghese.in uses WordPress.com; owner can see Add a record in domain DNS. Preserve the main website. Proposed library subdomain is only a test option, not a product dependency.
- Owner has no Cloudflare account. Free temporary Quick Tunnel was discussed; no account/domain needed for that temporary test. No cloudflared command was found on PATH during inspection. Nothing installed or started.

## Resume sequence

1. Confirm current installed app is the updated build and check OPAC/staff hosting settings; do not assume it was installed because the installer exists.
2. Prepare a temporary library and review the actual HTTPS proxy/tunnel integration before publishing. Follow WEB_HOSTING.md: loopback boundary, configured public origin, correct forwarded Host/protocol, buffered uploads, approved routes only; never publish legacy LAN APIs.
3. Establish the temporary HTTPS endpoint, configure the exact public origin, verify OPAC results and staff authentication/permissions. No WordPress DNS edits needed for this initial test.
4. Owner opens /opac and /staff on a phone using mobile data, Wi-Fi off. Record actual result; same-PC checks do not prove outside-network reachability.
5. Once the test works, plan a stable custom subdomain or always-on deployment. Review account/DNS requirements before any WordPress nameserver change. No server purchase requested.

## Hosting decisions

The library PC remains default. If it is off, its portals stop. An optional rented always-on server runs Pusthakam and stores the authoritative database; PCs/phones connect to it. Hosted mode needs automatic startup, HTTPS, maintenance, independent backups and verified restore/migration. It does not automatically synchronize two writable databases. Windows console server is locally verified; Linux/public deployment still need acceptance.

## Remaining limits and saved work

Physical phones, Safari, real soft keyboards and full workflow/hardware acceptance remain separate from touch emulation. Existing source/docs are saved locally but have not been committed or pushed. Read PROGRESS.md, PROGRESS.json and SESSION_HANDOFF.md before continuing. No owner data, firewall, DNS or installed app was modified for this checkpoint. Do not resume automatically tomorrow.
