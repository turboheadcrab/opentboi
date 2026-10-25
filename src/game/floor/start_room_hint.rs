//! The start room's controls drawn on the floor: the picture without its keys, and over each
//! spot the control of the device the player used last (keys, or the pad's buttons).

use glam::Vec2;

use crate::game::draw::{DrawItem, Rgba};
use crate::game::input_state::InputDevice;
use crate::game::texture_id::TextureId;

/// The drawing is part of the floor: above the room art, below doors and decals.
const DRAWING_DEPTH: f32 = 1.2;
/// The hint sits on the drawing.
const HINT_DEPTH: f32 = 1.21;
/// Without artwork the drawing is simply left out.
const FALLBACK_COLOR: Rgba = Rgba {
    red: 0.0,
    green: 0.0,
    blue: 0.0,
    alpha: 0.0,
};
/// A control the start room's drawing names, under its little figure.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StartRoomControl {
    /// "Move": the W, A, S, D keys; a pad's D-pad, one picture per direction (Rebirth's hint).
    Move,
    /// "Attack": the arrow keys; a pad's four face buttons, top, left, bottom, right (Rebirth's).
    Attack,
    /// "Bomb": the E key; a pad's left bumper.
    Bomb,
    /// "Item": the space bar; a pad's left trigger.
    Item,
}

impl StartRoomControl {
    pub const ALL: [Self; 4] = [Self::Move, Self::Attack, Self::Bomb, Self::Item];

    /// Where the drawing's keys for it are, from the room center in world units (x right, y up);
    /// measured on the drawing, with a unit of margin around the ink.
    pub fn offset(self) -> Vec2 {
        match self {
            Self::Move => Vec2::new(-188.3, 17.3),
            Self::Attack => Vec2::new(-60.5, 17.0),
            Self::Bomb => Vec2::new(67.75, 16.5),
            Self::Item => Vec2::new(194.5, 16.5),
        }
    }

    /// The area around those keys that is cut out of the drawing and covered by the hint.
    pub fn size(self) -> Vec2 {
        match self {
            Self::Move | Self::Attack => Vec2::new(90.0, 22.0),
            Self::Bomb => Vec2::new(28.0, 22.0),
            Self::Item => Vec2::new(66.0, 22.0),
        }
    }
}

pub fn append_draw_items(input_device: InputDevice, draw_items: &mut Vec<DrawItem>) {
    draw_items.push(DrawItem::textured(
        TextureId::StartRoomControls,
        Vec2::ZERO,
        1.0,
        FALLBACK_COLOR,
        DRAWING_DEPTH,
    ));
    draw_items.extend(StartRoomControl::ALL.map(|control| {
        DrawItem::textured(
            TextureId::StartRoomHint(control, input_device),
            control.offset(),
            1.0,
            FALLBACK_COLOR,
            HINT_DEPTH,
        )
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::input_state::GamepadKind;

    fn hints_for(input_device: InputDevice) -> Vec<TextureId> {
        let mut draw_items = Vec::new();
        append_draw_items(input_device, &mut draw_items);
        draw_items
            .iter()
            .filter_map(|item| item.texture)
            .filter(|texture| matches!(texture, TextureId::StartRoomHint(..)))
            .collect()
    }

    #[test]
    fn every_control_names_the_device() {
        let devices = std::iter::once(InputDevice::KeyboardAndMouse).chain(GamepadKind::ALL.map(InputDevice::Gamepad));
        for device in devices {
            let expected: Vec<TextureId> = StartRoomControl::ALL
                .map(|control| TextureId::StartRoomHint(control, device))
                .into();
            assert_eq!(hints_for(device), expected);
        }
    }

    #[test]
    fn the_cut_out_areas_do_not_overlap() {
        let area = |control: StartRoomControl| {
            (
                control.offset() - control.size() / 2.0,
                control.offset() + control.size() / 2.0,
            )
        };
        for (index, first) in StartRoomControl::ALL.into_iter().enumerate() {
            for second in StartRoomControl::ALL.into_iter().skip(index + 1) {
                let ((first_min, first_max), (second_min, second_max)) = (area(first), area(second));
                assert!(
                    first_max.x < second_min.x || second_max.x < first_min.x,
                    "{first:?} and {second:?}"
                );
                assert!(first_min.y < first_max.y && second_min.y < second_max.y);
            }
        }
    }
}
