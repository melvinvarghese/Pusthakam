# Known Issues

| ID | Severity | Issue | Status |
|---|---|---|---|
| KI-001 | Blocker | Toolchain missing (Rust, MSVC Build Tools) and low disk on `C:`. User approved install to `D:\dev` on 2026-11-14; Rust installed and verified, Build Tools install in progress. | IN PROGRESS |
| KI-002 | Low | `git` not on PATH. GitHub Desktop's bundled git works (path in CLAUDE.md). | Workaround |
| KI-003 | Info | Node v26 is very new; frontend typecheck/lint/tests pass with it. | Monitoring |
| KI-004 | Medium | Tauri commands (`get_setting`, `set_setting`, `app_info`) are unauthenticated. Acceptable only while no library data exists; every data command added after Phase 2 must enforce permissions in Rust. | Open until Phase 2 |
| KI-005 | Low | App icon is a generated placeholder (open book on charcoal). Final brand icon needs design before release. | Open |
| KI-006 | Low | `npm audit` reports 7 moderate advisories in dev dependencies; to be reviewed before release (none ship in the app bundle unless proven otherwise). | Open |
| KI-007 | Low | C: drive has under 10 GB free; build output is redirected to D: via `CARGO_TARGET_DIR`. `node_modules` and `dist` remain in the repo on C:. | Monitoring |
