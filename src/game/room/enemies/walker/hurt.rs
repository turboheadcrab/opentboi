//! What hits do to a walker.

use glam::Vec2;

use super::{Hurt, Walker, WalkerKind, WalkerState};
use crate::game::original_units::{Damage, Frames, HitPoints};
use crate::game::random::Random;
use crate::game::room::blood::Splatter;
use crate::game::room::enemies::EnemyParameters;

impl Walker {
    /// `hurt()`: a Gaper whose hit left it above -25 hit points turns into a headless body instead of
    /// dying, one time in two, at full health and without a sound, spilling blood where it stands
    /// (`splater()`). A Mulligan that dies without being blown far past death owes the room its flies
    /// (`deathscripts()`, DoAction 13717–13741).
    pub fn hurt(&mut self, damage: Damage, parameters: &EnemyParameters, random: &mut Random) -> Hurt {
        if !self.is_alive() {
            return Hurt::Survived;
        }
        self.hit_points -= damage;
        if self.hit_points < HitPoints::ZERO {
            let headless = parameters.headless;
            if self.kind == WalkerKind::Gaper
                && random.happens(headless.conversion_odds)
                && self.hit_points > headless.conversion_floor
            {
                self.lose_head(parameters);
                let splats = &parameters.splats;
                self.spilled = Some(Splatter::any_look(self.position, splats.beheaded, splats, random));
                return Hurt::Beheaded;
            }
            self.state = WalkerState::Dying(Frames::ZERO);
            self.velocity = Vec2::ZERO;
            // A Mulligan blown far past death leaves no flies.
            self.blob.owes_flies =
                self.kind == WalkerKind::Mulligan && self.hit_points > parameters.mulligans.flies_above;
            return Hurt::Died;
        }
        self.flash_left = parameters.flies.flash_after_hit(damage);
        Hurt::Survived
    }

    /// Flashing red from a hit (`hurtcol`).
    pub fn is_flashing(&self) -> bool {
        self.flash_left > Frames::ZERO && self.is_alive()
    }

    /// The blood it spilled losing its head, if the room has not taken it yet.
    pub(in crate::game::room::enemies) fn take_spilled(&mut self) -> Option<Splatter> {
        self.spilled.take()
    }
}
