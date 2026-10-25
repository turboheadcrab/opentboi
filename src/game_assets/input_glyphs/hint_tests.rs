//! The start room's pad hints: which pictures, in which order, and how the D-pad is drawn.

use super::pictures::{dpad, face_buttons, left_bumper};
use super::tests::{alpha_at, hint, layers_of, render};
use super::*;
use crate::game::{GamepadKind, StartRoomControl};

#[test]
fn start_room_hints_name_the_controls_that_do_it() {
    for kind in GamepadKind::ALL {
        assert_eq!(start_room_glyphs(StartRoomControl::Move, kind), dpad(kind));
        assert_eq!(start_room_glyphs(StartRoomControl::Attack, kind), face_buttons(kind));
        assert_eq!(start_room_glyphs(StartRoomControl::Bomb, kind), [left_bumper(kind)]);
        assert_eq!(start_room_glyphs(StartRoomControl::Item, kind), [left_trigger(kind)]);
    }
}

/// Rebirth's order: the D-pad's up, left, down, right; the face buttons top, left, bottom, right.
#[test]
fn the_four_directions_are_four_different_pictures_side_by_side() {
    for kind in GamepadKind::ALL {
        for row in [dpad(kind), face_buttons(kind)] {
            assert_eq!(row.len(), 4, "{kind:?}");
            for (index, picture) in row.iter().enumerate() {
                assert!(!row[index + 1..].contains(picture), "{kind:?}");
            }
        }
        let texture_id = hint(StartRoomControl::Move, kind);
        let width = texture_id.world_size().x;
        let quarters: Vec<usize> = layers_of(texture_id)
            .iter()
            .map(|layer| {
                let bounds = layer.path.bounds();
                crate::whole_numbers::clamped_usize(f64::from(
                    (bounds.x() + bounds.width() / 2.0 + width / 2.0) / (width / 4.0),
                ))
            })
            .collect();
        assert!(quarters.is_sorted(), "{kind:?}: left to right");
        assert_eq!(
            quarters.first().zip(quarters.last()),
            Some((&0, &3)),
            "{kind:?}: across the width"
        );
    }
    assert_eq!(
        face_buttons(GamepadKind::Xbox)[0],
        face_buttons(GamepadKind::Switch)[1],
        "Y sits left on a Switch pad"
    );
    assert_ne!(dpad(GamepadKind::PlayStation), dpad(GamepadKind::Xbox));
}

/// The pressed arm of Kenney's filled D-pad is a second, coloured path: it becomes an outline, so
/// the pad is solid and the pressed arm hollow, as in Rebirth.
#[test]
fn the_pressed_arm_of_the_dpad_is_hollow() {
    for kind in GamepadKind::ALL {
        let pressed_up = picture_paths(dpad(kind)[0]).expect("readable");
        let expected = if kind == GamepadKind::PlayStation { 1 } else { 2 };
        assert_eq!(pressed_up.len(), expected, "{kind:?}: the pad and its pressed arm");
    }
    // The Switch's pressed arm holds an arrow in its middle, and the PlayStation's outlined
    // picture fills the pressed arrow instead.
    for kind in [GamepadKind::Xbox, GamepadKind::SteamDeck] {
        let image = render(hint(StartRoomControl::Move, kind));
        let up_arm_middle = (image.width / 8, image.height / 4);
        assert_eq!(
            alpha_at(&image, up_arm_middle.0, up_arm_middle.1),
            0,
            "{kind:?}: open inside the pressed arm"
        );
    }
}
