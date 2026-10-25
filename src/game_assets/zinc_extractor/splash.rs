//! The projector's splash image: Zinc shows `isaacload.bmp` (800×455) while the game starts. The
//! container stores it as an entry of compressed size, uncompressed size, then the zlib-compressed
//! file under the container keystream (restarting where the stream starts).

use super::{ExtractError, ZINC_CONTAINER_MAGIC, apply_keystream, inflate_at_most, overlay_offset, read_u32};

const ENTRY_SIZES_LENGTH: usize = 8;
const UNCOMPRESSED_SIZE_OFFSET: usize = 4;
/// zlib streams start with method byte 0x78 and a header whose big-endian value divides by 31.
const ZLIB_METHOD_BYTE: u8 = 0x78;
const ZLIB_HEADER_LENGTH: usize = 2;
const ZLIB_HEADER_CHECK_DIVISOR: u16 = 31;
const BITMAP_SIGNATURE: &[u8; 2] = b"BM";
/// Far above the real 1.4 MB; rejects entries whose sizes are random bytes.
const MAXIMUM_SPLASH_BYTES: usize = 64 * 1024 * 1024;

/// Returns the splash as a Windows BMP file.
pub fn extract_splash_bitmap(executable_bytes: &[u8]) -> Result<Vec<u8>, ExtractError> {
    let container = &executable_bytes[overlay_offset(executable_bytes)?..];
    if !container.starts_with(&ZINC_CONTAINER_MAGIC) {
        return Err(ExtractError::NoZincContainer);
    }
    (ENTRY_SIZES_LENGTH..container.len().saturating_sub(ZLIB_HEADER_LENGTH))
        .find_map(|stream_start| bitmap_entry_at(container, stream_start))
        .ok_or(ExtractError::SplashNotFound)
}

fn bitmap_entry_at(container: &[u8], stream_start: usize) -> Option<Vec<u8>> {
    let sizes_start = stream_start - ENTRY_SIZES_LENGTH;
    let compressed_length = read_u32(container, sizes_start)? as usize;
    let uncompressed_length = read_u32(container, sizes_start + UNCOMPRESSED_SIZE_OFFSET)? as usize;
    let plausible = ZLIB_HEADER_LENGTH < compressed_length
        && compressed_length < uncompressed_length
        && uncompressed_length <= MAXIMUM_SPLASH_BYTES;
    if !plausible {
        return None;
    }
    let stream = container.get(stream_start..stream_start.checked_add(compressed_length)?)?;
    let first_key = stream[0] ^ ZLIB_METHOD_BYTE;
    let header = apply_keystream(&stream[..ZLIB_HEADER_LENGTH], first_key);
    if !u16::from_be_bytes([header[0], header[1]]).is_multiple_of(ZLIB_HEADER_CHECK_DIVISOR) {
        return None;
    }
    let plain = apply_keystream(stream, first_key);
    let mut bitmap = Vec::with_capacity(uncompressed_length);
    inflate_at_most(&plain, uncompressed_length, &mut bitmap).ok()?;
    (bitmap.len() == uncompressed_length && bitmap.starts_with(BITMAP_SIGNATURE)).then_some(bitmap)
}
