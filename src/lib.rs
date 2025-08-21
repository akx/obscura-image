mod png;
mod tiff;
mod typ;
mod utils;

use anyhow::Result;
use png::encode_png;
use typ::{DecodeResult, Image, Output};
use wasm_bindgen::prelude::*;
#[wasm_bindgen(js_name = "decodeTiff")]
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

fn encode_result(res: DecodeResult) -> Result<Output> {
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
