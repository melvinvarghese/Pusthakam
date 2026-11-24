# OPAC design concepts

Current status (2026-11-24T14:12:02.123Z): development and design work paused by owner. Existing concepts/prototypes are preserved; await explicit resume and owner selection before further OPAC implementation.

Owner request, 24 November 2026: prepare three visual UI directions and wait for selection before implementation. This authorizes concept exploration; no OPAC application code or LAN/firewall behavior changed. Preserve approved branding/pusthakam-logo.png as the production mark.

## 1. Open Shelves

Bright, approachable public catalogue. Palette: white #FFFFFF, cloud #F3F7F7, ink #172331, teal #168B85, pale teal #DBF1EE. Hanken Grotesk UI with restrained literary display type; book covers carry most visual interest. Large left-aligned search, modest topic browsing and a spacious cover grid; mobile uses one search column and a two-cover grid.

## 2. Reading Room

Literary discovery with deep teal and warm paper contrast. Palette: forest #123A36, paper #FAFAF6, teal #218F87, mint #D5E9DE, ink #172723. Newsreader-style serif display paired with Hanken Grotesk UI. Asymmetric search introduction, a quiet book-spine composition and horizontal reading selections. Mobile stacks the search before browsing. No decorative gradients or generic marketing statistics.

## 3. Catalogue Desk

Fast catalogue lookup with legible holdings data. Palette: white #FFFFFF, shelf ink #172331, slate #EEF2F6, teal #168B85, muted ink #65758A. Hanken Grotesk with Courier Prime only for classification/copy data. Compact search toolbar, optional filter rail and book-result rows showing author, classification, copies and shelf/location; mobile collapses controls into a filter button and readable result cards.

## Scope and acceptance for a later implementation

These are visual proposals with illustrative catalogue content, not live data or implemented features. All directions prioritize title/author/ISBN search, availability, location and book details already represented by public catalogue data. Cover retrieval, topic browsing, filters and curated selections require implementation review if chosen; current public payload has no cover field. Do not imply working patron sign-in, reservations, saved favourites or borrowing through these mockups. Actual production must stay bundled/local, keyboard-accessible and responsive; no remote font, stock-image or source-quotation requests at runtime. The remote-device ERR_CONNECTION_REFUSED issue is separate and remains unresolved. Wait for owner selection of 1, 2 or 3 before changing OPAC UI.

Generated previews are attached to the conversation for comparison. The exact approved logo asset remains unchanged; generated mockup placement is illustrative. No concept has been selected yet.

## Delivery checkpoint - 24 November 2026

All three numbered previews were generated and visually reviewed in the conversation. No direction selected yet. Concept 1 prioritizes approachable browsing, concept 2 literary atmosphere, concept 3 fast holdings lookup. Generated book-cover art and example classification/count data are illustrative; use actual records and the exact original logo asset in implementation. Image generation first failed on Windows local-path references, then succeeded with the displayed approved logo reference. No source code changed, and no application tests or native builds were required for these concept images.

## Superseding direction: coded prototypes

Owner rejected all three generated images and requested actual browser prototypes. The image concepts above are superseded, not approved. New standalone source is design/opac/: The Shelf (visual catalogue), The Index (compact result list), Reading Table (coverless editorial browsing). All operate on clearly marked demonstration holdings. Production OPAC/LAN is unchanged until selection. Search, filters, sort and dialogs are implemented locally. Browser review passed 12 responsive layouts and 24 interactions, with zero page exceptions or external page HTTP requests. Six actual browser screenshots and review-results.json are saved in design/opac/previews/. Run `node design/opac/server.mjs` from the repository root; open http://127.0.0.1:5193/design/opac/?view=shelf (or ?view=index / ?view=table). The restricted server exposes only prototype assets. Await owner selection. Search placeholder includes ISBN as a proposed production capability; current demonstration records do not include ISBN fixtures.
