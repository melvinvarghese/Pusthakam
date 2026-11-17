use crate::error::{AppError, AppResult};
use crate::services::{audit, auth::Session};
use base64::{engine::general_purpose::STANDARD, Engine};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use std::io::Cursor;

const MAX_BYTES: usize = 2 * 1024 * 1024;
const MAX_EDGE: u32 = 512;

fn decode_photo(data: &str) -> AppResult<Vec<u8>> {
    let invalid = || AppError::Validation("Choose a valid member photo up to 512 pixels per side.".into());
    if data.len() > MAX_BYTES.div_ceil(3) * 4 {
        return Err(invalid());
    }
    let bytes = STANDARD.decode(data).map_err(|_| invalid())?;
    if bytes.len() > MAX_BYTES {
        return Err(invalid());
    }
    let mut decoder = png::Decoder::new(Cursor::new(bytes));
    decoder.set_limits(png::Limits { bytes: 4 * MAX_BYTES });
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().map_err(|_| invalid())?;
    let info = reader.info();
    if info.width > MAX_EDGE || info.height > MAX_EDGE || info.animation_control.is_some() {
        return Err(invalid());
    }
    let size = reader.output_buffer_size().filter(|&size| size <= MAX_BYTES).ok_or_else(invalid)?;
    let mut pixels = vec![0; size];
    let frame = reader.next_frame(&mut pixels).map_err(|_| invalid())?;

    // Store the decoded image without the upload's text or location metadata.
    let mut photo = Vec::new();
    let mut encoder = png::Encoder::new(&mut photo, frame.width, frame.height);
    encoder.set_color(frame.color_type);
    encoder.set_depth(frame.bit_depth);
    let mut writer = encoder.write_header().map_err(|_| invalid())?;
    writer.write_image_data(&pixels[..frame.buffer_size()]).map_err(|_| invalid())?;
    writer.finish().map_err(|_| invalid())?;
    Ok(photo)
}

pub fn get(conn: &Connection, id: i64) -> AppResult<Option<String>> {
    let photo: Option<Vec<u8>> = conn.query_row(
        "SELECT p.png FROM members m LEFT JOIN member_photos p ON p.member_id = m.id WHERE m.id = ?1",
        [id], |r| r.get(0),
    ).optional()?.ok_or_else(|| AppError::NotFound("Member not found.".into()))?;
    Ok(photo.map(|bytes| STANDARD.encode(bytes)))
}

pub fn set(conn: &mut Connection, actor: &Session, id: i64, data: Option<&str>) -> AppResult<()> {
    let photo = data.map(decode_photo).transpose()?;
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let active: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM members WHERE id = ?1 AND archived_at IS NULL)",
        [id], |r| r.get(0),
    )?;
    if !active {
        return Err(AppError::NotFound("Active member not found.".into()));
    }
    let (changed, action, summary) = if let Some(photo) = photo {
        let changed = tx.execute(
            "INSERT INTO member_photos (member_id, png) VALUES (?1, ?2)
             ON CONFLICT(member_id) DO UPDATE SET png = excluded.png",
            params![id, photo],
        )?;
        (changed, "member.photo_set", "Updated member photo")
    } else {
        let changed = tx.execute("DELETE FROM member_photos WHERE member_id = ?1", [id])?;
        (changed, "member.photo_remove", "Removed member photo")
    };
    if changed > 0 {
        audit::record(&tx, Some(actor), action, "member", Some(id), summary)?;
    }
    tx.commit()?;
    Ok(())
}
