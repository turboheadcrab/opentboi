use std::time::Instant;

use super::*;
use crate::game_assets::{EXTRACTED_SWF_FILE_NAME, cache_directory};

const BYTES_PER_MEBIBYTE: f64 = 1024.0 * 1024.0;

/// Startup cost breakdown on this machine. Run with
/// `cargo test --release asset_load_timings -- --ignored --nocapture`.
/// A 1280×960 window's density, the common case on the developer's machine.
const TIMING_PIXELS_PER_WORLD_UNIT: f32 = 1.98;

#[test]
#[ignore = "requires the game SWF extracted into the user cache"]
fn asset_load_timings() {
    let swf_path = cache_directory()
        .expect("no cache directory")
        .join(EXTRACTED_SWF_FILE_NAME);
    let started = Instant::now();
    let swf_bytes = std::fs::read(&swf_path).expect("cannot read the SWF");
    let read_done = Instant::now();
    let swf_buffer = swf::decompress_swf(&swf_bytes[..]).expect("cannot decompress the SWF");
    let movie = swf::parse_swf(&swf_buffer).expect("cannot parse the SWF");
    let parse_done = Instant::now();
    let library = CharacterLibrary::new(&movie);
    let index_done = Instant::now();
    let mut unsupported = UnsupportedFeatureCounts::default();
    let mut texture_count = 0;
    let mut texture_bytes = 0;
    for texture_id in TextureId::all() {
        let image = render_texture(&library, texture_id, TIMING_PIXELS_PER_WORLD_UNIT, &mut unsupported)
            .expect("render failed");
        texture_count += 1;
        texture_bytes += image.width as usize * image.height as usize * RgbaImage::BYTES_PER_PIXEL;
    }
    let render_done = Instant::now();
    let _sounds = sound_bank_from_movie(&movie);
    let sounds_done = Instant::now();
    println!(
        "read {:?}, parse {:?}, index {:?}, render {} textures {:?} ({:.1} MiB of pixels), sounds {:?}",
        read_done - started,
        parse_done - read_done,
        index_done - parse_done,
        texture_count,
        render_done - index_done,
        texture_bytes as f64 / BYTES_PER_MEBIBYTE,
        sounds_done - render_done,
    );
}
