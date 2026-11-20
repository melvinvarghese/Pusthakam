use crate::error::{AppError, AppResult};
use quick_xml::escape::escape;
use serde::Serialize;
use std::io::{Cursor, Write};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportExportFile {
    pub file_name: String,
    pub mime_type: String,
    pub data_base64: String,
}

pub fn generate_xlsx(
    sheet_name: &str,
    headers: &[String],
    rows: &[Vec<String>],
) -> AppResult<Vec<u8>> {
    let mut all_rows = Vec::with_capacity(rows.len() + 1);
    all_rows.push(headers.to_vec());
    all_rows.extend_from_slice(rows);

    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut part = |name: &str, content: &str| -> AppResult<()> {
        zip.start_file(
            name,
            zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated),
        )
        .map_err(|e| AppError::Internal(format!("Zip start error: {e}")))?;
        zip.write_all(content.as_bytes())
            .map_err(|e| AppError::Internal(format!("Zip write error: {e}")))
    };

    part(
        "[Content_Types].xml",
        r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/><Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/></Types>"#,
    )?;
    part(
        "_rels/.rels",
        r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/></Relationships>"#,
    )?;
    let clean_sheet = escape(sheet_name);
    part(
        "xl/workbook.xml",
        &format!(
            r#"<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets><sheet name="{clean_sheet}" sheetId="1" r:id="rId1"/></sheets></workbook>"#
        ),
    )?;
    part(
        "xl/_rels/workbook.xml.rels",
        r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/></Relationships>"#,
    )?;

    let mut xml = String::from(
        r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData>"#,
    );
    for (i, row) in all_rows.iter().enumerate() {
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

    Ok(zip
        .finish()
        .map_err(|e| AppError::Internal(format!("Zip finish error: {e}")))?
        .into_inner())
}

fn escape_pdf_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '(' => out.push_str("\\("),
            ')' => out.push_str("\\)"),
            '\\' => out.push_str("\\\\"),
            '\r' | '\n' | '\t' => out.push(' '),
            c if (c as u32) >= 32 && (c as u32) <= 126 => out.push(c),
            c if (c as u32) >= 160 && (c as u32) <= 255 => out.push(c),
            _ => out.push('?'),
        }
    }
    out
}

pub fn generate_pdf(
    title: &str,
    subtitle: &str,
    headers: &[String],
    rows: &[Vec<String>],
    alignments: &[String],
    simple: bool,
) -> AppResult<Vec<u8>> {
    let landscape = headers.len() > 5;
    let (page_width, page_height) = if landscape {
        (841.89f32, 595.28f32)
    } else {
        (595.28f32, 841.89f32)
    };

    let margin = 36.0f32;
    let available_width = page_width - (margin * 2.0);

    // Calculate column widths
    let num_cols = headers.len().max(1);
    let mut col_lens = Vec::with_capacity(num_cols);
    for c in 0..num_cols {
        let h_len = headers.get(c).map(|h| h.len()).unwrap_or(4);
        let max_data = rows
            .iter()
            .map(|r| r.get(c).map(|s| s.len()).unwrap_or(0))
            .max()
            .unwrap_or(0);
        col_lens.push(h_len.max(max_data).max(4));
    }
    let total_len: usize = col_lens.iter().sum::<usize>().max(1);

    let mut col_widths: Vec<f32> = col_lens
        .iter()
        .map(|l| ((*l as f32) / (total_len as f32) * available_width).max(35.0))
        .collect();
    let current_total: f32 = col_widths.iter().sum();
    for w in &mut col_widths {
        *w = (*w / current_total) * available_width;
    }

    let row_height = 18.0f32;
    let page1_rows = if landscape { 22 } else { 34 };
    let subsequent_rows = if landscape { 25 } else { 38 };

    // Paginate rows
    let mut pages: Vec<Vec<&[String]>> = Vec::new();
    let mut remaining = rows;
    let mut first = true;
    while !remaining.is_empty() || (pages.is_empty() && rows.is_empty()) {
        let chunk_size = if first { page1_rows } else { subsequent_rows };
        let take = chunk_size.min(remaining.len());
        let (chunk, rest) = remaining.split_at(take);
        pages.push(chunk.iter().map(|r| r.as_slice()).collect());
        remaining = rest;
        first = false;
        if remaining.is_empty() {
            break;
        }
    }
    if pages.is_empty() {
        pages.push(Vec::new());
    }

    let num_pages = pages.len();

    // Build PDF objects
    let mut buf: Vec<u8> = Vec::with_capacity(16384);
    let mut offsets: Vec<usize> = Vec::new();

    // 0-th dummy offset
    offsets.push(0);

    buf.extend_from_slice(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n");

    // Obj 1: Catalog
    offsets.push(buf.len());
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    // Obj 2: Pages root
    offsets.push(buf.len());
    let mut kids = String::from("<< /Type /Pages /Kids [");
    for i in 0..num_pages {
        kids.push_str(&format!("{} 0 R ", 5 + i * 2));
    }
    kids.push_str(&format!("] /Count {} >>\nendobj\n", num_pages));
    buf.extend_from_slice(format!("2 0 obj\n{kids}").as_bytes());

    // Obj 3: Font F1 (Helvetica)
    offsets.push(buf.len());
    buf.extend_from_slice(
        b"3 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n",
    );

    // Obj 4: Font F2 (Helvetica-Bold)
    offsets.push(buf.len());
    buf.extend_from_slice(
        b"4 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold >>\nendobj\n",
    );

    for (p, page_rows) in pages.iter().enumerate() {
        let page_obj = 5 + p * 2;
        let content_obj = 6 + p * 2;

        // Page Object
        offsets.push(buf.len());
        buf.extend_from_slice(
            format!(
                "{page_obj} 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {page_width:.2} {page_height:.2}] /Contents {content_obj} 0 R /Resources << /Font << /F1 3 0 R /F2 4 0 R >> >> >>\nendobj\n"
            )
            .as_bytes(),
        );

        // Content stream
        let mut stream = String::with_capacity(4096);

        // Header / title
        if p == 0 {
            let esc_title = escape_pdf_str(title);
            let esc_sub = escape_pdf_str(subtitle);
            stream.push_str(&format!(
                "BT /F2 13 Tf {margin:.2} {:.2} Td ({esc_title}) Tj ET\n",
                page_height - margin - 14.0
            ));
            if !subtitle.is_empty() {
                stream.push_str(&format!(
                    "BT /F1 8.5 Tf 0.3 0.3 0.3 rg {margin:.2} {:.2} Td ({esc_sub}) Tj 0 0 0 rg ET\n",
                    page_height - margin - 28.0
                ));
            }
        } else {
            let esc_title = escape_pdf_str(title);
            stream.push_str(&format!(
                "BT /F2 9.5 Tf 0.3 0.3 0.3 rg {margin:.2} {:.2} Td ({esc_title} - continued) Tj 0 0 0 rg ET\n",
                page_height - margin - 12.0
            ));
        }

        let header_y = if p == 0 {
            page_height - margin - 48.0
        } else {
            page_height - margin - 26.0
        };

        // Table header bar
        if !simple {
            stream.push_str(&format!(
                "0.92 0.94 0.96 rg {margin:.2} {:.2} {available_width:.2} 18.00 re f\n",
                header_y - 4.0
            ));
        }
        stream.push_str(&format!(
            "0.75 0.78 0.82 RG 0.5 w {margin:.2} {:.2} {available_width:.2} 18.00 re s\n",
            header_y - 4.0
        ));

        // Column headers text
        let mut col_x = margin;
        for (c, h) in headers.iter().enumerate() {
            let w = col_widths.get(c).copied().unwrap_or(40.0);
            let align = alignments.get(c).map(|s| s.as_str()).unwrap_or("left");
            let esc_h = escape_pdf_str(h);
            let text_x = if align == "right" {
                col_x + w - 4.0 - ((h.len() as f32) * 5.0).min(w - 8.0)
            } else {
                col_x + 4.0
            };
            stream.push_str(&format!(
                "BT /F2 8.5 Tf 0.1 0.1 0.1 rg {:.2} {:.2} Td ({esc_h}) Tj 0 0 0 rg ET\n",
                text_x,
                header_y + 1.0
            ));
            col_x += w;
        }

        // Table rows
        for (r_idx, row) in page_rows.iter().enumerate() {
            let row_y = header_y - 4.0 - ((r_idx + 1) as f32 * row_height);
            let is_total = row
                .first()
                .map(|s| s.starts_with("Total") || s.starts_with("All"))
                .unwrap_or(false);

            if is_total {
                if !simple {
                    stream.push_str(&format!(
                        "0.90 0.92 0.95 rg {margin:.2} {:.2} {available_width:.2} {row_height:.2} re f\n",
                        row_y
                    ));
                }
                stream.push_str(&format!(
                    "0.65 0.65 0.65 RG 0.75 w {margin:.2} {:.2} m {:.2} {:.2} l S\n",
                    row_y + row_height,
                    margin + available_width,
                    row_y + row_height
                ));
            } else if !simple && r_idx % 2 == 1 {
                stream.push_str(&format!(
                    "0.98 0.98 0.99 rg {margin:.2} {:.2} {available_width:.2} {row_height:.2} re f\n",
                    row_y
                ));
            }

            // Cell bottom border
            stream.push_str(&format!(
                "0.90 0.90 0.90 RG 0.4 w {margin:.2} {:.2} m {:.2} {:.2} l S\n",
                row_y,
                margin + available_width,
                row_y
            ));

            let font = if is_total { "/F2 8.0 Tf" } else { "/F1 8.0 Tf" };
            let mut cell_x = margin;
            for (j, cell) in row.iter().enumerate() {
                let w = col_widths.get(j).copied().unwrap_or(40.0);
                let align = alignments.get(j).map(|s| s.as_str()).unwrap_or("left");
                let esc_cell = escape_pdf_str(cell);
                let text_x = if align == "right" {
                    cell_x + w - 4.0 - ((cell.len() as f32) * 4.4).min(w - 8.0)
                } else {
                    cell_x + 4.0
                };
                stream.push_str(&format!(
                    "BT {font} {:.2} {:.2} Td ({esc_cell}) Tj ET\n",
                    text_x,
                    row_y + 5.0
                ));
                cell_x += w;
            }
        }

        // Footer
        stream.push_str(&format!(
            "BT /F1 7.5 Tf 0.4 0.4 0.4 rg {margin:.2} 18.00 Td (Pusthakam Library Management System) Tj ET\n"
        ));
        let page_text = format!("Page {} of {}", p + 1, num_pages);
        stream.push_str(&format!(
            "BT /F1 7.5 Tf 0.4 0.4 0.4 rg {:.2} 18.00 Td ({page_text}) Tj 0 0 0 rg ET\n",
            page_width - margin - 50.0
        ));

        // Content stream object
        offsets.push(buf.len());
        let stream_bytes = stream.as_bytes();
        buf.extend_from_slice(
            format!(
                "{content_obj} 0 obj\n<< /Length {} >>\nstream\n",
                stream_bytes.len()
            )
            .as_bytes(),
        );
        buf.extend_from_slice(stream_bytes);
        buf.extend_from_slice(b"\nendstream\nendobj\n");
    }

    // Xref
    let startxref = buf.len();
    let num_objects = offsets.len() - 1;
    buf.extend_from_slice(format!("xref\n0 {}\n", num_objects + 1).as_bytes());
    buf.extend_from_slice(b"0000000000 65535 f \r\n");
    for &offset in &offsets[1..] {
        buf.extend_from_slice(format!("{offset:010} 00000 n \r\n").as_bytes());
    }

    // Trailer
    buf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            num_objects + 1,
            startxref
        )
        .as_bytes(),
    );

    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_xlsx() {
        let headers = vec!["Date".into(), "Issued".into(), "Returned".into()];
        let rows = vec![
            vec!["2026-11-18".into(), "10".into(), "8".into()],
            vec!["2026-11-19".into(), "15".into(), "12".into()],
            vec!["Total".into(), "25".into(), "20".into()],
        ];
        let bytes = generate_xlsx("Activity", &headers, &rows).unwrap();
        assert!(bytes.len() > 100);
        // ZIP magic bytes PK\x03\x04
        assert_eq!(&bytes[0..4], b"\x50\x4B\x03\x04");
    }

    #[test]
    fn test_generate_pdf() {
        let headers = vec!["Date".into(), "Issued".into(), "Returned".into()];
        let rows = vec![
            vec!["2026-11-18".into(), "10".into(), "8".into()],
            vec!["2026-11-19".into(), "15".into(), "12".into()],
            vec!["Total".into(), "25".into(), "20".into()],
        ];
        let alignments = vec!["left".into(), "right".into(), "right".into()];
        let bytes_polished = generate_pdf(
            "Circulation Activity",
            "Date range: 2026-11-18 to 2026-11-19",
            &headers,
            &rows,
            &alignments,
            false,
        )
        .unwrap();
        assert!(bytes_polished.starts_with(b"%PDF-1.4"));
        assert!(bytes_polished.ends_with(b"%%EOF\n"));
        let pdf_str = String::from_utf8_lossy(&bytes_polished);
        assert!(pdf_str.contains("Circulation Activity"));
        assert!(pdf_str.contains("2026-11-18 to 2026-11-19"));
        assert!(pdf_str.contains("Page 1 of 1"));

        // Also verify simple design mode generates valid PDF without errors
        let bytes_simple = generate_pdf(
            "Circulation Activity",
            "Date range: 2026-11-18 to 2026-11-19",
            &headers,
            &rows,
            &alignments,
            true,
        )
        .unwrap();
        assert!(bytes_simple.starts_with(b"%PDF-1.4"));
        assert!(bytes_simple.ends_with(b"%%EOF\n"));
    }
}
