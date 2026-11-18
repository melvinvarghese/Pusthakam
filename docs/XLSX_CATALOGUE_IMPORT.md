# XLSX catalogue import

Choose Import records, then an `.xlsx` file. The workbook must contain exactly one worksheet, with unique text column headings in row 1 starting at A1. Remove unused sheets before importing. Legacy `.xls` and macro-enabled `.xlsm` files are not supported.

The preview recognises the same columns as CSV and lets you change column mappings. Rows with the same ISBN, or the same title, first author and year, are grouped into one title with multiple copies. Review warnings, copies and any new locations or categories before confirming. Preview writes nothing. Duplicate policies and confirmed transactional import use the existing catalogue import workflow.

Store ISBNs, barcodes and accession numbers as **text**, including identifiers made only of digits. Numeric identifier cells are rejected because Excel may already have removed leading zeroes or rounded long numbers. Changing a damaged number's cell format cannot recover its original value; restore it from the source.

Use values instead of formulas. Store acquisition dates as `YYYY-MM-DD` text. Prices are in major currency units, for example `125.50`, not `12550` minor units. Negative prices and spreadsheet error cells are rejected. Other field validation uses the shared CSV warnings and import rules.

The importer bounds expanded workbook data to 100 MB and accepts at most 100,000 data rows, 128 columns and 1,000,000 rectangular cells. Excessive sparse cell coordinates are rejected before loading a range. Nothing is extracted to disk and formulas are never evaluated.

## Windows verification

Use a disposable library and `docs/examples/catalogue-import.xlsx`:

1. Preview: one title and two copies, with no records saved yet. Check the column mappings.
2. Confirm and open `XLSX verification book`.
3. Check barcodes/accessions `000XLSX123` and `000XLSX124`, call number `020 XLSX`, location `XLSX MAIN`, acquisition date `2026-11-18`, and price `125.50` on each copy. The second copy is damaged.
4. Preview `catalogue-import-formula.xlsx`. It must show an error without adding titles or copies.
5. Optional negative samples cover numeric identifiers, excessive sparse dimensions and multiple worksheets. Each must be rejected at preview.
