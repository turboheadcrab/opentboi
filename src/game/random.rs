//! Small deterministic random source for gameplay variation (xorshift32), so the game core needs no
//! extra dependency and runs are reproducible from a seed.

use glam::Vec2;

use crate::game::odds::Odds;
use crate::game::original_units::{Frames, Pixels};
use crate::game::pickups::trinket::{Trinket, TrinketPool};

const DEFAULT_SEED: u32 = 0x9E37_79B9;
const UNIT_INTERVAL_STEPS: f32 = (1u32 << 24) as f32;
const MANTISSA_SHIFT: u32 = 8;

#[derive(Clone, Debug)]
pub struct Random {
    state: u32,
    /// The trinkets not handed out yet this run: a deck drawn from without replacement, which is
    /// why it lives with the run's randomness.
    trinkets: TrinketPool,
}

impl Random {
    pub fn with_seed(seed: u32) -> Self {
        Self {
            state: if seed == 0 { DEFAULT_SEED } else { seed },
            trinkets: TrinketPool::default(),
        }
    }

    /// `junx()`: a trinket the run has not handed out yet (always the Counterfeit Penny for a gold
    /// pickup), or `None` once they are all out.
    pub fn trinket(&mut self, golden: bool) -> Option<Trinket> {
        let mut pool = std::mem::replace(&mut self.trinkets, TrinketPool::empty());
        let trinket = pool.take(self, golden);
        self.trinkets = pool;
        trinket
    }

    /// Like Flash's random number of a limit: a whole number below `limit`, or 0 when `limit` is
    /// not positive.
    pub fn below(&mut self, limit: i32) -> i32 {
        if limit <= 0 {
            return 0;
        }
        crate::whole_numbers::clamped_i32(self.next_unit() * limit as f32).min(limit - 1)
    }

    /// Whether a roll of one in `odds` comes up.
    pub fn happens(&mut self, odds: Odds) -> bool {
        self.below(i32::try_from(odds.chances()).unwrap_or(i32::MAX)) == 0
    }

    /// [`Self::below`] for a small count or choice.
    pub fn below_u8(&mut self, limit: u8) -> u8 {
        u8::try_from(self.below(i32::from(limit))).unwrap_or_default()
    }

    /// A whole number of pixels below `limit`.
    pub fn pixels_below(&mut self, limit: Pixels) -> Pixels {
        Pixels(self.below(crate::whole_numbers::clamped_i32(limit.get())) as f32)
    }

    /// A whole number of frames below `limit`.
    pub fn frames_below(&mut self, limit: Frames) -> Frames {
        Frames::from_number(
            u32::try_from(self.below(i32::try_from(limit.whole()).unwrap_or(i32::MAX))).unwrap_or_default(),
        )
    }

    /// [`Self::below`] used to pick from a list of `length` things: an index into it (0 for an
    /// empty list).
    pub fn index(&mut self, length: usize) -> usize {
        let limit = i32::try_from(length).unwrap_or(i32::MAX);
        usize::try_from(self.below(limit)).unwrap_or_default()
    }

    /// The original's `crand()` (DoAction 1440–1451), called twice: `length` pixels in a random direction, the cosine
    /// and the sine of one random angle.
    pub fn direction(&mut self, length: f32) -> Vec2 {
        self.unit_direction() * length
    }

    /// A random direction one pixel long (`crand()`'s angle): `x` is the cosine, `y` the sine.
    pub fn unit_direction(&mut self) -> Vec2 {
        Vec2::from_angle(self.next_unit() * FULL_TURN)
    }

    /// The original's `rand()` (DoAction 3972): uniform in `-0.5..0.5`, centred on zero.
    pub fn centered(&mut self) -> f32 {
        self.next_unit() - CENTERING_OFFSET
    }

    /// Uniform value in `0.0..1.0`, like Flash's random fraction.
    pub fn next_unit(&mut self) -> f32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 17;
        self.state ^= self.state << 5;
        (self.state >> MANTISSA_SHIFT) as f32 / UNIT_INTERVAL_STEPS
    }
}

impl Default for Random {
    fn default() -> Self {
        Self::with_seed(DEFAULT_SEED)
    }
}

/// Half of the unit interval, which `centered` subtracts.
const CENTERING_OFFSET: f32 = 0.5;

/// A whole turn in radians, for random directions (the angle is a random fraction of it, DoAction 1444).
pub const FULL_TURN: f32 = std::f32::consts::TAU;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centered_values_fall_on_both_sides_of_zero() {
        let mut random = Random::default();
        let values: Vec<f32> = (0..10_000).map(|_| random.centered()).collect();
        assert!(values.iter().all(|value| (-0.5..0.5).contains(value)));
        assert!(values.iter().any(|&value| value < -0.4) && values.iter().any(|&value| value > 0.4));
    }

    #[test]
    fn values_stay_in_unit_interval() {
        let mut random = Random::default();
        assert!(
            (0..10_000)
                .map(|_| random.next_unit())
                .all(|value| (0.0..1.0).contains(&value))
        );
    }

    #[test]
    fn an_index_is_the_same_draw_as_below() {
        let (mut first, mut second) = (Random::with_seed(7), Random::with_seed(7));
        for length in [1_usize, 2, 5, 22, 80] {
            let limit = i32::try_from(length).expect("small");
            assert_eq!(
                first.index(length),
                usize::try_from(second.below(limit)).expect("not negative")
            );
        }
        assert_eq!(first.index(0), 0, "an empty list gives 0, like random(0)");
    }

    #[test]
    fn a_direction_has_the_asked_length() {
        let mut random = Random::default();
        for _ in 0..100 {
            assert!((random.direction(5.0).length() - 5.0).abs() < 1e-4);
        }
    }
}
