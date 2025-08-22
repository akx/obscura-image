use crate::typ::{DecodeResult, DecodedImage, ImageDecodeError, ImageMetadata};
use anyhow::Result;
use std::io::Cursor;
use tiff::{
    ColorType,
    decoder::{Decoder, DecodingResult},
};

#[inline]
fn convert_16_to_8(value: u16) -> u8 {
    (value.saturating_add(128) >> 8) as u8
}

pub fn decode_tiff(tiff_data: &[u8]) -> Result<DecodeResult> {
    let cursor = Cursor::new(tiff_data);
    let mut decoder = Decoder::new(cursor)?;

    let mut images = Vec::new();
    let mut errors = Vec::new();
    let mut image_index = 0;

    loop {
        match decode_single_image(&mut decoder, image_index) {
            Ok(decoded) => {
                images.push(decoded);
            }
            Err(e) => {
                errors.push(ImageDecodeError {
                    image_index,
                    message: format!("{e}"),
                });
            }
        }

        image_index += 1;

        if !decoder.more_images() {
            break;
        }
        match decoder.next_image() {
            Ok(()) => continue,
            Err(e) => {
                // Error moving to next image - report it
                errors.push(ImageDecodeError {
                    image_index,
                    message: format!("Failed to move to next image: {e}"),
                });
                break;
            }
        }
    }

    Ok(DecodeResult { images, errors })
}

fn decode_single_image(
    decoder: &mut Decoder<Cursor<&[u8]>>,
    image_index: usize,
) -> Result<DecodedImage> {
    let (width, height) = decoder.dimensions()?;
    let colortype = decoder.colortype()?;
    let image_data = decoder.read_image()?;

    let (rgb_data, png_color_type) = match (image_data, colortype) {
        (DecodingResult::U8(data), ColorType::Gray(1)) => (
            data.iter().map(|&b| if b != 0 { 255 } else { 0 }).collect(),
            png::ColorType::Grayscale,
        ),
        (DecodingResult::U8(data), ColorType::Gray(8)) => {
            // Convert grayscale to RGB
            let mut rgb_data = Vec::with_capacity(data.len() * 3);
            for gray in data {
                rgb_data.extend_from_slice(&[gray, gray, gray]);
            }
            (rgb_data, png::ColorType::Rgb)
        }
        (DecodingResult::U8(data), ColorType::RGB(8)) => (data, png::ColorType::Rgb),
        (DecodingResult::U8(data), ColorType::RGBA(8)) => (data, png::ColorType::Rgba),
        (DecodingResult::U16(data), ColorType::Gray(16)) => {
            // Convert 16-bit grayscale to 8-bit RGB
            let mut rgb_data = Vec::with_capacity(data.len() * 3);
            for gray in data {
                let gray_8 = convert_16_to_8(gray);
                rgb_data.extend_from_slice(&[gray_8, gray_8, gray_8]);
            }
            (rgb_data, png::ColorType::Rgb)
        }
        (DecodingResult::U16(data), ColorType::RGB(16)) => {
            // Convert 16-bit RGB to 8-bit RGB
            (
                data.iter().map(|&c| convert_16_to_8(c)).collect(),
                png::ColorType::Rgb,
            )
        }
        (DecodingResult::U16(data), ColorType::RGBA(16)) => {
            // Convert 16-bit RGBA to 8-bit RGBA
            (
                data.iter().map(|&c| convert_16_to_8(c)).collect(),
                png::ColorType::Rgba,
            )
        }
        _ => {
            anyhow::bail!("Unsupported TIFF color type: {:?}", colortype);
        }
    };

    let (bit_depth, color_type_str) = match colortype {
        ColorType::Gray(depth) => (depth, "Grayscale".to_string()),
        ColorType::RGB(depth) => (depth, "RGB".to_string()),
        ColorType::RGBA(depth) => (depth, "RGBA".to_string()),
        ColorType::CMYK(depth) => (depth, "CMYK".to_string()),
        ColorType::YCbCr(depth) => (depth, "YCbCr".to_string()),
        ColorType::Palette(depth) => (depth, "Palette".to_string()),
        ColorType::GrayA(depth) => (depth, "GrayscaleAlpha".to_string()),
        ColorType::CMYKA(depth) => (depth, "CMYKA".to_string()),
        ColorType::Multiband {
            bit_depth,
            num_samples,
        } => (bit_depth, format!("Multiband{num_samples}")),
        _ => (0, "Unknown".to_string()),
    };

    let metadata = ImageMetadata {
        image_index,
        width,
        height,
        color_type: color_type_str,
        bit_depth,
    };

    Ok(DecodedImage {
        width,
        height,
        color_type: png_color_type,
        data: rgb_data,
        metadata,
    })
}
