//! Bounded catalogue exchange files from an explicit selection and one read snapshot.
use crate::{
    error::{AppError, AppResult},
    services::{
        catalogue,
        import::{ImportCopy, ImportRecord},
    },
};
use base64::Engine;
use rusqlite::Connection;
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    io::{Cursor, Write},
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportFile {
    pub file_name: String,
    pub mime_type: String,
    pub data_base64: String,
    pub records: usize,
    pub copies: usize,
    pub warnings: Vec<String>,
}

fn validation(message: &str) -> AppError {
    AppError::Validation(message.into())
}
fn io_error() -> AppError {
    AppError::Internal("Could not create the catalogue export.".into())
}

fn lookup(conn: &Connection, sql: &str) -> AppResult<HashMap<i64, String>> {
    Ok(conn
        .prepare(sql)?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<Result<_, _>>()?)
}

pub fn export(conn: &mut Connection, ids: &[i64], format: &str) -> AppResult<ExportFile> {
    if ids.is_empty() || ids.len() > 100 || ids.iter().any(|id| *id <= 0) {
        return Err(validation("Select between 1 and 100 catalogue records."));
    }
    let (ext, mime) = match format {
        "json" => ("json", "application/json"),
        "csv" => ("csv", "text/csv;charset=utf-8"),
        "xlsx" => (
            "xlsx",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        ),
        "marc" => ("mrc", "application/marc"),
        "marcxml" => ("xml", "application/xml"),
        _ => return Err(validation("Choose CSV, XLSX, JSON, MARC or MARCXML.")),
    };
    let tx = conn.transaction()?;
    let locations = lookup(&tx, "SELECT id, name FROM locations")?;
    let shelves = lookup(&tx, "SELECT id, code FROM shelves")?;
    let collections = lookup(&tx, "SELECT id, name FROM collections")?;
    let categories = lookup(&tx, "SELECT id, name FROM item_categories")?;
    let schemes = lookup(&tx, "SELECT id, code FROM classification_schemes")?;
    let name = |map: &HashMap<i64, String>, id: Option<i64>| id.and_then(|i| map.get(&i).cloned());
    let mut records = Vec::new();
    let mut warnings = Vec::new();
    let mut seen = HashSet::new();
    let mut copies = 0;
    for &id in ids {
        if !seen.insert(id) {
            continue;
        }
        let r = catalogue::get_record(&tx, id)?;
        if r.archived && !warnings.iter().any(|s| s == "Archived titles are exported as ordinary catalogue records; archive state is not migrated.") {
            warnings.push("Archived titles are exported as ordinary catalogue records; archive state is not migrated.".to_string());
        }
        let mut items = Vec::new();
        for c in &r.copies {
            let status = if ["available", "lost", "damaged", "withdrawn", "in_repair"]
                .contains(&c.status.as_str())
            {
                c.status.clone()
            } else {
                let warning = "Loan, hold and transit states are represented as available. This export does not migrate circulation or transfer history.";
                if !warnings.iter().any(|s| s == warning) {
                    warnings.push(warning.into());
                }
                "available".into()
            };
            if c.acquisition_source != "purchase" || c.invoice_ref.is_some() {
                let warning = "Acquisition source and invoice references are not part of this catalogue exchange; use a database backup to preserve them.";
                if !warnings.iter().any(|s| s == warning) {
                    warnings.push(warning.into());
                }
            }
            items.push(ImportCopy {
                barcode: Some(c.barcode.clone()),
                accession: Some(c.accession_no.clone()),
                call_number: c.call_number.clone(),
                location: name(&locations, c.location_id),
                shelf: name(&shelves, c.shelf_id),
                collection: name(&collections, c.collection_id),
                item_category: name(&categories, Some(c.item_category_id)),
                acquisition_date: c.acquisition_date.clone(),
                price_minor: c.price_minor,
                vendor: c.vendor.clone(),
                notes: c.notes.clone(),
                status: Some(status),
                skip_reason: None,
            });
        }
        copies += items.len();
        if copies > 25_000 {
            return Err(validation(
                "This selection has too many copies. Export fewer titles at a time.",
            ));
        }
        records.push(ImportRecord {
            title: r.title,
            subtitle: r.subtitle,
            contributors: r.contributors,
            isbn: r.isbn13.or(r.isbn10),
            publisher: r.publisher,
            place: r.publication_place,
            year: r.publication_year,
            edition: r.edition,
            language: r.language,
            pagination: r.pagination,
            physical: r.physical_description,
            series: r.series,
            subjects: r.subjects,
            classification: r.classification_number,
            classification_scheme: name(&schemes, r.classification_scheme_id),
            abstract_text: r.abstract_text,
            notes: r.notes,
            keywords: r.keywords,
            copies: items,
            ..Default::default()
        });
    }
    tx.commit()?;
    let bytes = match format {
        "json" => json_bytes(&records)?,
        "csv" | "xlsx" => {
            warnings.push("Tabular exchange preserves author and additional-author names. Other contributor roles and per-copy notes are omitted; JSON preserves them.".into());
            let rows = table(&records);
            if rows.len() * rows[0].len() > 1_000_000 {
                return Err(validation(
                    "Too many spreadsheet cells. Export fewer titles.",
                ));
            }
            if format == "xlsx" {
                xlsx(&rows)?
            } else {
                if rows.iter().flatten().any(|v| {
                    v.trim_start().starts_with(['=', '+', '-', '@'])
                        || v.starts_with(['\t', '\r', '\n'])
                }) {
                    return Err(validation("Some values can be treated as spreadsheet formulas in CSV. Choose XLSX or JSON to preserve them safely."));
                }
                let mut out = csv::Writer::from_writer(Vec::new());
                for row in &rows {
                    out.write_record(row).map_err(|_| io_error())?;
                }
                out.into_inner().map_err(|_| io_error())?
            }
        }
        _ => super::marc_export::encode(&records, format == "marcxml", &mut warnings)?,
    };
    if bytes.len() > 50 * 1024 * 1024 {
        return Err(validation("Export exceeds 50 MB. Select fewer titles."));
    }
    Ok(ExportFile {
        file_name: format!("pusthakam-catalogue.{ext}"),
        mime_type: mime.into(),
        data_base64: base64::engine::general_purpose::STANDARD.encode(bytes),
        records: records.len(),
        copies,
        warnings,
    })
}

fn json_bytes(records: &[ImportRecord]) -> AppResult<Vec<u8>> {
    let mut values = Vec::new();
    for r in records {
        let mut value = serde_json::to_value(r).map_err(|_| io_error())?;
        if let Some(object) = value.as_object_mut() {
            for key in ["row", "warnings", "errors", "duplicateOf"] {
                object.remove(key);
            }
            if let Some(copies) = object.get_mut("copies").and_then(|v| v.as_array_mut()) {
                for c in copies {
                    if let Some(c) = c.as_object_mut() {
                        c.remove("skipReason");
                    }
                }
            }
        }
        values.push(value);
    }
    serde_json::to_vec_pretty(&values).map_err(|_| io_error())
}

pub(super) fn money(value: Option<i64>) -> String {
    value
        .map(|v| format!("{}.{:02}", v / 100, v % 100))
        .unwrap_or_default()
}

fn table(records: &[ImportRecord]) -> Vec<Vec<String>> {
    let mut rows = vec![vec![
        "Title",
        "Subtitle",
        "Author",
        "Additional Authors",
        "Publisher",
        "Place",
        "Year",
        "ISBN",
        "Edition",
        "Language",
        "Pagination",
        "Physical Description",
        "Series",
        "Subjects",
        "Keywords",
        "Classification",
        "Classification Scheme",
        "Abstract",
        "Notes",
        "Barcode",
        "Accession",
        "Call Number",
        "Location",
        "Shelf",
        "Collection",
        "Category",
        "Date Acquired",
        "Price",
        "Vendor",
        "Status",
        "Copies",
    ]
    .into_iter()
    .map(String::from)
    .collect()];
    for r in records {
        let base = vec![
            r.title.clone(),
            text(&r.subtitle),
            r.contributors
                .iter()
                .filter(|c| c.role == "author")
                .map(|c| c.name.as_str())
                .collect::<Vec<_>>()
                .join("; "),
            r.contributors
                .iter()
                .filter(|c| c.role == "additional_author")
                .map(|c| c.name.as_str())
                .collect::<Vec<_>>()
                .join("; "),
            text(&r.publisher),
            text(&r.place),
            r.year.map(|v| v.to_string()).unwrap_or_default(),
            text(&r.isbn),
            text(&r.edition),
            text(&r.language),
            text(&r.pagination),
            text(&r.physical),
            text(&r.series),
            r.subjects.join("; "),
            text(&r.keywords),
            text(&r.classification),
            text(&r.classification_scheme),
            text(&r.abstract_text),
            text(&r.notes),
        ];
        for c in r.copies.iter().map(Some).chain(if r.copies.is_empty() {
            Some(None)
        } else {
            None
        }) {
            let mut row = base.clone();
            if let Some(c) = c {
                row.extend([
                    text(&c.barcode),
                    text(&c.accession),
                    text(&c.call_number),
                    text(&c.location),
                    text(&c.shelf),
                    text(&c.collection),
                    text(&c.item_category),
                    text(&c.acquisition_date),
                    money(c.price_minor),
                    text(&c.vendor),
                    text(&c.status),
                    "1".into(),
                ]);
            } else {
                row.extend(vec![String::new(); 11]);
                row.push("0".into());
            }
            rows.push(row);
        }
    }
    rows
}
fn text(value: &Option<String>) -> String {
    value.clone().unwrap_or_default()
}
pub(super) fn escape(value: &str) -> String {
    quick_xml::escape::escape(value).into_owned()
}

// Text cells are intentional: identifiers, dates and prices must not be reformatted by Excel.
fn xlsx(rows: &[Vec<String>]) -> AppResult<Vec<u8>> {
    if rows.iter().flatten().any(|s| {
        s.chars()
            .any(|c| c < ' ' && !['\t', '\n', '\r'].contains(&c))
    }) {
        return Err(validation(
            "Unsupported control characters in spreadsheet text. Choose JSON.",
        ));
    }
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut part = |name: &str, content: &str| -> AppResult<()> {
        zip.start_file(
            name,
            zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated),
        )
        .map_err(|_| io_error())?;
        zip.write_all(content.as_bytes()).map_err(|_| io_error())
    };
    part(
        "[Content_Types].xml",
        r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/><Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/></Types>"#,
    )?;
    part(
        "_rels/.rels",
        r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/></Relationships>"#,
    )?;
    part(
        "xl/workbook.xml",
        r#"<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets><sheet name="Catalogue" sheetId="1" r:id="rId1"/></sheets></workbook>"#,
    )?;
    part(
        "xl/_rels/workbook.xml.rels",
        r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/></Relationships>"#,
    )?;
    let mut xml = String::from(
        r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData>"#,
    );
    for (i, row) in rows.iter().enumerate() {
        xml.push_str(&format!("<row r=\"{}\">", i + 1));
        for (j, cell) in row.iter().enumerate() {
            let col = if j < 26 {
                ((b'A' + j as u8) as char).to_string()
            } else {
                format!("A{}", (b'A' + (j - 26) as u8) as char)
            };
            xml.push_str(&format!(
                "<c r=\"{col}{}\" t=\"inlineStr\"><is><t xml:space=\"preserve\">{}</t></is></c>",
                i + 1,
                escape(cell)
            ));
        }
        xml.push_str("</row>");
    }
    xml.push_str("</sheetData></worksheet>");
    part("xl/worksheets/sheet1.xml", &xml)?;
    Ok(zip.finish().map_err(|_| io_error())?.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db,
        services::import::{self, CommitOptions, DuplicatePolicy, PreviewRequest},
    };

    fn load(conn: &mut Connection, bytes: &[u8], format: &str) -> Vec<i64> {
        let request = PreviewRequest {
            file_name: "exchange".into(),
            data_base64: base64::engine::general_purpose::STANDARD.encode(bytes),
            format: format.into(),
            profile: "koha".into(),
            mapping: None,
        };
        let (pending, preview) = import::prepare(conn, &request).unwrap();
        assert_eq!(preview.totals.with_errors, 0);
        let report = import::commit(
            conn,
            None,
            &pending.records,
            &CommitOptions {
                duplicate_policy: DuplicatePolicy::New,
                create_missing: true,
                default_item_category_id: None,
                barcode_as_accession: false,
            },
            "exchange",
        )
        .unwrap();
        assert_eq!(
            report.failed_records,
            0,
            "{:?}",
            report
                .problems
                .iter()
                .map(|p| &p.message)
                .collect::<Vec<_>>()
        );
        conn.prepare("SELECT id FROM bibliographic_records ORDER BY id")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
    }

    fn seed(conn: &mut Connection) -> Vec<i64> {
        load(conn, r#"[{"title":"Books & reading é","subtitle":"A guide","isbn":"9780306406157","contributors":[{"name":"Sample Author","role":"author"},{"name":"Sample Translator","role":"translator"}],"subjects":["Reading"],"publisher":"Small Press","place":"Kottayam","year":2020,"language":"English","classification":"82-3","classificationScheme":"udc","physical":"20 cm","pagination":"120 pages","notes":"Catalogue note","copies":[{"barcode":"000123","accession":"000045","itemCategory":"General","location":"Main","shelf":"A1","collection":"Fiction","priceMinor":12550,"status":"damaged","notes":"Copy note","vendor":"Bookshop","acquisitionDate":"2026-11-18","callNumber":"82-3 SAM"}]},{"title":"Title without copies"}]"#.as_bytes(), "json")
    }

    #[test]
    fn every_format_round_trips_titles_copies_scheme_identifiers_and_money() {
        let dir = tempfile::tempdir().unwrap();
        let mut source = db::open(&dir.path().join("source.db")).unwrap();
        let ids = seed(&mut source);
        for format in ["json", "csv", "xlsx", "marc", "marcxml"] {
            let out = export(&mut source, &[ids[0], ids[1], ids[0]], format).unwrap();
            assert_eq!((out.records, out.copies), (2, 1));
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(out.data_base64)
                .unwrap();
            let mut target = db::open(&dir.path().join(format!("{format}.db"))).unwrap();
            let imported = load(&mut target, &bytes, format);
            assert_eq!(imported.len(), 2, "{format}");
            let r = catalogue::get_record(&target, imported[0]).unwrap();
            assert_eq!(r.title, "Books & reading é", "{format}");
            assert_eq!(r.physical_description.as_deref(), Some("20 cm"));
            assert_eq!(r.classification_number.as_deref(), Some("82-3"));
            let scheme: String = target
                .query_row(
                    "SELECT code FROM classification_schemes WHERE id = ?1",
                    [r.classification_scheme_id],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(scheme, "udc");
            assert_eq!(r.copies.len(), 1);
            let c = &r.copies[0];
            assert_eq!(
                (&*c.barcode, &*c.accession_no, c.price_minor, &*c.status),
                ("000123", "000045", Some(12550), "damaged")
            );
            assert_eq!(c.call_number.as_deref(), Some("82-3 SAM"));
            assert_eq!(c.acquisition_date.as_deref(), Some("2026-11-18"));
            assert!(
                catalogue::get_record(&target, imported[1])
                    .unwrap()
                    .copies
                    .is_empty(),
                "{format} must retain titles with no holdings"
            );
            if format == "json" {
                assert_eq!(r.contributors.len(), 2);
                assert_eq!(c.notes.as_deref(), Some("Copy note"));
            }
        }
    }

    #[test]
    fn validates_selection_and_warns_about_live_states_and_archives() {
        let dir = tempfile::tempdir().unwrap();
        let mut conn = db::open(&dir.path().join("source.db")).unwrap();
        let ids = seed(&mut conn);
        assert!(export(&mut conn, &[], "json").is_err());
        assert!(export(&mut conn, &ids, "unknown").is_err());
        assert!(export(&mut conn, &[999999], "json").is_err());
        conn.execute("UPDATE physical_items SET status='in_transit'", [])
            .unwrap();
        conn.execute(
            "UPDATE bibliographic_records SET archived_at='2026-11-18'",
            [],
        )
        .unwrap();
        let out = export(&mut conn, &ids, "json").unwrap();
        assert!(out.warnings.iter().any(|w| w.contains("transit")));
        assert!(out.warnings.iter().any(|w| w.contains("Archived")));
        conn.execute("UPDATE bibliographic_records SET title='=unsafe()'", [])
            .unwrap();
        assert!(export(&mut conn, &ids, "csv").is_err());
        let file = export(&mut conn, &ids, "xlsx").unwrap();
        let parsed = import::xlsx_import::parse(
            &base64::engine::general_purpose::STANDARD
                .decode(file.data_base64)
                .unwrap(),
            None,
        )
        .unwrap();
        assert_eq!(parsed.records[0].title, "=unsafe()");
    }
}
