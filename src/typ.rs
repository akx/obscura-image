use crate::metadata::MetadataMap;
use serde::Serialize;
#[derive(Serialize)]
pub struct ImageInfo {
    pub image_index: usize,
    pub width: u32,
    pub height: u32,
    pub color_type: String,
    pub bit_depth: u8,
    pub metadata: Option<MetadataMap>,
}

#[derive(Serialize)]
pub struct Image {
    #[serde(with = "serde_bytes")]
    pub png_data: Vec<u8>,
    pub info: ImageInfo,
}

#[derive(Serialize, Debug)]
pub struct ImageDecodeError {
    pub image_index: usize,
    pub message: String,
}

#[derive(Serialize)]
pub struct Output {
    pub images: Vec<Image>,
    pub errors: Vec<ImageDecodeError>,
    pub total_images: usize,
    pub metadata: Option<MetadataMap>,
}

pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    pub color_type: png::ColorType,
    pub data: Vec<u8>,
    pub info: ImageInfo,
}

pub struct DecodeResult {
    pub images: Vec<DecodedImage>,
    pub errors: Vec<ImageDecodeError>,
    pub metadata: Option<MetadataMap>,
}
