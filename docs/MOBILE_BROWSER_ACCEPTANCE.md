# Mobile OPAC and staff workspace

The freshly compiled application passed115 checks in Edge with touch emulation against an isolated real server/database on25 November2026. STAFF_BROWSER_REVIEW.json records the exact binary SHA256, individual checks and limitations. The fresh Windows installer contains these changes; the same115 checks and11 headless restore/delayed-packet checks pass against its exact bundled release server SHA256. WEB_RELEASE_VERIFICATION.json records installer/server hashes. The owner installed application has not been replaced automatically.

| Area | Change | Evidence |
|---|---|---|
| Staff frame | Sidebar collapses below1024px; full-width workspace; modal navigation with inert background | All14 staff sections and internal role=tab views exercised at320,390,768px with real APIs and no workspace overflow |
| Navigation focus | Restoration after navigation, Escape and backdrop dismissal | Real touch checks; backdrop prevents default mousedown focus loss; AppShell component regressions |
| Controls/authentication | 44px buttons/fields,16px editable text, compact account header | Real book dialog geometry, populated member actions, touch signout/keyboard-input sign-in and landscape menu |
| Catalogue/members | Labelled phone records, readable titles, wrapping member actions | Real seeded records and visually inspected screenshots |
| Search | Desktop autofocus retained; phone focus user initiated | Actual browser focus assertions; route entry no longer requests a phone keyboard |
| Tabs/toolbars | Internally scrollable tabs; wrapping filters/actions | Real review caught/fixed Classification scheme controls, Acquisitions tabs/Purchase Orders toolbar and Periodicals filters |
| Forms/tables | Single-column forms, bounded dialogs, internally scrollable wide tables,16px page padding | Book dialog/route/tab geometry; Classification MARC table visually reviewed |
| Public OPAC | Compact search/holdings and reachable details | Actual embedded public API results/details/privacy at320,390,768px; empty search/reset; earlier template fixture review24 checks |
| Browser backups | Download actual SQLite snapshot; verify; queue/cancel restore with host instructions | Real API/browser transport checks |
| Optional hosting | Each library can configure its own public HTTPS origin; library PC remains default | PUBLIC_WEB_HOSTING_FEATURE.md; no personal-domain dependency or live public deployment implied |

Actual-app screenshots: design/mobile/previews/actual-staff-*-390.png and actual-opac-390.png. Earlier review-mobile-layout.mjs output uses source classes/mobile CSS and old compiled base CSS: historical fixture geometry, not functional proof.

The full frontend suite passed252 tests before the shared backdrop correction. The subsequent253-test run passed42 files and failed2 still-loading query checks in Dashboard/Settings under concurrent build contention. No assertions or timeouts were weakened. A silent one-worker full rerun was stopped after prolonged execution and an unavailable report path; focused unchanged Dashboard/Settings/AppShell rerun passed all27 tests in75.12s, including both previously failing checks and the new backdrop regression. Full-project ESLint passes. This is combined full-suite/focused-rerun evidence, not a single all253-pass run. Fresh TypeScript/Vite/debug-server builds pass. Owner explicitly approved the previously blocked tooling connection for this rebuild; do not request the same approval again.

Remaining: physical Android/iPhone and Safari, real soft keyboard/zoom/screen readers, complete circulation/import/upload workflows, printing/scanning and public HTTPS deployment. Touch emulation does not prove these. Owner already confirms both same-network portals work on another device; do not repeat firewall changes.
