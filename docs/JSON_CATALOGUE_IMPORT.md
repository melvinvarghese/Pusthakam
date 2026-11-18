# JSON catalogue import

Choose **Catalogue → Import → Catalogue records (JSON file)**. Preview does not write records. Review the warnings, copy details, duplicate policy and default item category before confirming import. The same transactional importer and permissions used for CSV/MARC apply.

The file must be UTF-8 JSON containing an array of record objects. Maximum file size is 100 MB. UTF-8 BOM is accepted. This is a documented interchange shape, not an arbitrary JSON reader or a database backup format.

```json
[
  {
    "title": "Animal Farm",
    "contributors": [{ "name": "George Orwell", "role": "author" }],
    "subjects": ["Fiction"],
    "year": 1945,
    "copies": [
      {
        "barcode": "000123",
        "accession": "000045",
        "location": "Reading room",
        "priceMinor": 12550,
        "status": "available"
      }
    ]
  }
]
```

Record text fields: `title`, `subtitle`, `isbn`, `publisher`, `place`, `edition`, `language`, `pagination`, `physical`, `series`, `classification`, `classificationScheme`, `abstractText`, `notes`, `keywords`. `year` is an integer. `subjects` is an array of text. `contributors` is an array of objects containing `name` and `role`. Supported roles: `author`, `additional_author`, `editor`, `translator`, `illustrator`, `other`. Missing titles are reported by the preview and cannot be imported. Without copies, only the bibliographic record is imported.

Copy fields: `barcode`, `accession`, `callNumber`, `location`, `shelf`, `collection`, `itemCategory`, `acquisitionDate`, `priceMinor`, `vendor`, `notes`, `status`. Identifiers must be strings to preserve leading zeroes. `priceMinor` is an integer in the configured currency's minor units; for INR, 12550 means ₹125.50. Dates use `YYYY-MM-DD`. Imported non-available conditions follow the existing importer: `lost`, `damaged`, `withdrawn`, `in_repair`; circulation must be migrated through the circulation workflow, not invented through a copy status.

Omit optional fields when absent. Optional text fields may also be null; arrays must be arrays, not null. Do not include database IDs, preview warnings/errors, duplicate IDs or copy skip markers. Unsupported keys anywhere in records, copies or contributors, invalid field types and malformed JSON reject the file before a preview can be committed. No data is silently discarded to make an unsupported shape fit. Map source fields into this shape before importing.

After parsing, normal preview validation still reports ISBN problems, duplicate titles by ISBN, missing titles, and skipped or conflicting copies. Review those reports: confirming an import with invalid records or copies may skip them using the existing importer policy. A fatal transaction failure rolls back all writes.

This format covers catalogue and copy data only. Covers, members, circulation, finances and settings are not imported from this file. Use the backup system for a complete database recovery.

`classificationScheme` is an optional code such as `ddc`, `udc` or `lcc`, and must be configured in the target library. If omitted, classification numbers use the existing Dewey default.
