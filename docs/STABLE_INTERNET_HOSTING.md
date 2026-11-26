# Stable public OPAC and staff address

## DEFERRED: owner only requested an internet-access test

2026-11-26T12:30:20.259Z: The owner clarified that permanent hosting/domain setup is not requested. The temporary both-portal cross-network test already passed. Stop this plan; do not add providers, migrate email, activate the pending Cloudflare zone or change WordPress nameservers. Historical plans below are reference only and require a new explicit request before proceeding. WordPress website and Gmail forwarding remain unchanged.

## Current owner decision: WordPress DNS and email stay unchanged

2026-11-26T12:26:38.149Z: Owner declined moving existing email forwarding. Retain WordPress nameservers and email@melvinvarghese.in forwarding to the same existing Gmail. Do not activate the pending full-DNS Cloudflare zone, change nameservers or migrate Email Routing. Earlier full-zone migration instructions below are superseded and must not be followed. No agent cutover performed.

Revised direction: use an external reachable Pusthakam endpoint compatible with adding only a separate library subdomain record at WordPress.com. A hosted server with public ingress is one architectural option; provider, budget, endpoint and deployment remain undecided, and no purchase is authorized. A private192.168 address is not a public endpoint. A standard free full-setup Cloudflare custom-hostname tunnel is not compatible with the selected no-nameserver-change approach. Temporary Quick Tunnel proof remains valid as a test, not a permanent domain deployment.

No live DNS or mail edits are requested at this checkpoint.

## Active forwarding confirmed; migration not authorized

2026-11-26T12:24:44.504Z: Owner actively uses email@melvinvarghese.in forwarding to an existing Gmail inbox. Keep current WordPress nameservers while the mail approach is decided. Cloudflare Email Routing is a free replacement option, retaining the same alias and Gmail destination but changing the forwarding provider. The owner has not authorized this migration.

Proposed reviewable sequence if selected: verify the existing Gmail destination privately in Cloudflare; configure the same alias; capture exact Cloudflare-generated MX/SPF requirements; preserve applicable WordPress DKIM and verification records and merge required SPF mechanisms into one SPF record; review full before/after DNS and original nameserver rollback; then explicitly authorize cutover and test receipt from a different sender account. Do not promise zero interruption during DNS propagation. Keep WordPress website records DNS-only. Do not remove the existing WordPress forward during preparation.

Alternative: retain WordPress DNS and forwarding, and reconsider the stable-hosting connection. No account routing, MX/SPF, forwarding or nameserver edit has occurred. Sources: https://developers.cloudflare.com/email-service/get-started/route-emails/ and https://developers.cloudflare.com/dns/manage-dns-records/how-to/email-records/.

## Current hold: WordPress free email forwarding

Checked 2026-11-26T12:22:00.858Z: WordPress support says free email forwarding requires WordPress.com nameservers. Copying MX/SPF/DKIM records to Cloudflare alone is not a verified way to preserve this service. Before any cutover, ask whether owner uses domain email forwarding. If yes, keep existing nameservers until a compatible routing/replacement plan is explicitly agreed and verified. The earlier generic migration steps below do not authorize cutover past this dependency.

Source: https://wordpress.com/support/add-email/email-forwarding/ (Required name servers missing). Do not copy old WordPress NS records into the Cloudflare zone. _domainconnect is provider-discovery configuration, not needed for Pusthakam. No instruction to add it is issued now.

Prepared 2026-11-26T08:49:12.754Z. Owner selected internet-hosting work; Malayalam remains deferred. Both portals already passed the temporary external-device/network test. No permanent address or always-on deployment has been configured.

## Proposed next deployment

Use a named Cloudflare Tunnel with a library-owned subdomain, initially on the library PC with an isolated persistent test library. Proposed owner test address: library.melvinvarghese.in, serving /opac and /staff. This is an example for this owner, not a hardcoded dependency for other libraries. A stable address still requires the PC/host and connector to run; it does not provide hosting when the PC is off. Optional always-on server migration remains supported separately.

## Concrete next steps

1. Owner creates a free Cloudflare account and signs in. Do not request passwords/tunnel tokens in chat. No paid service or domain purchase requested.
2. Add melvinvarghese.in to the free Cloudflare DNS plan and review/import its complete existing DNS records. Before changing nameservers, obtain the full WordPress DNS record inventory/export and verify website, email forwarding, TXT/verification and other subdomains. The observed public records in STABLE_HOSTING_DNS_OBSERVATIONS.json are partial and not a migration backup.
3. Prepare a reviewable before/after DNS change and rollback plan. Preserve WordPress website routing and email. WordPress recommends DNS-only mode for its website records; the library tunnel record has its own routing policy. Do not guess a WordPress.com backing hostname from the domain name.
4. After owner approval of the concrete DNS change, activate Cloudflare DNS and confirm the main site and email records remain correct. No nameserver change has been made.
5. Configure a named tunnel for the selected library subdomain to the validated loopback Caddy proxy. Match the exact public origin in Pusthakam and preserve the tested route allowlist, forwarding headers and upload bounds. Keep legacy LAN APIs private. Configure persistent storage and supervised startup; do not publish the actual owner database until explicitly authorized.
6. Test fresh HTTPS login/session security, data permissions, downloads/uploads, reboot/reconnection and both portals from another network. Only then label the permanent deployment verified.

## Current account dependency

The owner previously had no Cloudflare account. No account access or named-tunnel credentials are available to the agent. The next owner action is account creation/sign-in, not a DNS or nameserver edit. No service, DNS, firewall, library database or installed-app changes occurred during this preparation.

## References

- https://developers.cloudflare.com/tunnel/get-started/
- https://developers.cloudflare.com/cloudflare-one/networks/connectors/cloudflare-tunnel/routing-to-tunnel/
- https://wordpress.com/support/cloudflare-dns/
- https://developers.cloudflare.com/support/third-party-software/content-management-system-cms/wordpresscom-and-cloudflare/

The temporary Quick Tunnel is a successful test, not a production availability guarantee. A named tunnel/domain and an always-on rented server are different choices: the former stabilizes the address, the latter changes where the authoritative library runs.


## Account checkpoint26 November2026

Owner confirms logged into Cloudflare. Next: Domains > Onboard a domain, enter melvinvarghese.in, quick scan/import existing records, Free plan, stop at DNS review. Review full WordPress records against scan before changing nameservers; no cutover performed. Earlier no-account dependency is resolved by owner confirmation.


## Scanned DNS screenshot checkpoint

Owner reached the Cloudflare record-review screen on Free/full setup. Both WordPress root A records and www CNAME match the earlier public observations; email-forwarding MX and several TXT records are visible. Website A/CNAME records are currently orange-cloud Proxied; set the two root WordPress A records and www to DNS only per WordPress guidance. The link subdomain service is not assessed. Full TXT values and scan completeness cannot be verified from this screenshot. Compare with the full WordPress DNS table before nameserver cutover. No external DNS edit or activation performed by agent.


## WordPress DNS comparison: missing DKIM records

Owner screenshot shows two CNAME records absent from the earlier Cloudflare scan table: wpcloud1._domainkey -> wpcloud1._domainkey.wpcloud.com and wpcloud2._domainkey -> wpcloud2._domainkey.wpcloud.com. Add both as DNS-only CNAMEs before any cutover. Root A is displayed as Handled by WordPress.com; link A52.72.49.79 also present. Lower WordPress rows still need review; full comparison incomplete. No nameserver change or external record edit performed by agent.


Updated Cloudflare screenshot: both WordPress root A records and www are now DNS only; two DKIM CNAME rows are present and DNS only. Their exact target strings remain truncated, so preserve earlier instructed values and confirm during full comparison. Link remains proxied, purpose unassessed. Await lower WordPress DNS-table screenshot/full record inventory before activation; no nameserver cutover confirmed.


## Pending-zone cleanup requested

Owner confirms only Cloudflare records were edited; nothing was changed in WordPress and no nameserver cutover occurred. Owner wants to remove the unused pending Cloudflare domain entry. Official path: domain Overview > Advanced Actions > Remove from Cloudflare > Confirm. Removal is not yet confirmed. Preserve WordPress website/email and domain registration; do not activate/cut over for cleanup.


## Cleanup complete — owner confirmation

Owner confirmed removal of the unused pending Cloudflare domain entry. No WordPress changes or nameserver cutover had been made. Internet-access testing for both portals is complete; permanent domain/email migration is not requested. This confirmation does not stop a still-running temporary test launcher or implement Malayalam.
