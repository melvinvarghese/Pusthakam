use crate::error::{AppError, AppResult};
use base64::{engine::general_purpose::STANDARD, Engine};
use std::io::Cursor;
const MAX_BYTES: usize = 2 * 1024 * 1024;
const MAX_EDGE: u32 = 512;

pub fn decode_png(data: &str) -> AppResult<Vec<u8>> {
    let invalid = || AppError::Validation("Choose a valid image up to 512 pixels per side.".into());
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
