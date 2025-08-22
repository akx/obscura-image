mod png;
pub mod tiff;
pub mod typ;
mod utils;

use anyhow::Result;
use png::encode_png;
use typ::{DecodeResult, Image, Output};
use wasm_bindgen::prelude::*;

// TODO: it would be nicer to generate these automatically, with e.g.
//       `tsify`, but I couldn't get it to work with `serde_bytes`
//       and `wasm-bindgen` so we'd still keep emitting an Uint8Array...
#[wasm_bindgen(typescript_custom_section)]
const TS_CUSTOM_TYPES: &'static str = r#"
export interface ImageMetadata {
  image_index: number;
  width: number;
  height: number;
  color_type: string;
  bit_depth: number;
}

export interface Image {
  png_data: Uint8Array;
  metadata: ImageMetadata;
}

export interface ImageDecodeError {
  image_index: number;
  message: string;
}

export interface Output {
  images: Image[];
  errors: ImageDecodeError[];
  total_images: number;
}
"#;

#[wasm_bindgen(js_name = "decodeTiff", unchecked_return_type = "Output")]
pub fn js_decode_tiff(
    #[wasm_bindgen(js_name = "tiffData")] tiff_data: &[u8],
) -> std::result::Result<JsValue, JsValue> {
    utils::set_panic_hook();

    tiff::decode_tiff(tiff_data)
        .and_then(encode_result)
        .and_then(|result| {
            serde_wasm_bindgen::to_value(&result)
                .map_err(|e| anyhow::anyhow!("Failed to serialize result: {e}"))
        })
        .map_err(|e| JsValue::from_str(&format!("{e}")))
}

pub fn encode_result(res: DecodeResult) -> Result<Output> {
    let mut successful_results = Vec::new();

    for decoded in res.images {
        match encode_png(&decoded) {
            Ok(png_data) => {
                successful_results.push(Image {
                    png_data,
                    metadata: decoded.metadata,
                });
            }
            Err(e) => {
                // If PNG encoding fails, we could add it to errors, but for now we'll
                // let the error bubble up since this is less likely than TIFF decode errors
                return Err(e);
            }
        }
    }

    let total_images = successful_results.len() + res.errors.len();

    Ok(Output {
        images: successful_results,
        errors: res.errors,
        total_images,
    })
}
