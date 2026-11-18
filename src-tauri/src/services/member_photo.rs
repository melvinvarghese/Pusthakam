use crate::error::{AppError, AppResult};
use crate::services::{audit, auth::Session};
use base64::{engine::general_purpose::STANDARD, Engine};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use crate::services::image_upload::decode_png as decode_photo;



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
