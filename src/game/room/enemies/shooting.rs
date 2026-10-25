//! Shots leaving Pooters, flying, and ending: on the floor (`splatter0`–`2.wav` one time in two),
//! against walls and solid things, or on the player. Each leaves a blood splash where it was drawn,
//! turned at random (`parc()`).

use glam::Vec2;

use crate::game::attacks::splash::{Splash, SplashKind};
use crate::game::odds::Odds;
use crate::game::original_units::{Frames, Hearts, PixelsPerFrame, distance};
use crate::game::player::death::Killer;
use crate::game::random::Random;
use crate::game::room::enemies::fly::FlySurroundings;
use crate::game::room::enemies::shot::{EnemyShot, ShotEnd};
use crate::game::room::enemies::surroundings::Grid;
use crate::game::room::enemies::{Enemies, PLAYER_SIZE};
use crate::game::room::enemies::{WalkerShot, walker::WalkerTurn};
use crate::game::room::stage_to_world;
use crate::game::sound_event::SoundEvent;
use crate::game::texture_id::FlyKind;

/// One time in two a landing shot splatters (one of three sounds).
const SPLATTER_ODDS: Odds = Odds::one_in(2);
const SPLASH_SCALE: f32 = 1.0;

impl Enemies {
    /// `shots()`: the fly at `index` fires at `target` (stage); the two-shot Pooter fires two
    /// shots spread either side.
    pub(super) fn fire(&mut self, index: usize, target: Vec2, sounds: &mut Vec<SoundEvent>) {
        let Some(fly) = self.flies.get(index) else { return };
        let parameters = self.parameters;
        let aim = (target - fly.position).normalize_or_zero() * parameters.shots.speed.get();
        let (forward, side) = (parameters.pooters.spread_forward, parameters.pooters.spread_side);
        let velocities = if fly.kind == FlyKind::SuperPooter {
            vec![aim * forward + aim.perp() * side, aim * forward - aim.perp() * side]
        } else {
            vec![aim]
        };
        let (position, shooter) = (fly.position, Killer::Enemy(fly.kind));
        let frame = self.frames.floor();
        sounds.extend(velocities.iter().map(|_| SoundEvent::BloodShot));
        self.shots.extend(
            velocities
                .into_iter()
                .map(|velocity| EnemyShot::new(position, velocity, shooter, parameters.shots).born_in(frame)),
        );
    }

    /// What the walker at `index` did this frame: growled, or let a shot go.
    pub(super) fn walker_turned(&mut self, index: usize, turn: WalkerTurn, sounds: &mut Vec<SoundEvent>) {
        if turn.growled {
            sounds.push(SoundEvent::GaperGrowled);
        }
        if turn.squished {
            sounds.push(SoundEvent::ClottySquished);
        }
        if turn.whimpered {
            sounds.push(SoundEvent::MulliganWhimpered);
        }
        if turn.landed {
            sounds.push(SoundEvent::HopperLanded);
        }
        let Some(shot) = turn.shot else { return };
        let parameters = self.parameters;
        let position = self.walkers[index].position;
        let frame = self.frames.floor();
        match shot {
            WalkerShot::Aimed(target) => {
                let aim = (target - position).normalize_or_zero() * parameters.shots.speed.get();
                self.shots
                    .push(EnemyShot::new(position, aim, Killer::Horf, parameters.shots).born_in(frame));
                sounds.push(SoundEvent::HorfShot);
            }
            WalkerShot::Cross(speed) => {
                let killer = self.walkers[index].kind.killer();
                self.shoot_cross(position, speed, killer);
                sounds.push(SoundEvent::BloodShot);
            }
            WalkerShot::Spurted(velocity) => {
                self.shots
                    .push(EnemyShot::spurted(position, velocity, Killer::Headless, parameters.shots).born_in(frame));
                sounds.push(SoundEvent::BloodShot);
            }
        }
    }

    /// `quadf()` (DoAction 9180–9197): a shot along each axis.
    pub(super) fn shoot_cross(&mut self, position: Vec2, speed: PixelsPerFrame, killer: Killer) {
        let frame = self.frames.floor();
        let parameters = self.parameters;
        let along_the_axes = [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y];
        self.shots.extend(along_the_axes.map(|direction| {
            EnemyShot::new(position, direction * speed.get(), killer, parameters.shots).born_in(frame)
        }));
    }

    pub(super) fn advance_shots(
        &mut self,
        elapsed: Frames,
        grid: &Grid,
        random: &mut Random,
        sounds: &mut Vec<SoundEvent>,
    ) {
        for splash in &mut self.splashes {
            splash.advance(elapsed);
        }
        self.splashes.retain(|splash| !splash.is_finished());
        let parameters = self.parameters.shots;
        let mut ended = Vec::new();
        self.shots.retain_mut(|shot| {
            shot.advance(elapsed, parameters);
            let Some(end) = shot.end(grid.is_solid(shot.position), parameters) else {
                return true;
            };
            if end == ShotEnd::Floor && random.happens(SPLATTER_ODDS) {
                sounds.push(SoundEvent::ShotSplattered);
            }
            ended.push(shot.drawn_point());
            false
        });
        for point in ended {
            self.splash_at(point, random);
        }
    }

    /// A shot reaching the player at `player` (world) ends there; the damage and its shooter.
    pub(super) fn shot_hitting(&mut self, player: Vec2, random: &mut Random) -> Option<(Hearts, Killer)> {
        let reach = PLAYER_SIZE + self.parameters.shots.size;
        let frame = self.frames.floor();
        let index = self
            .shots
            .iter()
            .position(|shot| !shot.is_new_in(frame) && distance(stage_to_world(shot.position), player) < reach)?;
        let shot = self.shots.swap_remove(index);
        self.splash_at(shot.drawn_point(), random);
        Some((self.parameters.shots.damage, shot.shooter))
    }

    fn splash_at(&mut self, stage_point: Vec2, random: &mut Random) {
        self.splashes.push(Splash::new(
            SplashKind::Blood,
            stage_to_world(stage_point),
            SPLASH_SCALE,
            random,
        ));
    }
}
