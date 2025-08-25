use obscura_image::encode_result;
use obscura_image::tiff::decode_tiff;
use obscura_image::typ::Image;
use std::fs;

#[test]
fn test_rgb8() {
    test_tiff(&fs::read("tests/rgb8.tiff").unwrap(), 1, (64, 64));
}

#[test]
fn test_rgb16() {
    test_tiff(&fs::read("tests/rgb16.tiff").unwrap(), 1, (64, 64));
}

#[test]
fn test_gray8() {
    test_tiff(&fs::read("tests/gray8.tiff").unwrap(), 1, (64, 64));
}

#[test]
fn test_multipage() {
    test_tiff(&fs::read("tests/multipage.tiff").unwrap(), 2, (64, 64));
}

#[test]
fn test_bilevel() {
    test_tiff(&fs::read("tests/bilevel.tiff").unwrap(), 1, (128, 128));
}

#[test]
fn test_broken() {
    let tiff_data = &fs::read("tests/broken-at-byte-6155.tiff").unwrap();
    if decode_tiff(tiff_data).is_ok() {
        panic!("Expected decode_tiff to fail for broken TIFF data");
    }
}

#[test]
fn test_cmyk_unsupported() {
    let tiff_data = &fs::read("tests/cmyk-lzw.tiff").unwrap();
    let res = decode_tiff(tiff_data).unwrap();
    assert!(res.images.is_empty(),);
    let msg = &res.errors[0].message;
    assert_eq!(msg, "Unsupported TIFF color type: CMYK(8)");
}

fn test_tiff(tiff_data: &[u8], num_images: usize, dimensions: (u32, u32)) {
    let decode_result = decode_tiff(tiff_data).unwrap();
    let output = encode_result(decode_result).unwrap();

    assert!(output.errors.is_empty(), "{:?}", output.errors);
    assert_eq!(output.total_images, num_images);
    for image in &output.images {
        assert_eq!(image.info.width, dimensions.0);
        assert_eq!(image.info.height, dimensions.1);
        verify_png(image);
    }
}

fn verify_png(image: &Image) {
    let png = &image.png_data;
    assert!(png.len() > 100, "PNG data seems too small");
    assert!(png.len() < 1_000_000, "PNG data seems too large");
    assert!(
        png.starts_with(b"\x89PNG\r\n\x1a\n"),
        "PNG data does not start with valid signature"
    );
}
