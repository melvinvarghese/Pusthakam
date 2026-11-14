# Architecture Decision Records

| ID | Decision | Status |
|---|---|---|
| D-001 | Stack: Tauri 2 + React/TS + Rust + SQLite as specified. | Accepted |
| D-002 | `rusqlite` with `bundled` feature (ships SQLite with FTS5; no system dependency, same SQLite on all OSes). | Accepted |
| D-003 | One database file = one library in v1. A `libraries` table holds a single row; scoped uniqueness (e.g. accession numbers) is enforced database-wide. Avoids `library_id` on every table; multi-library can be added by opening different DB files. | Accepted |
| D-004 | Hand-written mirrored DTOs + contract test at first; evaluate `tauri-specta` for generated bindings in Phase 1 and adopt only if stable with Tauri 2. | Proposed |
| D-005 | Passwords: `argon2` crate (argon2id, per-user salt). Sessions in Rust memory only. | Accepted |
| D-006 | PDF generation approach deferred to Phase 5 after a spike (print-HTML vs Rust PDF crate). CSV/XLSX in Rust. | Open |
| D-007 | Dates `YYYY-MM-DD` TEXT; timestamps UTC ISO-8601 TEXT; money INTEGER minor units; booleans INTEGER 0/1. | Accepted |
| D-008 | Primary keys: INTEGER autoincrement internal IDs; human-facing identifiers (accession no., member no., barcode) are separate UNIQUE columns. | Accepted |
| D-009 | Soft-delete (`archived_at`) for catalogue/members/vendors; circulation, fines, payments, audit are append-only. | Accepted |
| D-010 | Fonts bundled locally via npm (`@fontsource-variable/inter`); Malayalam brand word rendered with OS fallback fonts, never required. | Accepted |
| D-011 | TanStack Query + react-hook-form + zod on the frontend; Rust is the source of truth for validation. | Accepted |
| D-012 | Marc21/ISO 2709 and authority data deferred to Phase 6; schema keeps contributor/subject/classification normalised so import maps cleanly. | Accepted |
| D-013 | Toolchain (Rust 1.100 via rustup, VS 2022 Build Tools C++ workload) installed with user approval on 2026-11-14 to `D:\dev` (low free space on C:). Env vars set at user level: `RUSTUP_HOME`, `CARGO_HOME`, `CARGO_TARGET_DIR=D:\dev\cargo-target`; `D:\dev\cargo\bin` added to user PATH. | Accepted |
| D-014 | Pinned older majors for ecosystem compatibility: TypeScript ~5.9 (v7 is a native rewrite; typescript-eslint support unproven), ESLint 9 (eslint-plugin-react-hooks peers stop at 9), Tauri 2 (crates.io latest is 3.0 alpha). | Accepted |
| D-015 | Bundle identifier `app.pusthakam.desktop` is a technical string only (no domain claim). **It determines the OS data folder, so it must be final before the first real release** or users' data location changes. | Accepted |
| D-016 | Hash-based routing (`HashRouter`) since the app is served from a custom protocol, not a web server. | Accepted |
| D-017 | Theme preference currently in `localStorage` (per-viewer convenience); move to `application_settings` once the settings UI exists. | Accepted |
| D-018 | Unbuilt modules stay visible in navigation but open an honest "not available yet (phase N)" page, so the product's shape is visible without fake functionality. | Accepted |

## Deviations from the brief
None so far.
