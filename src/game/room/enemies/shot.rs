//! Enemy shots (type 9, clip `bullet1`): a blood ball flying straight at 7 pixels per frame,
//! rising then falling (DoAction 14982–14988), ending on the floor, against a wall or anything
//! solid (`mhity()`), or on the player, hurting by half a heart. Kept in stage coordinates (y down)
//! like the enemies.

use glam::Vec2;

use crate::game::original_units::{Frames, Pixels, PixelsPerFrame, PixelsPerFrameSquared};
use crate::game::parameters::EnemyShotParameters;
use crate::game::player::death::Killer;
use crate::game::room::stage_area::blocks_flight;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShotEnd {
    /// `dy > -5`: it fell to the floor.
    Floor,
    /// Against a wall or something solid.
    Wall,
}

#[derive(Clone, Debug)]
pub struct EnemyShot {
    /// The floor point under it (`xp`, `yp`).
    pub position: Vec2,
    pub previous_position: Vec2,
    velocity: Vec2,
    /// `dy` (negative is up) and its rate `dm`.
    height: Pixels,
    previous_height: Pixels,
    rise: PixelsPerFrame,
    age: Frames,
    /// `sss`: who shot it, for the death note.
    pub shooter: Killer,
    /// What the falling rate gains besides `fall_gain` each frame (`fd`).
    gravity: PixelsPerFrameSquared,
    /// The room frame it was created in (`fra`); it touches nothing before the next one.
    born_frame: Frames,
}

impl EnemyShot {
    pub fn new(position: Vec2, velocity: Vec2, shooter: Killer, parameters: EnemyShotParameters) -> Self {
        Self {
            position,
            previous_position: position,
            velocity,
            height: parameters.start_height,
            previous_height: parameters.start_height,
            rise: parameters.start_rise,
            age: Frames::ZERO,
            shooter,
            gravity: parameters.gravity,
            born_frame: Frames::ZERO,
        }
    }

    /// Created during room frame `frame`.
    pub fn born_in(self, frame: Frames) -> Self {
        Self {
            born_frame: frame,
            ..self
        }
    }

    #[cfg(test)]
    pub fn velocity(&self) -> Vec2 {
        self.velocity
    }

    /// Whether it was created in room frame `frame`, so that it touches nothing yet.
    pub fn is_new_in(&self, frame: Frames) -> bool {
        self.born_frame == frame
    }

    /// The headless body's blood ball: it rises faster and falls harder than a Pooter's.
    pub fn spurted(position: Vec2, velocity: Vec2, shooter: Killer, parameters: EnemyShotParameters) -> Self {
        Self {
            rise: parameters.spurt_start_rise,
            gravity: parameters.spurt_gravity,
            ..Self::new(position, velocity, shooter, parameters)
        }
    }

    /// Moves by `elapsed`, and falls one step per whole frame crossed.
    pub fn advance(&mut self, elapsed: Frames, parameters: EnemyShotParameters) {
        self.previous_position = self.position;
        self.previous_height = self.height;
        let first_frame = self.age.whole() + 1;
        self.age += elapsed;
        for _ in first_frame..=self.age.whole() {
            self.rise = self.rise * parameters.fall_retention + (parameters.fall_gain + self.gravity) * Frames::ONE;
            self.height += self.rise * Frames::ONE;
        }
        self.position += self.velocity * elapsed.get();
    }

    /// Whether it has ended, given whether its floor point is in something solid.
    pub fn end(&self, in_solid: bool, parameters: EnemyShotParameters) -> Option<ShotEnd> {
        if blocks_flight(self.position, || in_solid) {
            Some(ShotEnd::Wall)
        } else {
            (self.height > parameters.landing_height).then_some(ShotEnd::Floor)
        }
    }

    /// Where the ball is drawn: its floor point raised by its height (stage coordinates).
    pub fn drawn_point(&self) -> Vec2 {
        self.position + Vec2::new(0.0, self.height.get())
    }

    pub fn previous_drawn_point(&self) -> Vec2 {
        self.previous_position + Vec2::new(0.0, self.previous_height.get())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::parameters::GameParameters;

    const PARAMETERS: EnemyShotParameters = GameParameters::ORIGINAL.enemy_shots;

    #[test]
    fn a_shot_rises_then_falls_to_the_floor() {
        let mut shot = EnemyShot::new(
            Vec2::new(320.0, 280.0),
            Vec2::ZERO,
            Killer::Enemy(crate::game::texture_id::FlyKind::Pooter),
            PARAMETERS,
        );
        let mut peak = shot.height;
        let mut frames = 0;
        while shot.end(false, PARAMETERS).is_none() {
            shot.advance(Frames(1.0), PARAMETERS);
            peak = peak.min(shot.height);
            frames += 1;
            assert!(frames < 100, "it lands");
        }
        assert!(peak < PARAMETERS.start_height, "it rises first");
        assert_eq!(shot.end(false, PARAMETERS), Some(ShotEnd::Floor));
        assert!((15..40).contains(&frames), "lands after {frames} frames");
    }

    #[test]
    fn a_shot_stops_at_walls_and_solid_things() {
        let shot = EnemyShot::new(
            Vec2::new(10.0, 280.0),
            Vec2::ZERO,
            Killer::Enemy(crate::game::texture_id::FlyKind::Pooter),
            PARAMETERS,
        );
        assert_eq!(shot.end(false, PARAMETERS), Some(ShotEnd::Wall));
        let shot = EnemyShot::new(
            Vec2::new(320.0, 280.0),
            Vec2::ZERO,
            Killer::Enemy(crate::game::texture_id::FlyKind::Pooter),
            PARAMETERS,
        );
        assert_eq!(shot.end(true, PARAMETERS), Some(ShotEnd::Wall));
    }
}
