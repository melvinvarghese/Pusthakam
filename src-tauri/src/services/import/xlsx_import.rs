//! XLSX decoding into the existing spreadsheet mapping and grouping pipeline.
use super::{csv_import, MAX_FILE_BYTES};
use crate::error::{AppError, AppResult};
use calamine::{Data, Reader, Xlsx};
use quick_xml::events::Event;
use std::{
    collections::{HashMap, HashSet},
    io::{Cursor, Read},
};

fn invalid(message: &str) -> AppError {
    AppError::Validation(format!("{message} Nothing was imported."))
}

fn check_position(value: &str) -> AppResult<()> {
    for part in value.split(':') {
        let mut col = 0usize;
        let mut row = 0usize;
        let mut digits = false;
        for c in part.bytes() {
            if c.is_ascii_digit() {
                digits = true;
                row = row.saturating_mul(10).saturating_add((c - b'0') as usize);
            } else if c.is_ascii_uppercase() && !digits {
                col = col
                    .saturating_mul(26)
                    .saturating_add((c - b'A' + 1) as usize);
            } else {
                return Err(invalid("Invalid worksheet cell reference."));
            }
        }
        if row == 0 || col == 0 || row > 100_001 || col > 128 || row.saturating_mul(col) > 1_000_000
        {
            return Err(invalid("Worksheet is too large. Use at most 100,000 data rows, 128 columns and 1,000,000 cells."));
        }
    }
    Ok(())
}

// Bound expansion and sparse cell coordinates before calamine allocates a rectangular range.
fn preflight(bytes: &[u8]) -> AppResult<()> {
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|_| invalid("This is not a readable XLSX workbook."))?;
    if zip.len() > 512 {
        return Err(invalid(
            "The workbook has too many parts. Export a single worksheet.",
        ));
    }
    let mut total = 0usize;
    let mut names = HashSet::new();
    for i in 0..zip.len() {
        let mut file = zip
            .by_index(i)
            .map_err(|_| invalid("The XLSX archive is damaged or encrypted."))?;
        let name = file.name().to_owned();
        if !names.insert(name.clone()) {
            return Err(invalid("The workbook contains duplicate parts."));
        }
        if file.size() > MAX_FILE_BYTES as u64 {
            return Err(invalid("Expanded workbook exceeds 100 MB."));
        }
        let mut content = Vec::new();
        (&mut file)
            .take((MAX_FILE_BYTES - total + 1) as u64)
            .read_to_end(&mut content)
            .map_err(|_| invalid("The XLSX archive is damaged."))?;
        total += content.len();
        if total > MAX_FILE_BYTES {
            return Err(invalid("Expanded workbook exceeds 100 MB."));
        }
        if name.starts_with("xl/worksheets/") && name.ends_with(".xml") {
            let mut xml = quick_xml::Reader::from_reader(content.as_slice());
            loop {
                match xml
                    .read_event()
                    .map_err(|_| invalid("Worksheet XML is malformed."))?
                {
                    Event::Start(e) | Event::Empty(e) => {
                        let local = e.local_name();
                        if local.as_ref() == "f" {
                            return Err(invalid(
                                "Replace worksheet formulas with values before importing.",
                            ));
                        }
                        if local.as_ref() == "c" || local.as_ref() == "dimension" {
                            for a in e.attributes() {
                                let a = a.map_err(|_| invalid("Invalid worksheet attributes."))?;
                                if a.key.as_ref() == "r" || a.key.as_ref() == "ref" {
                                    check_position(
                                        &a.normalized_value(quick_xml::XmlVersion::Implicit1_0)
                                            .map_err(|_| invalid("Invalid cell reference."))?,
                                    )?;
                                }
                            }
                        }
                    }
                    Event::DocType(_) => return Err(invalid("Worksheet DTDs are not supported.")),
                    Event::Eof => break,
                    _ => (),
                }
            }
        }
    }
    Ok(())
}

pub fn parse(
    bytes: &[u8],
    mapping: Option<&HashMap<String, String>>,
) -> AppResult<csv_import::CsvResult> {
    preflight(bytes)?;
    let mut book: Xlsx<_> = Xlsx::new(Cursor::new(bytes))
        .map_err(|_| invalid("The XLSX workbook could not be read."))?;
    let sheets = book.sheet_names();
    if sheets.len() != 1 {
        return Err(invalid("Use a workbook with exactly one worksheet."));
    }
    let range = book
        .worksheet_range(&sheets[0])
        .map_err(|_| invalid("The worksheet could not be read."))?;
    if range.start() != Some((0, 0)) || range.height() < 2 {
        return Err(invalid(
            "Put column names in row 1 starting at A1, followed by catalogue rows.",
        ));
    }
    if range.width() > 128
        || range.height() > 100_001
        || range.width().saturating_mul(range.height()) > 1_000_000
    {
        return Err(invalid("The worksheet is too large."));
    }
    let mut rows = range.rows();
    let header = rows
        .next()
        .ok_or_else(|| invalid("The worksheet is empty."))?;
    let headers: Vec<String> = header
        .iter()
        .map(|v| match v {
            Data::String(s) => s.trim().to_owned(),
            _ => String::new(),
        })
        .collect();
    let mut unique = HashSet::new();
    if headers
        .iter()
        .any(|s| s.is_empty() || !unique.insert(s.to_lowercase()))
    {
        return Err(invalid(
            "Every column needs a unique, nonempty text heading.",
        ));
    }
    let effective = mapping
        .cloned()
        .unwrap_or_else(|| csv_import::auto_mapping(&headers));
    let identifier_columns: HashSet<usize> = ["isbn", "barcode", "accession"]
        .iter()
        .filter_map(|key| effective.get(*key))
        .filter_map(|h| headers.iter().position(|v| v == h))
        .collect();
    let price_column = effective
        .get("price")
        .and_then(|h| headers.iter().position(|v| v == h));
    let mut csv = csv::WriterBuilder::new()
        .quote_style(csv::QuoteStyle::Always)
        .from_writer(Vec::new());
    csv.write_record(&headers)
        .map_err(|_| invalid("Could not read worksheet headings."))?;
    for (index, row) in rows.enumerate() {
        let mut values = Vec::with_capacity(row.len());
        for (col, cell) in row.iter().enumerate() {
            if identifier_columns.contains(&col) && !matches!(cell, Data::String(_) | Data::Empty) {
                return Err(invalid(&format!("Row {}: store ISBNs, barcodes and accession numbers as text, then preview again.", index + 2)));
            }
            values.push(match cell {
                Data::Empty => String::new(),
                Data::String(s) => s.clone(),
                Data::Int(n) => n.to_string(),
                Data::Float(n) if n.is_finite() => n.to_string(),
                Data::Bool(b) => b.to_string(),
                _ => return Err(invalid(&format!("Row {} contains an error or a date/time cell. Correct errors and store dates as YYYY-MM-DD text.", index + 2))),
            });
        }
        if price_column
            .and_then(|col| values.get(col))
            .is_some_and(|v| v.contains('-') || v.contains('('))
        {
            return Err(invalid(&format!(
                "Row {}: copy prices cannot be negative.",
                index + 2
            )));
        }
        csv.write_record(values)
            .map_err(|_| invalid("Could not read worksheet values."))?;
    }
    let csv = csv
        .into_inner()
        .map_err(|_| invalid("Could not read worksheet values."))?;
    Ok(csv_import::parse_table_csv(&csv, Some(&effective)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    const VALID: &[u8] = include_bytes!("../../../../docs/examples/catalogue-import.xlsx");
    fn changed_sheet(from: &str, to: &str) -> Vec<u8> {
        let mut input = zip::ZipArchive::new(Cursor::new(VALID)).unwrap();
        let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for i in 0..input.len() {
            let mut file = input.by_index(i).unwrap();
            let name = file.name().to_owned();
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).unwrap();
            if name == "xl/worksheets/sheet1.xml" {
                bytes = String::from_utf8(bytes)
                    .unwrap()
                    .replace(from, to)
                    .into_bytes();
            }
            output
                .start_file(name, zip::write::SimpleFileOptions::default())
                .unwrap();
            output.write_all(&bytes).unwrap();
        }
        output.finish().unwrap().into_inner()
    }
    #[test]
    fn reads_grouped_copies_and_exact_identifiers_and_price() {
        let r = parse(VALID, None).unwrap();
        assert_eq!(r.records.len(), 1);
        assert_eq!(r.records[0].copies.len(), 2);
        assert_eq!(
            r.records[0].copies[0].barcode.as_deref(),
            Some("000XLSX123")
        );
        assert_eq!(r.records[0].copies[0].price_minor, Some(12550));
        assert_eq!(r.records[0].copies[1].status.as_deref(), Some("damaged"));
    }
    #[test]
    fn rejects_malformed_formulas_numeric_identifiers_and_sparse_ranges() {
        for bytes in [
            b"not a workbook".as_slice(),
            include_bytes!("../../../../docs/examples/catalogue-import-formula.xlsx"),
            include_bytes!("../../../../docs/examples/catalogue-import-numeric-id.xlsx"),
            include_bytes!("../../../../docs/examples/catalogue-import-sparse.xlsx"),
            include_bytes!("../../../../docs/examples/catalogue-import-multiple-sheets.xlsx"),
        ] {
            assert!(parse(bytes, None).is_err());
        }
    }
    #[test]
    fn manual_mapping_is_used_for_identifier_validation() {
        let mut mapping = csv_import::auto_mapping(&parse(VALID, None).unwrap().headers);
        mapping.insert("barcode".into(), "Year".into());
        assert!(parse(VALID, Some(&mapping)).is_err());
        mapping.insert("barcode".into(), "Barcode".into());
        mapping.insert("author".into(), "Location".into());
        assert_eq!(
            parse(VALID, Some(&mapping)).unwrap().records[0].contributors[0].name,
            "XLSX MAIN"
        );
    }
    #[test]
    fn rejects_duplicate_headings_negative_money_and_cell_errors() {
        for (from, to) in [
            ("<t>Author</t>", "<t>Title</t>"),
            ("<v>125.5</v>", "<v>-125.5</v>"),
            (
                "<c r=\"E2\"><v>125.5</v></c>",
                "<c r=\"E2\" t=\"e\"><v>#VALUE!</v></c>",
            ),
        ] {
            assert!(parse(&changed_sheet(from, to), None).is_err());
        }
    }
    #[test]
    fn quoted_headers_and_unicode_text_survive_the_shared_table_parser() {
        let bytes = changed_sheet("<t>Title</t>", "<t>Book; heading, title</t>");
        let bytes = {
            // The custom heading requires a manual mapping; punctuation must not affect delimiters.
            let mut mapping = csv_import::auto_mapping(&parse(VALID, None).unwrap().headers);
            mapping.insert("title".into(), "Book; heading, title".into());
            let r = parse(&bytes, Some(&mapping)).unwrap();
            assert_eq!(r.records[0].title, "XLSX verification book");
            changed_sheet("XLSX verification book", "Books &amp; reading \u{e9}")
        };
        assert_eq!(
            parse(&bytes, None).unwrap().records[0].title,
            "Books & reading \u{e9}"
        );
    }
}
