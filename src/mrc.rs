use crate::metadata::MetadataValue;
use crate::typ::{DecodeResult, DecodedImage, ImageDecodeError, ImageInfo};
use anyhow::{Result, anyhow};
use mrc::{Header, Mode};
use png::ColorType;
use std::collections::HashMap;
use std::fmt::Display;

macro_rules! md_item {
    ($key:expr, $val:expr) => {
        ($key.to_string(), MetadataValue::from(*$val))
    };
}
macro_rules! md_item_string {
    ($key:expr, $val:expr) => {
        ($key.to_string(), MetadataValue::String($val))
    };
}
enum Endianness {
    Little,
    Big,
    Unknown(u32),
}

impl Display for Endianness {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Endianness::Little => write!(f, "Little"),
            Endianness::Big => write!(f, "Big"),
            Endianness::Unknown(code) => write!(f, "Unknown({code:#X})"),
        }
    }
}

impl Endianness {
    fn from_machst(machst: &[u8; 4]) -> Self {
        match u32::from_le_bytes(*machst) {
            0x44440000 => Endianness::Little,
            0x44410000 => Endianness::Little,
            0x11110000 => Endianness::Big,
            other => Endianness::Unknown(other),
        }
    }
}

fn mrc_header_to_metadata(header: &Header) -> HashMap<String, MetadataValue> {
    let Header {
        nx,
        ny,
        nz,
        mode: _mode,
        nxstart,
        nystart,
        nzstart,
        mx,
        my,
        mz,
        xlen,
        ylen,
        zlen,
        alpha,
        beta,
        gamma,
        mapc,
        mapr,
        maps,
        dmin,
        dmax,
        dmean,
        ispg,
        nsymbt: _nsymbt,
        extra: _extra,
        origin,
        map: _map,
        machst,
        rms,
        nlabl,
        label,
    } = header;
    let endianness = Endianness::from_machst(machst);

    let machst_hex = machst
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join("");

    let mut metadata = HashMap::from_iter([
        md_item!("nx", nx),
        md_item!("ny", ny),
        md_item!("nz", nz),
        md_item!("nx_start", nxstart),
        md_item!("ny_start", nystart),
        md_item!("nz_start", nzstart),
        md_item!("sampling_x", mx),
        md_item!("sampling_y", my),
        md_item!("sampling_z", mz),
        md_item!("cell_dim_x", xlen),
        md_item!("cell_dim_y", ylen),
        md_item!("cell_dim_z", zlen),
        md_item!("cell_ang_x", alpha),
        md_item!("cell_ang_y", beta),
        md_item!("cell_ang_z", gamma),
        md_item!("map_c", mapc),
        md_item!("map_r", mapr),
        md_item!("map_s", maps),
        md_item!("density_min", dmin),
        md_item!("density_max", dmax),
        md_item!("density_mean", dmean),
        md_item!("ispg", ispg),
        md_item!("rms_deviation", rms),
        md_item!("exttyp", &header.exttyp()),
        md_item!("nversion", &header.nversion()),
        md_item_string!("endianness", format!("{}", endianness)),
        md_item_string!("machst", machst_hex),
        md_item!("origin_x", &origin[0]),
        md_item!("origin_y", &origin[1]),
        md_item!("origin_z", &origin[2]),
    ]);
    for n in 0..(*nlabl as usize) {
        let slice = &label[n * 80..(n + 1) * 80];
        let label = String::from_utf8_lossy(slice).to_string();
        if !label.trim().is_empty() {
            metadata.insert(format!("label_{n}"), MetadataValue::String(label));
        }
    }
    metadata
}

pub fn decode_mrc(data: &[u8]) -> Result<DecodeResult> {
    // Parse the header from the byte data
    if data.len() < 1024 {
        anyhow::bail!("MRC file too small to contain valid header");
    }

    // This parsing should be safe:
    // * Header is #[repr(C)] and has no padding
    // * We checked data.len() >= 1024 above
    // * Header fields are all plain data (integers and floats, no pointers)
    let header = unsafe { std::ptr::read_unaligned(data.as_ptr() as *const Header) };

    let mut images = Vec::new();
    let mut errors = Vec::new();

    // MRC files can contain multiple 2D slices in a 3D volume
    let nz = header.nz as usize;

    for z in 0..nz {
        match decode_slice(data, &header, z) {
            Ok(image) => images.push(image),
            Err(e) => errors.push(ImageDecodeError {
                image_index: z,
                message: format!("Failed to decode slice {z}: {e}"),
            }),
        }
    }

    Ok(DecodeResult {
        images,
        errors,
        metadata: Some(mrc_header_to_metadata(&header)),
    })
}

fn decode_slice(data: &[u8], header: &Header, slice_index: usize) -> Result<DecodedImage> {
    let width = header.nx as u32;
    let height = header.ny as u32;
    let pixels_per_slice = (width * height) as usize;

    let mode =
        Mode::from_i32(header.mode).ok_or_else(|| anyhow!("Unknown MRC mode: {}", header.mode))?;
    let bytes_per_pixel = mode.byte_size();
    let slice_size = pixels_per_slice * bytes_per_pixel;
    let offset = header.data_offset() + slice_index * slice_size;

    if offset + slice_size > data.len() {
        anyhow::bail!("Slice {} extends beyond file boundaries", slice_index);
    }

    let slice_data = &data[offset..offset + slice_size];

    let (png_color_type, converted_data, range) = match mode {
        Mode::Int8 => {
            let converted_data: Vec<u8> = slice_data
                .iter()
                .map(|&val| (val as i8 as i16 + 128) as u8)
                .collect();
            (
                ColorType::Grayscale,
                converted_data,
                (i8::MIN as f32, i8::MAX as f32),
            )
        }
        Mode::Int16 => {
            let int16_data: &[i16] = bytemuck::cast_slice(slice_data);
            let converted_data: Vec<u8> = int16_data
                .iter()
                .map(|&val| ((val as i32 + 32768) / 256) as u8)
                .collect();
            (
                ColorType::Grayscale,
                converted_data,
                (i16::MIN as f32, i16::MAX as f32),
            )
        }
        Mode::Float32 => {
            // 32-bit float -> convert to 8-bit grayscale
            let float_data: &[f32] = bytemuck::cast_slice(slice_data);
            f32_to_grayscale(float_data)?
        }
        Mode::Int16Complex => {
            // Complex 16-bit -> convert magnitude to 8-bit grayscale
            let complex_data: &[[i16; 2]] = bytemuck::cast_slice(slice_data);

            let magnitudes: Vec<f32> = complex_data
                .iter()
                .map(|&[real, imag]| ((real as f32).powi(2) + (imag as f32).powi(2)).sqrt())
                .collect();

            f32_to_grayscale(&magnitudes)?
        }
        Mode::Uint8 => {
            // 8-bit unsigned integer -> direct copy
            (
                ColorType::Grayscale,
                slice_data.to_vec(),
                (u8::MIN as f32, u8::MAX as f32),
            )
        }
        Mode::Float32Complex => {
            // Complex 32-bit float -> convert magnitude to 8-bit grayscale
            let complex_data: &[[f32; 2]] = bytemuck::cast_slice(slice_data);

            let magnitudes: Vec<f32> = complex_data
                .iter()
                .map(|&[real, imag]| (real.powi(2) + imag.powi(2)).sqrt())
                .collect();

            f32_to_grayscale(&magnitudes)?
        }
        Mode::Float16 => {
            // 16-bit half-precision float -> convert to 8-bit grayscale
            let float16_data: &[u16] = bytemuck::cast_slice(slice_data);

            // Convert half-precision floats to f32
            let float_values: Vec<f32> = float16_data
                .iter()
                .map(|&bits| half::f16::from_bits(bits).to_f32())
                .collect();

            f32_to_grayscale(&float_values)?
        }
        _ => anyhow::bail!("Unsupported MRC mode: {:?}", mode),
    };

    let metadata = HashMap::from([
        md_item!("min_value", &range.0),
        md_item!("max_value", &range.1),
    ]);
    let metadata = ImageInfo {
        image_index: slice_index,
        width,
        height,
        color_type: format!("{mode:?}"),
        bit_depth: 8,
        metadata: Some(metadata),
    };

    Ok(DecodedImage {
        width,
        height,
        color_type: png_color_type,
        data: converted_data,
        info: metadata,
    })
}

fn f32_to_grayscale(values: &[f32]) -> Result<(ColorType, Vec<u8>, (f32, f32))> {
    let (min_val, max_val) = values
        .iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), &val| {
            (min.min(val), max.max(val))
        });
    let range = max_val - min_val;
    let converted_data: Vec<u8> = if range == 0.0 {
        vec![128u8; values.len()]
    } else {
        values
            .iter()
            .map(|&val| ((val - min_val) / range * 255.0) as u8)
            .collect()
    };
    Ok((ColorType::Grayscale, converted_data, (min_val, max_val)))
}
