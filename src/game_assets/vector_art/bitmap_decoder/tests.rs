use std::borrow::Cow;
use std::io::Write;

use super::*;

fn compressed(data: &[u8]) -> Vec<u8> {
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::best());
    encoder.write_all(data).unwrap();
    encoder.finish().unwrap()
}

fn bitmap(width: u16, height: u16, data: Vec<u8>) -> DefineBitsLossless<'static> {
    DefineBitsLossless {
        version: VERSION_WITH_ALPHA,
        id: 1,
        format: BitmapFormat::Rgb32,
        width,
        height,
        data: Cow::Owned(data),
    }
}

#[test]
fn a_small_bitmap_decodes() {
    let pixels = [255, 10, 20, 30].repeat(4);
    let pixmap = decode_bitmap(&bitmap(2, 2, compressed(&pixels))).expect("decodes");
    assert_eq!((pixmap.width(), pixmap.height()), (2, 2));
    assert_eq!(&pixmap.data()[..4], &[10, 20, 30, 255]);
}

#[test]
fn oversized_or_short_bitmaps_are_refused() {
    assert!(
        decode_bitmap(&bitmap(u16::MAX, u16::MAX, compressed(&[0; 16]))).is_none(),
        "too large"
    );
    assert!(
        decode_bitmap(&bitmap(4, 4, compressed(&[0; 8]))).is_none(),
        "data runs short"
    );
    assert!(decode_bitmap(&bitmap(2, 2, vec![1, 2, 3])).is_none(), "not zlib");
}

#[test]
fn data_expanding_far_past_the_image_is_cut_off() {
    // 2 × 2 pixels need 16 bytes; 64 MiB of data is only read that far.
    let pixmap = decode_bitmap(&bitmap(2, 2, compressed(&vec![255; 64 * 1024 * 1024])));
    assert!(pixmap.is_some());
}
