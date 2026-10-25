use super::*;
use crate::game_assets::vector_art::CharacterLibrary;
use crate::game_assets::{EXTRACTED_SWF_FILE_NAME, cache_directory};

/// The browser demo, if the user put it there (never committed).
const DEMO_SWF: &str = "downloads/581168_Isaac_demo_Secure.swf";

/// Meshes every shape in `swf_bytes`; returns (shapes, triangles).
fn mesh_every_shape(swf_bytes: &[u8]) -> (usize, usize) {
    let swf_buffer = swf::decompress_swf(swf_bytes).expect("SWF should decompress");
    let movie = swf::parse_swf(&swf_buffer).expect("SWF should parse");
    let library = CharacterLibrary::new(&movie);
    let mut mesher = RuffleMesher::new(&library);
    let mut shapes = 0;
    let mut triangles = 0;
    let mut smoothed_bitmap_fills = 0;
    for tag in &movie.tags {
        if let swf::Tag::DefineShape(shape) = tag {
            let mesh = mesher.mesh(shape);
            for draw in &mesh.draws {
                assert_eq!(draw.indices.len() % 3, 0, "shape {}", shape.id);
                match &draw.fill {
                    MeshFill::Solid => {}
                    MeshFill::Gradient { matrix, gradient } => {
                        assert!(!gradient.stops.is_empty() && matrix[2][2] != 0.0, "shape {}", shape.id);
                    }
                    MeshFill::Bitmap {
                        matrix,
                        bitmap_id,
                        smoothed,
                        repeating,
                    } => {
                        assert!(*bitmap_id > 0 && matrix[2][2] != 0.0, "shape {}", shape.id);
                        smoothed_bitmap_fills += usize::from(*smoothed && !*repeating);
                    }
                }
                assert!(draw.indices.iter().all(|&index| (index as usize) < draw.vertices.len()));
            }
            shapes += 1;
            triangles += mesh.draws.iter().map(|draw| draw.indices.len() / 3).sum::<usize>();
        }
    }
    println!("{smoothed_bitmap_fills} smoothed, clamped bitmap fills");
    (shapes, triangles)
}

#[test]
#[ignore = "requires the game SWF extracted into the user cache"]
fn every_shape_of_the_game_becomes_triangles() {
    let path = cache_directory()
        .expect("no cache directory")
        .join(EXTRACTED_SWF_FILE_NAME);
    let (shapes, triangles) = mesh_every_shape(&std::fs::read(path).expect("extracted SWF should exist"));
    println!("game: {shapes} shapes, {triangles} triangles");
    assert!(shapes > 1_000);
}

#[test]
#[ignore = "requires the browser demo SWF in downloads/"]
fn every_shape_of_the_demo_becomes_triangles() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(DEMO_SWF);
    let (shapes, triangles) = mesh_every_shape(&std::fs::read(path).expect("demo SWF should exist"));
    println!("demo: {shapes} shapes, {triangles} triangles");
    assert!(shapes > 1_000);
}
