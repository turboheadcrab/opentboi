//! Choices between the original's behavior and alternatives (Rebirth's, or debugging aids).

use crate::game::input_state::InputDevice;
use crate::game::original_units::DEFAULT_TICKS_PER_SECOND;

/// Whose behavior a choice follows: the original Flash game (the default) or Rebirth.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Style {
    #[default]
    Original,
    Rebirth,
}

/// How buttons pressed between two ticks reach the game.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputSampling {
    /// Each tick sees the buttons as they are at that moment, as the original's `Key.isDown()`
    /// once per frame: a tap shorter than a tick can be missed.
    #[default]
    Original,
    /// Presses seen between ticks are kept for the next tick (bombs, cards, shooting), so even a
    /// tap shorter than a tick counts, once.
    Latched,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools, reason = "independent options")]
pub struct GameSettings {
    /// Rooms slide into view like the original; off with `--instant-room-changes`.
    pub slide_between_rooms: bool,
    /// A bombed-open secret entrance closes once the player leaves the secret room (original) or
    /// stays open for the rest of the floor (Rebirth).
    pub secret_entrances: Style,
    /// A secret room next to the curse room has an ordinary spiked curse door on both sides
    /// (original: the door setup checks the curse room before the secret room) or a wall to bomb
    /// whose hole does not hurt (Rebirth).
    pub curse_secret_door: Style,
    /// The HUD's look. Only the original's exists so far.
    pub hud: Style,
    /// Simulation steps per second (`--tick-rate`); each covers `30 / ticks_per_second` of the
    /// original's frames.
    pub ticks_per_second: u32,
    /// Draw between ticks for smooth motion (on); `--no-interpolation` draws whole ticks.
    pub interpolation: bool,
    /// Rebirth's stats panel on the left, for debugging (on for now; `--no-stats`).
    pub show_stats: bool,
    /// Developer showcases in the floor's rooms (`--showcase`): every pickup kind in the boss
    /// room, every machine in the shop, every card in the treasure room. Off in normal play.
    pub showcase: bool,
    /// `--input original|latched`.
    pub input_sampling: InputSampling,
    /// Whose button names the hints show until real input arrives (`--hints`; for checking the
    /// pictures of pads nobody has at hand).
    pub initial_input_device: InputDevice,
}

impl Default for GameSettings {
    /// The original's behavior.
    fn default() -> Self {
        Self {
            slide_between_rooms: true,
            secret_entrances: Style::Original,
            curse_secret_door: Style::Original,
            hud: Style::Original,
            ticks_per_second: DEFAULT_TICKS_PER_SECOND,
            interpolation: true,
            show_stats: true,
            showcase: false,
            input_sampling: InputSampling::Original,
            initial_input_device: InputDevice::KeyboardAndMouse,
        }
    }
}
