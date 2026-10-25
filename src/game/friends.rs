//! Isaac's blue flies (`friends()`, DoAction 11827–12210; made in `onEnterFrame()`, 29725–29740). One
//! appears above Isaac each frame while some are owed. Each circles him on a ring shared with the
//! others, or, with a one-in-three chance a frame, goes for the enemy hurt last; it hurts every enemy
//! it touches, once in 14 frames, and a hit costs it its life one time in two. Enemy shots they
//! meet are gone too. Up to ten follow him into the next room.

use glam::Vec2;

use crate::game::Game;
use crate::game::bounds::Bounds;
use crate::game::draw::{DrawItem, Rgba, actor_depth};
use crate::game::original_units::{Damage, Frames, Pixels};
use crate::game::parameters::BlueFlyParameters;
use crate::game::random::Random;
use crate::game::room::enemies::Enemies;
use crate::game::sound_event::SoundEvent;
use crate::game::texture_id::{BLUE_FLY_FRAME_COUNT, TextureId};

/// An enemy shot's size (`sizes[9]`), for where it meets a familiar.
const ENEMY_SHOT_SIZE: Pixels = Pixels(5.0);
const FALLBACK_COLOR: Rgba = Rgba::opaque(0.3, 0.4, 0.9);

#[derive(Clone, Copy, Debug, PartialEq)]
enum Aim {
    Player,
    Enemy(u32),
}

#[derive(Clone, Debug)]
struct BlueFly {
    /// World coordinates (y up).
    position: Vec2,
    previous_position: Vec2,
    velocity: Vec2,
    aim: Aim,
    /// Where it is heading (`xpp`, `ypp`).
    goal: Vec2,
    age: Frames,
    /// The enemies it has hurt lately (`hh`), forgotten every 14th frame.
    hurt: Vec<u32>,
}

/// What the flies need of the world each tick.
pub struct Surroundings<'room> {
    pub player: Vec2,
    pub player_busy: bool,
    pub damage: Damage,
    pub interior: Bounds,
    pub enemies: &'room mut Enemies,
}

#[derive(Clone, Debug, Default)]
pub struct BlueFlies {
    /// `bluf`: flies still to appear.
    owed: u32,
    flies: Vec<BlueFly>,
    /// How many there were last frame (`ablub`), which spaces them around the ring.
    crowd: usize,
    /// The enemy hurt last frame (`ablufer`).
    recent: Option<u32>,
    /// Time in the room (`fra`).
    frames: Frames,
}

impl BlueFlies {
    /// `bluf += count`.
    pub fn owe(&mut self, count: u32) {
        self.owed += count;
    }

    #[cfg(test)]
    pub fn count(&self) -> usize {
        self.flies.len()
    }

    /// Leaving the room: up to ten flies wait to follow (DoAction 12831: the smaller of how many there
    /// were and ten).
    pub fn leave_room(&mut self, parameters: &BlueFlyParameters) {
        self.owed = u32::try_from(self.flies.len().min(parameters.carried_between_rooms)).unwrap_or(u32::MAX);
        self.flies.clear();
        self.frames = Frames::ZERO;
        self.crowd = 0;
        self.recent = None;
    }

    pub fn remember_positions(&mut self) {
        for fly in &mut self.flies {
            fly.previous_position = fly.position;
        }
    }

    pub fn advance(
        &mut self,
        elapsed: Frames,
        parameters: &BlueFlyParameters,
        world: &mut Surroundings,
        random: &mut Random,
        sounds: &mut Vec<SoundEvent>,
    ) {
        let first = self.frames.whole() + 1;
        self.frames += elapsed;
        for number in first..=self.frames.whole() {
            self.frame(Frames::from_number(number), parameters, world, random, sounds);
        }
        for fly in &mut self.flies {
            fly.age += elapsed;
        }
    }

    /// One original frame.
    fn frame(
        &mut self,
        frame: Frames,
        parameters: &BlueFlyParameters,
        world: &mut Surroundings,
        random: &mut Random,
        sounds: &mut Vec<SoundEvent>,
    ) {
        if self.owed > 0 && !world.player_busy {
            self.owed -= 1;
            let position = world.player + Vec2::new(0.0, parameters.spawn_lift.get());
            self.flies.push(BlueFly {
                position,
                previous_position: position,
                velocity: Vec2::ZERO,
                aim: Aim::Player,
                goal: position,
                age: Frames::ZERO,
                hurt: Vec::new(),
            });
        }
        let targets = world.enemies.friend_targets();
        let crowd = self.crowd;
        let recent = self.recent;
        let damage = world.damage * parameters.damage_scale + Damage(parameters.damage_bonus);
        let mut survivors = Vec::with_capacity(self.flies.len());
        for (index, mut fly) in std::mem::take(&mut self.flies).into_iter().enumerate() {
            if frame.is_on_cycle(parameters.hit_memory, Frames::ZERO) {
                fly.hurt.clear();
            }
            behaviour::steer(
                &mut fly,
                index,
                crowd,
                frame,
                parameters,
                world.player,
                &targets,
                recent,
                random,
            );
            fly.position = world
                .interior
                .shrunk_by(parameters.size)
                .clamp(fly.position + fly.velocity);
            if !behaviour::touches(&mut fly, damage, parameters, &targets, world, random, sounds) {
                survivors.push(fly);
            }
        }
        self.crowd = survivors.len();
        self.flies = survivors;
        self.recent = world.enemies.take_last_hurt();
    }

    pub fn append_draw_items(&self, draw_items: &mut Vec<DrawItem>) {
        draw_items.extend(self.flies.iter().map(|fly| DrawItem {
            motion: fly.position - fly.previous_position,
            ..DrawItem::textured(
                TextureId::BlueFly(fly.age.looped_frame(BLUE_FLY_FRAME_COUNT)),
                fly.position,
                1.0,
                FALLBACK_COLOR,
                actor_depth(fly.position.y),
            )
        }));
    }
}

impl Game {
    /// The blue flies' frames: they keep to the current room, which is why they are told its enemies.
    pub(crate) fn advance_blue_flies(&mut self, elapsed: Frames) {
        let mut world = Surroundings {
            player: self.player.position(),
            player_busy: self.player.is_animating(),
            damage: self.player_stats().damage,
            interior: crate::game::room::interior_bounds(),
            enemies: &mut self.floor.current_room_mut().enemies,
        };
        self.blue_flies.advance(
            elapsed,
            &self.parameters.blue_flies,
            &mut world,
            &mut self.random,
            &mut self.pending_sound_events,
        );
    }
}

mod behaviour;
#[cfg(test)]
mod tests;
