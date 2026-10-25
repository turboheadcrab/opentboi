//! Decodes `DefineBitsLossless`/`DefineBitsLossless2` bitmaps into premultiplied `tiny-skia` pixmaps.

use std::io::Read;

use swf::{BitmapFormat, DefineBitsLossless};
use tiny_skia::{IntSize, Pixmap};

const BYTES_PER_OUTPUT_PIXEL: usize = 4;
/// Rows of palette and 15-bit bitmaps are padded to a multiple of four bytes.
const ROW_ALIGNMENT: usize = 4;
const RGB_PALETTE_ENTRY_SIZE: usize = 3;
const RGBA_PALETTE_ENTRY_SIZE: usize = 4;
const RGB15_BYTES_PER_PIXEL: usize = 2;
const RGB32_BYTES_PER_PIXEL: usize = 4;
const FIVE_BIT_MAXIMUM: u16 = 0x1f;
const OPAQUE: u8 = u8::MAX;
/// `DefineBitsLossless2` carries alpha; version 1 is always opaque.
const VERSION_WITH_ALPHA: u8 = 2;
/// Flash Player's own limit on a bitmap (16,777,215 pixels); a damaged header claiming more is
/// refused before anything is allocated.
const MAXIMUM_PIXELS: usize = 16_777_215;

/// `None` for a damaged or oversized bitmap.
pub fn decode_bitmap(bitmap: &DefineBitsLossless) -> Option<Pixmap> {
    let width = usize::from(bitmap.width);
    let height = usize::from(bitmap.height);
    if width.checked_mul(height)? > MAXIMUM_PIXELS {
        return None;
    }
    let has_alpha = bitmap.version >= VERSION_WITH_ALPHA;
    // Only what the format needs is inflated, however far the data would expand.
    let needed = inflated_size(bitmap.format, width, height, has_alpha);
    let mut inflated = Vec::with_capacity(needed);
    flate2::read::ZlibDecoder::new(&bitmap.data[..])
        .take(needed as u64)
        .read_to_end(&mut inflated)
        .ok()?;
    let premultiplied_rgba = match bitmap.format {
        BitmapFormat::ColorMap8 { num_colors } => {
            decode_palette(&inflated, width, height, usize::from(num_colors) + 1, has_alpha)?
        }
        BitmapFormat::Rgb15 => decode_rgb15(&inflated, width, height)?,
        BitmapFormat::Rgb32 => decode_rgb32(&inflated, width * height, has_alpha)?,
    };
    Pixmap::from_vec(
        premultiplied_rgba,
        IntSize::from_wh(bitmap.width.into(), bitmap.height.into())?,
    )
}

/// Bytes of inflated data a bitmap of this format and size holds.
fn inflated_size(format: BitmapFormat, width: usize, height: usize, has_alpha: bool) -> usize {
    match format {
        BitmapFormat::ColorMap8 { num_colors } => {
            let entry_size = if has_alpha {
                RGBA_PALETTE_ENTRY_SIZE
            } else {
                RGB_PALETTE_ENTRY_SIZE
            };
            (usize::from(num_colors) + 1) * entry_size + padded_row_size(width) * height
        }
        BitmapFormat::Rgb15 => padded_row_size(width * RGB15_BYTES_PER_PIXEL) * height,
        BitmapFormat::Rgb32 => width * height * RGB32_BYTES_PER_PIXEL,
    }
}

fn padded_row_size(unpadded: usize) -> usize {
    unpadded.div_ceil(ROW_ALIGNMENT) * ROW_ALIGNMENT
}

fn premultiply(red: u8, green: u8, blue: u8, alpha: u8) -> [u8; BYTES_PER_OUTPUT_PIXEL] {
    let scale = |channel: u8| u8::try_from((u16::from(channel) * u16::from(alpha) + 127) / 255).unwrap_or(u8::MAX);
    [scale(red), scale(green), scale(blue), alpha]
}

fn decode_palette(data: &[u8], width: usize, height: usize, color_count: usize, has_alpha: bool) -> Option<Vec<u8>> {
    let entry_size = if has_alpha {
        RGBA_PALETTE_ENTRY_SIZE
    } else {
        RGB_PALETTE_ENTRY_SIZE
    };
    let (palette_bytes, indices) = data.split_at_checked(color_count * entry_size)?;
    let palette: Vec<[u8; BYTES_PER_OUTPUT_PIXEL]> = palette_bytes
        .chunks_exact(entry_size)
        .map(|entry| premultiply(entry[0], entry[1], entry[2], if has_alpha { entry[3] } else { OPAQUE }))
        .collect();
    let row_size = padded_row_size(width);
    let mut output = Vec::with_capacity(width * height * BYTES_PER_OUTPUT_PIXEL);
    for row in 0..height {
        let row_indices = indices.get(row * row_size..row * row_size + width)?;
        for &index in row_indices {
            output.extend_from_slice(palette.get(usize::from(index))?);
        }
    }
    Some(output)
}

fn decode_rgb15(data: &[u8], width: usize, height: usize) -> Option<Vec<u8>> {
    let row_size = padded_row_size(width * RGB15_BYTES_PER_PIXEL);
    let expand =
        |five_bits: u16| u8::try_from((five_bits * 255 + FIVE_BIT_MAXIMUM / 2) / FIVE_BIT_MAXIMUM).unwrap_or(u8::MAX);
    let mut output = Vec::with_capacity(width * height * BYTES_PER_OUTPUT_PIXEL);
    for row in 0..height {
        let row_bytes = data.get(row * row_size..row * row_size + width * RGB15_BYTES_PER_PIXEL)?;
        for pixel in row_bytes.as_chunks::<RGB15_BYTES_PER_PIXEL>().0 {
            let packed = u16::from_be_bytes([pixel[0], pixel[1]]);
            let red = expand((packed >> 10) & FIVE_BIT_MAXIMUM);
            let green = expand((packed >> 5) & FIVE_BIT_MAXIMUM);
            let blue = expand(packed & FIVE_BIT_MAXIMUM);
            output.extend_from_slice(&[red, green, blue, OPAQUE]);
        }
    }
    Some(output)
}

/// Pixels are stored as ARGB; version 2 alpha is already premultiplied.
fn decode_rgb32(data: &[u8], pixel_count: usize, has_alpha: bool) -> Option<Vec<u8>> {
    let pixels = data.get(..pixel_count * RGB32_BYTES_PER_PIXEL)?;
    let mut output = Vec::with_capacity(pixel_count * BYTES_PER_OUTPUT_PIXEL);
    for pixel in pixels.as_chunks::<RGB32_BYTES_PER_PIXEL>().0 {
        let alpha = if has_alpha { pixel[0] } else { OPAQUE };
        // Clamp so color never exceeds alpha, which premultiplied storage requires.
        output.extend_from_slice(&[pixel[1].min(alpha), pixel[2].min(alpha), pixel[3].min(alpha), alpha]);
    }
    Some(output)
}

#[cfg(test)]
mod tests;
