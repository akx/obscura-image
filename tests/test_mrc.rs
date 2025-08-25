use obscura_image::mrc::decode_mrc;
use std::fs;

#[test]
fn test_mrc_decode() {
    let mrc_data = fs::read("tests/EMD-3197.mrc").expect("Failed to read test MRC file");
    let result = decode_mrc(&mrc_data).expect("Failed to decode MRC");

    println!(
        "Decoded {} images with {} errors",
        result.images.len(),
        result.errors.len()
    );

    let result_meta = result.metadata.as_ref().unwrap();
    assert_eq!(
        format!("{}", result_meta["label_0"]).trim(),
        "::::EMDATABANK.org::::EMD-3197::::"
    );

    for (key, value) in result_meta {
        println!("File Metadata: {key} = {value}");
    }

    // Should have decoded at least one image
    assert!(
        !result.images.is_empty(),
        "Should decode at least one image"
    );

    // Print some basic info about the first image
    if let Some(image) = result.images.first() {
        let info = &image.info;
        println!(
            "First image: {}x{} pixels, {} color type",
            info.width, info.height, info.color_type
        );
        let img_meta = info.metadata.as_ref().unwrap();
        assert!(img_meta.contains_key("min_value"));
        assert!(img_meta.contains_key("max_value"));
        for (key, value) in img_meta {
            println!("Image Metadata: {key} = {value}");
        }

        assert!(info.width > 0, "Width should be positive");
        assert!(info.height > 0, "Height should be positive");
        assert_eq!(info.bit_depth, 8, "Should be 8-bit output");
    }
}
