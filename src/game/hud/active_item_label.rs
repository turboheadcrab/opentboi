//! The label above the active item's box. The original's HUD has "SPACE" baked into the box (a shape
//! of its own: light lettering in a thick black outline); here the HUD texture lacks it, and the
//! label is drawn over the spot: the original's shape for the keyboard, the item button of the pad
//! in use (its left trigger, outlined the same way) for a gamepad.

use glam::Vec2;

use crate::game::draw::{DrawItem, Rgba};
use crate::game::input_state::InputDevice;
use crate::game::room::stage_to_world;
use crate::game::texture_id::TextureId;

/// Where the original's label sits, in stage coordinates: the center of its shape (-0.05, -32.8)
/// inside the item box's clip, which the HUD places at (409.5, 39.9) scaled by (0.9656, 0.8647).
const LABEL_CENTER: Vec2 = Vec2::new(409.45, 11.54);
/// The label as it appears on the stage.
const LABEL_SIZE: Vec2 = Vec2::new(39.1, 14.53);
/// A pad's button picture is a badge, not a word: it gets a little more height than "SPACE" over
/// the same center, so it reads as large.
const GAMEPAD_LABEL_SIZE: Vec2 = Vec2::new(39.1, 18.0);
/// The original's label shape on its own, before the HUD clip scales it, plus a margin of 0.2 on
/// every side so that its edges are not cut.
const KEYBOARD_LABEL_TEXTURE_SIZE: Vec2 = Vec2::new(40.5, 16.8);
/// Over the HUD's counters and hearts, and over the item box it belongs to.
const LABEL_DEPTH: f32 = 802.0;
const NO_FALLBACK: Rgba = Rgba {
    red: 0.0,
    green: 0.0,
    blue: 0.0,
    alpha: 0.0,
};

/// The size of the label's texture: the original's shape as it is, or for a pad a picture made at
/// the size it is drawn at.
pub fn texture_size(input_device: InputDevice) -> Vec2 {
    match input_device {
        InputDevice::KeyboardAndMouse => KEYBOARD_LABEL_TEXTURE_SIZE,
        InputDevice::Gamepad(_) => GAMEPAD_LABEL_SIZE,
    }
}

pub fn draw_item(input_device: InputDevice) -> DrawItem {
    let mut item = DrawItem::textured(
        TextureId::ActiveItemLabel(input_device),
        stage_to_world(LABEL_CENTER),
        1.0,
        NO_FALLBACK,
        LABEL_DEPTH,
    );
    item.size = match input_device {
        InputDevice::KeyboardAndMouse => LABEL_SIZE,
        InputDevice::Gamepad(_) => GAMEPAD_LABEL_SIZE,
    };
    item
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::input_state::GamepadKind;

    #[test]
    fn every_device_draws_the_label_over_the_same_spot() {
        for device in std::iter::once(InputDevice::KeyboardAndMouse).chain(GamepadKind::ALL.map(InputDevice::Gamepad)) {
            let item = draw_item(device);
            assert_eq!(item.center, stage_to_world(LABEL_CENTER));
            assert!(
                (item.size.x - LABEL_SIZE.x).abs() < f32::EPSILON,
                "as wide as the original's word"
            );
            assert!(item.size.y >= LABEL_SIZE.y);
            assert_eq!(item.texture, Some(TextureId::ActiveItemLabel(device)));
        }
    }

    #[test]
    fn the_keyboard_shape_is_squeezed_like_the_original_hud_squeezes_it() {
        let squeezed = KEYBOARD_LABEL_TEXTURE_SIZE * Vec2::new(0.9656, 0.8647);
        assert!((squeezed - LABEL_SIZE).abs().max_element() < 0.1);
    }
}
