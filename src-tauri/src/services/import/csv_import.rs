//! CSV import with automatic column recognition (and manual override) plus grouping of rows into
//! titles with several copies.

use super::{parse_price_minor, ImportCopy, ImportRecord};
use crate::domain::{isbn, validate};
use crate::services::catalogue::ContributorDto;
use std::collections::HashMap;

/// (field key, recognised column names). Names are compared lower-cased with only letters and digits.
pub const FIELDS: &[(&str, &[&str])] = &[
    ("title", &["title", "booktitle", "name", "titleproper"]),
    ("subtitle", &["subtitle", "othertitleinformation"]),
    ("author", &["author", "authors", "mainauthor", "creator", "writer", "authorname"]),
    ("additionalAuthors", &["additionalauthors", "coauthor", "coauthors", "otherauthors", "contributors"]),
    ("publisher", &["publisher", "publishername"]),
    ("place", &["place", "publicationplace", "placeofpublication", "city"]),
    ("year", &["year", "publicationyear", "pubyear", "publishedyear", "yearofpublication", "date"]),
    ("isbn", &["isbn", "isbn13", "isbn10", "isbnnumber"]),
    ("edition", &["edition", "editionstatement"]),
    ("language", &["language", "lang"]),
    ("pagination", &["pages", "pagination", "pagecount", "extent", "numberofpages"]),
    ("series", &["series", "seriestitle"]),
    ("subjects", &["subject", "subjects", "subjectheadings", "topic", "topics"]),
    ("keywords", &["keywords", "keyword", "tags"]),
    ("classification", &["ddc", "classification", "classificationnumber", "deweynumber", "dewey", "classno", "classnumber"]),
    ("classificationScheme", &["classificationscheme"]),
    ("physical", &["physicaldescription"]),
    ("callNumber", &["callnumber", "callno", "shelfmark", "callnum"]),
    ("barcode", &["barcode", "itembarcode", "bookbarcode"]),
    ("accession", &["accessionnumber", "accessionno", "accno", "accession", "accessionnum"]),
    ("price", &["price", "cost", "amount", "mrp"]),
    ("location", &["location", "branch", "library", "section"]),
    ("shelf", &["shelf", "shelfcode", "shelflocation", "rack"]),
    ("collection", &["collection", "collectioncode"]),
    ("category", &["category", "itemtype", "itemcategory", "type", "materialtype"]),
    ("dateAcquired", &["dateacquired", "acquisitiondate", "dateaccessioned", "accessioned", "dateofpurchase", "purchasedate"]),
    ("vendor", &["vendor", "source", "supplier", "donor", "sourceofacquisition"]),
    ("status", &["status", "itemstatus", "condition"]),
    ("copies", &["copies", "numberofcopies", "noofcopies", "quantity", "qty", "copy"]),
    ("notes", &["notes", "note", "remarks", "comments"]),
    ("abstract", &["abstract", "summary", "description"]),
];

fn norm(h: &str) -> String {
    h.chars().filter(|c| c.is_alphanumeric()).collect::<String>().to_lowercase()
}

/// Guesses which column holds which field. Each column and each field is used at most once.
pub fn auto_mapping(headers: &[String]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut used = vec![false; headers.len()];
    for (key, aliases) in FIELDS {
        if let Some(i) = headers.iter().enumerate().position(|(i, h)| !used[i] && aliases.contains(&norm(h).as_str())) {
            used[i] = true;
            map.insert((*key).to_string(), headers[i].clone());
        }
    }
    map
}

fn decode(bytes: &[u8]) -> String {
    let b = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    match std::str::from_utf8(b) {
        Ok(s) => s.to_string(),
        Err(_) => b.iter().map(|&c| c as char).collect(),
    }
}

fn detect_delimiter(text: &str) -> u8 {
    let first = text.lines().next().unwrap_or("");
    [b',', b';', b'\t', b'|']
        .into_iter()
        .max_by_key(|d| first.bytes().filter(|b| b == d).count())
        .filter(|d| first.bytes().any(|b| b == *d))
        .unwrap_or(b',')
}

fn split_list(s: &str) -> Vec<String> {
    s.split([';', '|', '\n']).map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect()
}

fn status_from_text(s: &str) -> Option<String> {
    let t = s.to_ascii_lowercase();
    if t.contains("lost") { Some("lost".into()) }
    else if t.contains("damag") { Some("damaged".into()) }
    else if t.contains("withdraw") || t.contains("discard") { Some("withdrawn".into()) }
    else if t.contains("repair") { Some("in_repair".into()) }
    else { None }
}

pub struct CsvResult {
    pub records: Vec<ImportRecord>,
    pub headers: Vec<String>,
    pub mapping: HashMap<String, String>,
    pub errors: Vec<String>,
}

pub fn parse_csv(bytes: &[u8], mapping_override: Option<&HashMap<String, String>>) -> CsvResult {
    let text = decode(bytes);
    parse_delimited(&text, detect_delimiter(&text), mapping_override)
}

pub(super) fn parse_table_csv(bytes: &[u8], mapping: Option<&HashMap<String, String>>) -> CsvResult {
    parse_delimited(&decode(bytes), b',', mapping)
}

fn parse_delimited(text: &str, delimiter: u8, mapping_override: Option<&HashMap<String, String>>) -> CsvResult {
    let mut rdr = csv::ReaderBuilder::new().delimiter(delimiter).flexible(true).has_headers(true).from_reader(text.as_bytes());
    let headers: Vec<String> = match rdr.headers() {
        Ok(h) => h.iter().map(|s| s.trim().to_string()).collect(),
        Err(e) => return CsvResult { records: vec![], headers: vec![], mapping: HashMap::new(), errors: vec![format!("The file could not be read as CSV: {e}")] },
    };
    let mapping = match mapping_override {
        Some(m) => m.iter().filter(|(_, h)| headers.contains(h)).map(|(k, v)| (k.clone(), v.clone())).collect(),
        None => auto_mapping(&headers),
    };
    let index: HashMap<&str, usize> = mapping
        .iter()
        .filter_map(|(k, h)| headers.iter().position(|x| x == h).map(|i| (k.as_str(), i)))
        .collect();

    let mut errors = Vec::new();
    let mut out: Vec<ImportRecord> = Vec::new();
    let mut by_key: HashMap<String, usize> = HashMap::new();

    for (i, row) in rdr.records().enumerate() {
        let line = i + 2; // header is line 1
        let row = match row {
            Ok(r) => r,
            Err(e) => {
                errors.push(format!("Line {line}: {e}"));
                continue;
            }
        };
        if row.iter().all(|c| c.trim().is_empty()) {
            continue;
        }
        let get = |k: &str| index.get(k).and_then(|&c| row.get(c)).map(str::trim).filter(|s| !s.is_empty());

        let mut r = ImportRecord { row: line, ..Default::default() };
        r.title = get("title").unwrap_or("").to_string();
        r.subtitle = get("subtitle").map(String::from);
        for a in get("author").map(split_list).unwrap_or_default() {
            r.contributors.push(ContributorDto { name: a, role: "author".into() });
        }
        for a in get("additionalAuthors").map(split_list).unwrap_or_default() {
            r.contributors.push(ContributorDto { name: a, role: "additional_author".into() });
        }
        r.publisher = get("publisher").map(String::from);
        r.place = get("place").map(String::from);
        if let Some(y) = get("year") {
            // Accept "1999" or "1999-05-01" / "May 1999": use the first four digits in a row.
            let digits: String = y.chars().skip_while(|c| !c.is_ascii_digit()).take_while(|c| c.is_ascii_digit()).collect();
            match digits.parse::<i64>() {
                Ok(v) if digits.len() == 4 && (1000..=2200).contains(&v) => r.year = Some(v),
                _ => r.warnings.push(format!("The year \"{y}\" was not understood and was left empty.")),
            }
        }
        r.isbn = get("isbn").map(String::from);
        r.edition = get("edition").map(String::from);
        r.language = get("language").map(String::from);
        r.pagination = get("pagination").map(String::from);
        r.series = get("series").map(String::from);
        r.subjects = get("subjects").map(split_list).unwrap_or_default();
        r.keywords = get("keywords").map(String::from);
        r.classification = get("classification").map(String::from);
        r.classification_scheme = get("classificationScheme").map(String::from);
        r.physical = get("physical").map(String::from);
        r.abstract_text = get("abstract").map(String::from);
        r.notes = get("notes").map(String::from);

        // Copies: one row = one copy when it has a barcode/accession, or N unnumbered copies from a count column.
        let barcode = get("barcode").map(String::from);
        let accession = get("accession").map(String::from);
        let count = get("copies").and_then(|c| c.parse::<usize>().ok()).unwrap_or(1).clamp(1, 500);
        let has_copy_data = barcode.is_some() || accession.is_some() || get("copies").is_some() || get("callNumber").is_some() || get("location").is_some();
        if has_copy_data && get("copies") != Some("0") {
            let make = |barcode: Option<String>, accession: Option<String>| -> ImportCopy {
                let mut c = ImportCopy { barcode, accession, ..Default::default() };
                c.call_number = get("callNumber").map(String::from);
                c.location = get("location").map(String::from);
                c.shelf = get("shelf").map(String::from);
                c.collection = get("collection").map(String::from);
                c.item_category = get("category").map(String::from);
                c.vendor = get("vendor").map(String::from);
                c.notes = None;
                c.status = get("status").and_then(status_from_text);
                c.price_minor = get("price").and_then(parse_price_minor);
                c
            };
            let mut copies: Vec<ImportCopy> = if barcode.is_some() || accession.is_some() {
                vec![make(barcode, accession)]
            } else {
                (0..count).map(|_| make(None, None)).collect()
            };
            if let Some(d) = get("dateAcquired") {
                match validate::date("Date", d) {
                    Ok(v) => copies.iter_mut().for_each(|c| c.acquisition_date = Some(v.clone())),
                    Err(_) => r.warnings.push(format!("The date \"{d}\" is not YYYY-MM-DD and was left empty.")),
                }
            }
            if get("price").is_some() && copies[0].price_minor.is_none() {
                r.warnings.push(format!("The price \"{}\" was not understood and was left empty.", get("price").unwrap_or("")));
            }
            r.copies = copies;
        }

        // Group rows of the same title (same ISBN, or same title + first author + year) into one record.
        let key = match r.isbn.as_deref().and_then(isbn::both_forms) {
            Some((_, i13)) => format!("isbn:{i13}"),
            None if r.title.is_empty() => format!("row:{line}"),
            None => format!(
                "t:{}|{}|{}",
                r.title.to_lowercase(),
                r.contributors.first().map(|c| c.name.to_lowercase()).unwrap_or_default(),
                r.year.map(|y| y.to_string()).unwrap_or_default()
            ),
        };
        match by_key.get(&key) {
            Some(&idx) => {
                let existing = &mut out[idx];
                existing.copies.append(&mut r.copies);
                existing.warnings.append(&mut r.warnings);
            }
            None => {
                by_key.insert(key, out.len());
                out.push(r);
            }
        }
    }
    CsvResult { records: out, headers, mapping, errors }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> CsvResult {
        parse_csv(text.as_bytes(), None)
    }

    #[test]
    fn recognises_common_column_names() {
        let r = parse("Book Title,Author,ISBN,Publisher,Year,Call No.,Barcode,Price\nBalyakalasakhi,Basheer,0-306-40615-2,DC Books,1944,894.8123 BAS,B1,250.50\n");
        assert_eq!(r.mapping["title"], "Book Title");
        assert_eq!(r.mapping["callNumber"], "Call No.");
        let rec = &r.records[0];
        assert_eq!((rec.title.as_str(), rec.year, rec.publisher.as_deref()), ("Balyakalasakhi", Some(1944), Some("DC Books")));
        assert_eq!(rec.copies.len(), 1);
        assert_eq!((rec.copies[0].barcode.as_deref(), rec.copies[0].price_minor, rec.copies[0].call_number.as_deref()), (Some("B1"), Some(25050), Some("894.8123 BAS")));
    }

    #[test]
    fn detects_semicolon_and_tab_delimiters_and_a_bom() {
        let semi = parse("title;author;barcode\nA;X;1\n");
        assert_eq!(semi.records[0].copies[0].barcode.as_deref(), Some("1"));
        let tab = parse("title\tauthor\nA\tX\n");
        assert_eq!(tab.records[0].contributors[0].name, "X");
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice("title,author\nA,X\n".as_bytes());
        assert_eq!(parse_csv(&bytes, None).records[0].title, "A");
    }

    #[test]
    fn rows_of_one_title_become_copies_of_one_record() {
        let r = parse("title,author,year,barcode\nWings of Fire,Kalam,1999,B1\nWings of Fire,Kalam,1999,B2\nOther,Someone,2000,B3\n");
        assert_eq!(r.records.len(), 2);
        assert_eq!(r.records[0].copies.len(), 2);
        let by_isbn = parse("title,isbn,barcode\nA,0-306-40615-2,1\nA again,9780306406157,2\n");
        assert_eq!(by_isbn.records.len(), 1, "same ISBN in either form groups together");
    }

    #[test]
    fn a_copies_column_creates_unnumbered_copies() {
        let r = parse("title,copies\nBook,3\n");
        assert_eq!(r.records[0].copies.len(), 3);
        assert!(r.records[0].copies.iter().all(|c| c.barcode.is_none()));
        let none = parse("title,author\nBook,X\n");
        assert!(none.records[0].copies.is_empty(), "no copy columns means a title-only record");
    }

    #[test]
    fn bad_values_become_warnings_not_silent_drops() {
        let r = parse("title,year,price,date acquired,barcode\nBook,nineteen,lots,31/12/2020,B1\n");
        let rec = &r.records[0];
        assert_eq!(rec.year, None);
        assert_eq!(rec.warnings.len(), 3, "{:?}", rec.warnings);
    }

    #[test]
    fn authors_split_on_semicolons_and_keep_surname_first_names_intact() {
        let r = parse("title,author,additional authors\nBook,\"Roy, Arundhati; Kalam, A. P. J.\",\"Nair, X|Menon, Y\"\n");
        let names: Vec<_> = r.records[0].contributors.iter().map(|c| (c.name.as_str(), c.role.as_str())).collect();
        assert_eq!(names, [("Roy, Arundhati", "author"), ("Kalam, A. P. J.", "author"), ("Nair, X", "additional_author"), ("Menon, Y", "additional_author")]);
    }

    #[test]
    fn manual_mapping_overrides_detection_and_ignores_unknown_columns() {
        let mut m = HashMap::new();
        m.insert("title".to_string(), "Heading".to_string());
        m.insert("author".to_string(), "Nope".to_string());
        let r = parse_csv(b"Heading,Writer\nBook,X\n", Some(&m));
        assert_eq!(r.records[0].title, "Book");
        assert!(r.records[0].contributors.is_empty());
    }

    #[test]
    fn blank_rows_are_skipped_and_empty_titles_are_kept_for_reporting() {
        let r = parse("title,author\nA,X\n,\n,Y\n");
        assert_eq!(r.records.len(), 2);
        assert_eq!(r.records[1].title, "");
        assert_eq!(r.records[1].row, 4);
    }

    #[test]
    fn garbage_does_not_panic() {
        let _ = parse_csv(&[0xFF, 0xFE, 0x00, 0x01], None);
        let _ = parse_csv(b"", None);
        let _ = parse_csv(b"\"unterminated,quote\nx", None);
    }
}
