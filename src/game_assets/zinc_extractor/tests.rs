use std::io::Write;

use super::*;

const SYNTHETIC_HEADERS_SIZE: usize = 0x100;
const SYNTHETIC_IMAGE_END: u32 = 0x200;
const SYNTHETIC_PE_HEADER: usize = 0x40;
const SYNTHETIC_CONTAINER_HEADER_FILLER: usize = 37;
const SYNTHETIC_TRAILER_SIZE: usize = 16;
const REAL_FIRST_KEY: u8 = 0xa1;

/// Minimal PE with one section ending at `SYNTHETIC_IMAGE_END`, followed by a Zinc-like container.
fn synthetic_executable(payload_plain: &[u8], first_key: u8) -> Vec<u8> {
    let mut executable = vec![0u8; SYNTHETIC_IMAGE_END as usize];
    executable[..DOS_SIGNATURE.len()].copy_from_slice(DOS_SIGNATURE);
    executable[PE_HEADER_POINTER_OFFSET..PE_HEADER_POINTER_OFFSET + 4]
        .copy_from_slice(&u32::try_from(SYNTHETIC_PE_HEADER).expect("small").to_le_bytes());
    executable[SYNTHETIC_PE_HEADER..SYNTHETIC_PE_HEADER + PE_SIGNATURE.len()].copy_from_slice(PE_SIGNATURE);
    let section_count_field = SYNTHETIC_PE_HEADER + SECTION_COUNT_OFFSET;
    executable[section_count_field..section_count_field + 2].copy_from_slice(&1u16.to_le_bytes());
    let section_header = SYNTHETIC_PE_HEADER + COFF_HEADER_END;
    let raw_size_field = section_header + SECTION_RAW_SIZE_OFFSET;
    let raw_pointer_field = section_header + SECTION_RAW_POINTER_OFFSET;
    let section_bytes = SYNTHETIC_IMAGE_END - u32::try_from(SYNTHETIC_HEADERS_SIZE).expect("small");
    executable[raw_size_field..raw_size_field + 4].copy_from_slice(&section_bytes.to_le_bytes());
    executable[raw_pointer_field..raw_pointer_field + 4]
        .copy_from_slice(&u32::try_from(SYNTHETIC_HEADERS_SIZE).expect("small").to_le_bytes());

    executable.extend_from_slice(&ZINC_CONTAINER_MAGIC);
    executable.extend_from_slice(&[0xAA; SYNTHETIC_CONTAINER_HEADER_FILLER]);
    executable.extend_from_slice(&u32::try_from(payload_plain.len()).expect("small").to_le_bytes());
    executable.extend_from_slice(&apply_keystream(payload_plain, first_key));
    executable.extend_from_slice(&[0x55; SYNTHETIC_TRAILER_SIZE]);
    executable
}

fn compressed_swf(body: &[u8]) -> Vec<u8> {
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(body).unwrap();
    let mut swf = COMPRESSED_SWF_SIGNATURE.to_vec();
    swf.push(8);
    swf.extend_from_slice(
        &u32::try_from(SWF_HEADER_SIZE + body.len())
            .expect("small")
            .to_le_bytes(),
    );
    swf.extend_from_slice(&encoder.finish().unwrap());
    swf
}

#[test]
fn keystream_decodes_real_container_string() {
    // "Isaac" as stored in the Steam build; Zinc strings start their keystream at 0x92 + length.
    let stored_bytes = [0xde, 0xcb, 0xb8, 0x9b, 0x78];
    let string_first_key = 0x92 + u8::try_from(stored_bytes.len()).expect("a short string");
    assert_eq!(apply_keystream(&stored_bytes, string_first_key), b"Isaac");
}

#[test]
fn extracts_and_inflates_compressed_swf() {
    let body = b"tags and more tags".repeat(50);
    let executable = synthetic_executable(&compressed_swf(&body), REAL_FIRST_KEY);
    let swf_bytes = extract_swf(&executable).unwrap();
    assert!(swf_bytes.starts_with(UNCOMPRESSED_SWF_SIGNATURE));
    assert_eq!(&swf_bytes[SWF_HEADER_SIZE..], body.as_slice());
}

#[test]
fn rejects_executable_without_container() {
    let mut executable = synthetic_executable(&compressed_swf(b"body"), REAL_FIRST_KEY);
    executable[SYNTHETIC_IMAGE_END as usize] = 0;
    assert!(matches!(extract_swf(&executable), Err(ExtractError::NoZincContainer)));
}

#[test]
fn rejects_non_executable() {
    assert!(matches!(
        extract_swf(b"not an executable at all"),
        Err(ExtractError::NotAPortableExecutable)
    ));
}

const SPLASH_FIRST_KEY: u8 = 0xe4;
const ORIGINAL_SPLASH_WIDTH: u32 = 800;
const ORIGINAL_SPLASH_HEIGHT: u32 = 455;
const BITMAP_WIDTH_OFFSET: usize = 18;
const BITMAP_HEIGHT_OFFSET: usize = 22;

/// An executable whose container also holds a splash entry: sizes, then encrypted zlib data.
fn executable_with_splash(bitmap: &[u8]) -> Vec<u8> {
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(bitmap).unwrap();
    let compressed = encoder.finish().unwrap();
    let mut executable = synthetic_executable(&compressed_swf(b"body"), REAL_FIRST_KEY);
    executable.extend_from_slice(&u32::try_from(compressed.len()).expect("small").to_le_bytes());
    executable.extend_from_slice(&u32::try_from(bitmap.len()).expect("small").to_le_bytes());
    executable.extend_from_slice(&apply_keystream(&compressed, SPLASH_FIRST_KEY));
    executable
}

#[test]
fn extracts_the_compressed_splash_bitmap() {
    let mut bitmap = b"BM".to_vec();
    bitmap.extend(std::iter::repeat_n(0x42, 4000));
    let executable = executable_with_splash(&bitmap);
    assert_eq!(extract_splash_bitmap(&executable).unwrap(), bitmap);
}

#[test]
fn reports_a_missing_splash() {
    let executable = synthetic_executable(&compressed_swf(b"body"), REAL_FIRST_KEY);
    assert!(matches!(
        extract_splash_bitmap(&executable),
        Err(ExtractError::SplashNotFound)
    ));
}

/// Needs the game installed through Steam; run with `cargo test -- --ignored`.
#[test]
#[ignore = "requires the game installed through Steam"]
fn real_executable_holds_the_original_splash_and_swf() {
    let game_directory = crate::game_assets::locate_game_directory().expect("the game should be installed");
    let executable = std::fs::read(game_directory.join(GAME_EXECUTABLE_NAME)).expect("Isaac.exe should be readable");
    let bitmap = extract_splash_bitmap(&executable).expect("the splash should be found");
    assert_eq!(read_u32(&bitmap, BITMAP_WIDTH_OFFSET), Some(ORIGINAL_SPLASH_WIDTH));
    assert_eq!(read_u32(&bitmap, BITMAP_HEIGHT_OFFSET), Some(ORIGINAL_SPLASH_HEIGHT));
    let swf_bytes = extract_swf(&executable).expect("the SWF should be found");
    assert!(swf_bytes.starts_with(UNCOMPRESSED_SWF_SIGNATURE));
    assert_eq!(
        read_u32(&swf_bytes, SWF_LENGTH_OFFSET),
        Some(u32::try_from(swf_bytes.len()).expect("small"))
    );
}
