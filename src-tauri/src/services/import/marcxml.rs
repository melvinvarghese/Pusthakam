//! Strict MARC21 XML reader. DTDs and custom entities are never accepted.
use super::marc::{Field, MarcRecord};
use crate::error::{AppError, AppResult};
use quick_xml::{events::Event, Reader};

fn append_text(stack: &[String], text: &mut String, value: &str) -> AppResult<()> {
    if !matches!(
        stack.last().map(String::as_str),
        Some("subfield" | "controlfield" | "leader")
    ) && !value.trim().is_empty()
    {
        return Err(invalid());
    }
    text.push_str(value);
    Ok(())
}

fn invalid() -> AppError {
    AppError::Validation("Invalid MARCXML. Use a UTF-8 MARC21 record or collection without DTDs or custom entities. Nothing was imported.".into())
}

pub fn parse(bytes: &[u8]) -> AppResult<Vec<MarcRecord>> {
    std::str::from_utf8(bytes).map_err(|_| invalid())?;
    let mut reader = Reader::from_reader(bytes);
    reader.config_mut().expand_empty_elements = true;
    let mut stack: Vec<String> = Vec::new();
    let mut records = Vec::new();
    let mut record = MarcRecord::default();
    let mut field: Option<Field> = None;
    let mut code = None;
    let mut text = String::new();
    let mut roots = 0;
    loop {
        match reader.read_event().map_err(|_| invalid())? {
            Event::Start(e) => {
                let name = e.local_name().as_ref().to_string();
                let parent = stack.last().map(String::as_str);
                let valid = matches!(
                    (parent, name.as_str()),
                    (None, "collection" | "record")
                        | (Some("collection"), "record")
                        | (Some("record"), "leader" | "controlfield" | "datafield")
                        | (Some("datafield"), "subfield")
                );
                if !valid {
                    return Err(invalid());
                }
                if parent.is_none() {
                    roots += 1;
                    if roots != 1 {
                        return Err(invalid());
                    }
                }
                let mut tag = None;
                code = None;
                for attr in e.attributes() {
                    let attr = attr.map_err(|_| invalid())?;
                    let value = attr
                        .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                        .map_err(|_| invalid())?
                        .into_owned();
                    match attr.key.as_ref() {
                        "tag" => tag = Some(value),
                        "code" => {
                            if value.chars().count() != 1 {
                                return Err(invalid());
                            }
                            code = value.chars().next();
                        }
                        _ => (),
                    }
                }
                if name == "record" {
                    record = MarcRecord::default();
                }
                if name == "datafield" || name == "controlfield" {
                    let tag = tag.ok_or_else(invalid)?;
                    if tag.len() != 3 || !tag.bytes().all(|b| b.is_ascii_digit()) {
                        return Err(invalid());
                    }
                    field = Some(Field {
                        tag,
                        control: None,
                        subfields: Vec::new(),
                    });
                }
                if name == "subfield" && code.is_none() {
                    return Err(invalid());
                }
                text.clear();
                stack.push(name);
            }
            Event::Text(e) => {
                let value = e.xml_content(quick_xml::XmlVersion::Implicit1_0);
                append_text(&stack, &mut text, &value)?;
            }
            Event::GeneralRef(e) => {
                let entity = e.as_ref();
                let value = quick_xml::escape::unescape(&format!("&{entity};"))
                    .map_err(|_| invalid())?
                    .into_owned();
                append_text(&stack, &mut text, &value)?;
            }
            Event::CData(e) => append_text(&stack, &mut text, e.as_ref())?,
            Event::End(_) => {
                let name = stack.pop().ok_or_else(invalid)?;
                match name.as_str() {
                    "subfield" => field
                        .as_mut()
                        .ok_or_else(invalid)?
                        .subfields
                        .push((code.ok_or_else(invalid)?, text.clone())),
                    "controlfield" => {
                        let mut f = field.take().ok_or_else(invalid)?;
                        f.control = Some(text.clone());
                        record.fields.push(f);
                    }
                    "datafield" => record.fields.push(field.take().ok_or_else(invalid)?),
                    "record" => records.push(std::mem::take(&mut record)),
                    _ => (),
                }
                text.clear();
            }
            Event::DocType(_) => return Err(invalid()),
            Event::Eof => break,
            Event::Decl(_) | Event::Comment(_) | Event::PI(_) => (),
            _ => return Err(invalid()),
        }
    }
    if roots != 1 || !stack.is_empty() {
        return Err(invalid());
    }
    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reads_namespaced_koha_and_entities() {
        let xml = r#"<m:collection xmlns:m="http://www.loc.gov/MARC21/slim"><m:record><m:datafield tag="245" ind1="0" ind2="0"><m:subfield code="a">Books &amp; reading &#233;</m:subfield></m:datafield><m:datafield tag="952"><m:subfield code="p">000123</m:subfield><m:subfield code="g">125.50</m:subfield></m:datafield></m:record></m:collection>"#;
        let rows = parse(xml.as_bytes()).unwrap();
        let r = super::super::marc::to_import_record(&rows[0], 1, true);
        assert_eq!(r.title, "Books & reading \u{e9}");
        assert_eq!(r.copies[0].barcode.as_deref(), Some("000123"));
        assert_eq!(r.copies[0].price_minor, Some(12550));
        assert!(super::super::marc::to_import_record(&rows[0], 1, false)
            .copies
            .is_empty());
    }
    #[test]
    fn rejects_malformed_or_unsafe_whole_files() {
        for xml in [
            "<record>",
            "<record></collection>",
            "<foo/>",
            "<record/><record/>",
            "<!DOCTYPE record><record/>",
            "<record><datafield tag='bad'/></record>",
            "<record><datafield tag='245'><subfield>Book</subfield></datafield></record>",
            "<record><leader>&custom;</leader></record>",
            "<record><![CDATA[discarded]]></record>",
            "<record>&amp;</record>",
            "<record><datafield tag='245'><![CDATA[discarded]]></datafield></record>",
        ] {
            assert!(parse(xml.as_bytes()).is_err(), "accepted {xml}");
        }
    }
}
