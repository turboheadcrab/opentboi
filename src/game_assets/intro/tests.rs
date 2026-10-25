use std::io::Cursor;
use std::path::Path;
use std::time::{Duration, Instant};

use rodio::Source;

use super::{INTRO_FRAME_COUNT, intro_music, render_intro_frame};
use crate::game_assets::vector_art::{CharacterLibrary, UnsupportedFeatureCounts};
use crate::game_assets::{EXTRACTED_SWF_FILE_NAME, cache_directory, write_png};

const OUTPUT_DIRECTORY: &str = "target/intro";
const CHECKED_FRAME_STEP: usize = 12;
/// The intro is 373 frames at 30 fps; its music runs about as long.
const EXPECTED_MUSIC_SECONDS: std::ops::Range<f32> = 10.0..16.0;
/// Frames must render within one frame at the 30 fps they play at.
const FRAME_BUDGET: Duration = Duration::from_millis(33);
const RENDER_WIDTH: u32 = 800;

/// Needs the SWF extracted from a real Steam install; run with `cargo test -- --ignored`.
/// Writes every 12th frame to `target/intro/` for comparison with FFDec's renders.
#[test]
#[ignore = "requires the game SWF extracted into the user cache"]
fn intro_frames_render_and_music_decodes() {
    let swf_path = cache_directory()
        .expect("no cache directory")
        .join(EXTRACTED_SWF_FILE_NAME);
    let swf_bytes = std::fs::read(swf_path).expect("extracted SWF should exist");
    let swf_buffer = swf::decompress_swf(&swf_bytes[..]).expect("SWF should decompress");
    let movie = swf::parse_swf(&swf_buffer).expect("SWF should parse");
    let library = CharacterLibrary::new(&movie);
    let output_directory = Path::new(env!("CARGO_MANIFEST_DIR")).join(OUTPUT_DIRECTORY);
    std::fs::create_dir_all(&output_directory).expect("cannot create the output directory");

    let mut slowest = Duration::ZERO;
    for frame_index in (0..INTRO_FRAME_COUNT).step_by(CHECKED_FRAME_STEP) {
        let mut unsupported = UnsupportedFeatureCounts::default();
        let started = Instant::now();
        let image = render_intro_frame(&library, frame_index, RENDER_WIDTH, &mut unsupported)
            .unwrap_or_else(|render_error| panic!("frame {frame_index}: {render_error}"));
        slowest = slowest.max(started.elapsed());
        assert_eq!(unsupported, UnsupportedFeatureCounts::default(), "frame {frame_index}");
        write_png(&output_directory.join(format!("frame_{frame_index:03}.png")), &image);
    }
    println!(
        "slowest intro frame at {RENDER_WIDTH} px: {:.1} ms",
        slowest.as_secs_f64() * 1000.0
    );
    assert!(slowest < FRAME_BUDGET, "slowest frame took {slowest:?}");

    let music = intro_music(&movie).expect("the intro should have streamed music");
    let decoder = rodio::Decoder::new(Cursor::new(music.to_vec())).expect("intro music should decode");
    let sample_rate = decoder.sample_rate().get() as f32;
    let channels = f32::from(decoder.channels().get());
    let seconds = decoder.count() as f32 / sample_rate / channels;
    assert!(EXPECTED_MUSIC_SECONDS.contains(&seconds), "music lasts {seconds} s");
}
