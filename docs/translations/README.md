# Malayalam interface

## Latest review file: 519 drafts

2026-11-27T03:24:51.045Z

Use [Pusthakam_Malayalam_Drafts_519.xlsx](Pusthakam_Malayalam_Drafts_519.xlsx). Original 319 rows and their review cells are preserved; 200 new source-verified draft phrases are appended. Filter Batch to New 200 to review only additions. The 319-row original stays unchanged.

The additions in additional-draft-pairs.json / additional-draft-review.json are review-only and are NOT integrated in the app. Earlier core source coverage remains unchanged. Mark OK/Changed/Question and use Your correction/Notes; unreviewed rows remain drafts. Development is still paused awaiting review/explicit resume. Reconcile returned IDs and source wording before applying any approval.


English is the default. Malayalam is optional and selected per device in setup, sign-in or Library Settings. The setup request also stores the selected library locale; selecting a language on another device does not change that library record.

Use everyday Malayalam and familiar terms such as library, barcode and password. `approved-glossary.json` contains the 20 terms explicitly approved by the owner. Those terms override drafts. Further wording is a draft for small contextual reviews; the owner is not expected to translate the entire spreadsheet.

## Implemented source coverage

| Area | Current scope |
| --- | --- |
| Setup and authentication | Setup steps, sign-in, account recovery, selectors |
| Shared interface | Navigation, command search in either language, theme labels, startup messages, pagination |
| Dashboard | Counts, date controls, circulation alerts, module links and status labels |
| Catalogue | Search, results, availability, record details, record form, saved searches and archive confirmation |
| Circulation | Issue/return/renew tabs, lookup dialogs, confirmation dialogs, completion messages, current and overdue loan lists |
| Members | List, search, registration/edit/archive, loan history |

Copy editing/adding, cover messages, fines, reservations, imports/exports, printed documents, most settings, other modules and the independent public OPAC still need further work. Backend errors without a matching translation remain in English. Original quotations remain as quoted; no translated quotation is presented as an original.

## Implementation rules

- Draft resources live in `src/lib/setup-translations.ts` and `src/lib/core-translations.ts`; shared wording and glossary merging are in `src/lib/translations.ts`.
- Translate complete sentences with named placeholders. Preserve every placeholder between English and Malayalam resources.
- Insert placeholder values once, literally. Never interpret user data as translation keys, replacement expressions or HTML.
- Translate role/status labels only when displaying them. API values, permissions, query keys, routes and persisted identifiers retain their original values.
- Language switching must preserve unsaved form state and must not repeat a transaction. Circulation completion messages retain structured values and translate when rendered.
- Never report a module fully translated merely because its navigation label is translated.

## Verification and remaining acceptance

Focused tests cover approved terminology, placeholder parity, literal interpolation, language switching, unsaved member forms, unchanged catalogue data and barcode/renewal API payloads. Existing English workflow tests also run.

Browser rendering remains unverified for these additions: prior isolated Edge review attempts timed out before screen inspection. Malayalam font bundling, shaping, narrow phone headers, longer labels, dark mode and installed native acceptance are still required. The existing installer does not contain these source changes.

`english-review.json` and the XLSX are an earlier source snapshot of candidate phrases, not a complete live coverage count. Preserve IDs when reviewing a returned workbook and reconcile against current source before integration.


## Current owner review and pause

2026-11-26T15:12:32.816Z

Development is paused until the owner explicitly resumes. Use [Pusthakam_Malayalam_Drafts_319.xlsx](Pusthakam_Malayalam_Drafts_319.xlsx) for this batch. It contains English and Malayalam drafts side by side: correct only awkward wording, mark reviewed rows OK/Changed/Question, leave other rows Not reviewed. A partial file is sufficient. The previous larger workbook is preserved.

The exact review snapshot is core-draft-review.json. On return, match IDs and English source, check placeholders, and apply only approved wording; blank correction cells do not approve unreviewed drafts. Current language coverage and pending visual/native checks above remain unchanged. No application implementation was performed during this pause/save operation.


## Malayalam font decision

Owner confirmed Nirmala UI rendered poorly and selected Noto Sans Malayalam. The app bundles its own Noto Sans Malayalam Variable WOFF2 files through Fontsource; public OPAC embeds the Malayalam subset. All font loading is local; SIL OFL licence accompanies both builds. Workbook generators select Noto Sans Malayalam; do not overwrite owner review files to change fonts. Existing installer must be rebuilt before distribution. Browser/fresh-install glyph acceptance remains pending. This is application-local bundling, not system-wide registration for other applications.


## Automatic status in future review sheets

Typing a correction must automatically make Review status Changed. It is a computed column; use the separate optional Your review (OK / Question) column for unchanged drafts. Clearing a correction returns to that choice or Not reviewed. Preserve original review inputs when copying/expanding a workbook. Excel automatic calculation is enabled; verify formulas by recalculating the next generated deliverable before sending. Current 519 workbook remains unchanged.


Contributor now follows the new owner-approved glossary entry and grammatical forms; preserve existing owner review files.


Owner currently edits 519; only the 319 file was updated with Noto font, approved contributor labels and auto-status formulas. Do not overwrite owner519 work when reconciling overlapping rows. Excel runtime recalculation check was incomplete; formulas recalculate on open.


## Active combined review file updated

The owner authorized updating 519 in place. Noto Sans Malayalam, approved contributor labels and auto-status formulas applied; all13 saved corrections and notes/IDs/batches preserved. Use519 as the review file, no duplicate319 review needed. Backup path and preservation checks: WORKBOOK_519_UPDATE.json. ColumnE is computed; type corrections inD and optional OK/Question inI. Live Excel calculation remains unverified; automatic calculation on open enabled.
