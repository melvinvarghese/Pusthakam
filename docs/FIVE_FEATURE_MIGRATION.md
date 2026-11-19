# Five-feature Phase 4 migration batch

Owner-approved one-time batch: reservation history, assessed fines, payments, waivers and reconciliation. One Windows test build/checklist will cover all five. These features are implemented in the neutral history bundle; all automated checks and the native build passed; owner accepted the joint native walkthrough in PROGRESS.json. Full IMP-LOANS remains in progress for broader source mapping and remaining limitations.

## Workflow

Administrators choose **Circulation > Import reservations and fines**. Import members, copies and source-linked loans first. Choose a UTF-8 JSON bundle, preview, compare **Calculated destination** with **Expected source**, then explicitly confirm. Any problem disables confirmation. No record is silently skipped.

The entire bundle commits in one immediate transaction. Source records, stock events, assessed fines, allocated settlements, append-only provenance, saved reconciliation and audit are written together. The backend rereads the inserted records and compares destination totals before finalizing. Any failure rolls everything back. Preview fingerprints cover the file and relevant database state; changed source records, stock, loans, policies, queues or balances require a new preview.

After success, the saved summary is shown. Reopen the dialog and expand **Recent saved reconciliations** to inspect the latest 20 batches. This is a saved per-batch snapshot, not a live balance report; subsequent normal payments can change the current ledger.

## Strict bundle version 1

See docs/examples/history-bundle-example.json and history-bundle-loans.csv. Those are synthetic documentation examples requiring existing member 001 and barcode 0002; never import example history into a production library. The local Windows verification folder has separately generated examples matching that disposable library.

All root fields are required: version (1), sourceSystem, reservations, fines, payments, waivers and expected. Use empty arrays for sections absent from this bundle. Unknown/duplicate JSON fields and wrong types are rejected. Maximum 10 MB, with 1–25,000 records across the four arrays. Money uses positive integer minor units, never decimals; each amount and currency total must fit the exact integer range through 9007199254740991. Currency codes are uppercase three-letter codes. Timestamps are UTC YYYY-MM-DDTHH:MM:SSZ. Surrounding whitespace in identifiers is rejected. Source names and IDs are case-sensitive; keep the same source name across related imports.

| Section | Required fields | Optional/null fields |
|---|---|---|
| reservations | sourceId, memberNumber, barcode, placedAt, status, sourceStaff | expiresAt, closedAt, queuePosition (rules below) |
| fines | sourceId, loanSourceId, amountMinor, currency, assessedAt, sourceStaff | None |
| payments | sourceId, fineSourceId, amountMinor, currency, recordedAt, method, sourceStaff | reason must be absent/null |
| waivers | sourceId, fineSourceId, amountMinor, currency, recordedAt, reason, sourceStaff | method must be absent/null |

Source IDs are unique within a source and record kind. Payments and waivers share the settlement namespace. Repeat file digests and previously imported source IDs are rejected. Member numbers/barcodes preserve leading zeros; barcode lookup does not guess accession numbers or choose a copy for a title-level request. References resolve existing source-linked records, never source numeric database IDs as local IDs.

## 1. Reservation history

Supported statuses: pending, ready, fulfilled, cancelled and expired. Placement cannot be in the future. Closed/expiry timestamps cannot precede placement; closure cannot be in the future. Active pending/ready records require null closedAt and positive queuePosition. Closed history requires closedAt and null queuePosition. Ready requires a future expiresAt.

For each copy, positions must be unique and contiguous from 1. A ready reservation must be first; insertion order preserves that queue in normal issue/return/cancel processing. Do not merge a source queue into an existing local active queue: explicitly resolve it before import. Active records must reference eligible, unarchived members/categories/titles and cannot duplicate a member or reserve their currently borrowed copy.

A queue headed by ready requires an available copy, changed atomically to reserved_hold with a stock event. An entirely pending queue requires an on_loan copy with an actual open loan. The importer does not silently promote a source pending record or silently expire a source ready record. Normal subsequent circulation processing can promote/expire reservations using current rules. Closed reservation history changes no current stock. Full original fields, including source queue position/staff, are retained in provenance.

## 2. Assessed fines

Fines link to already imported **closed** loans using loanSourceId under the same sourceSystem. Preserve the assessment timestamp and amount; do not recalculate historical charges. Assessment must not precede issue or be in the future.

The current schema supports one assessed fine per loan. Open-loan charges, non-loan charges, multiple charge types on one loan, credits and refunds require a separate mapping/schema extension. They are explicitly rejected, not merged, converted or lost. Existing fines on a target loan also block importing another fine. The neutral format does not claim full Koha financial compatibility.

## 3. Payment history

Payments allocate to a fine in this bundle or a previously imported source fine. Currency must match that fine. Supported methods: cash, card, bank_transfer, other. Recorded time cannot precede assessment or be in the future. No waiver reason may be attached to a payment. Preserve separate allocation records; never manufacture a payment from a net balance.

## 4. Waiver history

Waivers allocate to the same supported fine references. Require a nonblank source reason, at most 500 characters, and no payment method. Amount/date/currency validation matches payments. Combined payments and waivers must not exceed assessed amount. Follow-up payment/waiver-only bundles are supported and must reconcile the full affected fine balances, including earlier entries.

Imported settlement history displays the original staff label with “(imported source)”. The importing account remains the local recording operator in provenance. No historical staff accounts are fabricated. Waiver reasons retain the existing circulation-management permission restriction. Settlement history sorts by original recorded timestamp, with local ID as tie-breaker.

## 5. Reconciliation

expected.counts contains reservations, fines, payments and waivers counts for **new records in this bundle**. expected.balances is an array of currency, assessedMinor, paidMinor, waivedMinor and outstandingMinor. Each affected fine contributes once. Balances cover the complete affected fines **after import**, including existing settlements; they are not deltas and not all fines in the whole library. Unaffected currencies are omitted. Currency rows may be supplied in any order; duplicate currency rows fail comparison. Reservations-only bundles use an empty balance array.

Expected source totals must match calculated destination totals exactly. The backend also checks the actual saved destination before commit. Saved summaries are immutable snapshots. Do not claim real-source reconciliation from synthetic tests; an anonymised source export is still needed to validate a source-specific mapping.

## Schema and verification

Migration 20 adds append-only history_import_batches and history_import_sources. Source system/ID, record kind/local ID, staff label and all mapped source fields are retained with batch importer/time, file digest and reconciliation JSON. Local staff foreign keys identify the recording operator. No prior migration is edited.

Focused IPC tests cover all five features, two currencies, ready/pending/closed reservations, normal promotion, full/follow-up allocations, duplicate and stale conflicts, invalid source references, chronology, methods/reasons, precision limits, reconciliation mismatch, post-write changes, forced audit rollback, immutable provenance and denied commands/reason redaction. Final regression/build/native evidence lives in PROGRESS.json.

## One Windows acceptance checklist

The local test package contains pusthakam-phase4-five-feature.exe (schema 20), five-feature-loans.csv, five-feature-history.json, five-feature-mismatch.json and FIVE_FEATURE_TEST.txt. Use source name Five-feature test for the prerequisite CSV. Expected counts: 1 reservation, 2 fines, 2 payments, 1 waiver. INR outstanding 5.00; USD outstanding 4.00. Preparation writes files only; it does not insert records into the test library.

1. Quit the older app via tray > Quit, then open the batch build. Import the prerequisite loan CSV with the source name in its local instructions.
2. In Import reservations and fines, preview the mismatched JSON: confirmation must be disabled. Preview the valid JSON and check all four record counts plus currency-separated amounts.
3. Confirm once. Check saved reconciliation and reopen it through Recent saved reconciliations.
4. Check Reservations for the imported ready queue. Check Members > Fines/Circulation for INR and USD assessed amounts, payments, waiver reason and remaining balances; inspect original settlement dates/staff labels.
5. Retry the bundle: it must be rejected without duplicate records. Check a read-only account cannot access the import action. Report the results together.

Owner confirmed the joint five-feature Windows walkthrough works fine on 19 November 2026. This accepts the provided synthetic package; real-source compatibility/reconciliation and unsupported source mappings remain open. LAN/browser access and OPAC remain Phase 7b.

Final checks passed: 122 Rust unit tests, 24 IPC tests, 28 focused frontend tests, production build/typecheck, lint and native standalone build. Owner confirmed the joint walkthrough works fine. Documentation examples use placeholder identifiers; the machine-local package uses read-only lookup of the disposable library and does not auto-import any rows.

Owner result: combined reservation, assessed-fine, payment, waiver and saved-reconciliation Windows test accepted. Do not reuse imported source IDs for later fixtures. This was a one-time five-feature batch trial, not a standing requirement to batch future work.

Session paused at owner request after native acceptance. Source and progress/handoff checkpoint saved locally. Next session reviews remaining Phase 4 source/schema limitations; no additional feature work started. Git sync is still needed before using another computer.
