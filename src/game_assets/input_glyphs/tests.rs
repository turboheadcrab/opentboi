use super::pictures::left_bumper;
use super::svg_path::parse_path_data;
use super::*;
use crate::game::{GamepadKind, StartRoomControl};
use crate::game_assets::vector_art::CharacterLibrary;
use crate::game_assets::vector_mesh::{MeshFill, RuffleMesher, ShapeMesher};

const TEST_DENSITY: f32 = 4.0;
const VISUAL_CHECK_DENSITY: f32 = 12.0;
const OPAQUE: u8 = u8::MAX;
const MOVIE_VERSION: u8 = 8;

pub(super) fn alpha_at(image: &RgbaImage, column: u32, row: u32) -> u8 {
    let index = (row * image.width + column) as usize * RgbaImage::BYTES_PER_PIXEL;
    image.pixels[index + RgbaImage::ALPHA_CHANNEL]
}

pub(super) fn layers_of(texture_id: TextureId) -> Vec<GlyphLayer> {
    glyph_layers(texture_id)
        .expect("a pad's picture is made here")
        .expect("its SVGs are readable")
}

pub(super) fn hint(control: StartRoomControl, kind: GamepadKind) -> TextureId {
    TextureId::StartRoomHint(control, InputDevice::Gamepad(kind))
}

pub(super) fn render(texture_id: TextureId) -> RgbaImage {
    render_layers(&layers_of(texture_id), texture_id.world_size(), TEST_DENSITY).expect("glyph renders")
}

#[test]
fn extra_pairs_repeat_the_command() {
    let path = parse_path_data("M0 0 L10 0 10 10 Q0 10 0 5 0 3 0 0").expect("valid path");
    assert_eq!((path.bounds().width(), path.bounds().height()), (10.0, 10.0));
}

#[test]
fn unknown_commands_and_dangling_numbers_are_rejected() {
    assert!(parse_path_data("M0 0 A5 5 0 0 1 10 10").is_none());
    assert!(parse_path_data("M0 0 L10").is_none());
    assert!(parse_path_data("10 10").is_none());
    assert!(parse_path_data("").is_none());
}

#[test]
fn every_make_has_its_own_picture() {
    for (index, first) in GamepadKind::ALL.into_iter().enumerate() {
        for second in GamepadKind::ALL.into_iter().skip(index + 1) {
            assert_ne!(left_bumper(first), left_bumper(second), "{first:?} and {second:?}");
        }
    }
}

#[test]
fn every_picture_fills_its_texture_up_to_the_margin() {
    for (control, kind) in StartRoomControl::ALL
        .into_iter()
        .flat_map(|control| GamepadKind::ALL.map(|kind| (control, kind)))
    {
        let texture_id = hint(control, kind);
        let image = render(texture_id);
        let size = texture_id.world_size();
        assert_eq!(
            image.width,
            crate::whole_numbers::clamped_u32(size.x * TEST_DENSITY),
            "{control:?} {kind:?}"
        );
        assert_eq!(
            image.height,
            crate::whole_numbers::clamped_u32(size.y * TEST_DENSITY),
            "{control:?} {kind:?}"
        );
        let inked = |column: u32, row: u32| alpha_at(&image, column, row) > 0;
        let columns: Vec<u32> = (0..image.width)
            .filter(|&column| (0..image.height).any(|row| inked(column, row)))
            .collect();
        let rows: Vec<u32> = (0..image.height)
            .filter(|&row| (0..image.width).any(|column| inked(column, row)))
            .collect();
        let margin = crate::whole_numbers::clamped_u32(MARGIN * TEST_DENSITY);
        assert!(
            columns[0] >= margin - 1 && rows[0] >= margin - 1,
            "{kind:?} starts inside the margin"
        );
        assert!(
            *columns.last().unwrap() < image.width - margin + 1 && *rows.last().unwrap() < image.height - margin + 1,
            "{kind:?} ends inside the margin"
        );
        let touches_a_side = columns[0] <= margin || rows[0] <= margin;
        assert!(touches_a_side, "{kind:?} is as big as the box allows");
    }
}

#[test]
fn pictures_are_hollow_rings_in_the_drawings_ink() {
    let image = render(hint(StartRoomControl::Bomb, GamepadKind::Xbox));
    assert_eq!(alpha_at(&image, 0, 0), 0, "the corner is empty");
    let opaque = image
        .pixels
        .chunks(RgbaImage::BYTES_PER_PIXEL)
        .find(|pixel| pixel[RgbaImage::ALPHA_CHANNEL] == OPAQUE);
    assert_eq!(&opaque.expect("some solid pixel")[..3], &START_ROOM_INK);
    let ring_gap = (0..image.width).any(|column| alpha_at(&image, column, image.height / 2) == 0);
    assert!(
        ring_gap,
        "the middle of the button is open between its outline and its name"
    );
}

#[test]
fn every_make_has_its_own_trigger_picture() {
    for (index, first) in GamepadKind::ALL.into_iter().enumerate() {
        for second in GamepadKind::ALL.into_iter().skip(index + 1) {
            assert_ne!(left_trigger(first), left_trigger(second), "{first:?} and {second:?}");
        }
        assert_ne!(left_trigger(first), left_bumper(first));
    }
}

#[test]
fn only_a_pads_button_pictures_are_made_here() {
    for control in StartRoomControl::ALL {
        assert!(glyph_layers(TextureId::StartRoomHint(control, InputDevice::KeyboardAndMouse)).is_none());
    }
    assert!(glyph_layers(TextureId::ActiveItemLabel(InputDevice::KeyboardAndMouse)).is_none());
    assert!(glyph_layers(TextureId::HeldCardLabel(InputDevice::KeyboardAndMouse)).is_none());
    assert!(glyph_layers(TextureId::Hud).is_none());
}

#[test]
fn the_item_label_is_a_light_trigger_in_a_black_outline_on_nothing() {
    for kind in GamepadKind::ALL {
        let texture_id = TextureId::ActiveItemLabel(InputDevice::Gamepad(kind));
        let layers = layers_of(texture_id);
        let colors: Vec<_> = layers.iter().map(|layer| layer.color).collect();
        assert_eq!(colors, [LABEL_OUTLINE_COLOR, LABEL_INK], "{kind:?}");
        let size = texture_id.world_size();
        let outline = layers[0].path.bounds();
        assert!(
            outline.width() <= size.x + 0.01 && outline.height() <= size.y + 0.01,
            "{kind:?}: the outline stays inside the texture"
        );
        let image = render_layers(&layers, size, TEST_DENSITY).expect("label renders");
        assert_eq!(alpha_at(&image, 0, 0), 0, "{kind:?}: the corner shows the HUD through");
    }
}

#[test]
fn the_card_label_is_a_light_bumper_in_a_black_outline_on_nothing() {
    for kind in GamepadKind::ALL {
        let texture_id = TextureId::HeldCardLabel(InputDevice::Gamepad(kind));
        let layers = layers_of(texture_id);
        let colors: Vec<_> = layers.iter().map(|layer| layer.color).collect();
        assert_eq!(colors, [LABEL_OUTLINE_COLOR, LABEL_INK], "{kind:?}");
        let size = texture_id.world_size();
        let outline = layers[0].path.bounds();
        assert!(
            outline.width() <= size.x + 0.01 && outline.height() <= size.y + 0.01,
            "{kind:?}: the outline stays inside the texture"
        );
    }
}

/// The meshed layers cover what the filled layers do: every layer becomes triangles of its own
/// color, inside the layer's bounds.
#[test]
fn layers_mesh_into_triangles_of_their_colors() {
    let empty_movie = swf::Swf {
        header: swf::HeaderExt::default_with_swf_version(MOVIE_VERSION),
        tags: Vec::new(),
    };
    let library = CharacterLibrary::new(&empty_movie);
    for kind in GamepadKind::ALL {
        let texture_id = TextureId::ActiveItemLabel(InputDevice::Gamepad(kind));
        let layers = layers_of(texture_id);
        let mesh = RuffleMesher::new(&library).mesh(&layers_to_shape(&layers));
        assert_eq!(
            mesh.draws.len(),
            1,
            "{kind:?}: one solid draw per fill style, merged by color"
        );
        let mut seen_colors: Vec<[u8; 3]> = Vec::new();
        for draw in &mesh.draws {
            assert!(matches!(draw.fill, MeshFill::Solid));
            assert!(!draw.indices.is_empty(), "{kind:?}: triangles to draw");
            for vertex in &draw.vertices {
                let [red, green, blue, alpha] = vertex.color;
                assert_eq!(alpha, OPAQUE);
                if !seen_colors.contains(&[red, green, blue]) {
                    seen_colors.push([red, green, blue]);
                }
            }
        }
        assert_eq!(
            seen_colors.len(),
            2,
            "{kind:?}: the outline and the glyph keep their colors"
        );
    }
}

/// Writes each make's start room hints and item label to `target/textures/` next to the SWF-derived textures.
#[test]
#[ignore = "writes files for a visual check"]
fn write_every_glyph_for_a_visual_check() {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/textures");
    std::fs::create_dir_all(&directory).expect("cannot create the output directory");
    for kind in GamepadKind::ALL {
        let hints = StartRoomControl::ALL.map(|control| hint(control, kind));
        for texture_id in hints.into_iter().chain([
            TextureId::ActiveItemLabel(InputDevice::Gamepad(kind)),
            TextureId::HeldCardLabel(InputDevice::Gamepad(kind)),
        ]) {
            let image = render_layers(&layers_of(texture_id), texture_id.world_size(), VISUAL_CHECK_DENSITY)
                .expect("glyph renders");
            crate::game_assets::write_png(&directory.join(format!("{texture_id:?}.png")), &image);
        }
    }
}
