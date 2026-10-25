//! Which of the pack's pictures names each control, per make of pad.

use crate::game::{GamepadKind, StartRoomControl};

macro_rules! glyph {
    ($file:literal) => {
        include_str!(concat!("../../../assets/input-glyphs/kenney/", $file))
    };
}

/// Each make's pictures in the pack's style: the buttons outlined (a ring around the button's name).
const XBOX_LEFT_BUMPER: &str = glyph!("xbox_lb_outline.svg");
const PLAYSTATION_LEFT_BUMPER: &str = glyph!("playstation_trigger_l1_outline.svg");
const SWITCH_LEFT_BUMPER: &str = glyph!("switch_button_l_outline.svg");
const STEAM_DECK_LEFT_BUMPER: &str = glyph!("steamdeck_button_l1_outline.svg");
const XBOX_RIGHT_BUMPER: &str = glyph!("xbox_rb_outline.svg");
const PLAYSTATION_RIGHT_BUMPER: &str = glyph!("playstation_trigger_r1_outline.svg");
const SWITCH_RIGHT_BUMPER: &str = glyph!("switch_button_r_outline.svg");
const STEAM_DECK_RIGHT_BUMPER: &str = glyph!("steamdeck_button_r1_outline.svg");
const XBOX_LEFT_TRIGGER: &str = glyph!("xbox_lt_outline.svg");
const PLAYSTATION_LEFT_TRIGGER: &str = glyph!("playstation_trigger_l2_outline.svg");
const SWITCH_LEFT_TRIGGER: &str = glyph!("switch_button_zl_outline.svg");
const STEAM_DECK_LEFT_TRIGGER: &str = glyph!("steamdeck_button_l2_outline.svg");

/// The D-pad's arrows, each with one direction lit, in the hints' order: up, left, down, right
/// (Rebirth's). The cross is the filled picture (drawn solid, the pressed arm hollow, like
/// Rebirth's); the PlayStation's four separate arrows look heavy that way, so theirs is the outlined
/// picture (the pressed arrow solid). The pack draws the Steam Deck's D-pad as the Xbox one.
const XBOX_DPAD: [&str; 4] = [
    glyph!("xbox_dpad_up.svg"),
    glyph!("xbox_dpad_left.svg"),
    glyph!("xbox_dpad_down.svg"),
    glyph!("xbox_dpad_right.svg"),
];
const PLAYSTATION_DPAD: [&str; 4] = [
    glyph!("playstation_dpad_up_outline.svg"),
    glyph!("playstation_dpad_left_outline.svg"),
    glyph!("playstation_dpad_down_outline.svg"),
    glyph!("playstation_dpad_right_outline.svg"),
];
const SWITCH_DPAD: [&str; 4] = [
    glyph!("switch_dpad_up.svg"),
    glyph!("switch_dpad_left.svg"),
    glyph!("switch_dpad_down.svg"),
    glyph!("switch_dpad_right.svg"),
];

/// The lettered face buttons; the pack draws the same letters for Xbox, Switch and the Steam Deck.
const BUTTON_A: &str = glyph!("xbox_button_a_outline.svg");
const BUTTON_B: &str = glyph!("xbox_button_b_outline.svg");
const BUTTON_X: &str = glyph!("xbox_button_x_outline.svg");
const BUTTON_Y: &str = glyph!("xbox_button_y_outline.svg");
/// Face buttons by position, in the hints' order: top, left, bottom, right (Rebirth's Y, X, A, B).
/// Buttons are bound by position (see `frontend::controls`), so a Switch pad's top button is its X.
const XBOX_FACE_BUTTONS: [&str; 4] = [BUTTON_Y, BUTTON_X, BUTTON_A, BUTTON_B];
const SWITCH_FACE_BUTTONS: [&str; 4] = [BUTTON_X, BUTTON_Y, BUTTON_B, BUTTON_A];
const PLAYSTATION_FACE_BUTTONS: [&str; 4] = [
    glyph!("playstation_button_triangle_outline.svg"),
    glyph!("playstation_button_square_outline.svg"),
    glyph!("playstation_button_cross_outline.svg"),
    glyph!("playstation_button_circle_outline.svg"),
];

/// The picture of the bumper on the pad's left shoulder, which drops bombs.
pub fn left_bumper(kind: GamepadKind) -> &'static str {
    match kind {
        GamepadKind::Xbox => XBOX_LEFT_BUMPER,
        GamepadKind::PlayStation => PLAYSTATION_LEFT_BUMPER,
        GamepadKind::Switch => SWITCH_LEFT_BUMPER,
        GamepadKind::SteamDeck => STEAM_DECK_LEFT_BUMPER,
    }
}

/// The picture of the bumper on the pad's right shoulder, which uses the card.
pub fn right_bumper(kind: GamepadKind) -> &'static str {
    match kind {
        GamepadKind::Xbox => XBOX_RIGHT_BUMPER,
        GamepadKind::PlayStation => PLAYSTATION_RIGHT_BUMPER,
        GamepadKind::Switch => SWITCH_RIGHT_BUMPER,
        GamepadKind::SteamDeck => STEAM_DECK_RIGHT_BUMPER,
    }
}

/// The picture of the trigger on the pad's left shoulder, which uses the active item.
pub fn left_trigger(kind: GamepadKind) -> &'static str {
    match kind {
        GamepadKind::Xbox => XBOX_LEFT_TRIGGER,
        GamepadKind::PlayStation => PLAYSTATION_LEFT_TRIGGER,
        GamepadKind::Switch => SWITCH_LEFT_TRIGGER,
        GamepadKind::SteamDeck => STEAM_DECK_LEFT_TRIGGER,
    }
}

/// The D-pad's four directions, which move Isaac.
pub fn dpad(kind: GamepadKind) -> &'static [&'static str] {
    match kind {
        GamepadKind::Xbox | GamepadKind::SteamDeck => &XBOX_DPAD,
        GamepadKind::PlayStation => &PLAYSTATION_DPAD,
        GamepadKind::Switch => &SWITCH_DPAD,
    }
}

/// The four face buttons, which shoot.
pub fn face_buttons(kind: GamepadKind) -> &'static [&'static str] {
    match kind {
        GamepadKind::Xbox | GamepadKind::SteamDeck => &XBOX_FACE_BUTTONS,
        GamepadKind::PlayStation => &PLAYSTATION_FACE_BUTTONS,
        GamepadKind::Switch => &SWITCH_FACE_BUTTONS,
    }
}

/// The pad's controls for one of the start room's hints, left to right.
pub fn start_room_glyphs(control: StartRoomControl, kind: GamepadKind) -> Vec<&'static str> {
    match control {
        StartRoomControl::Move => dpad(kind).to_vec(),
        StartRoomControl::Attack => face_buttons(kind).to_vec(),
        StartRoomControl::Bomb => vec![left_bumper(kind)],
        StartRoomControl::Item => vec![left_trigger(kind)],
    }
}
