# Testing Strategy

## Standalone and Wi-Fi acceptance

A cargo build success does not prove standalone launch: Tauri dev mode expects a Vite server. Build with npm.cmd run build:windows or npm.cmd run build:standalone-debug, then verify the app loads with no listener on development port 1420. Confirm the configured OPAC listener is running, the host-local OPAC and its network-address URL work, and a second device on the same Wi-Fi opens the exact displayed shared URL. Never use 127.0.0.1 from a remote device. Verify internet-disconnected search independently of optional ISBN covers. Do not label a same-host TCP test as a second-device Wi-Fi test.

Cover tests must check stored/offline images, automatic ISBN lookup, read-only thumbnail display, invalid/missing ISBN, provider failure, retry, and public cover denial when OPAC is disabled or the record is archived. Browser cover tests block the provider so tests are deterministic and do not prove provider availability. Saved covers, valid fallback URL construction and offline placeholders are covered separately.


## Staff HTTP and console-host verification

Build the frontend before compiling the server; do not replace dist during native compilation. `cargo test --manifest-path src-tauri/Cargo.toml --test staff_web` exercises the actual TCP gateway, session separation, role enforcement, cookies/CSRF/origins, public proxy boundary, backup downloads, atomic public-address configuration and 100 ms delayed headers/body (KI-025).

The existing 35-test IPC contract suite can also use real HTTP by setting `PUSTHAKAM_TEST_TRANSPORT=http`. In that mode run it with these five exclusions: `manual_backups_are_permission_checked_complete_and_retained`, `restore_waits_for_other_instances_and_preserves_the_previous_database`, `app_info_works_before_sign_in_and_reports_the_current_schema`, `lan_full_workflow_via_ipc`, and `lan_settings_start_and_stop_the_actual_http_listener`. These assert native folder paths or a stopped/default listener and are retained in the unmodified desktop run. The 30 remaining tests cover full application workflows through HTTP, including imports and permission failures. Clear the environment variable before the native run.

`node tools/review-staff-web.mjs` launches an isolated console host and Edge profile, creates a temporary library and checks all staff module screens, mobile layout, backup creation/download/verify and restore queue/cancellation. `node tools/test-headless-restore.mjs` verifies an actual process stop/restart applies a queued snapshot, restores account access, removes a post-snapshot change and retains the pre-restore recovery copy. Both default to the debug server executable; set `PUSTHAKAM_SERVER_BINARY` to test a release executable. They never use the owner's database. Results are saved in STAFF_BROWSER_REVIEW.json and HEADLESS_HOST_REVIEW.json.

Actual second-device Wi-Fi access, configured public HTTPS ingress, printer/scanner behavior, host migration and outage/reconnect remain operational acceptance checks. A spoofed-proxy test on loopback is a gateway boundary check, not a deployed TLS test.

## Principles
Build success ≠ working app. Domain logic is pure and unit-tested; database behaviour is tested against real SQLite (in-memory and temp-file); workflows are tested end-to-end through the service layer; UI is tested in a browser. Tests are never weakened to pass; each bug fix adds a regression test.

## Layers
| Layer | Tool | Covers |
|---|---|---|
| Rust unit | `cargo test` | due-date calc (holidays, closed days), fine calc (grace, cap), accession formats, barcode generation, validation, permission matrix, classification utils |
| Rust DB | `cargo test` + temp SQLite | migrations (fresh + upgrade + checksum tamper), FK enforcement, UNIQUE (accession, barcode, active loan), transactions/rollback, soft-delete, history preservation, FTS sync, backup/restore round-trip, corrupted backup rejection |
| Rust integration | `cargo test` (services) | Workflow: create library → record → copies → member → issue → renew → return → report → backup → restore |
| Frontend unit | Vitest + Testing Library | api wrappers, forms/validation, tables, theme, shortcuts |
| UI e2e | Playwright (Vite dev server, IPC stubbed) | navigation, forms, search, filters, dialogs, keyboard, light/dark |
| App e2e | Manual checklist (+ tauri-driver where feasible) | install, launch, persistence across restart |
| Offline | Network disabled run + static check | no remote URLs in bundle (CI grep for `http(s)://` outside allowlist), CSP has no remote origins, app works with adapter disabled |
| Security | review + tests | argon2 hashes only, permission denied paths, path traversal rejection, SQL parameterisation (no string-built SQL) |

## Acceptance gates
- CI: `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`, `npm run typecheck`, `npm run lint`, `npm test`, Playwright smoke.
- A requirement becomes VERIFIED only after its mapped tests ran green and results are recorded in PROGRESS.md.
- Platform support claims require a build + smoke test on that OS.

## Linux package acceptance

CI runs `bash tools/release/linux-package-smoke.sh` after packaging. Exactly one Debian package is required; absent packages fail. It verifies package identity, installed binary/desktop/icon assets, launches and restarts the binary under Xvfb with isolated XDG directories, checks migrated database integrity, then uninstalls and checks binary removal. Failures clean up the package. Temporary logs/data remain for diagnosis; no owner library is used.

Process/database checks do not prove WebView rendering or first-launch setup. REL-LINUX requires a native Ubuntu walkthrough: install, visually inspect setup, configure a disposable library, sign in, restart and check persistence, then uninstall. AppImage runtime checks remain separate. Run tooling regressions with `python -m unittest discover -s tools/release -p 'test_*.py' -v`; fake processes/databases are used. Windows runs the three Python checks and skips the Bash wrapper test if Bash is absent. The earlier Linux session passed all four checks; native Linux compilation was blocked by unavailable prerequisites/system-package permissions.

## Mobile source and fresh-bundle distinction

See MOBILE_BROWSER_ACCEPTANCE.md. The OPAC review uses current production template/CSS/JS with explicit public API fixtures; the staff geometry review uses source shell classes/mobile CSS plus the last compiled base CSS and fixture records. It cannot replace React interaction tests or a rebuilt application. Record source geometry, component tests, fresh server/browser checks and physical phone/keyboard acceptance separately; never infer mobile usability merely because document.scrollWidth fits the viewport.


Mobile release verification25 November2026: the compiled embedded app passes115 Edge touch-emulation checks against a temporary real library at320/390/768px; exact release also passes11 headless delayed-packet/restore checks. Reports and binary hashes are in WEB_RELEASE_VERIFICATION.json. Actual screenshots are under design/mobile/previews/actual-*.png. Earlier source-layout fixtures are not functional app proof. Physical phones, Safari, actual soft keyboards and complete workflow/hardware acceptance remain separate.
