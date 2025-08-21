use crate::typ::DecodedImage;
use anyhow::Result;
use png::{BitDepth, Compression, Encoder};
use std::io::Cursor;

pub fn encode_png(raw_image_data: &DecodedImage) -> Result<Vec<u8>> {
    let mut png_data = Vec::new();
    {
        let cursor = Cursor::new(&mut png_data);
        let mut encoder = Encoder::new(cursor, raw_image_data.width, raw_image_data.height);
        encoder.set_color(raw_image_data.color_type);
        encoder.set_depth(BitDepth::Eight);
        encoder.set_compression(Compression::Fast);

        let mut writer = encoder.write_header()?;
        writer.write_image_data(&raw_image_data.data)?;
    }
    Ok(png_data)
}
