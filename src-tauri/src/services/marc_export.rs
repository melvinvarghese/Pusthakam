//! MARC21 UTF-8 exchange with Koha 952 holdings. No internal database identifiers.
use super::{
    catalogue_export::{escape, money},
    import::ImportRecord,
};
use crate::error::{AppError, AppResult};

type Fields = Vec<(String, Vec<(char, String)>)>;
fn add(fields: &mut Fields, tag: &str, code: char, value: Option<&str>) {
    if let Some(value) = value.filter(|v| !v.is_empty()) {
        fields.push((tag.into(), vec![(code, value.into())]));
    }
}

pub fn encode(
    records: &[ImportRecord],
    xml: bool,
    warnings: &mut Vec<String>,
) -> AppResult<Vec<u8>> {
    let mut output = if xml {
        b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><collection xmlns=\"http://www.loc.gov/MARC21/slim\">".to_vec()
    } else {
        Vec::new()
    };
    for r in records {
        let mut f = Fields::new();
        let mut title = vec![('a', r.title.clone())];
        if let Some(subtitle) = &r.subtitle {
            title.push(('b', subtitle.clone()));
        }
        f.push(("245".into(), title));
        add(&mut f, "020", 'a', r.isbn.as_deref());
        add(&mut f, "250", 'a', r.edition.as_deref());
        let imprint: Vec<_> = [('a', r.place.as_deref()), ('b', r.publisher.as_deref())]
            .into_iter()
            .filter_map(|(c, v)| v.map(|s| (c, s.to_owned())))
            .collect();
        let mut imprint = imprint;
        if let Some(year) = r.year {
            imprint.push(('c', year.to_string()));
        }
        if !imprint.is_empty() {
            f.push(("264".into(), imprint));
        }
        let physical: Vec<_> = [('a', r.pagination.as_deref()), ('c', r.physical.as_deref())]
            .into_iter()
            .filter_map(|(c, v)| v.map(|s| (c, s.to_owned())))
            .collect();
        if !physical.is_empty() {
            f.push(("300".into(), physical));
        }
        for c in &r.contributors {
            if c.role == "author" {
                add(&mut f, "100", 'a', Some(&c.name));
            } else {
                f.push((
                    "700".into(),
                    vec![('a', c.name.clone()), ('e', c.role.clone())],
                ));
            }
        }
        for subject in &r.subjects {
            add(&mut f, "650", 'a', Some(subject));
        }
        add(&mut f, "041", 'a', r.language.as_deref());
        add(&mut f, "490", 'a', r.series.as_deref());
        add(&mut f, "520", 'a', r.abstract_text.as_deref());
        add(&mut f, "500", 'a', r.notes.as_deref());
        add(&mut f, "653", 'a', r.keywords.as_deref());
        let tag = match r.classification_scheme.as_deref().unwrap_or("ddc") {
            "ddc" => "082", "udc" => "080", "lcc" => "050",
            _ => return Err(AppError::Validation("MARC export supports Dewey, UDC and LC classifications. Choose JSON, CSV or XLSX for a custom scheme.".into())),
        };
        add(&mut f, tag, 'a', r.classification.as_deref());
        for c in &r.copies {
            let mut holding: Vec<_> = [
                ('p', c.barcode.as_deref()),
                ('i', c.accession.as_deref()),
                ('o', c.call_number.as_deref()),
                ('b', c.location.as_deref()),
                ('c', c.shelf.as_deref()),
                ('8', c.collection.as_deref()),
                ('y', c.item_category.as_deref()),
                ('d', c.acquisition_date.as_deref()),
                ('e', c.vendor.as_deref()),
                ('z', c.notes.as_deref()),
            ]
            .into_iter()
            .filter_map(|(code, value)| value.map(|s| (code, s.to_owned())))
            .collect();
            if c.price_minor.is_some() {
                holding.push(('g', money(c.price_minor)));
            }
            match c.status.as_deref() {
                Some("lost") => holding.push(('1', "1".into())),
                Some("withdrawn") => holding.push(('0', "1".into())),
                Some("damaged") => holding.push(('4', "1".into())),
                Some("in_repair") => {
                    let warning = "MARC has no mapped repair status; copies in repair are represented as available. JSON, CSV and XLSX preserve repair status.";
                    if !warnings.iter().any(|s| s == warning) {
                        warnings.push(warning.into());
                    }
                }
                _ => (),
            }
            f.push(("952".into(), holding));
        }
        if xml {
            output.extend_from_slice(b"<record><leader>00000nam a2200000   4500</leader>");
            for (tag, subs) in f {
                output.extend_from_slice(
                    format!("<datafield tag=\"{tag}\" ind1=\" \" ind2=\" \">").as_bytes(),
                );
                for (code, value) in subs {
                    output.extend_from_slice(
                        format!("<subfield code=\"{code}\">{}</subfield>", escape(&value))
                            .as_bytes(),
                    );
                }
                output.extend_from_slice(b"</datafield>");
            }
            output.extend_from_slice(b"</record>");
        } else {
            output.extend(iso2709(&f)?);
        }
    }
    if xml {
        output.extend_from_slice(b"</collection>");
    }
    warnings.push("MARC reimport applies MARC punctuation, subject and language normalization. Use the Koha profile to read 952 holdings; 952$i carries the accession/inventory number.".into());
    Ok(output)
}

fn iso2709(fields: &Fields) -> AppResult<Vec<u8>> {
    let mut directory = Vec::new();
    let mut data = Vec::new();
    for (tag, subs) in fields {
        let start = data.len();
        data.extend_from_slice(b"  ");
        for (code, value) in subs {
            if value.bytes().any(|b| [0x1d, 0x1e, 0x1f].contains(&b)) {
                return Err(AppError::Validation(
                    "MARC control characters in catalogue text cannot be exported.".into(),
                ));
            }
            data.extend_from_slice(&[0x1f, *code as u8]);
            data.extend_from_slice(value.as_bytes());
        }
        data.push(0x1e);
        let len = data.len() - start;
        if len > 9999 || start > 99999 {
            return Err(AppError::Validation(
                "A MARC field is too large. Use MARCXML or JSON.".into(),
            ));
        }
        directory.extend_from_slice(format!("{tag}{len:04}{start:05}").as_bytes());
    }
    let base = 24 + directory.len() + 1;
    let total = base + data.len() + 1;
    if total > 99999 {
        return Err(AppError::Validation(
            "A MARC record exceeds 99,999 bytes. Use MARCXML or JSON.".into(),
        ));
    }
    let mut out = format!("{total:05}nam a22{base:05}   4500").into_bytes();
    out.extend(directory);
    out.push(0x1e);
    out.extend(data);
    out.push(0x1d);
    Ok(out)
}
