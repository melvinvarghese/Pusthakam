//! ISO 2709 (MARC) reader and mapping from MARC21 to import records, with a Koha profile.
//!
//! Koha keeps its holdings in field 952 (one field per item). Subfield meanings used here follow Koha's
//! default MARC21 framework (a = home library, b = holding library, c = shelving location, d = date
//! accessioned, e = source of acquisition, g = price, o = call number, p = barcode, y = item type,
//! 8 = collection code, 0/1/4/7 = withdrawn / lost / damaged / not-for-loan flags). Libraries can change
//! their framework, so the preview always shows what will be imported before anything is saved.

use super::{parse_price_minor, ImportCopy, ImportRecord};
use crate::domain::validate;
use crate::services::catalogue::ContributorDto;

#[derive(Debug, Clone)]
pub struct Field {
    pub tag: String,
    pub control: Option<String>,
    pub subfields: Vec<(char, String)>,
}

impl Field {
    pub fn sub(&self, code: char) -> Option<&str> {
        self.subfields
            .iter()
            .find(|(c, _)| *c == code)
            .map(|(_, v)| v.as_str())
            .filter(|v| !v.is_empty())
    }
    pub fn subs(&self, code: char) -> Vec<&str> {
        self.subfields
            .iter()
            .filter(|(c, _)| *c == code)
            .map(|(_, v)| v.as_str())
            .filter(|v| !v.is_empty())
            .collect()
    }
}

#[derive(Debug, Clone, Default)]
pub struct MarcRecord {
    pub fields: Vec<Field>,
    pub warnings: Vec<String>,
}

impl MarcRecord {
    fn fields<'a>(&'a self, tag: &'a str) -> impl Iterator<Item = &'a Field> + 'a {
        self.fields.iter().filter(move |f| f.tag == tag)
    }
    fn first_sub<'a>(&'a self, tag: &'a str, code: char) -> Option<&'a str> {
        self.fields(tag).find_map(|f| f.sub(code))
    }
}

fn digits(b: &[u8]) -> Option<usize> {
    std::str::from_utf8(b).ok()?.trim().parse::<usize>().ok()
}

fn decode(bytes: &[u8], note: &mut Option<String>) -> String {
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => {
            *note = Some("Some text was not valid UTF-8 and was read as Latin-1; accented characters may look wrong.".into());
            bytes.iter().map(|&b| b as char).collect()
        }
    }
}

/// Reads every record in an ISO 2709 file. Damaged records are reported and skipped, never silently dropped.
pub fn read_iso2709(data: &[u8]) -> (Vec<MarcRecord>, Vec<String>) {
    let mut records = Vec::new();
    let mut errors = Vec::new();
    let mut pos = 0usize;
    let mut n = 0usize;
    if data.starts_with(&[0xEF, 0xBB, 0xBF]) {
        pos = 3;
    }
    while pos < data.len() {
        while pos < data.len() && matches!(data[pos], b'\n' | b'\r' | b' ' | 0x00 | 0x1A) {
            pos += 1;
        }
        if pos >= data.len() {
            break;
        }
        n += 1;
        let rest = &data[pos..];
        if rest.len() < 24 {
            errors.push(format!(
                "Record {n}: the file ends in the middle of a record."
            ));
            break;
        }
        let declared = digits(&rest[0..5]);
        let end = match declared {
            Some(l) if l >= 24 && l <= rest.len() && rest[l - 1] == 0x1D => pos + l,
            // Wrong or missing length: resynchronise on the next record terminator.
            _ => match rest.iter().position(|&b| b == 0x1D) {
                Some(p) => pos + p + 1,
                None => data.len(),
            },
        };
        let bytes = &data[pos..end];
        pos = end;
        match parse_record(bytes) {
            Ok(r) => records.push(r),
            Err(e) => errors.push(format!("Record {n}: {e}")),
        }
    }
    (records, errors)
}

fn parse_record(rec: &[u8]) -> Result<MarcRecord, String> {
    if rec.len() < 25 {
        return Err("the record is too short.".into());
    }
    let base = digits(&rec[12..17]).ok_or("the leader has no valid base address.")?;
    if base < 25 || base > rec.len() {
        return Err("the base address points outside the record.".into());
    }
    let dir_end = if rec[base - 1] == 0x1E {
        base - 1
    } else {
        base
    };
    let dir = &rec[24..dir_end];
    if dir.len() % 12 != 0 {
        return Err("the directory is damaged.".into());
    }
    let mut note: Option<String> = None;
    let mut fields = Vec::new();
    for e in dir.chunks(12) {
        let tag = String::from_utf8_lossy(&e[0..3]).to_string();
        let len = digits(&e[3..7]).ok_or("a directory entry has no length.")?;
        let start = digits(&e[7..12]).ok_or("a directory entry has no start position.")?;
        let from = base + start;
        let to = from + len;
        if to > rec.len() {
            return Err(format!("field {tag} runs past the end of the record."));
        }
        let mut data = &rec[from..to];
        if data.last() == Some(&0x1E) {
            data = &data[..data.len() - 1];
        }
        if tag.as_str() < "010" {
            fields.push(Field {
                tag,
                control: Some(decode(data, &mut note)),
                subfields: vec![],
            });
            continue;
        }
        let body = if data.len() >= 2 { &data[2..] } else { &[][..] };
        let mut subfields = Vec::new();
        for chunk in body.split(|&b| b == 0x1F).skip(1) {
            if chunk.is_empty() {
                continue;
            }
            let code = chunk[0] as char;
            subfields.push((code, decode(&chunk[1..], &mut note).trim().to_string()));
        }
        fields.push(Field {
            tag,
            control: None,
            subfields,
        });
    }
    Ok(MarcRecord {
        fields,
        warnings: note.into_iter().collect(),
    })
}

// ---------- text clean-up ----------

/// Removes trailing ISBD punctuation such as " /" or " :" from a title part.
fn isbd(s: &str) -> String {
    s.trim()
        .trim_end_matches([' ', '/', ':', ';', ',', '='])
        .trim()
        .to_string()
}

fn person(s: &str) -> String {
    let t = s.trim().trim_end_matches([',', ' ']).trim();
    // Drop a closing full stop after a word ("Kalam.") but keep initials ("A. P. J.").
    if let Some(stripped) = t.strip_suffix('.') {
        let last_word = stripped.rsplit(' ').next().unwrap_or("");
        if last_word.chars().count() > 2 {
            return stripped.to_string();
        }
    }
    t.to_string()
}

fn first_year(s: &str) -> Option<i64> {
    let b: Vec<char> = s.chars().collect();
    for w in b.windows(4) {
        if w.iter().all(|c| c.is_ascii_digit()) {
            let y: String = w.iter().collect();
            if let Ok(v) = y.parse::<i64>() {
                if (1000..=2200).contains(&v) {
                    return Some(v);
                }
            }
        }
    }
    None
}

fn language_name(code: &str) -> String {
    match code.trim().to_ascii_lowercase().as_str() {
        "eng" => "English",
        "mal" => "Malayalam",
        "hin" => "Hindi",
        "tam" => "Tamil",
        "tel" => "Telugu",
        "kan" => "Kannada",
        "san" => "Sanskrit",
        "ben" => "Bengali",
        "urd" => "Urdu",
        "ara" => "Arabic",
        "fre" | "fra" => "French",
        "ger" | "deu" => "German",
        "spa" => "Spanish",
        "lat" => "Latin",
        other => return other.to_string(),
    }
    .to_string()
}

fn ddc(raw: &str) -> String {
    let cleaned: String = raw
        .split_whitespace()
        .next()
        .unwrap_or("")
        .chars()
        .filter(|c| *c != '/' && *c != '\'')
        .collect();
    cleaned
}

fn role_from_relator(rel: Option<&str>) -> &'static str {
    let r = rel.unwrap_or("").to_ascii_lowercase();
    if r.starts_with("ed") {
        "editor"
    } else if r.starts_with("tr") {
        "translator"
    } else if r.starts_with("ill") {
        "illustrator"
    } else {
        "additional_author"
    }
}

fn flagged(f: &Field, code: char) -> bool {
    f.sub(code).is_some_and(|v| v != "0" && !v.is_empty())
}

// ---------- mapping ----------

/// Maps one MARC21 record to an import record. With `koha`, field 952 holdings become copies.
pub fn to_import_record(m: &MarcRecord, row: usize, koha: bool) -> ImportRecord {
    let mut r = ImportRecord {
        row,
        warnings: m.warnings.clone(),
        ..Default::default()
    };

    if let Some(f) = m.fields("245").next() {
        let mut title = isbd(f.sub('a').unwrap_or(""));
        for part in f.subs('n').into_iter().chain(f.subs('p')) {
            title = format!("{title}. {}", isbd(part));
        }
        r.title = title;
        r.subtitle = f.sub('b').map(isbd).filter(|s| !s.is_empty());
    }

    for f in m
        .fields("100")
        .chain(m.fields("110"))
        .chain(m.fields("111"))
    {
        if let Some(a) = f.sub('a') {
            r.contributors.push(ContributorDto {
                name: person(a),
                role: "author".into(),
            });
        }
    }
    for f in m
        .fields("700")
        .chain(m.fields("710"))
        .chain(m.fields("711"))
    {
        if let Some(a) = f.sub('a') {
            r.contributors.push(ContributorDto {
                name: person(a),
                role: role_from_relator(f.sub('e')).into(),
            });
        }
    }

    r.isbn = m
        .fields("020")
        .filter_map(|f| f.sub('a'))
        .map(|v| v.split_whitespace().next().unwrap_or("").to_string())
        .find(|v| !v.is_empty());
    r.edition = m.first_sub("250", 'a').map(isbd);
    let imprint = m
        .fields("264")
        .find(|f| f.sub('c').is_some() || f.sub('b').is_some())
        .or_else(|| m.fields("260").next());
    if let Some(f) = imprint {
        r.place = f.sub('a').map(isbd).filter(|s| !s.is_empty());
        r.publisher = f.sub('b').map(isbd).filter(|s| !s.is_empty());
        r.year = f.sub('c').and_then(first_year);
    }
    if let Some(f) = m.fields("300").next() {
        r.pagination = f.sub('a').map(isbd).filter(|s| !s.is_empty());
        let phys: Vec<String> = [f.sub('b'), f.sub('c')]
            .into_iter()
            .flatten()
            .map(isbd)
            .filter(|s| !s.is_empty())
            .collect();
        if !phys.is_empty() {
            r.physical = Some(phys.join("; "));
        }
    }
    r.series = m
        .fields("490")
        .chain(m.fields("440"))
        .find_map(|f| f.sub('a'))
        .map(isbd)
        .filter(|s| !s.is_empty());
    r.abstract_text = m.first_sub("520", 'a').map(str::to_string);
    let notes: Vec<&str> = m.fields("500").filter_map(|f| f.sub('a')).collect();
    if !notes.is_empty() {
        r.notes = Some(notes.join("; "));
    }
    for f in m
        .fields("650")
        .chain(m.fields("600"))
        .chain(m.fields("651"))
        .chain(m.fields("610"))
    {
        if let Some(a) = f.sub('a') {
            // Subject headings end with a full stop in MARC; it is punctuation, not part of the heading.
            let stop = |s: &str| isbd(s).trim_end_matches('.').trim().to_string();
            let mut h = stop(a);
            for x in f
                .subs('x')
                .into_iter()
                .chain(f.subs('z'))
                .chain(f.subs('v'))
            {
                h = format!("{h} -- {}", stop(x));
            }
            if !h.is_empty() {
                r.subjects.push(h);
            }
        }
    }
    let kw: Vec<String> = m
        .fields("653")
        .flat_map(|f| f.subs('a'))
        .map(isbd)
        .collect();
    if !kw.is_empty() {
        r.keywords = Some(kw.join("; "));
    }
    r.classification = m.first_sub("082", 'a').map(ddc).filter(|s| !s.is_empty());
    if r.classification.is_some() {
        r.classification_scheme = Some("ddc".into());
    } else {
        for (tag, scheme) in [("080", "udc"), ("050", "lcc")] {
            if let Some(number) = m.first_sub(tag, 'a') {
                r.classification = Some(number.to_string());
                r.classification_scheme = Some(scheme.into());
                break;
            }
        }
    }

    // Language: 008 positions 35-37, else 041 $a.
    let lang = m
        .fields("008")
        .find_map(|f| f.control.as_deref())
        .and_then(|c| c.get(35..38).map(str::to_string))
        .filter(|c| {
            c.trim().chars().all(|ch| ch.is_ascii_alphabetic()) && c.trim().len() == 3 && c != "   "
        })
        .or_else(|| m.first_sub("041", 'a').map(str::to_string));
    r.language = lang.map(|c| language_name(&c));

    if koha {
        let default_type = m.first_sub("942", 'c').map(str::to_string);
        if let Some(bn) = m.first_sub("999", 'c') {
            let note = format!("Imported from Koha (biblionumber {bn}).");
            r.notes = Some(match r.notes.take() {
                Some(n) => format!("{n} {note}"),
                None => note,
            });
        }
        for f in m.fields("952") {
            let mut c = ImportCopy {
                accession: f.sub('i').map(str::to_string),
                barcode: f.sub('p').map(str::to_string),
                call_number: f.sub('o').map(str::to_string),
                ..Default::default()
            };
            c.location = f.sub('b').or_else(|| f.sub('a')).map(str::to_string);
            c.shelf = f.sub('c').map(str::to_string);
            c.collection = f.sub('8').map(str::to_string);
            c.item_category = f
                .sub('y')
                .map(str::to_string)
                .or_else(|| default_type.clone());
            c.vendor = f.sub('e').map(str::to_string);
            if let Some(d) = f.sub('d') {
                match validate::date("Date", d) {
                    Ok(v) => c.acquisition_date = Some(v),
                    Err(_) => r.warnings.push(format!(
                        "Copy {}: date \"{d}\" is not YYYY-MM-DD and was left empty.",
                        c.barcode.as_deref().unwrap_or("?")
                    )),
                }
            }
            c.price_minor = f.sub('g').and_then(parse_price_minor);
            // Item flags: any non-zero value means the flag applies. The most serious wins.
            c.status = if flagged(f, '1') {
                Some("lost".into())
            } else if flagged(f, '0') {
                Some("withdrawn".into())
            } else if flagged(f, '4') {
                Some("damaged".into())
            } else {
                None
            };
            let mut notes: Vec<String> = [f.sub('z'), f.sub('x')]
                .into_iter()
                .flatten()
                .map(str::to_string)
                .collect();
            if flagged(f, '7') {
                notes.push("Marked not for loan in Koha.".into());
            }
            if !notes.is_empty() {
                c.notes = Some(notes.join(" "));
            }
            r.copies.push(c);
        }
    }
    r
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// Builds a valid ISO 2709 record for tests. Each field is (tag, indicators+subfields) where subfields are (code, text).
    pub fn build(fields: &[(&str, Vec<(char, &str)>)], control: &[(&str, &str)]) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        let mut dir: Vec<u8> = Vec::new();
        let mut push = |tag: &str, bytes: Vec<u8>| {
            let start = data.len();
            data.extend_from_slice(&bytes);
            data.push(0x1E);
            dir.extend_from_slice(format!("{tag}{:04}{:05}", bytes.len() + 1, start).as_bytes());
        };
        for (tag, text) in control {
            push(tag, text.as_bytes().to_vec());
        }
        for (tag, subs) in fields {
            let mut b = vec![b' ', b' '];
            for (code, text) in subs {
                b.push(0x1F);
                b.push(*code as u8);
                b.extend_from_slice(text.as_bytes());
            }
            push(tag, b);
        }
        let base = 24 + dir.len() + 1;
        let total = base + data.len() + 1;
        let mut rec = format!("{total:05}nam a22{base:05} a 4500").into_bytes();
        rec.extend_from_slice(&dir);
        rec.push(0x1E);
        rec.extend_from_slice(&data);
        rec.push(0x1D);
        rec
    }

    fn koha_record() -> Vec<u8> {
        build(
            &[
                ("020", vec![('a', "0-306-40615-2 (pbk.)")]),
                ("082", vec![('a', "894/.8123")]),
                ("100", vec![('a', "Basheer, Vaikom Muhammad,")]),
                ("245", vec![('a', "ബാല്യകാലസഖി /"), ('b', "a novel :")]),
                ("250", vec![('a', "5th ed.")]),
                (
                    "260",
                    vec![('a', "Kottayam :"), ('b', "DC Books,"), ('c', "c1944.")],
                ),
                ("300", vec![('a', "120 p. ;"), ('c', "20 cm.")]),
                ("500", vec![('a', "A classic.")]),
                ("650", vec![('a', "Malayalam fiction"), ('x', "History.")]),
                ("700", vec![('a', "Nair, Someone."), ('e', "translator.")]),
                ("942", vec![('c', "BK")]),
                (
                    "952",
                    vec![
                        ('a', "MAIN"),
                        ('b', "MAIN"),
                        ('c', "FIC"),
                        ('d', "2026-11-02"),
                        ('g', "250.00"),
                        ('o', "894.8123 BAS"),
                        ('p', "004217"),
                        ('y', "BK"),
                        ('8', "FICTION"),
                    ],
                ),
                (
                    "952",
                    vec![
                        ('a', "MAIN"),
                        ('b', "ANNEX"),
                        ('d', "2026-13-45"),
                        ('p', "004218"),
                        ('1', "2026-01-01"),
                    ],
                ),
                (
                    "952",
                    vec![
                        ('b', "MAIN"),
                        ('p', "004219"),
                        ('4', "1"),
                        ('7', "1"),
                        ('z', "Cover torn"),
                    ],
                ),
                ("999", vec![('c', "1234"), ('d', "1234")]),
            ],
            &[("008", "261102s1944    ii            000 0 mal d")],
        )
    }

    #[test]
    fn reads_records_and_subfields_including_unicode() {
        let (recs, errs) = read_iso2709(&koha_record());
        assert!(errs.is_empty(), "{errs:?}");
        assert_eq!(recs.len(), 1);
        let f245 = recs[0].fields("245").next().unwrap();
        assert_eq!(f245.sub('a'), Some("ബാല്യകാലസഖി /"));
        assert_eq!(recs[0].fields("952").count(), 3);
    }

    #[test]
    fn koha_record_maps_bibliographic_fields() {
        let (recs, _) = read_iso2709(&koha_record());
        let r = to_import_record(&recs[0], 1, true);
        assert_eq!(r.title, "ബാല്യകാലസഖി");
        assert_eq!(r.subtitle.as_deref(), Some("a novel"));
        assert_eq!(r.isbn.as_deref(), Some("0-306-40615-2"));
        assert_eq!(
            r.contributors[0],
            ContributorDto {
                name: "Basheer, Vaikom Muhammad".into(),
                role: "author".into()
            }
        );
        assert_eq!(
            r.contributors[1],
            ContributorDto {
                name: "Nair, Someone".into(),
                role: "translator".into()
            }
        );
        assert_eq!(
            (r.place.as_deref(), r.publisher.as_deref(), r.year),
            (Some("Kottayam"), Some("DC Books"), Some(1944))
        );
        assert_eq!(r.classification.as_deref(), Some("894.8123"));
        assert_eq!(r.pagination.as_deref(), Some("120 p."));
        assert_eq!(r.language.as_deref(), Some("Malayalam"));
        assert_eq!(r.subjects, ["Malayalam fiction -- History"]);
        assert!(r.notes.as_deref().unwrap().contains("biblionumber 1234"));
    }

    #[test]
    fn koha_952_becomes_copies_with_status_flags() {
        let (recs, _) = read_iso2709(&koha_record());
        let r = to_import_record(&recs[0], 1, true);
        assert_eq!(r.copies.len(), 3);
        let a = &r.copies[0];
        assert_eq!(
            (
                a.barcode.as_deref(),
                a.call_number.as_deref(),
                a.location.as_deref(),
                a.shelf.as_deref(),
                a.collection.as_deref(),
                a.item_category.as_deref(),
                a.acquisition_date.as_deref(),
                a.price_minor
            ),
            (
                Some("004217"),
                Some("894.8123 BAS"),
                Some("MAIN"),
                Some("FIC"),
                Some("FICTION"),
                Some("BK"),
                Some("2026-11-02"),
                Some(25000)
            )
        );
        let b = &r.copies[1];
        assert_eq!(
            (
                b.location.as_deref(),
                b.status.as_deref(),
                b.acquisition_date.as_deref()
            ),
            (Some("ANNEX"), Some("lost"), None)
        );
        assert!(
            r.warnings.iter().any(|w| w.contains("2026-13-45")),
            "a bad date is reported, not hidden"
        );
        assert_eq!(
            b.item_category.as_deref(),
            Some("BK"),
            "falls back to the record's 942 item type"
        );
        let c = &r.copies[2];
        assert_eq!(c.status.as_deref(), Some("damaged"));
        assert!(
            c.notes.as_deref().unwrap().contains("Cover torn")
                && c.notes.as_deref().unwrap().contains("not for loan")
        );
    }

    #[test]
    fn generic_marc_has_no_copies() {
        let (recs, _) = read_iso2709(&koha_record());
        assert!(to_import_record(&recs[0], 1, false).copies.is_empty());
    }

    #[test]
    fn handles_many_records_newlines_and_a_damaged_one() {
        let good = koha_record();
        let mut file = Vec::new();
        file.extend_from_slice(&good);
        file.extend_from_slice(b"\r\n");
        // A record whose declared length is wrong but which still ends with the terminator.
        let mut bad = good.clone();
        bad[0..5].copy_from_slice(b"99999");
        file.extend_from_slice(&bad);
        file.push(b'\n');
        // A record with a damaged directory.
        let mut broken = good.clone();
        broken[24] = b'X';
        broken[27..31].copy_from_slice(b"ZZZZ");
        file.extend_from_slice(&broken);
        file.extend_from_slice(&good);
        let (recs, errs) = read_iso2709(&file);
        assert_eq!(
            recs.len(),
            3,
            "the good records and the one with a wrong length are read"
        );
        assert_eq!(errs.len(), 1, "the damaged record is reported: {errs:?}");
        assert!(errs[0].starts_with("Record 3"));
    }

    #[test]
    fn truncated_files_and_garbage_do_not_panic() {
        let good = koha_record();
        for cut in [0, 5, 23, 24, 40, good.len() / 2, good.len() - 1] {
            let _ = read_iso2709(&good[..cut]);
        }
        let _ = read_iso2709(b"this is not a marc file at all, just text\n");
        let _ = read_iso2709(&[0xFF; 100]);
    }

    #[test]
    fn non_utf8_text_is_flagged_not_lost() {
        let mut rec = build(&[("245", vec![('a', "Caf\u{e9} title")])], &[]);
        // Replace the UTF-8 sequence for é (C3 A9) by the single Latin-1 byte E9 and fix the lengths.
        let pos = rec.windows(2).position(|w| w == [0xC3, 0xA9]).unwrap();
        rec[pos] = 0xE9;
        rec.remove(pos + 1);
        let total = rec.len();
        rec[0..5].copy_from_slice(format!("{total:05}").as_bytes());
        // Field length in the directory also shrinks by one.
        let len = digits(&rec[27..31]).unwrap() - 1;
        rec[27..31].copy_from_slice(format!("{len:04}").as_bytes());
        let (recs, errs) = read_iso2709(&rec);
        assert!(errs.is_empty(), "{errs:?}");
        assert!(recs[0].warnings[0].contains("Latin-1"));
        assert_eq!(
            recs[0].fields("245").next().unwrap().sub('a'),
            Some("Café title")
        );
    }

    #[test]
    fn punctuation_helpers() {
        assert_eq!(isbd("Title /"), "Title");
        assert_eq!(isbd("Subtitle :"), "Subtitle");
        assert_eq!(person("Kalam, A. P. J."), "Kalam, A. P. J.");
        assert_eq!(person("Roy, Arundhati."), "Roy, Arundhati");
        assert_eq!(first_year("c1944."), Some(1944));
        assert_eq!(first_year("[1999?]"), Some(1999));
        assert_eq!(first_year("n.d."), None);
        assert_eq!(ddc("894/.8123 23"), "894.8123");
    }
}
