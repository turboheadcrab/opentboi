//! The start room's controls drawn on the floor. The floor clip `bg` (6314, placed at (56.1, 47.95)
//! in the room clip) shows frame 50 in the first chapter's start room, which adds shape 6313: one
//! shape holding all the hints. The keys under each figure (W A S D, the arrows, E, the space bar)
//! are cut out of the picture and replaced by a small texture per kind of input device, so the
//! hints can follow the device the player uses: the keys for the keyboard (cut from the same
//! shape), the make's own buttons for a gamepad (Kenney's pictures, see
//! `input_glyphs`).

use swf::CharacterId;

use super::{ROOM_CENTER, TextureSource};
use crate::game::{InputDevice, StartRoomControl, TextureId};
use crate::game_assets::vector_art::{FrameSelection, NO_OVERRIDES, RgbaImage};

const CONTROLS_SHAPE: CharacterId = 6313;
const FLOOR_CLIP_OFFSET: (f32, f32) = (56.1, 47.95);

/// The shape's own point at the center of the room texture.
fn room_center_in_shape() -> (f32, f32) {
    (ROOM_CENTER.0 - FLOOR_CLIP_OFFSET.0, ROOM_CENTER.1 - FLOOR_CLIP_OFFSET.1)
}

pub(super) fn source_for(texture_id: TextureId) -> TextureSource {
    let (center_x, center_y) = room_center_in_shape();
    match texture_id {
        // The key itself, cut out of the same shape; the world's y axis points up, the shape's down.
        TextureId::StartRoomHint(control, InputDevice::KeyboardAndMouse) => TextureSource {
            character_id: CONTROLS_SHAPE,
            selection: FrameSelection::start_of(1),
            origin_x: center_x + control.offset().x,
            origin_y: center_y - control.offset().y,
            overrides: &NO_OVERRIDES,
        },
        // The whole drawing; a gamepad's hint has no clip of its own (see `input_glyphs`).
        _ => TextureSource {
            character_id: CONTROLS_SHAPE,
            selection: FrameSelection::start_of(1),
            origin_x: center_x,
            origin_y: center_y,
            overrides: &NO_OVERRIDES,
        },
    }
}

/// Clears the areas of the keys from the rendered controls drawing, which covers `ROOM_SIZE`.
pub(super) fn blank_hint_areas(image: &mut RgbaImage, pixels_per_world_unit: f32) {
    for control in StartRoomControl::ALL {
        blank_area(image, control, pixels_per_world_unit);
    }
}

fn blank_area(image: &mut RgbaImage, control: StartRoomControl, pixels_per_world_unit: f32) {
    let (offset, size) = (control.offset(), control.size());
    let room_size = TextureId::StartRoomControls.world_size();
    let left = (room_size.x / 2.0 + offset.x - size.x / 2.0) * pixels_per_world_unit;
    let top = (room_size.y / 2.0 - offset.y - size.y / 2.0) * pixels_per_world_unit;
    let whole = crate::whole_numbers::clamped_u32;
    let first_column = whole(left.floor());
    let first_row = whole(top.floor());
    let last_column = whole((left + size.x * pixels_per_world_unit).ceil()).min(image.width);
    let last_row = whole((top + size.y * pixels_per_world_unit).ceil()).min(image.height);
    for row in first_row..last_row {
        let start = (row * image.width + first_column) as usize * RgbaImage::BYTES_PER_PIXEL;
        let end = (row * image.width + last_column) as usize * RgbaImage::BYTES_PER_PIXEL;
        image.pixels[start..end].fill(0);
    }
}
