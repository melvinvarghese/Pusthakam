# Splash quotations: design

## Intent

Replace the current splash-screen closing line with one authentic quotation about books, reading, or libraries. The quotation should make Pusthakam feel more human without delaying startup or requiring an internet connection.

## Goals

- Bundle exactly 100 reviewed quotations with the application.
- Show one quotation and a compact author credit per app launch.
- Keep the chosen quotation stable while the splash is visible.
- Avoid repeating the immediately previous quotation when local storage is available.
- Preserve the existing three-second minimum splash duration, accessibility status updates, main tagline, and offline startup.
- Keep an auditable local source record for every quotation.

## Non-goals

- No runtime quote fetching, online service, telemetry, or generated quotations.
- No logo replacement or installer-icon work in this change.
- No longer splash duration to accommodate long quotations.
- No claim of copyright clearance beyond the use of short, reviewed public-domain excerpts.

## Approaches considered

1. **Public-domain primary-source collection — chosen.** Curate short quotations from authors and works that are demonstrably public domain, linking each entry to a primary digital edition or authoritative archive. This is the most reliable option for accuracy, offline bundling, and rights risk.
2. Mixed modern and public-domain quotations. This would give broader representation, but would need individual rights analysis and stronger source checks. It is not appropriate for the first collection.
3. A remote quote service. This conflicts with Pusthakam's offline-first promise and is excluded.

## Design

### Data

Create a local quotation module with exactly 100 entries. Each entry has:

- stable ID
- exact quotation text
- credited author
- work title and publication context where known
- source URL or locator
- verification note

All entries must be distinct. The audit record is source code or a companion documentation file; the splash itself shows only the quotation and author.

### Selection

At application startup, select one quote from the bundled collection. Read the previous quote ID from local storage. Exclude it when selecting the next quote, then persist the chosen ID. If storage is blocked or corrupt, still select a valid quote safely. Selection happens once per launch; it does not rotate during the splash.

### Presentation

The quotation replaces the current closing line in `StartupSplashScreen`. It appears in the existing lower splash area, with the author underneath. Text length is limited so it remains readable during the existing three seconds. The established primary tagline, **A Better Way to Manage Libraries.**, remains unchanged.

### Tests

- collection contains exactly 100 entries with unique IDs and text
- all required source metadata is present
- selection avoids an immediate repeat
- selection handles missing or corrupted local storage
- a splash render shows the quote and author
- the existing minimum-duration and accessibility behavior remains intact

## Implementation decisions

- Use the existing React and local browser-storage stack; add no network or third-party quotation dependency.
- Source the quotations only after a deliberate source review, using original texts or authoritative archives.
- Do not include a quotation if wording, attribution, source, or public-domain status cannot be supported.
