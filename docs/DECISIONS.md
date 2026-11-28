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
| D-010 | Fonts bundled locally via npm (`@fontsource-variable/hanken-grotesk` for UI, `@fontsource/courier-prime` for call numbers); Malayalam content falls back to OS fonts. The Malayalam brand word (പുസ്തകം) was removed from the app interface at the owner's request on 2026-11-15; it remains a brand fact only. | Accepted |
| D-019 | Visual world "Spine Label" chosen by the owner on 2026-11-15 after a decision round (alternatives: Ola Leaf, Terminal Yellow, Lexicon Shoulder, standard). Replaces the rejected ivory/sage/Inter shell. See `.impeccable/surfaces/src-app-appshell-tsx.md`. | Accepted |
| D-011 | TanStack Query + react-hook-form + zod on the frontend; Rust is the source of truth for validation. | Accepted |
| D-012 | Marc21/ISO 2709 and authority data deferred to Phase 6; schema keeps contributor/subject/classification normalised so import maps cleanly. | Accepted |
| D-013 | Toolchain (Rust 1.100 via rustup, VS 2022 Build Tools C++ workload) installed with user approval on 2026-11-14 to `D:\dev` (low free space on C:). Env vars set at user level: `RUSTUP_HOME`, `CARGO_HOME`, `CARGO_TARGET_DIR=D:\dev\cargo-target`; `D:\dev\cargo\bin` added to user PATH. | Accepted |
| D-014 | Pinned older majors for ecosystem compatibility: TypeScript ~5.9 (v7 is a native rewrite; typescript-eslint support unproven), ESLint 9 (eslint-plugin-react-hooks peers stop at 9), Tauri 2 (crates.io latest is 3.0 alpha). | Accepted |
| D-015 | Bundle identifier `app.pusthakam.desktop` is a technical string only (no domain claim). **It determines the OS data folder, so it must be final before the first real release** or users' data location changes. | Accepted |
| D-016 | Hash-based routing (`HashRouter`) since the app is served from a custom protocol, not a web server. | Accepted |
| D-017 | Theme preference currently in `localStorage` (per-viewer convenience); move to `application_settings` once the settings UI exists. | Accepted |
| D-018 | Unbuilt modules stay visible in navigation but open an honest "not available yet (phase N)" page, so the product's shape is visible without fake functionality. | Accepted |

| D-020 | "Online Tools" menu (owner-approved 2026-11-15): a rail-footer item listing optional web tools (DDC list, Library Toolkit). Links open in the system browser via `tauri-plugin-opener`, restricted by `capabilities/default.json` to `melvinvarghese.in` and `melvinvarghese.github.io`; every link carries `utm_source=pusthakam`. Only the address is opened; no library data is sent. Core functions never depend on it. New tools need an entry in `src/features/online-tools/tools.ts` and a capability entry. The permanent "Offline" text was removed from the UI at the owner's request. | Accepted |

| D-021 | Sign-in session is held only in Rust memory (`AppState.session`); closing the app signs everyone out. No token is stored in the browser. Permission checks live in the command layer (`AppState::require`). Lockout: 5 failed attempts lock the account for 5 minutes. | Accepted |
| D-022 | Setup wizard creates everything in one `BEGIN IMMEDIATE` transaction and can run once. Library-profile defaults are plain data (`domain/profiles.rs`), editable afterwards; an item category without a policy row for a member category means "not lendable". Fine defaults to 0 (no fines) so no currency is assumed. | Accepted |
| D-023 | Frontend dev preview (`?preview`) serves synthetic data in a plain browser for design review; gated by `import.meta.env.DEV` so it is absent from production. The command layer is tested separately through Tauri's mock IPC (`src-tauri/tests/ipc.rs`). | Accepted |
| D-024 | `argon2` pinned to 0.5 (0.6 changes the API); `rand_core` 0.6 with `getrandom` for salts. Argon2 compiled with `opt-level=3` even in debug builds. | Accepted |
| D-025 | Import from other systems (Koha first) is a first-class goal: MARC21/ISO 2709/MARCXML import moves to Phase 3. See DEVELOPMENT_ROADMAP "Migration from other systems". | Accepted |

| D-026 | Multi-computer use is a server/client design (Phase 7b), never a shared database file: SQLite over SMB/NFS is unsafe for several writers. One Library Server owns the database; clients use an encrypted LAN API with per-connection sessions. Until then the app is single-computer and the in-memory single session is acceptable. | Accepted |

## Deviations from the brief
- The brief says "completely offline"; LAN mode (D-026) uses the local network only and never needs the internet, and is off by default.
- Barcodes are drawn by the JsBarcode library (Code 128, bundled locally) instead of hand-written Rust, because it is widely tested; the data (barcode text) is still owned by the backend.
- The brief prohibits online dependencies; the Online Tools menu is an explicit, owner-approved optional exception (D-020), links only.
None so far.

## LAN browser and OPAC extension, 2026-11-18

Owner requested phone/computer browser access for staff and anonymous OPAC for people on the same reachable local network. Extend Phase 7b: one host database, browser staff sessions with server permissions, separate read-only public catalogue contract, locally served assets, host enable controls and certificate trust guidance. Desktop Tauri invoke and single in-memory session are not a browser API. No cloud dependency or public-internet hosting implied. Implementation remains not started.

## V3 hosting and staff browser parity — owner decision, 25 November 2026

Support both local network and internet access for OPAC and the full role-permitted staff app. Default host is the library PC; optionally deploy the same application services on an always-on host. Reuse the staff frontend and typed contracts with independent browser sessions. One authoritative database; explicit verified backup/restore migration when changing host. This supersedes earlier statements that public-internet access is outside all planned scope, while preserving optional/offline PC operation and v1 accounting. No deployment or public domain has been configured.

## OPAC browse categories replace DDC class buttons — owner decision, 27 November 2026

Melvin: members "don't wanna know the numbers. they just want to know the classification/subjects". Three options were offered (relabel the 10 DDC classes without numbers, staff-managed categories, or staff-picked subject headings); the owner chose **staff-managed browse categories**.

- A new `browse_categories` list, separate from subject headings and class numbers, is what members browse in the OPAC. It ships with 31 defaults (Malayalam Fiction, Malayalam Poetry, English Fiction, Children's Books, Physics, Mathematics, Management & Business, Competitive Exams, Reference, ...). Staff add, rename, re-rule or archive them under Classification & Authorities → Browse Categories.
- Each category may carry DDC prefix rules (e.g. Physics = 53, Malayalam Fiction = 894.8123). Rules place existing and new books automatically, matching only DDC (or unspecified-scheme) class numbers. Staff can add or remove categories on any book from its record form; a removal is stored as `excluded` so rules do not add it back.
- The OPAC shows only categories that hold visible books, with counts, and never shows class numbers in result rows. The book detail shows "Where to find it" (location plus shelf call number). Staff screens, spine labels and exports keep DDC unchanged.
- Later: an optional parent level (Fiction → Malayalam Fiction) can cover the "subject hierarchies" item in PENDING_WORK; Malayalam display names for categories are not built yet.

## Online web demo on GitHub Pages — owner request, 28 November 2026

Melvin asked for a demo staff account and an OPAC with a demo member login, so people can try Pusthakam before downloading it, as a web demo on GitHub Pages for now. Proposed approach (not yet confirmed by the owner): static demo build of the real frontend and OPAC with an in-browser stand-in backend and sample data, kept out of desktop builds the same way as the D-023 preview. See WEB_DEMO_PLAN.md. Not started.
