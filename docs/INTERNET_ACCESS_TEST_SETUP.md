# Temporary internet test

This test uses an isolated demo library. It does not publish the owner's database, change WordPress DNS, open a router port or install a Windows service. The existing desktop installer does not need rebuilding.

## Earlier download blocker, 26 November 2026

Official GitHub release metadata is reachable through Node's configured proxy/system certificate trust. PowerShell HTTPS fails with Windows security error 0x8009030E. Actual release binary downloads and GitHub's asset API timed out. Certificate checking was retained throughout. No tool binary was executed, no public URL was created and no internet access is confirmed.

Download these two official assets using the normal Windows browser, keeping their filenames and saving them in Downloads:

- [cloudflared-windows-amd64.exe, release 2026.11.2](https://github.com/cloudflare/cloudflared/releases/download/2026.11.2/cloudflared-windows-amd64.exe)
- [caddy_2.11.9_windows_amd64.zip](https://github.com/caddyserver/caddy/releases/download/v2.11.9/caddy_2.11.9_windows_amd64.zip)

No account or payment is required for the temporary Quick Tunnel. Leave the Caddy archive zipped. The preparation helper checks official release SHA256 digests and lengths before copying or extracting tools. If the latest upstream release changes, obtain matching current assets rather than bypassing the digest check.

## Prepared helpers

From the repository directory:

```powershell
node --use-system-ca --use-env-proxy tools/prepare-internet-test-tools.mjs
node tools/internet-test.mjs
```

The first helper checks the two specifically named files in Downloads before attempting network downloads, verifies published hashes, extracts Caddy into an isolated temporary directory and records portable-tool metadata. It changes no PATH or service settings.

The second helper creates a new demo library and random staff password outside the repository, starts the existing release console host on loopback, starts Caddy with an explicit public-route allowlist and HTTPS forwarding, and requests a temporary Cloudflare URL. It configures that exact public origin before checking staff cookies. Legacy LAN/staff APIs return404 at the proxy. Credentials stay in a test-login.json file outside the repository. Type quit into its console to stop its child processes. Keep the computer and launcher running while testing; restarting generates a new URL and new isolated library.

Preparation, extraction and seven actual local proxy/authentication checks now pass. Actual Cloudflare connectivity and public HTTPS acceptance remain unverified because the connector hit a socket-permission denial inside the agent environment. Do not treat a generated URL or local proxy check as proof of external access. After setup, verify real HTTPS staff sign-in/API/backup behavior and then ask the owner to open /opac and /staff on mobile data with Wi-Fi off. Record actual results in INTERNET_ACCESS_TEST.json and the progress/handover files. No owner library changes or WordPress DNS changes are needed.

This is a temporary developer test, not a production service installer or an automatic hosted-database migration. Follow WEB_HOSTING.md for permanent hosting and migration acceptance.


## Current state after owner downloads

Both files passed official SHA256/size verification and portable extraction completed using Windows tar.exe. Seven real local proxy checks pass, including OPAC/staff, public-origin Secure cookies, CSRF login/session rotation and blocked legacy routes; see INTERNET_PROXY_REVIEW.json. Caddy hostname/listen configuration and the Node Host-override test were corrected during preflight. No product source or owner library changed.

The agent execution environment denied the connector an outbound socket to api.trycloudflare.com. No tunnel remains running. From normal Windows PowerShell, run:

```powershell
Set-Location D:\Pusthakam
node tools/internet-test.mjs
```

Keep that console running. It prints OPAC/staff URLs and the local credentials-file path. Share the generated HTTPS URL with the agent for actual public checks, then test on mobile data. Type quit to stop. A successful local proxy check does not prove internet access. No administrator access, Cloudflare account, WordPress DNS change or paid server is needed for this temporary demo.


## Live demo update26 November2026

Owner launched the prepared helper successfully. Temporary origin: https://configure-habitat-relationships-commodities.trycloudflare.com (only while its console remains running). Public OPAC/search, Secure staff session/login, authenticated staff data, origin rejection and backup creation pass; legacy routes return404. See INTERNET_PUBLIC_REVIEW.json and latest SESSION_HANDOFF.md. No real library data or WordPress DNS changes. Owner mobile-data/browser acceptance remains pending; previous agent socket-denial notes are historical, not a current failure of this owner-launched tunnel. Credentials stay outside the repository.


## Latest availability: hostname unavailable

Owner browser reported ERR_NAME_NOT_RESOLVED. OS lookup fails and Google/Cloudflare public DNS both return NXDOMAIN, although all three recorded test processes remain alive. Previous successful HTTPS checks are historical, not proof of current availability. Restart the temporary launcher from normal PowerShell (quit, then node .\tools\internet-test.mjs in D:\Pusthakam), share its fresh URL and recheck public DNS/HTTPS. Do not change PC DNS or firewall based on this result. See INTERNET_DNS_REVIEW.json.


Replacement URL verified26 November2026: https://assistant-habits-putting-completing.trycloudflare.com. Both public resolvers return A records; OPAC/staff/search return200 and staff sets Secure cookie. Launcher origin matches. Old hostname failure is historical; current phone/mobile-data acceptance remains pending. Credentials changed with the new isolated demo and remain outside repository. See INTERNET_ENDPOINT_RECHECK.json.


Owner acceptance26 November2026: fresh temporary public address worked on another device using another network. This confirms cross-network reachability. Portal-specific coverage and staff authenticated workflows were not specified and need confirmation. Temporary demo/PC uptime requirements remain; no permanent domain or always-on deployment configured.


Owner follow-up26 November2026: BOTH OPAC and staff worked on another device on another network, answering the portal/sign-in question. Both-portal external-device acceptance passes for this temporary isolated demo. Full workflows and production availability remain separate; no permanent custom domain/always-on deployment exists.
