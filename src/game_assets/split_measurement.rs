//! Development aid: why frames cannot be vector pictures, and how much video memory a hybrid
//! (shapes without gradients as meshes, only the gradient shapes as a cropped texture) would
//! still need.

use std::collections::BTreeMap;

use super::asset_loader::TextureRenderer;
use super::texture_catalog::place_texture;
use super::vector_art::{Character, PlacedShape, ShapeSource, TWIPS_PER_PIXEL};
use super::vector_mesh::{MeshFill, RuffleMesher, ShapeMesh, ShapeMesher};
use super::{EXTRACTED_SWF_FILE_NAME, cache_directory, load_assets};
use crate::game::TextureId;

const PIXELS_PER_UNIT: f32 = 1.6;
const BYTES_PER_PIXEL: f32 = 4.0;
const MEBIBYTE: f32 = 1024.0 * 1024.0;

#[derive(Default)]
struct FamilyTotals {
    frames: u32,
    drawable: u32,
    unplaceable: u32,
    with_gradient: u32,
    gradient_frame_mebibytes: f32,
    gradient_crop_mebibytes: f32,
    mixed_shapes: u32,
    all_gradient_shapes: u32,
    /// How often each kind of gradient (shape, spread, stop count) occurs in those frames.
    kinds: BTreeMap<String, u32>,
}

/// The gradient shapes of one frame: how many are wholly gradient, how many mix fills, and the
/// box around them in Flash pixels.
struct GradientShapes {
    mixed: u32,
    all_gradient: u32,
    kinds: Vec<String>,
    low: [f32; 2],
    high: [f32; 2],
}

fn is_gradient(fill: &MeshFill) -> bool {
    matches!(fill, MeshFill::Gradient { .. })
}

fn mesh_of(renderer: &TextureRenderer, part: &PlacedShape) -> Option<ShapeMesh> {
    let shape = match &part.source {
        ShapeSource::Character(character_id) => match renderer.library.character(*character_id) {
            Some(Character::Shape(shape)) => *shape,
            _ => return None,
        },
        ShapeSource::Glyph(_, shape) => shape,
    };
    Some(RuffleMesher::new(&renderer.library).mesh(shape))
}

fn gradient_shapes(renderer: &TextureRenderer, placed: &[PlacedShape]) -> GradientShapes {
    let mut found = GradientShapes {
        mixed: 0,
        all_gradient: 0,
        kinds: Vec::new(),
        low: [f32::MAX; 2],
        high: [f32::MIN; 2],
    };
    for part in placed {
        let Some(mesh) = mesh_of(renderer, part) else {
            continue;
        };
        if !mesh.draws.iter().any(|draw| is_gradient(&draw.fill)) {
            continue;
        }
        for draw in &mesh.draws {
            if let MeshFill::Gradient { gradient, .. } = &draw.fill {
                found.kinds.push(format!(
                    "{:?}, {:?}, {} stops, focal {}, linear {} {:?}",
                    gradient.shape,
                    gradient.spread,
                    gradient.stops.len(),
                    gradient.focal_point != 0.0,
                    gradient.linear_rgb,
                    gradient.stops
                ));
            }
        }
        if mesh.draws.iter().all(|draw| is_gradient(&draw.fill)) {
            found.all_gradient += 1;
        } else {
            found.mixed += 1;
        }
        let transform = part.transform.pre_scale(TWIPS_PER_PIXEL, TWIPS_PER_PIXEL);
        for vertex in mesh.draws.iter().flat_map(|draw| &draw.vertices) {
            let [x, y] = vertex.position;
            let point = [
                transform.sx * x + transform.kx * y + transform.tx,
                transform.ky * x + transform.sy * y + transform.ty,
            ];
            found.low = [found.low[0].min(point[0]), found.low[1].min(point[1])];
            found.high = [found.high[0].max(point[0]), found.high[1].max(point[1])];
        }
    }
    found
}

fn add_frame(totals: &mut FamilyTotals, renderer: &TextureRenderer, texture_id: TextureId) {
    totals.frames += 1;
    let Some(placed) = place_texture(&renderer.library, texture_id) else {
        totals.unplaceable += 1;
        return;
    };
    let gradients = gradient_shapes(renderer, &placed);
    if gradients.mixed + gradients.all_gradient == 0 {
        totals.drawable += 1;
        return;
    }
    let size = texture_id.world_size() * PIXELS_PER_UNIT;
    let frame_mebibytes = size.x * size.y * BYTES_PER_PIXEL / MEBIBYTE;
    let crop_pixels = (gradients.high[0] - gradients.low[0]).max(0.0) * (gradients.high[1] - gradients.low[1]).max(0.0);
    let crop_mebibytes = crop_pixels * PIXELS_PER_UNIT * PIXELS_PER_UNIT * BYTES_PER_PIXEL / MEBIBYTE;
    totals.with_gradient += 1;
    totals.gradient_frame_mebibytes += frame_mebibytes;
    totals.gradient_crop_mebibytes += crop_mebibytes.min(frame_mebibytes);
    totals.mixed_shapes += gradients.mixed;
    totals.all_gradient_shapes += gradients.all_gradient;
    for kind in gradients.kinds {
        *totals.kinds.entry(kind).or_default() += 1;
    }
}

/// Run with `cargo test split_measurement -- --ignored --nocapture`.
#[test]
#[ignore = "requires the game SWF extracted into the user cache"]
fn split_measurement() {
    let swf_path = cache_directory()
        .expect("no cache directory")
        .join(EXTRACTED_SWF_FILE_NAME);
    let renderer = load_assets(&swf_path)
        .expect("extracted SWF should parse")
        .texture_renderer;
    let mut families: BTreeMap<String, FamilyTotals> = BTreeMap::new();
    for texture_id in TextureId::all().into_iter().chain(TextureId::on_demand_textures()) {
        let debug = format!("{texture_id:?}");
        let family = debug.split(['(', ' ', '{']).next().unwrap_or_default().to_string();
        add_frame(families.entry(family).or_default(), renderer, texture_id);
    }
    println!(
        "family, frames, drawable, unplaceable (masks etc.), with gradient, their MiB, MiB if cropped, shapes mixed/all-gradient"
    );
    for (family, totals) in families
        .iter()
        .filter(|(_, totals)| totals.with_gradient + totals.unplaceable > 0)
    {
        println!(
            "{family}: {} frames, {} drawable, {} unplaceable, {} with gradient, {:.1} MiB, {:.1} MiB cropped, shapes {}/{}",
            totals.frames,
            totals.drawable,
            totals.unplaceable,
            totals.with_gradient,
            totals.gradient_frame_mebibytes,
            totals.gradient_crop_mebibytes,
            totals.mixed_shapes,
            totals.all_gradient_shapes
        );
        for (kind, count) in &totals.kinds {
            println!("    {count:>5} x {kind}");
        }
    }
}
