# MARCXML catalogue import

Choose Import records and select a `.xml` or `.marcxml` file. Choose the Koha profile to import items in field 952, or the titles-only profile for bibliographic records without copies.

The file must be UTF-8 MARC21 XML: a single `record` or a `collection` of records. Namespace prefixes are supported. Standard XML entities and numeric character references are decoded. DTDs, custom entities, malformed XML and invalid field structures are rejected before a preview is staged.

Bibliographic fields and Koha 952 holdings use the same mapping as ISO 2709 MARC import. Review titles, copies, warnings, new locations and categories before confirming. The existing duplicate policies, barcode clash checks, category selection and transactional import apply.

Use `docs/examples/catalogue-import.xml` in a disposable verification library. Preview should show one title and one copy. After confirming, check title `MARCXML verification book`, barcode `XML-000123`, call number `020 XML`, location `XML MAIN` and price 125.50. Repeat with the titles-only profile to check that no copies are read. A truncated file must show an error without adding records.

For the negative check, choose `docs/examples/catalogue-import-malformed.xml`. This deliberately truncated sample must produce an error; `This file must not import` must not appear in the catalogue.
