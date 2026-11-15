---
version: 1
slug: "src-app-appshell-tsx"
primary_target: "src/app/AppShell.tsx"
related_targets: ["src/styles/index.css","src/features/dashboard/Dashboard.tsx"]
---

# Surface brief: Pusthakam application shell (all screens)

Mode: Operate. Visitor: a librarian at a desk, scanner in hand, 8 hours a day, dense records, queue pressure. Task: scan, look up, issue, return, catalogue, report. Constraints: offline, desktop 1024x640+, Tailwind v4 tokens, bundled fonts only, WCAG AA, light + dark. Chosen world: "Spine Label" (user confirmed in chat 2026-11-15; the card was IMPECCABLE'S PICK). Reference sketch: .impeccable/mocks/decision/spine-label.html.

## Direction contract

THESIS: A library tool should look like the shelf it serves: every book is identified by the stacked call-number label a librarian already reads at a glance. It refuses the pale admin-dashboard default (icon-and-card grid, sage-on-cream, Inter-everywhere).

OWN-WORLD: Deep shelf-ink (#16202e) rail and top bar, white work area (deep navy in dark theme), hairline cool-grey rules, label-stock white for labels, one grotesque (Hanken Grotesk) for UI and one typed monospace (Courier Prime) reserved for call numbers and accession codes. Ten Dewey class colours exist only on label bands; red and amber appear only for overdue and fines. Recognisable with content removed: the dark ink frame and a white label with a coloured top band.

STORY: A librarian understands within a glance where they are, what state the library is in, and trusts that data stays on this computer. They do their work from the keyboard.

FIRST VIEWPORT: 232px ink rail (brand, library name, four labelled groups of text-only nav, offline footer), 56px ink top bar (search-first trigger, theme control), white work area with a 28/20px page title and panels on a 24px grid. Primary action sits top right of its panel. The stacked call-number label is the signature component wherever a book appears.

FORM: Seed seed:7c93dab10bbfe9cd, IMPECCABLE'S PICK card "Spine Label" (my candidate 1 of 7, not the assigned direction); chosen by the user in chat.

FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance

## Unresolved
Final Dewey colour mapping for 10 classes (configurable per library later); real library name and user chip appear when Phase 2 and staff accounts ship.
