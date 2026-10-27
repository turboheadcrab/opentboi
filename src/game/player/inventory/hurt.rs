//! Getting hurt: the hit itself, the blink and shield that keep the next ones off, and dying.

use crate::game::original_units::{Frames, Hearts};
use crate::game::player::inventory::Inventory;

impl Inventory {
    /// `lasth = 0` before a hurt that must go through: no blink left from an earlier hit.
    pub fn end_invulnerability(&mut self) {
        self.invulnerable_frames = Frames::ZERO;
    }

    /// `playerhurt()`: soul hearts absorb the hit (the rest is lost), else the eternal heart softens
    /// it by half a heart and red hearts take it. Returns whether it hurt.
    pub fn hurt(&mut self, damage: Hearts) -> bool {
        // Not while invulnerable after a hit, nor in a bubble, nor on the unicorn.
        if self.invulnerable_frames > Frames::ZERO || self.is_shielded() || self.ride.is_riding() {
            return false;
        }
        if self.soul_hearts > Hearts::ZERO {
            self.soul_hearts = (self.soul_hearts - damage).max(Hearts::ZERO);
        } else {
            if self.eternal_heart {
                self.eternal_heart = false;
                self.red_hearts += Hearts::HALF;
            }
            self.red_hearts = (self.red_hearts - damage).max(Hearts::ZERO);
        }
        self.invulnerable_frames = self.parameters.invulnerable_frames_per_heart * damage.get();
        true
    }

    pub fn is_dead(&self) -> bool {
        self.red_hearts <= Hearts::ZERO && self.soul_hearts <= Hearts::ZERO && !self.eternal_heart
    }

    pub fn rides_unicorn(&self) -> bool {
        self.ride.is_riding()
    }

    pub fn is_invulnerable(&self) -> bool {
        self.invulnerable_frames > Frames::ZERO
    }

    pub fn advance(&mut self, elapsed: Frames) {
        self.invulnerable_frames = (self.invulnerable_frames - elapsed).max(Frames::ZERO);
        self.advance_shield(elapsed);
    }

    /// Whether the player is hidden this frame by the hurt blink.
    pub fn blinks_hidden(&self) -> bool {
        let frames_left = Frames(self.invulnerable_frames.get().ceil());
        let periods_left = (frames_left / self.parameters.blink_period_frames).ceil();
        frames_left > self.parameters.blink_until_frames_left && periods_left % 2.0 == 0.0
    }
}
