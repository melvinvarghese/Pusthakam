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
