use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::game::TextureId;
use crate::game_assets::vector_art::UnsupportedFeatureCounts;
use crate::game_assets::{
    EXTRACTED_SWF_FILE_NAME, RgbaImage, TextureRenderer, cache_directory, load_assets, write_png,
};

/// Generous density so the PNGs are easy to inspect.
const VISUAL_CHECK_PIXELS_PER_WORLD_UNIT: f32 = 3.0;
const OUTPUT_DIRECTORY: &str = "target/textures";
const PNG_EXTENSION: &str = "png";
/// Sprites and effect frames only need some visible pixels; late splash frames are tiny droplets.
const MINIMUM_VISIBLE_FRACTION: f32 = 0.0;
/// The room art covers its whole canvas, so almost every pixel should be opaque.
const MINIMUM_ROOM_OPAQUE_FRACTION: f32 = 0.9;
const OPAQUE_ALPHA: u8 = u8::MAX;
mod empty_frames;
use empty_frames::is_empty_in_the_original;

/// Needs the SWF extracted from a real Steam install; run with `cargo test -- --ignored`.
/// Writes every texture to `target/textures/` for comparison with FFDec's renders.
#[test]
#[ignore = "requires the game SWF extracted into the user cache"]
fn every_texture_renders_without_unsupported_features() {
    let swf_path = cache_directory()
        .expect("no cache directory")
        .join(EXTRACTED_SWF_FILE_NAME);
    let renderer = load_assets(&swf_path)
        .expect("extracted SWF should parse")
        .texture_renderer;

    let output_directory = Path::new(env!("CARGO_MANIFEST_DIR")).join(OUTPUT_DIRECTORY);
    std::fs::create_dir_all(&output_directory).expect("cannot create the output directory");
    let texture_ids = TextureId::all();
    let next_index = AtomicUsize::new(0);
    let problems = Mutex::new(Vec::new());
    let worker_count = std::thread::available_parallelism().map_or(1, usize::from);
    std::thread::scope(|scope| {
        for _ in 0..worker_count {
            scope.spawn(|| {
                while let Some(&texture_id) = texture_ids.get(next_index.fetch_add(1, Ordering::Relaxed)) {
                    if let Err(problem) = check_texture(renderer, texture_id, &output_directory) {
                        problems.lock().expect("a worker panicked").push(problem);
                    }
                }
            });
        }
    });
    // Workers finish in any order; sorting keeps the report the same on every machine.
    let mut problems = problems.into_inner().expect("a worker panicked");
    problems.sort();
    assert!(
        problems.is_empty(),
        "textures against expectations:\n{}",
        problems.join("\n")
    );
}

/// Renders one texture, checks it and writes its PNG; the error says what is wrong with it.
fn check_texture(renderer: &TextureRenderer, texture_id: TextureId, output_directory: &Path) -> Result<(), String> {
    let (image, unsupported) = renderer
        .render(texture_id, VISUAL_CHECK_PIXELS_PER_WORLD_UNIT)
        .map_err(|render_error| format!("{texture_id:?}: {render_error}"))?;
    if unsupported != UnsupportedFeatureCounts::default() {
        return Err(format!("{texture_id:?}: unsupported art features {unsupported:?}"));
    }
    let image = &image;
    let world_size = texture_id.world_size();
    let aspect_ratio = image.width as f32 / image.height as f32;
    if (aspect_ratio - world_size.x / world_size.y).abs() >= 0.05 {
        return Err(format!(
            "{texture_id:?}: aspect ratio {aspect_ratio} against {world_size}"
        ));
    }
    let (checked_fraction, minimum_fraction) = if matches!(texture_id, TextureId::RoomBackground(_)) {
        (
            fraction_of_pixels(image, |alpha| alpha == OPAQUE_ALPHA),
            MINIMUM_ROOM_OPAQUE_FRACTION,
        )
    } else {
        (fraction_of_pixels(image, |alpha| alpha > 0), MINIMUM_VISIBLE_FRACTION)
    };
    let file_name = format!("{texture_id:?}.{PNG_EXTENSION}");
    write_png(&output_directory.join(file_name), image);
    if (checked_fraction > minimum_fraction) == is_empty_in_the_original(texture_id) {
        return Err(format!("{texture_id:?}: empty or drawn against expectations"));
    }
    Ok(())
}

fn fraction_of_pixels(image: &RgbaImage, alpha_matches: impl Fn(u8) -> bool) -> f32 {
    let (pixels, _) = image.pixels.as_chunks::<{ RgbaImage::BYTES_PER_PIXEL }>();
    let matching_count = pixels
        .iter()
        .filter(|pixel| alpha_matches(pixel[RgbaImage::ALPHA_CHANNEL]))
        .count();
    matching_count as f32 / pixels.len() as f32
}

const MEASURING_CANVAS_SIZE: f32 = 640.0;
const MEASURING_PIXELS_PER_UNIT: f32 = 1.0;

/// Development aid: prints how large each texture's canvas must be (around its origin) to hold all
/// of its art. Run with `cargo test texture_extents -- --ignored --nocapture`.
#[test]
#[ignore = "requires the game SWF extracted into the user cache"]
fn texture_extents() {
    let swf_path = cache_directory()
        .expect("no cache directory")
        .join(EXTRACTED_SWF_FILE_NAME);
    let swf_bytes = std::fs::read(&swf_path).expect("cannot read the SWF");
    let swf_buffer = swf::decompress_swf(&swf_bytes[..]).expect("cannot decompress the SWF");
    let movie = swf::parse_swf(&swf_buffer).expect("cannot parse the SWF");
    let library = crate::game_assets::vector_art::CharacterLibrary::new(&movie);
    for texture_id in TextureId::all().into_iter().chain(TextureId::on_demand_textures()) {
        super::cards::read_names(&library);
        super::hud_values::read_messages(&library);
        let source = super::source_for(texture_id);
        let canvas = crate::game_assets::vector_art::Canvas {
            center_x: source.origin_x,
            center_y: source.origin_y,
            width: MEASURING_CANVAS_SIZE,
            height: MEASURING_CANVAS_SIZE,
            output_pixels_per_unit: MEASURING_PIXELS_PER_UNIT,
            // Extents only need coverage, not smooth edges.
            maximum_supersampling: 1,
        };
        let mut unsupported = crate::game_assets::vector_art::UnsupportedFeatureCounts::default();
        let image = crate::game_assets::vector_art::render_frame(
            &library,
            source.character_id,
            source.selection,
            &canvas,
            source.overrides,
            &mut unsupported,
        )
        .expect("render failed");
        let (needed_half_width, needed_half_height) = needed_half_extents(&image);
        let current = texture_id.world_size();
        let fits = needed_half_width * 2.0 <= current.x && needed_half_height * 2.0 <= current.y;
        if !fits {
            println!(
                "{texture_id:?}: needs {:.0}x{:.0}, has {}x{}",
                needed_half_width * 2.0,
                needed_half_height * 2.0,
                current.x,
                current.y
            );
        }
    }
}

/// How far from the canvas center the image has visible pixels, horizontally and vertically.
fn needed_half_extents(image: &RgbaImage) -> (f32, f32) {
    let half = MEASURING_CANVAS_SIZE / 2.0;
    let (pixels, _) = image.pixels.as_chunks::<{ RgbaImage::BYTES_PER_PIXEL }>();
    let mut needed: (f32, f32) = (0.0, 0.0);
    for (index, pixel) in pixels.iter().enumerate() {
        if pixel[RgbaImage::ALPHA_CHANNEL] > 0 {
            let x = (index % image.width as usize) as f32 + 0.5 - half;
            let y = (index / image.width as usize) as f32 + 0.5 - half;
            needed = (needed.0.max(x.abs()), needed.1.max(y.abs()));
        }
    }
    needed
}
