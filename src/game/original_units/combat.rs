//! What fights are measured in: how much a body can take, how much a hit takes, and how heavy
//! bodies are when they bump.

use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

unit!(
    /// What a body can take before it dies (the original's `hps`); it dies once below zero.
    HitPoints
);
unit!(
    /// What one hit takes off hit points.
    Damage
);
unit!(
    /// How heavy a body is: when two bump, the lighter one is pushed more.
    Mass
);

impl Sub<Damage> for HitPoints {
    type Output = Self;
    fn sub(self, damage: Damage) -> Self {
        Self(self.0 - damage.0)
    }
}

impl SubAssign<Damage> for HitPoints {
    fn sub_assign(&mut self, damage: Damage) {
        self.0 -= damage.0;
    }
}
