//! The card or pill held, in the HUD: its own picture (`hud.extra`, clip 782 frame 11 with `p` on the
//! card's frame) and its name on a black strip (`hud.pilll`, the name from `tart()`, or a pill's
//! effect once known).

use glam::Vec2;

use crate::game::draw::{DrawItem, Rgba};
use crate::game::input_state::InputDevice;
use crate::game::room::stage_to_world;
use crate::game::texture_id::TextureId;

/// `hud.extra` at (612.55, 94.05), mirrored and scaled to 82.3 %; `hud.pilll` at (587.8, 69.6).
const PICTURE_POSITION: Vec2 = Vec2::new(612.55, 94.05);
const PICTURE_SCALE: f32 = 0.822_753_9;
const NAME_POSITION: Vec2 = Vec2::new(587.8, 69.6);
/// `extra` (depth 266) is drawn over `pilll` (258), whose round end frames the picture.
const NAME_DEPTH: f32 = 801.0;
/// The original bakes a "Q" badge into the name strip (a shape of its own, light lettering in a
/// black ring); it is drawn separately so it can name the device's own button. Its center (8.275, 3.75)
/// is measured from the strip's origin; over the strip and over the picture's ring.
const LABEL_OFFSET: Vec2 = Vec2::new(8.275, 3.75);
const LABEL_DEPTH: f32 = 802.5;
/// The badge as the original draws it, and the size of a pad's button picture in its place.
pub const KEYBOARD_LABEL_SIZE: Vec2 = Vec2::new(18.85, 16.8);
pub const GAMEPAD_LABEL_SIZE: Vec2 = Vec2::new(24.0, 16.8);
/// Margin around the original's shape in its texture, so its edges are not cut.
const KEYBOARD_LABEL_MARGIN: f32 = 0.2;
const PICTURE_DEPTH: f32 = 802.0;
const NO_FALLBACK: Rgba = Rgba {
    red: 0.0,
    green: 0.0,
    blue: 0.0,
    alpha: 0.0,
};

/// `pilll` frame 2 when a card is taken (10 frames), frame 3 when it is used (8).
pub const SLOT_TAKEN_FRAME_COUNT: u8 = 10;
pub const SLOT_USED_FRAME_COUNT: u8 = 8;

/// The slot's animation, if one plays: whether a card was used (else taken), and its frame.
pub type SlotAnimation = Option<(bool, u8)>;

/// The size of the label's texture: the original's shape with a margin, or for a pad a picture made at
/// the size it is drawn at.
pub fn label_texture_size(input_device: InputDevice) -> Vec2 {
    match input_device {
        InputDevice::KeyboardAndMouse => KEYBOARD_LABEL_SIZE + Vec2::splat(2.0 * KEYBOARD_LABEL_MARGIN),
        InputDevice::Gamepad(_) => GAMEPAD_LABEL_SIZE,
    }
}

fn label_draw_item(input_device: InputDevice) -> DrawItem {
    let center = stage_to_world(NAME_POSITION + LABEL_OFFSET);
    let mut item = DrawItem::textured(
        TextureId::HeldCardLabel(input_device),
        center,
        1.0,
        NO_FALLBACK,
        LABEL_DEPTH,
    );
    item.size = match input_device {
        InputDevice::KeyboardAndMouse => KEYBOARD_LABEL_SIZE,
        InputDevice::Gamepad(_) => GAMEPAD_LABEL_SIZE,
    };
    item
}

/// The textures of what is held: its picture and its name.
pub type HeldTextures = Option<(TextureId, TextureId)>;

pub fn append_draw_items(
    held: HeldTextures,
    slot_animation: SlotAnimation,
    input_device: InputDevice,
    draw_items: &mut Vec<DrawItem>,
) {
    let name_position = stage_to_world(NAME_POSITION);
    if let Some((used, frame)) = slot_animation {
        let texture = TextureId::CardSlotAnimation { used, frame };
        draw_items.push(DrawItem::textured(texture, name_position, 1.0, NO_FALLBACK, NAME_DEPTH));
    }
    let Some((picture, name)) = held else { return };
    draw_items.push(DrawItem {
        flip_horizontally: true,
        ..DrawItem::textured(
            picture,
            stage_to_world(PICTURE_POSITION),
            PICTURE_SCALE,
            NO_FALLBACK,
            PICTURE_DEPTH,
        )
    });
    if slot_animation.is_none() {
        draw_items.push(DrawItem::textured(name, name_position, 1.0, NO_FALLBACK, NAME_DEPTH));
        draw_items.push(label_draw_item(input_device));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::input_state::GamepadKind;
    use crate::game::pickups::card::Card;

    fn card_textures(card: Card) -> (TextureId, TextureId) {
        (TextureId::HeldCard(card), TextureId::HeldCardName(card))
    }

    fn labels_for(held: HeldTextures, slot_animation: SlotAnimation, input_device: InputDevice) -> Vec<TextureId> {
        let mut draw_items = Vec::new();
        append_draw_items(held, slot_animation, input_device, &mut draw_items);
        draw_items
            .iter()
            .filter_map(|item| item.texture)
            .filter(|texture| matches!(texture, TextureId::HeldCardLabel(_)))
            .collect()
    }

    #[test]
    fn a_held_card_shows_the_button_of_the_device_in_use() {
        for device in std::iter::once(InputDevice::KeyboardAndMouse).chain(GamepadKind::ALL.map(InputDevice::Gamepad)) {
            assert_eq!(
                labels_for(Some(card_textures(Card::ALL[0])), None, device),
                [TextureId::HeldCardLabel(device)]
            );
        }
    }

    #[test]
    fn no_card_or_a_playing_slot_animation_shows_no_label() {
        let device = InputDevice::KeyboardAndMouse;
        assert_eq!(labels_for(None, None, device), []);
        assert_eq!(
            labels_for(Some(card_textures(Card::ALL[0])), Some((false, 0)), device),
            []
        );
    }
}
