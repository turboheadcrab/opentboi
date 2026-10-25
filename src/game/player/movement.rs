//! Isaac walking: approaching the input's velocity, stopping at walls and obstacles with the
//! original's rebound, sliding along walls, and spider webs.

use glam::Vec2;

use crate::game::bounds::Bounds;
use crate::game::original_units::{Frames, Retention};
use crate::game::parameters::MovementParameters;
use crate::game::player::{COLLISION_RADIUS, DIAGONAL_INPUT_MINIMUM, Player, Surroundings};
use crate::game::wall_contact::rebound;

impl Player {
    /// Exponential approach to the target velocity; matches the original's per-frame rule exactly
    /// at 30 frames per second and stays stable at any frame rate.
    pub(super) fn move_within(&mut self, movement: Vec2, elapsed: Frames, surroundings: &Surroundings) {
        let movement = self.slide_along_wall(movement, elapsed);
        let movement_parameters = MovementParameters {
            speed_stat: self.stats.speed,
            ..self.parameters.movement
        };
        let target_velocity = movement * movement_parameters.top_speed_per_frame();
        let smoothing = 1.0 - movement_parameters.velocity_retention_per_frame().over(elapsed);
        self.velocity += (target_velocity - self.velocity) * smoothing;
        self.apply_web_slowdown(elapsed, surroundings.spider_webs);
        self.step(elapsed, surroundings);
    }

    /// Keeps `retention` of the velocity every frame, deaf to the controls (dying).
    pub(super) fn coast(&mut self, retention: Retention, elapsed: Frames, surroundings: &Surroundings) {
        self.velocity *= retention.over(elapsed);
        self.step(elapsed, surroundings);
    }

    /// Moves by the velocity, kept out of walls and obstacles.
    fn step(&mut self, elapsed: Frames, surroundings: &Surroundings) {
        let walkable_area = surroundings.room_interior.shrunk_by(COLLISION_RADIUS);
        let target = self.position + self.velocity * elapsed;
        let mut position = walkable_area.clamp(target);
        for tile in surroundings.solid_tiles {
            position = tile.push_circle_out(position, COLLISION_RADIUS);
        }
        self.position = position;
        self.rebound_from_walls(target - position, elapsed);
    }

    /// `physix()` (DoAction 25960–26112): pressing into a wall or obstacle turns that part of the
    /// velocity around (so tears fired then do not inherit it), slows Isaac, and for a moment makes
    /// diagonal input slide along the wall.
    fn rebound_from_walls(&mut self, blocked: Vec2, elapsed: Frames) {
        if blocked == Vec2::ZERO {
            return;
        }
        let walls = self.parameters.walls;
        if blocked.dot(self.velocity) > 0.0 {
            self.slide = Some((blocked, walls.player_slide_frames));
        }
        self.velocity = rebound(self.velocity, blocked, COLLISION_RADIUS, walls);
        self.velocity *= walls.player_friction.over(elapsed);
    }

    /// Just after pressing into a wall (DoAction 28986),
    /// a diagonal keeps only the direction along the wall, at full speed (otherwise a diagonal is
    /// divided by √2).
    pub(super) fn slide_along_wall(&mut self, movement: Vec2, elapsed: Frames) -> Vec2 {
        let Some((towards_wall, frames_left)) = self.slide else {
            return movement;
        };
        let frames_left = frames_left - elapsed;
        self.slide = (frames_left > Frames::ZERO).then_some((towards_wall, frames_left));
        // The original only had keys, whose diagonals are exactly 45°; a stick held nearly
        // straight is not a diagonal.
        let strength = movement.length();
        if movement.x.abs().min(movement.y.abs()) < strength * DIAGONAL_INPUT_MINIMUM {
            return movement;
        }
        if towards_wall.x.abs() > towards_wall.y.abs() {
            Vec2::new(0.0, movement.y.signum() * strength)
        } else {
            Vec2::new(movement.x.signum() * strength, 0.0)
        }
    }

    #[cfg(test)]
    pub fn is_slowed(&self) -> bool {
        self.web_slow_frames > Frames::ZERO
    }

    /// `playslow = frames`: velocity decays by the web's extra factor for that long.
    pub fn slow_down_for(&mut self, frames: Frames) {
        self.web_slow_frames = frames;
    }

    /// Standing on a web restarts a short countdown; while it runs, velocity also decays by the
    /// web's extra per-frame factor.
    fn apply_web_slowdown(&mut self, elapsed: Frames, spider_webs: &[Bounds]) {
        let movement_parameters = self.parameters.movement;
        if spider_webs.iter().any(|web| web.contains(self.position)) {
            self.web_slow_frames = movement_parameters.web_slow_frames;
        }
        if self.web_slow_frames > Frames::ZERO {
            let slowing_frames = elapsed.min(self.web_slow_frames);
            self.velocity *= movement_parameters.web_velocity_retention.over(slowing_frames);
            self.web_slow_frames -= elapsed;
        }
    }
}
