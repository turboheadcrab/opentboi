//! Recovers the game SWF from the Steam build's `Isaac.exe`.
//!
//! The executable is an MDM Zinc 3 projector. Zinc appends a container after the PE image whose
//! payloads are XOR-encrypted with a keystream that advances by 0x21 per byte. The SWF is located
//! by known plaintext (its "CWS"/"FWS" signature) and validated by inflating to its declared size.

#[cfg(test)]
mod damaged_tests;
#[cfg(test)]
mod tests;

mod splash;

use std::io::Read;

pub use splash::extract_splash_bitmap;

pub const GAME_EXECUTABLE_NAME: &str = "Isaac.exe";
const ZINC_CONTAINER_MAGIC: [u8; 8] = [0x14, 0x09, 0x13, 0x49, 0x49, 0x13, 0x09, 0x14];
const KEYSTREAM_STEP: u8 = 0x21;
/// The SWF entry follows the project header closely; this bounds the search.
const ENTRY_SEARCH_LIMIT: usize = 64 * 1024;

const DOS_SIGNATURE: &[u8] = b"MZ";
const PE_SIGNATURE: &[u8] = b"PE\0\0";
const PE_HEADER_POINTER_OFFSET: usize = 0x3c;
const SECTION_COUNT_OFFSET: usize = 6;
const OPTIONAL_HEADER_SIZE_OFFSET: usize = 20;
const COFF_HEADER_END: usize = 24;
const SECTION_HEADER_SIZE: usize = 40;
const SECTION_RAW_SIZE_OFFSET: usize = 16;
const SECTION_RAW_POINTER_OFFSET: usize = 20;

const COMPRESSED_SWF_SIGNATURE: &[u8; 3] = b"CWS";
const UNCOMPRESSED_SWF_SIGNATURE: &[u8; 3] = b"FWS";
const SWF_HEADER_SIZE: usize = 8;
const SWF_LENGTH_OFFSET: usize = 4;
const LENGTH_PREFIX_SIZE: usize = 4;
/// Far above the real game's 46 MB (uncompressed); a header claiming more is damaged.
const MAXIMUM_SWF_BYTES: usize = 256 * 1024 * 1024;

#[derive(Debug)]
pub enum ExtractError {
    FileAccess(std::io::Error),
    NotAPortableExecutable,
    NoZincContainer,
    SwfNotFound,
    SplashNotFound,
    CorruptSwf(String),
}

impl From<std::io::Error> for ExtractError {
    fn from(io_error: std::io::Error) -> Self {
        Self::FileAccess(io_error)
    }
}

impl std::fmt::Display for ExtractError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileAccess(io_error) => write!(formatter, "I/O error: {io_error}"),
            Self::NotAPortableExecutable => write!(formatter, "{GAME_EXECUTABLE_NAME} is not a PE executable"),
            Self::NoZincContainer => write!(formatter, "{GAME_EXECUTABLE_NAME} has no Zinc container appended"),
            Self::SwfNotFound => write!(formatter, "no encrypted SWF found in the Zinc container"),
            Self::SplashNotFound => write!(formatter, "no splash bitmap found in the Zinc container"),
            Self::CorruptSwf(reason) => write!(formatter, "embedded SWF is corrupt: {reason}"),
        }
    }
}

/// Returns the game SWF in uncompressed (`FWS`) form.
pub fn extract_swf(executable_bytes: &[u8]) -> Result<Vec<u8>, ExtractError> {
    let container = &executable_bytes[overlay_offset(executable_bytes)?..];
    if !container.starts_with(&ZINC_CONTAINER_MAGIC) {
        return Err(ExtractError::NoZincContainer);
    }
    let search_end = ENTRY_SEARCH_LIMIT.min(container.len().saturating_sub(SWF_HEADER_SIZE));
    for payload_start in LENGTH_PREFIX_SIZE..search_end {
        let Some(payload_length) = read_u32(container, payload_start - LENGTH_PREFIX_SIZE) else {
            continue;
        };
        let payload_length = payload_length as usize;
        let Some(payload) = container.get(payload_start..payload_start.saturating_add(payload_length)) else {
            continue;
        };
        let Some(first_key) = find_swf_keystream_start(payload) else {
            continue;
        };
        if let Ok(swf_bytes) = decrypt_and_inflate(payload, first_key) {
            return Ok(swf_bytes);
        }
    }
    Err(ExtractError::SwfNotFound)
}

/// End of the last section's raw data, which is where appended data begins. Every offset the
/// headers give is checked against the file, so a truncated or damaged one is "not a PE".
fn overlay_offset(executable_bytes: &[u8]) -> Result<usize, ExtractError> {
    let not_pe = || ExtractError::NotAPortableExecutable;
    if !executable_bytes.starts_with(DOS_SIGNATURE) {
        return Err(not_pe());
    }
    let pe_header = read_u32(executable_bytes, PE_HEADER_POINTER_OFFSET).ok_or_else(not_pe)? as usize;
    let signature_end = pe_header.checked_add(PE_SIGNATURE.len()).ok_or_else(not_pe)?;
    if executable_bytes.get(pe_header..signature_end) != Some(PE_SIGNATURE) {
        return Err(not_pe());
    }
    let header_field = |offset: usize| read_u16(executable_bytes, pe_header.checked_add(offset)?);
    let section_count = usize::from(header_field(SECTION_COUNT_OFFSET).ok_or_else(not_pe)?);
    let optional_header_size = usize::from(header_field(OPTIONAL_HEADER_SIZE_OFFSET).ok_or_else(not_pe)?);
    let section_table = pe_header
        .checked_add(COFF_HEADER_END + optional_header_size)
        .ok_or_else(not_pe)?;
    let section_end = |section_index: usize| -> Option<usize> {
        let section_header = section_table.checked_add(section_index * SECTION_HEADER_SIZE)?;
        let raw_pointer = read_u32(
            executable_bytes,
            section_header.checked_add(SECTION_RAW_POINTER_OFFSET)?,
        )?;
        let raw_size = read_u32(executable_bytes, section_header.checked_add(SECTION_RAW_SIZE_OFFSET)?)?;
        (raw_pointer as usize).checked_add(raw_size as usize)
    };
    let image_end = (0..section_count).filter_map(section_end).max().ok_or_else(not_pe)?;
    if image_end > executable_bytes.len() {
        return Err(not_pe());
    }
    Ok(image_end)
}

/// Returns the first keystream byte if `payload` decrypts to a CWS or FWS signature.
fn find_swf_keystream_start(payload: &[u8]) -> Option<u8> {
    if payload.len() < SWF_HEADER_SIZE {
        return None;
    }
    [COMPRESSED_SWF_SIGNATURE, UNCOMPRESSED_SWF_SIGNATURE]
        .into_iter()
        .map(|signature| payload[0] ^ signature[0])
        .find(|&first_key| {
            let signature = apply_keystream(&payload[..COMPRESSED_SWF_SIGNATURE.len()], first_key);
            signature == COMPRESSED_SWF_SIGNATURE || signature == UNCOMPRESSED_SWF_SIGNATURE
        })
}

fn apply_keystream(bytes: &[u8], first_key: u8) -> Vec<u8> {
    bytes
        .iter()
        .zip((0..=u8::MAX).cycle())
        .map(|(&byte, index)| byte ^ first_key.wrapping_add(KEYSTREAM_STEP.wrapping_mul(index)))
        .collect()
}

fn decrypt_and_inflate(payload: &[u8], first_key: u8) -> Result<Vec<u8>, ExtractError> {
    let plain = apply_keystream(payload, first_key);
    let header = plain
        .get(..SWF_HEADER_SIZE)
        .ok_or_else(|| ExtractError::CorruptSwf("shorter than its header".to_string()))?;
    let declared_length = read_u32(header, SWF_LENGTH_OFFSET).unwrap_or_default() as usize;
    if !(SWF_HEADER_SIZE..=MAXIMUM_SWF_BYTES).contains(&declared_length) {
        return Err(ExtractError::CorruptSwf(format!(
            "header claims {declared_length} bytes"
        )));
    }
    let mut swf_bytes = Vec::with_capacity(declared_length);
    swf_bytes.extend_from_slice(UNCOMPRESSED_SWF_SIGNATURE);
    swf_bytes.extend_from_slice(&header[UNCOMPRESSED_SWF_SIGNATURE.len()..]);
    let body = &plain[SWF_HEADER_SIZE..];
    if plain.starts_with(COMPRESSED_SWF_SIGNATURE) {
        inflate_at_most(body, declared_length - SWF_HEADER_SIZE, &mut swf_bytes)
            .map_err(|inflate_error| ExtractError::CorruptSwf(inflate_error.to_string()))?;
    } else {
        swf_bytes.extend_from_slice(body);
    }
    if swf_bytes.len() != declared_length {
        return Err(ExtractError::CorruptSwf(format!(
            "{} bytes, header says {declared_length}",
            swf_bytes.len()
        )));
    }
    Ok(swf_bytes)
}

/// Inflates zlib `compressed` onto `output`, stopping one byte past `expected_length`: data that
/// expands further is not decoded in full, and the caller's length check rejects it.
fn inflate_at_most(compressed: &[u8], expected_length: usize, output: &mut Vec<u8>) -> std::io::Result<usize> {
    flate2::read::ZlibDecoder::new(compressed)
        .take(expected_length as u64 + 1)
        .read_to_end(output)
}

/// `None` past the end of `bytes`.
fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    let field = bytes.get(offset..offset.checked_add(2)?)?;
    Some(u16::from_le_bytes([field[0], field[1]]))
}

/// `None` past the end of `bytes`.
fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let field = bytes.get(offset..offset.checked_add(4)?)?;
    Some(u32::from_le_bytes([field[0], field[1], field[2], field[3]]))
}
