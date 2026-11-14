# Pusthakam (പുസ്തകം) — Project Instructions

Offline-first, cross-platform desktop **library management system**. Tagline: "A Better Way to Manage Libraries."
The name "Pusthakam" is fixed. Never rename or abbreviate it. Never claim trademark/domain clearance.

## Stack
Tauri 2 · Rust backend · SQLite (rusqlite, bundled, WAL, FTS5) · React + TypeScript + Vite · Tailwind CSS · shadcn/ui · Lucide · Recharts · Vitest · Playwright · cargo test.

## Hard rules
1. **Fully offline.** No CDN, remote fonts, telemetry, cloud auth, or required network calls. All assets bundled.
2. **Frontend never touches SQLite.** All data access and business rules live in Rust, exposed via typed Tauri commands.
3. **Layering (Rust):** `commands/` (thin, auth + validation) → `services/` (business rules, transactions) → `db/` (SQL, migrations). No SQL in commands.
4. **Schema changes only via new migration files** in `src-tauri/migrations/`. Never edit an applied migration.
5. **Never hard-delete history** (loans, fines, payments, audit events). Use status / `archived_at`.
6. Dates: `YYYY-MM-DD` TEXT; timestamps: UTC ISO-8601 TEXT; money: INTEGER minor units. No floats for money.
7. Passwords: argon2id via the `argon2` crate. No custom crypto. No secrets or user data in logs or Git.
8. Permission checks happen in Rust commands, never only in the UI.
9. User data lives in the OS app-data directory, never in the install dir or temp.
10. Backups use SQLite's backup API (not raw file copy of a live DB). Never overwrite the only backup; snapshot before restore.
11. No fake data in production code paths. Dev seed data lives in a separate dev-only module/script.
12. No dead buttons or placeholder pages shown as finished modules. Unbuilt modules are hidden or clearly marked "not yet available".
13. Never mark a requirement VERIFIED without running its tests. Never weaken a test to pass a build.

## Conventions
- Keep files small and focused. Reusable UI components in `src/components`; feature code in `src/features/<module>`.
- Design tokens only (CSS variables in `src/styles/tokens.css`); light + dark themes. Palette: charcoal, ivory, sand, sage, neutral grey.
- Errors: Rust `AppError` enum → serialised `{code, message}` → frontend toast/inline error. No `unwrap()` in non-test code.
- Every bug fix gets a regression test.

## Commands (once toolchain installed — see docs/KNOWN_ISSUES.md KI-001)
- Dev app: `npm run tauri dev`
- Frontend tests: `npm test` · Rust tests: `cargo test --manifest-path src-tauri/Cargo.toml`
- Lint/typecheck: `npm run lint`, `npm run typecheck`

## Project docs
`PROGRESS.md` (status, update after each milestone) · `CHANGELOG.md` · `docs/` (PRODUCT_SPECIFICATION, ARCHITECTURE, DATABASE_DESIGN, DEVELOPMENT_ROADMAP, TESTING_STRATEGY, DECISIONS, KNOWN_ISSUES).
Statuses: NOT STARTED · IN PROGRESS · IMPLEMENTED · VERIFIED · BLOCKED.

## Environment notes
Developer machine is Windows 10. `git` is not on PATH; GitHub Desktop's bundled git is at
`%LOCALAPPDATA%\GitHubDesktop\app-*\resources\app\git\cmd\git.exe`. Do not push, release, or force-anything without approval.
