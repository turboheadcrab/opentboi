pub mod active_item;
mod active_use;
pub mod animation;
pub mod boosts;
mod card_effects;
#[cfg(test)]
mod card_tests;
pub mod cards;
#[cfg(test)]
mod creep_tests;
pub mod death;
mod gambling;
pub mod hazards;
pub mod inventory;
#[cfg(test)]
mod item_tests;
pub mod items;
pub mod look;
mod movement;
mod pill_effects;
pub mod pill_state;
#[cfg(test)]
mod pill_tests;
mod red_chest_opening;
mod shooting_axes;
#[cfg(test)]
mod trinket_tests;
mod tuning;
pub mod unicorn_ride;
use glam::Vec2;

use crate::game::attacks::tear::Tear;
use crate::game::bounds::Bounds;
use crate::game::draw::DrawItem;
use crate::game::input_state::{InputState, snap_to_cardinal};
use crate::game::interpolation::set_motion;
use crate::game::original_units::{Frames, Pixels, Retention};
use crate::game::parameters::GameParameters;
use crate::game::player::animation::{PlayerAnimation, PlayingAnimation};
use crate::game::player::boosts::{PlayerStats, RoomBoosts};
use crate::game::player::look::{Outfit, PlayerLook};
use crate::game::player::pill_state::PillState;
use crate::game::random::Random;
use shooting_axes::ShootingAxes;

pub(super) const COLLISION_RADIUS: Pixels = Pixels(12.0);
/// sin(22.5°): input counts as diagonal when both directions are at least this share of it.
pub(super) const DIAGONAL_INPUT_MINIMUM: f32 = 0.382_683_43;

/// What the player moves among this frame.
pub struct Surroundings<'frame> {
    pub room_interior: Bounds,
    pub solid_tiles: &'frame [Bounds],
    pub spider_webs: &'frame [Bounds],
}

#[derive(Clone, Debug)]
pub struct Player {
    parameters: GameParameters,
    position: Vec2,
    /// Where it was when the current tick began.
    previous_position: Vec2,
    velocity: Vec2,
    fire_cooldown_frames: Frames,
    /// Alternates between +1 and -1 so consecutive tears leave from either side.
    next_tear_side: f32,
    look: PlayerLook,
    /// `playslow`: time left in which a spider web keeps slowing the player.
    web_slow_frames: Frames,
    /// A whole-body animation playing instead of the body and head.
    animation: Option<PlayingAnimation>,
    /// Speed, damage, range, size and flight with this room's card boosts.
    stats: PlayerStats,
    /// When each shooting axis was last let go, for which wins when both are held.
    shooting_axes: ShootingAxes,
    /// The wall last pressed into (direction towards it) and how long diagonal input still
    /// slides along it.
    slide: Option<(Vec2, Frames)>,
}

impl Player {
    pub fn new(position: Vec2, parameters: &GameParameters) -> Self {
        Self {
            parameters: *parameters,
            position,
            previous_position: position,
            velocity: Vec2::ZERO,
            fire_cooldown_frames: Frames::ZERO,
            next_tear_side: 1.0,
            look: PlayerLook::new(parameters.movement, parameters.firing),
            stats: RoomBoosts::default().stats(parameters, &PillState::default()),
            animation: None,
            web_slow_frames: Frames::ZERO,
            slide: None,
            shooting_axes: ShootingAxes::default(),
        }
    }

    /// Moves within `room_interior`; returns a new tear if one was fired this frame.
    pub fn update(
        &mut self,
        input: &InputState,
        elapsed: Frames,
        surroundings: &Surroundings,
        random: &mut Random,
    ) -> Option<Tear> {
        self.move_within(input.movement, elapsed, surroundings);
        let mouse_aim_origin = self.position + Vec2::new(0.0, self.parameters.firing.mouse_aim_height.get());
        let shoot_direction = match input.aim_target {
            Some(aim_target) => snap_to_cardinal(aim_target - mouse_aim_origin),
            None => self.shooting_axes.direction(input.shoot_axes, elapsed),
        };
        if let Some(animation) = &mut self.animation
            && !animation.advance(elapsed)
        {
            self.animation = None;
        }
        // On the unicorn the head turns where Isaac aims, but no tear comes.
        let (fired_tear, fired_direction) = if self.stats.riding_unicorn {
            (None, shoot_direction)
        } else {
            let tear = self.try_firing(shoot_direction, elapsed, random);
            let direction = tear.as_ref().and(shoot_direction);
            (tear, direction)
        };
        self.look.update(self.velocity, fired_direction, elapsed);
        fired_tear
    }

    /// Dying: Isaac slides to a stop and the death clip plays.
    pub fn update_dying(&mut self, retention: Retention, elapsed: Frames, surroundings: &Surroundings) {
        self.coast(retention, elapsed, surroundings);
        if let Some(animation) = &mut self.animation {
            animation.advance(elapsed);
        }
    }

    fn try_firing(&mut self, shoot_direction: Option<Vec2>, elapsed: Frames, random: &mut Random) -> Option<Tear> {
        self.fire_cooldown_frames = (self.fire_cooldown_frames - elapsed).max(Frames::ZERO);
        let shoot_direction = shoot_direction?;
        if self.fire_cooldown_frames > Frames::ZERO {
            return None;
        }
        self.fire_cooldown_frames = self.firing().frames_between_shots();
        let side = self.next_tear_side;
        self.next_tear_side = -side;
        Some(
            Tear::fire(
                self.position,
                shoot_direction,
                self.velocity,
                side,
                self.boosted_tears(),
                random,
            )
            .with_dark_look(self.stats.dark_tears),
        )
    }

    /// Puts the player somewhere new (entering a room), keeping the current velocity.
    pub fn place_at(&mut self, position: Vec2) {
        self.position = position;
        self.previous_position = position;
    }

    /// Pushed by something it bumped into, staying out of the walls and solid tiles.
    pub fn shove(&mut self, offset: Vec2, room_interior: Bounds, solid_tiles: &[Bounds]) {
        let mut position = room_interior.shrunk_by(COLLISION_RADIUS).clamp(self.position + offset);
        for tile in solid_tiles {
            position = tile.push_circle_out(position, COLLISION_RADIUS);
        }
        self.position = position;
    }

    /// Nothing replaces dying.
    pub fn play_animation(&mut self, animation: PlayerAnimation) {
        if self
            .animation
            .is_some_and(|playing| playing.animation == PlayerAnimation::Dying)
        {
            return;
        }
        self.animation = Some(PlayingAnimation::start(animation));
    }

    #[cfg(test)]
    pub fn is_playing(&self, animation: PlayerAnimation) -> bool {
        self.animation.is_some_and(|playing| playing.animation == animation)
    }

    /// Drawn as a whole-body animation rather than as himself.
    pub fn is_animating(&self) -> bool {
        self.animation.is_some()
    }

    pub fn remember_position(&mut self) {
        self.previous_position = self.position;
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn velocity(&self) -> Vec2 {
        self.velocity
    }

    /// Where he is and how fast he moves, for the enemies to take aim.
    pub fn sighting(&self) -> crate::game::room::PlayerSighting {
        crate::game::room::PlayerSighting {
            position: self.position,
            velocity: self.velocity,
        }
    }

    /// Adds to the velocity, keeping what it had.
    pub fn accelerate(&mut self, change: Vec2) {
        self.velocity += change;
    }

    /// A blast's shove: keeps `retention` of the velocity and adds `push`.
    pub fn knock_back(&mut self, retention: Retention, push: Vec2) {
        self.velocity = self.velocity * retention.0 + push;
    }

    pub fn append_draw_items(&self, draw_items: &mut Vec<DrawItem>) {
        let first_item = draw_items.len();
        let stats = self.stats;
        if let Some(animation) = self.animation {
            draw_items.push(animation.draw_item(self.position, stats.scale));
        } else {
            let outfit = Outfit {
                flying: stats.flying,
                riding_unicorn: stats.riding_unicorn,
                tint: stats.ride_tint,
                demonic: stats.demonic,
            };
            self.look
                .append_draw_items(self.position, stats.scale, outfit, draw_items);
        }
        set_motion(&mut draw_items[first_item..], self.position - self.previous_position);
    }
}

#[cfg(test)]
mod tests;
