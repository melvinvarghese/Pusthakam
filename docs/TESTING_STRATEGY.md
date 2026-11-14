# Testing Strategy

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
