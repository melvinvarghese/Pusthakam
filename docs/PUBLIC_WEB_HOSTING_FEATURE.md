# Optional public web hosting for each library

Owner clarification, 25 November 2026: a personal domain may be used for testing, but publication must be a product feature for any library. No owner domain is a required dependency or a hardcoded production address.

Each library installation owns its configuration and database. Local OPAC/staff access works without public hosting. The existing optional Public HTTPS address is saved in that library's configuration; a library may use its own domain with either the default library-PC host or an always-on console host. The database remains on its selected authoritative host. Staff continue to use their own permission-checked accounts.

The application gateway and configurable public origin exist. Real public access also requires an operator-managed domain, HTTPS ingress and a reachable host. Entering a domain in the app cannot create those services. The current host has no public endpoint configured. Publication is therefore optional and supported by the architecture, but a deployed public-hosting walkthrough remains unverified. See WEB_HOSTING.md for the deployment template and migration process.

Product acceptance must cover:

- A library can keep both portals local and leave its public address empty.
- A library can configure its own public HTTPS origin independently of every other installation.
- The configured origin works with either host mode and can be changed without losing library data.
- The deployment can publish OPAC alone, or include staff access through the supported HTTPS gateway; route exposure must match the operator's chosen proxy policy.
- Patrons can search on phones without signing in or pairing devices. Staff on phones can navigate all permitted modules and use readable forms, touch controls and downloads.
- Saved configuration is labelled as configured, never as proof of reachability. Actual external-device/domain, uploads, session security and outage/reconnect acceptance must be recorded before production release.

Mobile work retains Swiss Stacks for the public catalogue and the Spine Label staff design. A desktop sidebar squeezed into a phone, clipped tables, tiny fields or a screenshot without overflow are insufficient mobile acceptance. Check usable workspace width, menu/focus behavior, record readability, form input sizes, row actions, deliberate table scrolling and dialogs with the phone keyboard open.


## Saved owner test plan, 25 November 2026

The owner has WordPress.com DNS access for melvinvarghese.in and no Cloudflare account. A free temporary HTTPS tunnel was discussed as the first outside-network test, before custom-domain or paid-server setup. No tunnel, public origin or DNS changes have been made. Work is paused until the owner returns; see INTERNET_ACCESS_CHECKPOINT.md for exact next steps and safety boundaries.


## Resume26 November2026

Owner resumed the free temporary public test. Isolated-test tooling is prepared, syntax checked and pending execution; official binary download timeouts require browser downloads first. No public URL, account, purchase or WordPress DNS changes exist. See INTERNET_ACCESS_TEST_SETUP.md and the current SESSION_HANDOFF.md entry.


Update after owner downloads26 November2026: portable tools are verified and prepared; seven real local proxy/auth checks pass. Actual tunnel startup hit an outbound socket-permission denial in the agent environment, so the owner must run the prepared launcher from normal Windows PowerShell. No active public endpoint or outside-network acceptance exists yet. See INTERNET_ACCESS_TEST_SETUP.md.


## Live demo update26 November2026

Owner launched the prepared helper successfully. Temporary origin: https://configure-habitat-relationships-commodities.trycloudflare.com (only while its console remains running). Public OPAC/search, Secure staff session/login, authenticated staff data, origin rejection and backup creation pass; legacy routes return404. See INTERNET_PUBLIC_REVIEW.json and latest SESSION_HANDOFF.md. No real library data or WordPress DNS changes. Owner mobile-data/browser acceptance remains pending; previous agent socket-denial notes are historical, not a current failure of this owner-launched tunnel. Credentials stay outside the repository.


Owner acceptance26 November2026: fresh temporary public address worked on another device using another network. This confirms cross-network reachability. Portal-specific coverage and staff authenticated workflows were not specified and need confirmation. Temporary demo/PC uptime requirements remain; no permanent domain or always-on deployment configured.


Owner follow-up26 November2026: BOTH OPAC and staff worked on another device on another network, answering the portal/sign-in question. Both-portal external-device acceptance passes for this temporary isolated demo. Full workflows and production availability remain separate; no permanent custom domain/always-on deployment exists.


## Stable address preparation, 26 November2026

Owner resumed internet-hosting work; Malayalam remains deferred. Named-tunnel/domain preparation is saved in STABLE_INTERNET_HOSTING.md. Read-only partial DNS observations show WordPress website and email-forwarding records; a complete inventory is required before any nameserver change. No account or DNS access is configured. Next owner action: create/sign into free Cloudflare account; preserve current WordPress DNS until the actual change is reviewed.


## Domain cutover dependency, 26 November2026

WordPress DNS screenshot shows free email-forwarding configuration. Official WordPress guidance requires WordPress.com nameservers for forwarding. Hold Cloudflare activation/nameserver changes until actual usage is clarified and preservation/replacement agreed. Copied MX records alone do not establish continuity. See current STABLE_INTERNET_HOSTING.md and SESSION_HANDOFF.md. No external cutover performed.


Owner decision26 November2026: keep WordPress DNS/nameservers and existing Gmail email forwarding unchanged. Do not perform the previously considered Cloudflare full-DNS or mail migration. Future stable hosting must work through a separate subdomain record with WordPress remaining authoritative. No provider purchase or deployment selected. See latest STABLE_INTERNET_HOSTING.md and SESSION_HANDOFF.md; older Cloudflare cutover instructions are superseded.


## Final scope clarification: test complete

Owner only wanted proof that Pusthakam OPAC and staff can be reached over the internet. Both were confirmed working from another device/network using an isolated demo library and the release server. No permanent domain, hosting purchase, alternative provider or email/DNS migration is requested. Stop permanent setup; preserve WordPress website/nameservers and existing Gmail forwarding. The real owner library has not been published. Historical stable-hosting instructions are not active next steps. Current temporary URL availability was not rechecked at this documentation checkpoint.
