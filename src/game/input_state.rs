use glam::Vec2;

/// The make of a gamepad, for showing the button names printed on it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GamepadKind {
    /// Xbox pads, and any pad nobody can tell apart: SDL's standard layout uses Xbox's names.
    #[default]
    Xbox,
    PlayStation,
    /// Switch Pro and Joy-Cons.
    Switch,
    /// The Steam Deck's own controls.
    SteamDeck,
}

impl GamepadKind {
    pub const ALL: [Self; 4] = [Self::Xbox, Self::PlayStation, Self::Switch, Self::SteamDeck];
}

/// The kind of device the player last used, for showing the right button names.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum InputDevice {
    #[default]
    KeyboardAndMouse,
    Gamepad(GamepadKind),
}

/// Device-independent player intent for one frame. Frontends build it from keyboards, gamepads and
/// the mouse.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[allow(clippy::struct_excessive_bools, reason = "one flag per button")]
pub struct InputState {
    /// Length is at most 1; analog sticks may produce less than full speed.
    pub movement: Vec2,
    /// The shoot keys or stick, each axis -1 to 1; the player picks one of four directions from
    /// it (see `ShootingAxes`).
    pub shoot_axes: Vec2,
    /// World position of the mouse cursor while the shoot button is held. Overrides
    /// `shoot_direction`, as in the original.
    pub aim_target: Option<Vec2>,
    /// Held bomb button (E); a bomb drops whenever the cooldown allows.
    pub place_bomb: bool,
    /// Held restart key or buttons (Rebirth's R, or right stick plus right trigger).
    pub restart_held: bool,
    /// The card held is used (Q, or RB).
    pub use_card: bool,
    /// The active item is used (Space, or LT).
    pub use_item: bool,
    /// "Replay?" on the death note (Space; South on a gamepad).
    pub replay: bool,
    /// The device that gave input this frame (a key held, a button held, a stick pushed past its
    /// deadzone); `None` when nothing was touched.
    pub device_in_use: Option<InputDevice>,
}

impl InputState {
    /// `movement_axes` and `shoot_axes` are the sums of all devices' raw directions.
    pub fn from_raw_axes(movement_axes: Vec2, shoot_axes: Vec2) -> Self {
        Self {
            movement: movement_axes.clamp_length_max(1.0),
            shoot_axes: shoot_axes.clamp(Vec2::NEG_ONE, Vec2::ONE),
            aim_target: None,
            place_bomb: false,
            restart_held: false,
            use_card: false,
            use_item: false,
            replay: false,
            device_in_use: None,
        }
    }
}

/// Microsoft's documented XInput thumbstick deadzones (`XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE` 7849
/// and `..._RIGHT_THUMB_DEADZONE` 8689 out of 32767). Provisional: Rebirth's own values still have to
/// be read from its executable.
pub const LEFT_STICK_DEADZONE: f32 = 7849.0 / 32767.0;
pub const RIGHT_STICK_DEADZONE: f32 = 8689.0 / 32767.0;

/// Radial deadzone: deflection inside `deadzone` reads as zero, and the rest is rescaled so output
/// still ramps smoothly from 0 at the deadzone edge to 1 at full deflection.
pub fn apply_radial_deadzone(stick: Vec2, deadzone: f32) -> Vec2 {
    let deflection = stick.length();
    if deflection <= deadzone {
        return Vec2::ZERO;
    }
    let rescaled = ((deflection - deadzone) / (1.0 - deadzone)).min(1.0);
    stick / deflection * rescaled
}

/// The dominant axis of `direction` as a unit vector; ties go to the horizontal axis, as mouse
/// aiming does.
pub fn snap_to_cardinal(direction: Vec2) -> Option<Vec2> {
    if direction == Vec2::ZERO {
        None
    } else if direction.x.abs() >= direction.y.abs() {
        Some(Vec2::new(direction.x.signum(), 0.0))
    } else {
        Some(Vec2::new(0.0, direction.y.signum()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyboard_diagonal_movement_is_normalized() {
        let input = InputState::from_raw_axes(Vec2::new(1.0, 1.0), Vec2::ZERO);
        assert!((input.movement.length() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn aiming_snaps_to_dominant_axis_and_ties_go_sideways() {
        assert_eq!(snap_to_cardinal(Vec2::new(-0.9, 0.4)), Some(Vec2::NEG_X));
        assert_eq!(snap_to_cardinal(Vec2::new(0.5, -0.5)), Some(Vec2::X));
    }

    #[test]
    fn deadzone_ignores_drift_and_keeps_full_range() {
        assert_eq!(
            apply_radial_deadzone(Vec2::new(0.2, 0.1), LEFT_STICK_DEADZONE),
            Vec2::ZERO
        );
        assert!((apply_radial_deadzone(Vec2::X, LEFT_STICK_DEADZONE).length() - 1.0).abs() < 1e-6);
        let halfway = apply_radial_deadzone(Vec2::new(0.0, 0.62), LEFT_STICK_DEADZONE);
        assert!(halfway.x == 0.0 && (0.4..0.6).contains(&halfway.y));
    }

    #[test]
    fn no_shoot_input_means_no_direction() {
        assert_eq!(
            snap_to_cardinal(InputState::from_raw_axes(Vec2::ZERO, Vec2::ZERO).shoot_axes),
            None
        );
    }
}
