# Full staff browser access and hosting

Owner decision, 25 November 2026: support both same-network and internet addresses for OPAC and staff. The library PC is the default host; an always-on hosted server is also supported by the target architecture. `/staff` must provide every desktop feature available to that user's role. A circulation-only portal is insufficient.

This is an expanded v3 requirement. Existing v1 percentages do not measure completion of this requirement. No internet deployment or full browser parity is currently established.

## Hosting modes

| Mode | Database owner | Local access | Internet access | Availability |
|---|---|---|---|---|
| Library PC — default | The library PC | Host IP or configured local name | Configured HTTPS address through an authenticated, supported ingress/reverse proxy | PC and application must stay running |
| Always-on server | The designated hosted server | Browsers connect to the server; a local route requires reachable infrastructure | Configured HTTPS domain | Server must remain running and backed up |

There is one authoritative database per library. These options do not introduce automatic synchronization between two independent writable databases. Switching host must be an explicit backup/restore migration with a pre-move snapshot, verification and rollback. Never share a live SQLite file over Wi-Fi, SMB or a cloud-synced directory. The local desktop application remains usable without internet in PC-host mode.

An internet address cannot be created from a local IP alone. Domain/HTTPS ingress and host reachability must be configured for the selected deployment. No router forwarding, firewall rule, DNS change or public deployment has been performed. Publicly exposing the current minimal HTTP server is not an implementation of this plan.

## One application contract

The browser should reuse the React staff application and typed API, including theme and keyboard behavior. Desktop continues to use Tauri IPC. Browser mode uses a same-origin authenticated HTTP API with identical argument/result/error shapes. The server advertises browser mode only after the session/API is ready; the current portal does not emit that configuration.

The initial frontend transport foundation is in `src/lib/api/staffHttp.ts`. It is guarded by explicit server metadata on `/staff`, uses same-origin cookies, a CSRF header, no-store requests and normalized errors, and refuses arbitrary endpoints. It is not a working browser backend or an internet-ready feature.

## Required feature parity

| Area | Browser acceptance |
|---|---|
| Sign-in and roles | Separate sessions per browser, permission refresh, logout/revocation and lockout; never inherit the desktop account |
| Dashboard | Same live counts and links, within user permissions |
| Catalogue and classification | Search, records/copies, authority management, duplicate checks, bulk operations, ISBN covers and call labels |
| Members and staff | Full directories, categories, photos, permitted account management and recovery workflows |
| Circulation | Issue, return, renew, reservations, eligibility, due dates, fines/payments/waivers and concurrent-edit protection |
| Imports | Every supported format and reconciliation flow; file upload, preview/confirm, per-session pending state, rollback and errors |
| Exports and reports | Every current format, design choice and filter; download to the client, not arbitrary writes to the host |
| Printing | Labels, spine labels, member cards, slips and reports through browser-compatible print/download flows; hardware checks remain real-device acceptance |
| Collection operations | Inventory, stock checks, transfers, acquisitions and serials |
| Settings and audit | Library rules, holidays, locations, branding, permissions and audit access, all enforced on the server |
| Backups and restore | Authorized host backups/scheduling/verify/restore; browser download/upload equivalents and recoverable restart; protect the only good backup |
| Network operations | Host addresses, OPAC/staff enablement, device/session management and truthful reachability status |

`STAFF_BROWSER_COMMAND_INVENTORY.json` lists all registered commands. Generate with `node tools/browser-parity.mjs`; verify with `node tools/browser-parity.mjs --check`. The inventory marks unimplemented backend parity explicitly. Command count is not a feature-completion percentage.

## Backend isolation before HTTP dispatch

The desktop wrappers still take `State<AppState>`. The implemented ownership layer shares database, listener and logging lifetime through Arc, while independent contexts have separate session and import-preview slots. A browser API must never temporarily swap the desktop session. 162 commands now expose common typed execute/dispatch handlers; desktop wrappers delegate to the same logic. Two native-window save handlers remain browser-adapter work. The HTTP session manager and route dispatcher are not implemented yet.

Use an explicit allowlist mapped to typed argument decoders and service handlers. Never expose generic host filesystem paths or Tauri native window/dialog commands as remote RPC. Native save/export/backup paths need client download/upload adapters. Unauthenticated bootstrap must not reveal database paths or other host internals.

Internet delivery requires HTTPS, secure session handling, origin/CSRF controls, bounded HTTP parsing/uploads, session expiry/revocation and operational backups. The current one-read raw TCP request parser and broad CORS behavior require replacement/hardening before full browser writes or internet exposure. Host and browser restart/reconnect, competing transactions, uploads and role-denied actions must be tested.

## Implementation and rollback sequence

1. Record both hosting modes and inventory all commands. Add guarded HTTP transport without advertising a working portal.
2. Separate session/request ownership and share tested application handlers while preserving desktop IPC contracts.
3. Add typed authenticated HTTP dispatch, per-session uploads/import previews and browser-native file/print adapters.
4. Serve the shared application and bundled assets from `/staff`; replace the legacy portal only after end-to-end parity passes.
5. Add headless always-on hosting and configuration/health/shutdown support, then HTTPS deployment profiles for both hosting modes.
6. Verify the full feature/role matrix locally, across devices and through the configured internet address. Perform backup/restore migration and outage/reconnect acceptance before release.

Keep existing desktop IPC and the previous portal during the transition. No destructive schema change is authorized or required by the audit. Source rollback must preserve the existing library database and backups. Do not claim full browser functionality from frontend transport tests, a screenshot or build success.

Implementation checkpoint 2026-11-25T04:51:38.038Z: guarded HTTP client/inventory tests pass; desktop baseline 35/35 IPC passed before AppState Arc ownership refactor. Independent session contexts added with separate login/import slots and shared DB/runtime/log lifetime. Post-refactor state/IPC/security proof pending. No backend route advertised; full browser parity remains incomplete.

Shared-handler checkpoint 2026-11-25T05:04:30.091Z: 162 commands have common typed handlers and desktop wrappers; two native window exports remain separate. Original behavior proof: full 35 IPC and 7 security tests pass; three shared-context tests pass after correcting the administrator fixture. Preservation report compares all 164 command bodies (162 exact, two explicit borrowed-context reference adaptations). This is not an authenticated HTTP server or full browser parity.
