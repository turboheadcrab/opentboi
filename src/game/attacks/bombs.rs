//! Bombs the player places (`bomb()`): each burns its fuse, then explodes where it lies.

use crate::game::original_units::FIRST_CLIP_FRAME;
use glam::Vec2;

use crate::game::ball_contact::{Body, Contact, ball_hit};
use crate::game::bounds::Bounds;
use crate::game::draw::{DrawItem, Rgba, actor_depth};
use crate::game::original_units::{Frames, Mass, Pixels, Retention, distance};
use crate::game::parameters::{BombParameters, WallParameters};
use crate::game::texture_id::{BOMB_FUSE_FRAME_COUNT, TextureId};
use crate::game::wall_contact::rebound;

/// A bomb's size: how far it keeps from the walls.
const RADIUS: Pixels = Pixels(16.0);
/// `balljunk()` (DoAction 7523–7535): whatever touches a bomb within 4 frames of its last such touch
/// (or of being placed) passes through, otherwise it kicks the bomb with 0.4 of its own velocity and
/// they bump.
const PASS_THROUGH_WINDOW: Frames = Frames(4.0);
const KICK_SHARE: f32 = 0.4;
/// A tear hits a bomb within their two sizes (8 + 16) and weighs 8.
const TEAR_OVERLAP_REACH: Pixels = Pixels(24.0);
const TEAR_MASS: Mass = Mass(8.0);
/// A bomb keeps 95 % of its speed every frame.
const VELOCITY_RETENTION: Retention = Retention(0.95);
const FALLBACK_COLOR: Rgba = Rgba::opaque(0.2, 0.2, 0.25);

#[derive(Clone, Debug)]
struct Bomb {
    center: Vec2,
    /// Where it was when the current tick began.
    previous_center: Vec2,
    /// Pixels per frame; kicked by the player, slowing each frame.
    velocity: Vec2,
    age_frames: Frames,
    /// Time since it was placed or last passed through.
    since_passed_through: Frames,
}

impl Bomb {
    /// Something moving at `velocity` touches it; `false` if it passes through.
    fn touch(&mut self, velocity: Vec2) -> bool {
        if self.since_passed_through < PASS_THROUGH_WINDOW {
            self.since_passed_through = Frames::ZERO;
            return false;
        }
        self.velocity += velocity * KICK_SHARE;
        true
    }

    fn body(&self, mass: Mass) -> Body {
        Body {
            position: self.center,
            velocity: self.velocity,
            mass,
        }
    }
}

pub struct Bombs {
    parameters: BombParameters,
    walls: WallParameters,
    lit: Vec<Bomb>,
    /// `lastbo`: time since the last bomb was placed; starts ready.
    frames_since_placed: Frames,
}

impl Bombs {
    pub fn new(parameters: BombParameters, walls: WallParameters) -> Self {
        Self {
            parameters,
            walls,
            lit: Vec::new(),
            frames_since_placed: Frames(f32::INFINITY),
        }
    }

    /// Places a bomb at `position` if requested (the button held, with a bomb to spend) and the
    /// cooldown allows; returns whether one was placed.
    pub fn try_place(&mut self, place_requested: bool, position: Vec2) -> bool {
        let cooldown_frames = self.parameters.frames_between_placements;
        if !place_requested || self.frames_since_placed < cooldown_frames {
            return false;
        }
        self.frames_since_placed = Frames::ZERO;
        self.lit.push(Bomb {
            center: position,
            previous_center: position,
            velocity: Vec2::ZERO,
            age_frames: Frames::ZERO,
            since_passed_through: Frames::ZERO,
        });
        true
    }

    /// Burns the fuses, slides kicked bombs within `room_interior`, and returns where bombs
    /// exploded this update.
    pub fn advance(&mut self, elapsed: Frames, room_interior: Bounds) -> Vec<Vec2> {
        self.frames_since_placed += elapsed;
        let exploding_fuse_frame = self.parameters.exploding_fuse_frame;
        let walls = self.walls;
        self.bump_each_other(elapsed);
        let mut exploded = Vec::new();
        self.lit.retain_mut(|bomb| {
            bomb.age_frames += elapsed;
            bomb.since_passed_through += elapsed;
            bomb.velocity *= VELOCITY_RETENTION.over(elapsed);
            let target = bomb.center + bomb.velocity * elapsed;
            bomb.center = room_interior.shrunk_by(RADIUS).clamp(target);
            bomb.velocity = rebound(bomb.velocity, target - bomb.center, walls);
            let burnt_out = FIRST_CLIP_FRAME + bomb.age_frames >= exploding_fuse_frame;
            if burnt_out {
                exploded.push(bomb.center);
            }
            !burnt_out
        });
        exploded
    }

    /// `ballhit()` between the player and lit bombs (at half strength for the overlap); returns
    /// how the player is moved and slowed. A bomb placed under the player lets them walk off it
    /// (see [`PASS_THROUGH_WINDOW`]).
    pub fn bump(&mut self, player: Body, elapsed: Frames) -> Contact {
        let parameters = self.parameters;
        let mut total = Contact::default();
        for bomb in &mut self.lit {
            let reach = parameters.player_overlap_reach;
            if distance(bomb.center, player.position) >= reach || !bomb.touch(player.velocity) {
                continue;
            }
            let body = bomb.body(parameters.mass);
            let Some(contact) = ball_hit(player, body, reach, parameters.separation_scale, elapsed) else {
                continue;
            };
            bomb.center += contact.second_offset;
            bomb.velocity += contact.second_velocity_change;
            total.first_offset += contact.first_offset;
            total.first_velocity_change += contact.first_velocity_change;
        }
        total
    }

    /// A tear at `floor_point` hitting a lit bomb pushes it and splashes (`hurt()` leaves bombs
    /// alone, as they have no hit points); returns whether it hit one.
    pub fn hit_by_tear(&mut self, floor_point: Vec2, velocity: Vec2, elapsed: Frames) -> bool {
        let parameters = self.parameters;
        let tear = Body {
            position: floor_point,
            velocity,
            mass: TEAR_MASS,
        };
        for bomb in &mut self.lit {
            if distance(bomb.center, floor_point) >= TEAR_OVERLAP_REACH || !bomb.touch(velocity) {
                continue;
            }
            let body = bomb.body(parameters.mass);
            if let Some(contact) = ball_hit(tear, body, TEAR_OVERLAP_REACH, parameters.separation_scale, elapsed) {
                bomb.center += contact.second_offset;
                bomb.velocity += contact.second_velocity_change;
            }
            return true;
        }
        false
    }

    pub fn remember_positions(&mut self) {
        for bomb in &mut self.lit {
            bomb.previous_center = bomb.center;
        }
    }

    /// Lit bombs bump each other apart (`ballhit()` between two bombs, at half strength).
    fn bump_each_other(&mut self, elapsed: Frames) {
        let parameters = self.parameters;
        for first in 0..self.lit.len() {
            for second in first + 1..self.lit.len() {
                let body = |bomb: &Bomb| Body {
                    position: bomb.center,
                    velocity: bomb.velocity,
                    mass: parameters.mass,
                };
                let (first_body, second_body) = (body(&self.lit[first]), body(&self.lit[second]));
                let reach = parameters.bomb_overlap_reach;
                if let Some(contact) = ball_hit(first_body, second_body, reach, parameters.separation_scale, elapsed) {
                    self.lit[first].center += contact.first_offset;
                    self.lit[first].velocity += contact.first_velocity_change;
                    self.lit[second].center += contact.second_offset;
                    self.lit[second].velocity += contact.second_velocity_change;
                }
            }
        }
    }

    /// Leaving a room: lit bombs stay behind and are not tracked any more.
    pub fn clear(&mut self) {
        self.lit.clear();
    }

    pub fn append_draw_items(&self, draw_items: &mut Vec<DrawItem>) {
        draw_items.extend(self.lit.iter().map(|bomb| {
            let frame = bomb.age_frames.held_frame(BOMB_FUSE_FRAME_COUNT);
            DrawItem {
                motion: bomb.center - bomb.previous_center,
                ..DrawItem::textured(
                    TextureId::BombFuse(frame),
                    bomb.center,
                    1.0,
                    FALLBACK_COLOR,
                    actor_depth(bomb.center.y),
                )
            }
        }));
    }
}

#[cfg(test)]
mod tests;
