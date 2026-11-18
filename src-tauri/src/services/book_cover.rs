use crate::error::{AppError, AppResult};
use crate::services::{audit, auth::Session};
use base64::{engine::general_purpose::STANDARD, Engine};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use crate::services::image_upload::decode_png as decode_photo;



pub fn get(conn: &Connection, id: i64) -> AppResult<Option<String>> {
    let photo: Option<Vec<u8>> = conn.query_row(
        "SELECT p.png FROM bibliographic_records m LEFT JOIN book_covers p ON p.record_id = m.id WHERE m.id = ?1",
        [id], |r| r.get(0),
    ).optional()?.ok_or_else(|| AppError::NotFound("Catalogue record not found.".into()))?;
    Ok(photo.map(|bytes| STANDARD.encode(bytes)))
}

pub fn set(conn: &mut Connection, actor: &Session, id: i64, data: Option<&str>) -> AppResult<()> {
    let photo = data.map(decode_photo).transpose()?;
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let active: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM bibliographic_records WHERE id = ?1 AND archived_at IS NULL)",
        [id], |r| r.get(0),
    )?;
    if !active {
        return Err(AppError::NotFound("Active catalogue record not found.".into()));
    }
    let (changed, action, summary) = if let Some(photo) = photo {
        let changed = tx.execute(
            "INSERT INTO book_covers (record_id, png) VALUES (?1, ?2)
             ON CONFLICT(record_id) DO UPDATE SET png = excluded.png",
            params![id, photo],
        )?;
        (changed, "catalogue.cover_set", "Updated book cover")
    } else {
        let changed = tx.execute("DELETE FROM book_covers WHERE record_id = ?1", [id])?;
        (changed, "catalogue.cover_remove", "Removed book cover")
    };
    if changed > 0 {
        audit::record(&tx, Some(actor), action, "bibliographic_records", Some(id), summary)?;
    }
    tx.commit()?;
    Ok(())
}
