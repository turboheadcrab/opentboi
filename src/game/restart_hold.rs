//! Restarting the run by holding a key or buttons, as in Rebirth: the screen fades to black over
//! the two seconds the hold takes, then a new run starts, and another every two seconds while it is
//! held.

use crate::game::draw::{DrawItem, Rgba};
use crate::game::original_units::Frames;
use crate::game::room::{STAGE_CENTER, STAGE_SIZE};

/// Two seconds of the original's frames.
const HOLD_TO_RESTART: Frames = Frames(60.0);
/// Above everything, the view masks included.
const FADE_DEPTH: f32 = 950.0;
/// The fade covers well past the stage, whatever the window's shape.
const FADE_SCALE: f32 = 8.0;

#[derive(Clone, Copy, Debug, Default)]
pub struct RestartHold {
    held: Frames,
}

impl RestartHold {
    /// Whether to restart now; held on, it restarts again every two seconds.
    pub fn update(&mut self, held: bool, elapsed: Frames) -> bool {
        self.held = if held { self.held + elapsed } else { Frames::ZERO };
        self.held >= HOLD_TO_RESTART
    }

    pub fn append_draw_items(self, draw_items: &mut Vec<DrawItem>) {
        let darkness = (self.held / HOLD_TO_RESTART).clamp(0.0, 1.0);
        if darkness > 0.0 {
            let black = Rgba {
                red: 0.0,
                green: 0.0,
                blue: 0.0,
                alpha: darkness,
            };
            draw_items.push(DrawItem::solid(
                STAGE_CENTER,
                STAGE_SIZE * FADE_SCALE,
                black,
                FADE_DEPTH,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_two_seconds_of_holding_restarts() {
        let mut hold = RestartHold::default();
        assert!(!hold.update(true, Frames(59.0)));
        assert!(hold.update(true, Frames(1.0)));
        let mut hold = RestartHold::default();
        assert!(!hold.update(true, Frames(30.0)));
        assert!(!hold.update(false, Frames(1.0)));
        assert!(!hold.update(true, Frames(30.0)), "letting go starts over");
    }
}
