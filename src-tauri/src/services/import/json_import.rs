//! Strict JSON interchange input. Analysis and commit are shared with CSV/MARC imports.
use super::ImportRecord;
use crate::error::{AppError, AppResult};
use serde_json::Value;

const RECORD_FIELDS: &[&str] = &[
    "title", "subtitle", "contributors", "isbn", "publisher", "place", "year", "edition",
    "language", "pagination", "physical", "series", "subjects", "classification", "abstractText",
    "notes", "keywords", "copies",
];
const COPY_FIELDS: &[&str] = &[
    "barcode", "accession", "callNumber", "location", "shelf", "collection", "itemCategory",
    "acquisitionDate", "priceMinor", "vendor", "notes", "status",
];

fn check_fields(value: &Value, allowed: &[&str], context: &str) -> AppResult<()> {
    let object = value.as_object().ok_or_else(|| AppError::Validation(format!("{context} must be an object.")))?;
    if object.keys().any(|key| !allowed.contains(&key.as_str())) {
        // Do not echo arbitrary source values/keys that may contain private data.
        return Err(AppError::Validation(format!("{context} contains unsupported fields. Check the documented JSON format; nothing was imported.")));
    }
    Ok(())
}

pub fn parse(bytes: &[u8]) -> AppResult<Vec<ImportRecord>> {
    let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes);
    let value: Value = serde_json::from_slice(bytes).map_err(|error| AppError::Validation(format!(
        "Invalid JSON at line {}, column {}. Nothing was imported.", error.line(), error.column()
    )))?;
    let array = value.as_array().ok_or_else(|| AppError::Validation("JSON must contain an array of catalogue records.".into()))?;
    let mut records = Vec::with_capacity(array.len());
    for (index, value) in array.iter().enumerate() {
        let context = format!("Record {}", index + 1);
        check_fields(value, RECORD_FIELDS, &context)?;
        if let Some(copies) = value.get("copies") {
            let copies = copies.as_array().ok_or_else(|| AppError::Validation(format!("{context}: copies must be an array.")))?;
            for (copy_index, copy) in copies.iter().enumerate() {
                check_fields(copy, COPY_FIELDS, &format!("{context}, copy {}", copy_index + 1))?;
            }
        }
        if let Some(contributors) = value.get("contributors") {
            let contributors = contributors.as_array().ok_or_else(|| AppError::Validation(format!("{context}: contributors must be an array.")))?;
            for contributor in contributors {
                check_fields(contributor, &["name", "role"], &format!("{context}, contributor"))?;
            }
        }
        let mut record: ImportRecord = serde_json::from_value(value.clone()).map_err(|_| AppError::Validation(format!(
            "{context} has an invalid field type. Check text, integer, array and copy fields; nothing was imported."
        )))?;
        for copy in &record.copies {
            if copy.price_minor.is_some_and(|price| price < 0) {
                return Err(AppError::Validation(format!("{context}: copy prices cannot be negative.")));
            }
            if let Some(status) = copy.status.as_deref() {
                if !["available", "lost", "damaged", "withdrawn", "in_repair"].contains(&status) {
                    return Err(AppError::Validation(format!("{context}: unsupported copy status. Import circulation separately.")));
                }
            }
        }
        record.row = index + 1;
        records.push(record);
    }
    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_unicode_and_nested_copies_without_losing_identifiers_or_money() {
        let rows = parse(br#"[{"title":"Animal Farm","contributors":[{"name":"George Orwell","role":"author"}],"subjects":["Fiction"],"copies":[{"barcode":"000123","accession":"000045","priceMinor":12550,"status":"damaged"}]}]"#).unwrap();
        assert_eq!(rows[0].row, 1);
        assert_eq!(rows[0].contributors[0].name, "George Orwell");
        assert_eq!(rows[0].copies[0].barcode.as_deref(), Some("000123"));
        assert_eq!(rows[0].copies[0].price_minor, Some(12550));
        assert_eq!(parse("[{\"title\":\"മലയാളം\"}]".as_bytes()).unwrap()[0].title, "മലയാളം");
    }

    #[test]
    fn rejects_bad_shapes_types_and_unmapped_fields_including_later_records() {
        for bytes in [
            r#"{"title":"Book"}"#, r#"[{"title":"Good"},{"title":123}]"#,
            r#"[{"title":"Book","copies":[{"priceMinor":12.5}]}]"#,
            r#"[{"title":"Book","copies":[{"priceMinor":-1}]}]"#,
            r#"[{"title":"Book","copies":[{"status":"on_loan"}]}]"#,
            r#"[{"title":"Book","copies":[{"barcode":123}]}]"#,
            r#"[{"title":"Good"},{"title":"Book","unknown":"value"}]"#,
            r#"[{"title":"Book","copies":[{"skipReason":"ignore"}]}]"#,
            r#"[{"title":"Book","contributors":[{"name":"Writer","role":"author","extra":true}]}]"#,
            r#"[{"title":"Book"},"#, r#"[{"title":"Book","copies":null}]"#,
        ] { assert!(parse(bytes.as_bytes()).is_err(), "accepted {bytes}"); }
    }
}
