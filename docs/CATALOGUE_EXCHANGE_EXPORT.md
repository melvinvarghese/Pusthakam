# Catalogue and holdings export

In Catalogue, select up to 100 titles using the row checkboxes. Selection can span result pages. Choose **Export selected**, choose a format and **Prepare export**. Review the title/copy counts and any warnings, then **Save file**. On Windows, the Save As dialog lets you choose the folder and filename, confirms replacement of an existing file, and adds the format extension when omitted. The app shows the saved path after success. Cancelling writes nothing and keeps the export ready. A failed save shows an error and allows retry. Titles with no copies are included. The export reads one database snapshot and makes no catalogue changes.

Staff with catalogue read access can export. Every command checks permission in Rust. The existing Accession Register CSV export remains available for inventory work.

| Format | Supported exchange fields and limits |
|---|---|
| JSON | Supported bibliographic fields, contributor names/roles, subjects, classification number/scheme and nested copies including barcode, accession, call number, named lookups, acquisition date, price, vendor, copy note and manual status. Widest fidelity. |
| CSV / XLSX | Title, subtitle, author/additional-author names, publisher/place/year, ISBN, edition, language, pagination, physical description, series, subjects, keywords, classification number/scheme, abstract, record notes, and copy details except copy notes. Other contributor roles are omitted. Names/subjects use semicolon separation; embedded list separators are normalized on reimport. |
| MARC / MARCXML | UTF-8 MARC21 fields 020, 041, 050/080/082, 100/700, 245, 250, 264, 300, 490, 500, 520, 650, 653 and Koha-style 952 holdings. Use the Koha import profile to retain copies. MARC reimport normalizes punctuation, language labels, subject headings and supported contributor roles. Custom classification schemes require another format. |

XLSX uses explicit text cells, preserving leading zeroes and preventing formulas from executing. CSV values resembling formulas are refused with a message directing you to XLSX or JSON. Import CSV through the preview wizard, or import identifier columns as text when opening it in a spreadsheet application. Plain CSV cannot control Excel's automatic formatting.

MARC holdings map barcode to `952$p`, accession/inventory number to `$i`, call number to `$o`, location to `$b`, shelf to `$c`, collection to `$8`, category to `$y`, acquisition date to `$d`, vendor to `$e`, price in major units to `$g`, and copy notes to `$z`. Lost/withdrawn/damaged flags use `$1/$0/$4`. Repair status has no mapped MARC equivalent and becomes available, with a warning. JSON and tabular formats preserve repair status.

JSON and tabular imports accept an optional `classificationScheme` / `Classification Scheme` code. The target library must have the scheme configured. Without a scheme, imported classification numbers retain the previous Dewey default. MARC 080 and 050 now select UDC and LC respectively. A tabular row with `Copies` equal to `0` imports a title without inventing a holding.

These are catalogue exchange files, not backups. They omit internal IDs, credentials, members, loans, reservations, fines, transfer history, audit history, covers, acquisition source and invoice reference. Archive state is not migrated. Loan/hold/transit copy states become available with a warning; migrate operational history separately. Keep a database backup for complete recovery.

Selections are limited to 100 titles, 25,000 copies and a 50 MB output file; spreadsheets also respect the importer's cell limit. Oversized ISO 2709 fields/records are rejected rather than truncated; choose MARCXML or JSON instead.

## Windows acceptance

1. In the disposable test library, select the XLSX verification title. Export JSON and XLSX; each should report one title and two copies. Save both and open them in an appropriate reader.
2. Preview each saved file through Import. Check one title, two copies, barcodes `000XLSX123` / `000XLSX124`, price 125.50 and the second copy's damaged status. Cancel the preview to avoid duplicate import.
3. Repeat CSV, MARC and MARCXML. Use the Koha profile for MARC files. Reconcile the same supported fields.
4. Select a title with no copies and verify it previews without a new holding. Verify read-only staff can export while editing stays unavailable.

Save As is currently implemented for the Windows desktop application. Other platforms and browser previews retain their existing download behavior. The native command accepts prepared bytes and a fixed export filename, never a frontend-supplied destination path. Exports cannot replace files inside Pusthakam's private data directory. A completed temporary file is atomically published to the chosen location; a failed write preserves an existing destination.
