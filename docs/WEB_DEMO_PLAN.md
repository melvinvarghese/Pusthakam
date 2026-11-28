# Online web demo plan (GitHub Pages)

Owner request, 28 November 2026: "we have to setup a demo staff account and opac (with a demo member login). so users can experience the app before downloading the app. this should be a web demo done on GitHub pages (for now)."

Status: **NOT STARTED.** Planning only. Nothing below is built or published. Outside v1 product scope: no roadmap weight or completion credit. Build it only when the owner asks.

## What the visitor gets

From the public website (https://melvinvarghese.github.io/Pusthakam/), a **Try the demo** button opens a demo page with two ways in:

| Demo | Address (proposed) | Demo sign-in | What they can do |
|---|---|---|---|
| Staff app | `/Pusthakam/demo/app/` | Username `demo.librarian`, password `demo1234` | Dashboard, catalogue search and book records, copies, members and member profiles, issue / return / renew / reserve, fines, overdue list, reports on screen. |
| Library catalogue (OPAC) | `/Pusthakam/demo/opac/` | Member number `DEMO-001`, PIN `1234` | Search, browse categories, book details with "Where to find it", My account (loans, fines, holds), Request hold and Cancel hold. |

The credentials are printed on the demo page and pre-filled hints on both sign-in screens. They are not secrets: the demo has no server and protects nothing.

Credentials and URLs above are proposals; confirm or change them before building.

## How it works

GitHub Pages serves only static files, so there is no Rust backend and no SQLite in the demo. The demo is the real React frontend and the real OPAC page, with an **in-browser stand-in backend** that answers the same commands from seeded sample data.

### Staff app

- New build mode: `npm run build:demo` (`vite build --mode demo`), with `base: "/Pusthakam/demo/app/"` and `VITE_PUSTHAKAM_DEMO=true`. Output goes to `dist-demo/`, never to `dist/`.
- `call()` in `src/lib/api/index.ts` already has one seam for the dev `?preview` stand-in. Add a second branch: when the demo flag is set, dynamically import `src/demo/backend.ts` and answer from it. The flag is a build-time constant, so the normal desktop build drops the demo code and data (same mechanism as D-023). Add a test that a production `dist/` build contains no demo seed strings.
- `src/demo/` holds: `seed.ts` (sample library), `store.ts` (state), `backend.ts` (one handler per supported command). It can grow out of `src/features/dev/preview.ts`, which today answers only about a dozen of the ~176 commands.
- Handlers follow the same types as `src/lib/api/types.ts`, so `npm run typecheck` catches shape drift. Business rules (loan period, max loans, renewals, fine per day, hold queue) are applied from the seeded circulation policy table in a simplified form. The demo is an illustration; the Rust services stay the only source of truth.
- Commands the demo does not support return a clear error `DEMO_UNAVAILABLE`: "This is not part of the online demo. Download Pusthakam to try it." Out of scope for the demo: imports, exports and file downloads, backup and restore, printing to hardware, LAN and hosting settings, adding staff, online ISBN and cover lookups, Malayalam translation workflow.
- Read-only in the first version: acquisitions, periodicals, classification and authorities, inventory, audit log, settings.
- A thin banner on every demo screen: "Online demo with sample data. Changes stay in this browser." with a **Reset demo** button.

### OPAC

- The production OPAC lives in `src-tauri/src/services/opac/` (`index.html`, `styles.css`, `catalogue.js`) and calls eight endpoints: `/api/opac/categories`, `/api/opac/search`, `/api/opac/cover/{id}`, and `/api/opac/patron/{session,login,logout,hold,cancel-hold}`.
- The demo build copies those three files into `dist-demo/opac/` unchanged, and adds a small `demo-shim.js` loaded first that answers those `fetch` calls from the same seed and store as the staff demo. Production OPAC files are not edited for the demo, except, if needed, making the API base path configurable.
- Covers: no network calls. Use the existing placeholder cover style, or generated SVG covers. No Open Library requests from the demo.

### Shared state and reset

- Staff app and OPAC are on the same origin, so they share one store in `localStorage` (key e.g. `pusthakam-demo-v1`). A hold placed as `DEMO-001` in the OPAC shows up in the staff app's reservations, and an issue in the staff app shows up in My account. That round trip is the most convincing part of the demo.
- Each visitor gets their own copy; nothing is sent anywhere. **Reset demo** (and a bump of the key version on each deploy) restores the seed.
- Alternative if the owner prefers: read-only demo, or reset on every page load. Recommended: editable per browser with Reset, because issue / return is what librarians want to try.

### Sample data

- Demo library name: "Pusthakam Demo Library".
- About 80 titles and 150 copies spread over the default browse categories (Malayalam Fiction, Malayalam Poetry, English Fiction, Children's Books, Physics, Mathematics, Management & Business, Competitive Exams, Reference, and others), with DDC call numbers so spine labels look real.
- About 30 members with clearly sample names (for example "Demo Reader", "Sample Student 12"), never realistic personal names, phone numbers or emails. `DEMO-001` has two current loans, one overdue loan with a small fine, and one hold.
- Some open loans, some overdue, a few fines, holds and a reservation queue, so the dashboard and reports have something to show.
- Seed data lives only in `src/demo/`, per CLAUDE.md rule 11.

### Publishing on GitHub Pages

- Extend `.github/workflows/pages.yml`: set up Node 22.22.2, `npm ci`, `npm run build:demo`, then assemble one Pages artifact: `website/` at the root and `dist-demo/` under `demo/`. Add `src/**`, `src-tauri/src/services/opac/**`, `package*.json` and `vite.config.ts` to the trigger paths.
- `website/demo/index.html` is the landing page (indexable, in `sitemap.xml`). The app and OPAC demo pages carry `<meta name="robots" content="noindex">` so sample records do not show up in search results.
- Website links: **Try the demo** next to the main button on Home and on Get Pusthakam, and a line on Status. Wording must say it is a demo with sample data that runs in the browser; the real Pusthakam runs on the library's own computer. No "free" wording (commercial decision, 27 November 2026).
- GitHub Pages for a private repository needs a paid GitHub plan. The site already depends on the repository staying public; the demo adds the minified frontend bundle to the public site (no Rust code). Revisit both when moving to paid hosting at public launch.

## Build order

1. OPAC demo plus the demo landing page and website links. Smallest piece: eight endpoints.
2. Staff demo core: sign-in, dashboard, catalogue, members, circulation, fines, overdue, reports on screen.
3. Read-only screens for the remaining modules, banner and Reset, Pages workflow, `noindex`.
4. Owner review on a laptop and a phone, then publish.

## Acceptance checks

- Both demo sign-ins work with the printed credentials; wrong credentials show the normal error.
- Issue, return, renew, reserve and a fine payment work in the staff demo and are reflected in the OPAC's My account, and the other way round for holds.
- Reset demo restores the seed. A new browser starts from the seed.
- No network requests other than the demo's own static files (check the browser network tab).
- Unsupported actions show the `DEMO_UNAVAILABLE` message, never a broken screen or the "needs the desktop app" error.
- `npm run build` (desktop) output contains no demo code or seed data; existing Vitest, typecheck and lint stay green.
- OPAC demo usable on a phone (390 px wide). Staff demo usable at 1280 px and up.
- Direct loading of `/Pusthakam/demo/app/`, `/Pusthakam/demo/opac/` and a hash route such as `#/catalogue` works on Pages.

## Open owner decisions

- Credentials and URLs above, or different ones.
- Editable per browser with Reset (recommended), read-only, or reset on every load.
- Which modules besides the core ones should be in the first version.
- When to build it, relative to the Monday queue (Member Profile Batch 3, Malayalam reconciliation, PENDING_WORK section 1).
