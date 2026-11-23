# Splash quotations and logo: resume plan

Current checkpoint, 23 November: owner resumed. Runtime rotation exists, with 20 primary-text reviewed entries enabled; source access/context limitations are explicit. The previous 113 unchecked candidates are archived outside runtime. See SPLASH_QUOTE_SOURCES.md and the newest handoff; historical pause/implementation statements below describe 21 November only. Target of approximately 100 authentic entries and logo refinement remain open.

Checkpoint: 21 November 2026. Owner paused work until their usage allowance resets. Save the plan now; do not start further implementation while paused.

## What the owner requested

Replace the splash closing line with a collection of about **100 genuine quotations about books or reading**, showing a different quotation each time the application opens. The owner explicitly requires authentic quotations, not invented ones. The proposed book/P logo is good but the owner is **not fully satisfied**; it remains a draft.

## Current implemented state

- StartupSplashScreen.tsx has a branded loading screen, a progress indicator and status text. Gate.tsx uses a three-second minimum on the normal startup path and also waits for the initial application queries.
- The technical badge reading Local Sovereign Architecture / Offline Ready was removed. The current closing text in code is **More time for books and readers.**
- The established main tagline **A Better Way to Manage Libraries.** remains unchanged.
- BrandMark.tsx still uses the old PUS/THK spine-label symbol. The generated book/P concepts have not been integrated as the application mark or installer/tray icons.
- Four focused splash tests, TypeScript typecheck and lint passed after the wording change. Evidence: splash-closing-line-checks in PROGRESS.json. No new native executable or updated splash screenshot was produced after that wording edit.
- The earlier current-splash.png preview shows the older architecture badge, before the wording edit. It must not be presented as a screenshot of the latest source.

## Quotations: agreed direction, not implemented

1. Curate 100 distinct, genuine quotations about books, reading or libraries. Do not pad the collection with generic motivational lines or duplicate fragments to reach the count.
2. Verify wording and attribution against original works or reliable primary sources. Exclude doubtful internet quotations and misattributions. Do not create or paraphrase a sentence and present it in quotation marks as authentic.
3. Keep an auditable record for every entry: stable ID, exact text, credited author, work/speech/letter, original publication details where established, source URL/locator, and verification notes. For translations, identify the translator/edition. Distinguish a fictional character's words from an author's personal statement.
4. Prefer works that can appropriately be reused; check rights and quotation limits for the chosen wording. A public-domain author does not automatically make a recent translation public domain. Do not promise legal clearance without evidence.
5. Bundle the reviewed collection inside the app. No network calls, external quote service, telemetry or runtime source fetching.
6. Select one quotation once per launch and keep it stable through progress updates. Avoid an immediate repeat of the previous launch when storage is available; handle unavailable/corrupt storage safely. Do not rotate several quotes during a single short splash screen.
7. Display the author beneath the quotation. Keep detailed source references in the local source/documentation record; avoid a cluttered startup interface.
8. Preserve the minimum three-second startup behavior. Favor short quotations that remain readable in the available time. Any longer display-duration policy needs an explicit design decision rather than an accidental delay.
9. Test collection integrity, exactly 100 reviewed entries, unique IDs/text, one selection per launch, no immediate repeat, storage fallback, accessible rendering and unchanged startup/error behavior. Automated schema tests do not prove that a quotation is authentic: source review is required too.

No quote collection, verified quote list, random selection logic, author display or persistence behavior has been implemented yet.

## Logo refinement

See LOGO_DESIGN_DIRECTION.md. The visual direction is a compact book-shaped P in technology teal, shelf ink and light paper, informed by the product's existing identity. The owner likes the direction but has not approved a final logo.

Next revision should explore a flatter, more distinctive silhouette, cleaner page-fold geometry and stronger clarity at small sizes. Avoid tiny initials, gradients/material shading and fine details that disappear at 24–32 px. Review monochrome and light/dark versions. Use relevant available design guidance; do not apply another company's branding guidelines to Pusthakam.

Two generated concept images are saved in the conversation's images: Teal P-shaped open-book symbol.png and Pusthakam flat book icon.png. These are review concepts; their names do not establish that the artwork is a final flat/vector production asset. Do not assume the image bytes exist in the Git repository or that they travel with a clone. Do not replace app, installer, tray or website icons until the direction is accepted and appropriate final assets are ready.

## First steps after resuming

1. Read AGENTS.md, CLAUDE.md, PROGRESS.json, PROGRESS.md and SESSION_HANDOFF.md; inspect Git status and preserve all other edits.
2. Confirm the current splash source still has the implemented closing-line change. Resume quotation sourcing and logo refinement, using the owner's feedback above.
3. Record partial progress and actual checks immediately in both progress files and the handoff; never claim 100 verified quotes before sourcing them.
4. Rebuild a native verification executable and render a fresh screenshot only after the next actual implementation. The earlier temporary preview and executables may be stale or absent on another computer.
5. Keep macOS/Linux package verification as the underlying product release milestone. Preserve the report-design and website work already recorded; do not use this branding task to overwrite newer manual work.

All changes are saved locally. No Git commit or push has been performed by this pause/checkpoint operation. Commit/push, then pull on the other computer, to carry source and developer documentation between machines; never commit library databases, credentials or real patron data.

## Logo decision - 23 November 2026

Owner approved the latest teal book/P for use throughout. Approval supersedes the historical unapproved-concept notes above. Exact master is branding/pusthakam-logo.png; integration/check status is in the latest handoff. Quote target and source-review work remain separate.

## Splash presentation - 23 November 2026

Owner authorized presentation polish: larger approved mark, stronger title, larger balanced quotation, simple loading words, gentle entrance and sign-in/setup reveal, with reduced motion respected. No longer startup hold. Existing 3000 ms minimum and query/error gating remain authoritative. Exact current checks and build status are in the latest handoff. Catalogue count stays 20; source review and expansion remain open.

Presentation verification: all 213 frontend tests, lint, TypeScript, frontend/native Windows builds and actual desktop/narrow/reduced-motion visual checks passed. Evidence: splash-polish-final-checks-2026-11-23. First wording assertions/preview-port issues are retained in progress evidence as resolved. No extra startup hold; quotation count remains 20.
