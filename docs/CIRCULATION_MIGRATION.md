# Circulation and financial-history migration

Phase 4 task IMP-LOANS remains **in progress**. The neutral loan CSV preview and confirmed open/closed loan import are implemented. Renewal history has a separate source-linked CSV importer (below). Reservations, fines and settlements remain unimplemented. Real Koha field mappings remain unverified; the owner has no source export and authorized generic formats and synthetic tests.

## Neutral loan CSV v1

UTF-8 CSV, optional BOM, at most 10 MB and 25,000 records. Exact header order:

```csv
source_id,member_number,barcode,issued_at,due_on,returned_at,source_staff
```

Use docs/examples/loan-history-template.csv as an empty template. Every column is required. source_id must be unique in the source system; member_number and barcode resolve existing records, with case-insensitive matching and leading zeros preserved. Accession fallback is not inferred. Required identifiers and source_staff must have no surrounding whitespace. Timestamps are UTC YYYY-MM-DDTHH:MM:SSZ; due dates are YYYY-MM-DD. Leave returned_at empty for an open loan.

Import catalogue copies and members first. Source database IDs are never treated as Pusthakam IDs. Use a stable source-system/library name across later batches.

## Preview and confirmation

Administrators with admin.settings can choose **Circulation > Preview loan import**. Rust checks the same permission for preview and commit. Preview shows total/open/closed/invalid counts and up to 100 record-numbered problems. No invalid records are silently skipped; any invalid record blocks commit.

Preview reads one database snapshot. The fingerprint covers file bytes and relevant library tables. Confirmation rechecks the fingerprint, validates all rows and writes in an immediate transaction. Changes to members, copies, loans, categories, policies or import history require a new preview. This is deliberately conservative: an unrelated catalogue edit can also invalidate a preview.

Duplicate file digests are rejected. Within a named source, a source loan ID cannot be imported again, even in a changed file. Overlapping loans for the same copy are rejected at commit, including conflicts within a batch. A commit-only conflict may therefore appear after an otherwise valid preview. Loan dates and amounts are never inferred from a balance.

## Loan behavior and provenance

Open imports require an available copy with no local open loan, an active title/member and active member/item categories with a fine policy. They retain original issue and due dates and set the copy on loan, with a stock event. Current eligibility/expiry and maximum-loan checks are not applied retroactively to existing source loans.

This format does not carry historical fine terms. For open loans the administrator must explicitly confirm use of current policies. The current rate, grace, cap and library currency are snapshotted into loan_fine_terms. Future returns can assess overdue days from before import using those terms; the screen explains this consequence. Preserving exported historical terms needs a future format extension.

Closed imports retain issue/due/return dates, leave present copy status alone, assess no past fines and leave unknown return conditions unspecified. Past returns do not claim that a copy is available today.

Schema 18 adds append-only loan_import_batches and loan_import_sources. It stores source system, source ID, original source_staff label, all seven source field values, file digest, importing operator/time and fine-policy basis. The loan's local issued_by/returned_by references identify the importing recorder, **not the historical operator**; historical source labels are preserved separately and displayed in Members > Loans. No synthetic staff accounts or invented return-staff identities are created. A later real return records its actual local return operator normally.

Loans, fine terms, stock changes/events, provenance and the batch audit entry are committed together. Any error rolls back the entire batch. Historical source fields are retained; lexical CSV quoting/BOM are not preserved as an archival original file. Keep the original export separately.

## Remaining history types and schema gaps

Audited migrations 0006 through 0010 and the patron importer. The full task still requires:

| Source information | Destination constraint | Remaining work |
|---|---|---|
| Renewal events | Source-linked complete-chain importer implemented | Native acceptance pending. Incompatible/nonextending events are explicitly rejected; counts never manufacture events. |
| Reservations | Copy-specific queue uses local insertion order | Explicit copy mapping and source queue order; never choose an arbitrary copy for a title-level request. |
| Charges | One assessed fine per loan | Preserve/report non-loan charges and multiple charge types; do not silently merge them into overdue fines. |
| Settlements | Payments/waivers allocated to individual fines | Preserve allocation, currency, dates, method and reason; never infer payments from a net balance. |
| Historical policies and return conditions | Not present in neutral v1 | Extend the format and preview before accepting these fields. |

Financial reconciliation must keep currency totals separate. Test every supported history type, permissions, repeat import, stale preview and rollback before marking the full task implemented.

## Verification checkpoint

Final checks passed: **122 Rust unit tests, 22 IPC tests, 17 focused frontend tests, production build/typecheck and lint**. Includes permission denial, preserved dates/source labels, explicit policy choice, stale preview, source-ID repeats, overlap rejection and forced-audit rollback. Initial digest-format compilation and misplaced new assertions in an unrelated fine-history test were corrected; both failures remain in PROGRESS.json. No original financial checks were removed. Native standalone build passed; pusthakam-phase4-loan-import.exe is ready in the existing verification folder. Owner confirmed the supplied closed-loan preview/confirmation and member-history walkthrough worked. Other native checks remain open.

## Windows walkthrough

Quit an older test build via its tray menu; X only hides it. Use the new loan-import build against the disposable verification library, never a production library for experiments. Schema 18 is newer than the member-fines build.

1. Open Circulation > Preview loan import. Select a UTF-8 CSV whose member numbers and barcodes already exist in this test library.
2. Preview counts and problems. Invalid records disable confirmation.
3. Enter a stable source name. For open loans, read and select the current-fine-policy choice.
4. Confirm once. Check imported loans in Members > Loans, including original dates and source staff labels. An imported open copy appears on loan; importing closed history does not change its status.
5. Preview/confirm the same file again: it must be refused without duplicates. A library edit after preview requires repreview.
6. Check a read-only staff account cannot access the import action.

Native closed-loan import/history is owner-confirmed. Native repeat/invalid-file/read-only and open-loan checks, plus real anonymised source reconciliation, remain pending. Real exports must stay outside Git. When a sample becomes available, verify source-version field meanings against official documentation before adding a Koha-specific mapping; preserve joins while anonymising it.


For the local Windows test, loan-import-test-closed.csv and loan-import-test-invalid.csv are prepared beside the executable. They reference existing disposable test-library identifiers. Select the closed example, enter source name Windows test and confirm one historical loan; then inspect Members > Loans and retry the same file to verify rejection. The invalid example must block confirmation. No overdue copy is required. These CSVs are machine-local and not committed.

Next development step: native renewal-history acceptance, then source-linked reservation migration and financial records in Phase 4.


## Renewal history CSV v1 (schema 19)

Use **Circulation > Import renewal history** as an administrator. Import source loans first, then use exactly the same source-system name (case-sensitive). The empty template is docs/examples/renewal-history-template.csv.

```csv
source_id,loan_source_id,renewed_at,previous_due_on,due_on,source_staff
```

UTF-8 with optional BOM, at most 10 MB and 25,000 events. Headers must match exactly and in order. source_id identifies the renewal in this source; loan_source_id identifies the previously imported loan. Text identifiers preserve leading zeros. Dates/timestamps use the same strict forms as loan CSVs. Blank or whitespace-padded identifiers and invalid dates are refused.

Supply all renewal events for a loan in one batch. Input rows may be unordered; events are sorted by their UTC renewal timestamps. Events must lie within the loan's issue/return interval, have a strictly later due date, form an uninterrupted old-due/new-due chain with strictly increasing timestamps, and end at the already imported loan's final due date. The initial previous due date cannot precede issue. Existing renewal events on that loan block historical import; import history before conducting new local renewals. Counts alone cannot manufacture events.

Preview reports total events, distinct resolved loans and invalid records, with up to 100 problems. Invalid rows or chains block the entire import. The source name, file and current database state are bound to the review. A source/file/library change requires repreview. Duplicate files and source renewal IDs are refused. Confirmation writes events, append-only source provenance and the audit entry in one immediate transaction; audit failure rolls everything back.

Schema 19 preserves source labels and all six CSV field values in renewal_import_sources with a batch record, file digest, importing operator and time. loan_renewals.renewed_by identifies the local importing recorder, not the historical source staff. Original source labels remain separate; no staff accounts are fabricated.

This is historical event insertion, not a new renewal: it does not change loan due dates, stock, fines or fine terms, and does not run today's overdue, calendar, waiting-hold or renewal-limit rules against old events. Imported events do count in subsequent normal renewal-limit checks. Members > Loans displays the combined historical/local renewal count. Source event timestamps and due-date transitions are retained in the database; an event-by-event member history viewer is not part of this step.

All 122 Rust unit, 23 IPC and 22 focused frontend tests passed, along with production build/typecheck and lint. Native build passed; pusthakam-phase4-renewal-import.exe is ready in the existing test folder. Owner walkthrough remains open. Remaining Phase 4 migration work follows with reservations and financial records. Real source-system mapping remains unverified without an anonymised export.

### Native renewal test

Quit the older app via tray > Quit, then open pusthakam-phase4-renewal-import.exe. The same folder contains renewal-import-test.csv and RENEWAL_IMPORT_TEST.txt with the original source-system name. The CSV is a synthetic example linked to the previously imported historical loan; preparation only read the test database. In Circulation > Import renewal history, enter that source name, preview and confirm the CSV. Members > Loans should show Renewals: 1 on the historical loan, with due date and stock unchanged. Reimport must be rejected. No overdue copy or external export is needed. Schema 19 requires this refreshed build; older apps refuse a newer database. Owner confirmed renewal import/history works. Repeated-file rejection was not explicitly reported and remains an open native check.

## Next-session UI fix

Owner reports Choose file appears as random text. Make shared file-selection controls clearly button-shaped and recognizable, with focus/disabled/light/dark states. This is the first next step before reservation migration. Request recorded only; no styling change implemented in this checkpoint. Owner confirmed renewal import works and paused work until the next session.
