//! Damaged or hostile executables fail with an error, quickly and without panicking.

use std::io::Write;

use super::*;

/// A minimal PE with one section, like `tests.rs` builds, for corrupting.
fn executable_headers() -> Vec<u8> {
    const PE_HEADER: usize = 0x40;
    let mut executable = vec![0u8; 0x200];
    executable[..DOS_SIGNATURE.len()].copy_from_slice(DOS_SIGNATURE);
    executable[PE_HEADER_POINTER_OFFSET..PE_HEADER_POINTER_OFFSET + 4]
        .copy_from_slice(&u32::try_from(PE_HEADER).expect("small").to_le_bytes());
    executable[PE_HEADER..PE_HEADER + PE_SIGNATURE.len()].copy_from_slice(PE_SIGNATURE);
    executable[PE_HEADER + SECTION_COUNT_OFFSET..PE_HEADER + SECTION_COUNT_OFFSET + 2]
        .copy_from_slice(&1u16.to_le_bytes());
    executable
}

#[test]
fn every_truncation_of_the_headers_is_an_error() {
    let executable = executable_headers();
    for length in 0..executable.len() {
        assert!(extract_swf(&executable[..length]).is_err(), "{length} bytes");
        assert!(extract_splash_bitmap(&executable[..length]).is_err(), "{length} bytes");
    }
}

#[test]
fn offsets_past_the_end_are_an_error() {
    let mut executable = executable_headers();
    executable[PE_HEADER_POINTER_OFFSET..PE_HEADER_POINTER_OFFSET + 4].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(matches!(
        extract_swf(&executable),
        Err(ExtractError::NotAPortableExecutable)
    ));
    let mut executable = executable_headers();
    let section_header = 0x40 + COFF_HEADER_END;
    for field in [SECTION_RAW_POINTER_OFFSET, SECTION_RAW_SIZE_OFFSET] {
        executable[section_header + field..section_header + field + 4].copy_from_slice(&u32::MAX.to_le_bytes());
    }
    assert!(matches!(
        extract_swf(&executable),
        Err(ExtractError::NotAPortableExecutable)
    ));
}

#[test]
fn a_swf_claiming_too_much_is_refused_before_allocating() {
    let mut header = COMPRESSED_SWF_SIGNATURE.to_vec();
    header.push(8);
    header.extend_from_slice(&u32::MAX.to_le_bytes());
    header.extend_from_slice(&[0; 16]);
    let first_key = 0x11;
    let payload = apply_keystream(&header, first_key);
    assert!(matches!(
        decrypt_and_inflate(&payload, first_key),
        Err(ExtractError::CorruptSwf(_))
    ));
}

#[test]
fn data_expanding_past_its_declared_size_is_not_inflated_in_full() {
    const EXPANDED_BYTES: usize = 64 * 1024 * 1024;
    const DECLARED_BYTES: usize = 100;
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::best());
    encoder.write_all(&vec![0; EXPANDED_BYTES]).unwrap();
    let compressed = encoder.finish().unwrap();
    let mut output = Vec::new();
    inflate_at_most(&compressed, DECLARED_BYTES, &mut output).unwrap();
    assert_eq!(
        output.len(),
        DECLARED_BYTES + 1,
        "stops one byte past, for the length check"
    );
}
