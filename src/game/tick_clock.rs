//! Fixed ticks: the game advances in steps of `30 / ticks per second` original frames, however
//! often the frontend draws. Real time is carried over between frames until a tick is due.

use glam::Vec2;

use crate::game::original_units::{Frames, frames_per_tick};
use crate::game::settings::InputSampling;
use crate::game::{Game, InputState};

/// After a stall (a window drag, a breakpoint), catch up at most this many ticks at once.
const MAXIMUM_TICKS_PER_UPDATE: u32 = 8;

#[derive(Clone, Copy, Debug)]
pub struct TickClock {
    ticks_per_second: u32,
    carried_seconds: f32,
    sampling: InputSampling,
    /// With [`InputSampling::Latched`], what was pressed since the last tick.
    latched: Option<InputState>,
}

impl TickClock {
    pub fn new(ticks_per_second: u32, sampling: InputSampling) -> Self {
        Self {
            ticks_per_second: ticks_per_second.max(1),
            carried_seconds: 0.0,
            sampling,
            latched: None,
        }
    }

    /// Keeps what `current` presses for the next tick, if presses are latched.
    fn remember(&mut self, current: &InputState) {
        if self.sampling == InputSampling::Latched {
            self.latched = Some(self.latched.map_or(*current, |latched| combined(&latched, current)));
        }
    }

    /// The input the next tick sees: `current`, plus whatever was pressed since the last tick.
    fn input_for_tick(&mut self, current: &InputState) -> InputState {
        self.latched
            .take()
            .map_or(*current, |latched| combined(&latched, current))
    }

    /// How many ticks `real_seconds` more make due.
    fn ticks_due(&mut self, real_seconds: f32) -> u32 {
        let tick_seconds = 1.0 / self.ticks_per_second as f32;
        self.carried_seconds += real_seconds.max(0.0);
        let due = crate::whole_numbers::clamped_u32((self.carried_seconds / tick_seconds).floor());
        if due > MAXIMUM_TICKS_PER_UPDATE {
            self.carried_seconds = 0.0;
            return MAXIMUM_TICKS_PER_UPDATE;
        }
        self.carried_seconds -= due as f32 * tick_seconds;
        due
    }

    /// How far real time has got towards the next tick, from `0.0` to `1.0`.
    pub fn progress(self) -> f32 {
        (self.carried_seconds * self.ticks_per_second as f32).clamp(0.0, 1.0)
    }

    pub fn tick_length(self) -> Frames {
        frames_per_tick(self.ticks_per_second)
    }
}

/// `later` with the presses of `earlier` kept: buttons held in either, and the shoot direction or
/// mouse aim of `earlier` if `later` has none. Movement is `later`'s.
fn combined(earlier: &InputState, later: &InputState) -> InputState {
    InputState {
        device_in_use: later.device_in_use.or(earlier.device_in_use),
        shoot_axes: if later.shoot_axes == Vec2::ZERO {
            earlier.shoot_axes
        } else {
            later.shoot_axes
        },
        aim_target: later.aim_target.or(earlier.aim_target),
        place_bomb: earlier.place_bomb || later.place_bomb,
        use_card: earlier.use_card || later.use_card,
        use_item: earlier.use_item || later.use_item,
        replay: earlier.replay || later.replay,
        ..*later
    }
}

impl Game {
    /// Runs the ticks `real_seconds` of play make due with this frame's `input`; presses kept
    /// since the last tick (see [`InputSampling`]) reach only the first, so catching up never
    /// repeats them.
    pub fn update(&mut self, input: &InputState, real_seconds: f32) {
        if let Some(device) = input.device_in_use {
            self.last_input_device = device;
        }
        let due = self.clock.ticks_due(real_seconds);
        if due == 0 {
            self.clock.remember(input);
            return;
        }
        let first = self.clock.input_for_tick(input);
        self.tick(&first);
        for _ in 1..due {
            self.tick(input);
        }
    }

    /// One fixed step of the simulation.
    pub fn tick(&mut self, input: &InputState) {
        self.advance(input, self.clock.tick_length());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_time_is_carried_until_a_tick_is_due() {
        let mut clock = TickClock::new(60, InputSampling::Original);
        assert_eq!(clock.ticks_due(0.01), 0);
        assert_eq!(clock.ticks_due(0.01), 1);
        assert_eq!(clock.ticks_due(1.0 / 30.0), 2);
        assert_eq!(
            clock.ticks_due(10.0),
            MAXIMUM_TICKS_PER_UPDATE,
            "a stall is not replayed"
        );
        assert_eq!(clock.tick_length(), Frames(0.5));
        assert_eq!(clock.progress(), 0.0);
        clock.ticks_due(1.0 / 120.0);
        assert!((clock.progress() - 0.5).abs() < 1e-4);
    }

    #[test]
    fn a_tap_between_ticks_counts_once_only_when_latched() {
        let tap = InputState {
            use_card: true,
            ..InputState::default()
        };
        let mut original = TickClock::new(30, InputSampling::Original);
        original.remember(&tap);
        assert!(
            !original.input_for_tick(&InputState::default()).use_card,
            "missed, like Key.isDown"
        );
        let mut latched = TickClock::new(30, InputSampling::Latched);
        latched.remember(&tap);
        assert!(
            latched.input_for_tick(&InputState::default()).use_card,
            "kept for the next tick"
        );
        assert!(
            !latched.input_for_tick(&InputState::default()).use_card,
            "and only that one"
        );
    }
}
