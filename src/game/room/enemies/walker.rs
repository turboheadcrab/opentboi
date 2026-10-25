//! The enemies that stand or walk on the floor: the Gaper (type 10) appears, then walks after the
//! player around rocks and pits (`walker/walking.rs`); the headless body (type 11) is what half
//! of the Gapers become when killed, wandering and, one time in three, spurting blood balls
//! (`walker/headless.rs`); Horf (type 12) never moves of its own accord and shoots (`walker/horf.rs`);
//! the Clotty (type 15) wanders and shoots four ways (`walker/clotty.rs`); the Mulligan (type 16)
//! runs from the player, bursts when cornered and leaves flies (`walker/mulligan.rs`).
//! All hurt by half a heart on touch. Their state is kept in the original's units and stage
//! coordinates (y down), as the flies' is.

mod clotty;
mod flesh_mound;
mod headless;
mod hopper;
mod horf;
mod hurt;
mod kind;
mod movement;
mod mulligan;
mod outcomes;
mod walking;

pub use kind::WalkerKind;
use outcomes::{Blob, ClottyMode};
pub use outcomes::{Hurt, WalkerClip, WalkerShot, WalkerState, WalkerTurn};

use glam::Vec2;

use crate::game::original_units::{Frames, HitPoints};
use crate::game::random::Random;
use crate::game::room::enemies::EnemyParameters;
use crate::game::room::enemies::shooter::Shooter;
use crate::game::room::enemies::walk_map::Distances;

/// What it knows of the way to the player.
#[derive(Clone, Debug, Default)]
struct Route {
    /// `lastv`: the last look for a clear line to the player.
    line_clear: bool,
    /// `gridder`: steps to the player's tile from the last flood fill, and how long until the next
    /// (`gridtime`).
    distances: Option<Distances>,
    refill_timer: Frames,
    /// `firss`: the frame of its last growl.
    last_growl: Option<Frames>,
}

#[derive(Clone, Debug)]
pub struct Walker {
    pub kind: WalkerKind,
    /// `xp`, `yp`.
    pub position: Vec2,
    pub previous_position: Vec2,
    /// Pixels per frame.
    pub velocity: Vec2,
    hit_points: HitPoints,
    pub state: WalkerState,
    pub clip: WalkerClip,
    /// A Gaper's head clip `h` (from frame 0), which stays on its frame between walking and standing
    /// and only moves on at the sight of the player.
    pub head: u8,
    /// A plain marker's Gaper is frantic (`gonuts`): faster, its head shouting at the sight of the
    /// player; a marker with a fractional value makes a calm one.
    pub frantic: bool,
    /// `bnuts`: the body spurts blood (a Gaper's and a headless body's own, one time in three).
    pub spurts: bool,
    /// Mirrored (`sideflip()`: `_xscale` below zero).
    pub mirrored: bool,
    /// Drawn at this scale (`_xscale`): a headless body keeps its Gaper's.
    pub scale: f32,
    /// Frames since it appeared.
    pub age: Frames,
    /// Frames since its walking or standing clip started (the headless body's).
    pub clip_age: Frames,
    /// `e`: its place among the room's enemies.
    phase: u32,
    route: Route,
    /// The headless body's wandering: where it is heading (`xpp`, `ypp`), and `fire`, the frames until
    /// it spurts again.
    goal: Option<Vec2>,
    spurt_cooldown: Frames,
    /// Horf's shooting.
    pub shooter: Shooter,
    /// The Clotty's and the Mulligan's own state.
    blob: Blob,
    /// The Hopper's leaping.
    hop: hopper::Hop,
    /// Time left flashing red from a hit.
    flash_left: Frames,
    /// The frame the floor last hurt it (`lastfir`).
    pub(super) last_burned: Option<Frames>,
}

impl Walker {
    fn new(kind: WalkerKind, position: Vec2, phase: u32, parameters: &EnemyParameters) -> Self {
        let body = kind.body(parameters);
        Self {
            kind,
            position,
            previous_position: position,
            velocity: Vec2::ZERO,
            hit_points: body.hit_points,
            state: WalkerState::Appearing,
            clip: WalkerClip::Standing,
            head: 0,
            frantic: false,
            spurts: false,
            mirrored: false,
            scale: body.scale,
            age: Frames::ZERO,
            clip_age: Frames::ZERO,
            phase,
            route: Route::default(),
            goal: None,
            spurt_cooldown: Frames::ZERO,
            shooter: Shooter::with_phase(phase),
            blob: Blob::default(),
            hop: hopper::Hop::default(),
            flash_left: Frames::ZERO,
            last_burned: None,
        }
    }

    /// A Gaper, which spurts one time in three; `frantic` for a plain marker's.
    pub fn gaper(position: Vec2, frantic: bool, phase: u32, parameters: &EnemyParameters, random: &mut Random) -> Self {
        Self {
            frantic,
            // A frantic head is on its second frame from the start, not only once it is pulled.
            head: u8::from(frantic),
            spurts: random.happens(parameters.headless.spurting_odds),
            ..Self::new(WalkerKind::Gaper, position, phase, parameters)
        }
    }

    /// A layout's own headless body; `spurts` unless its marker has a fractional value.
    pub fn headless(position: Vec2, spurts: bool, phase: u32, parameters: &EnemyParameters) -> Self {
        Self {
            spurts,
            ..Self::new(WalkerKind::Headless, position, phase, parameters)
        }
    }

    pub fn horf(position: Vec2, phase: u32, parameters: &EnemyParameters) -> Self {
        Self::new(WalkerKind::Horf, position, phase, parameters)
    }

    pub fn clotty(position: Vec2, phase: u32, parameters: &EnemyParameters) -> Self {
        Self::new(WalkerKind::Clotty, position, phase, parameters)
    }

    pub fn mulligan(position: Vec2, phase: u32, parameters: &EnemyParameters) -> Self {
        Self::new(WalkerKind::Mulligan, position, phase, parameters)
    }

    pub fn hopper(position: Vec2, phase: u32, parameters: &EnemyParameters) -> Self {
        Self::new(WalkerKind::Hopper, position, phase, parameters)
    }

    pub fn flesh_mound(position: Vec2, phase: u32, parameters: &EnemyParameters) -> Self {
        Self::new(WalkerKind::FleshMound, position, phase, parameters)
    }

    /// The hop clip's frame while it plays (the Hopper's).
    pub fn hop_frame(&self) -> Option<u8> {
        self.hop.playing_frame()
    }

    /// Whether it still owes the room the flies it leaves behind; asking clears it.
    pub(super) fn take_owed_flies(&mut self) -> bool {
        std::mem::take(&mut self.blob.owes_flies)
    }

    /// Whether it is scared (a Mulligan running from the player).
    pub fn is_scared(&self) -> bool {
        self.blob.fear_looks > 0
    }

    pub fn hit_points(&self) -> HitPoints {
        self.hit_points
    }

    /// `e`: its place among the room's enemies.
    pub fn phase(&self) -> u32 {
        self.phase
    }

    pub fn is_alive(&self) -> bool {
        !matches!(self.state, WalkerState::Dying(_))
    }

    /// One original frame of its AI, `frame` being the room's `fra`; `enemy_count` the enemies in
    /// the room.
    pub(super) fn think(
        &mut self,
        frame: Frames,
        grid: &crate::game::room::enemies::surroundings::Grid,
        parameters: &EnemyParameters,
        enemy_count: usize,
        random: &mut Random,
    ) -> WalkerTurn {
        match self.state {
            WalkerState::Appearing => {
                self.velocity *= self.kind.body(parameters).appearing_retention.0;
                if self.age >= self.kind.body(parameters).appearing_frames {
                    self.state = WalkerState::Active;
                }
                WalkerTurn::default()
            }
            WalkerState::Dying(_) => WalkerTurn::default(),
            WalkerState::Active => match self.kind {
                WalkerKind::Gaper => WalkerTurn {
                    growled: self.walk(frame, grid, parameters, enemy_count, random),
                    ..WalkerTurn::default()
                },
                WalkerKind::Headless => self.wander(grid, parameters, random),
                WalkerKind::Horf => self.shoot(frame, grid, parameters),
                WalkerKind::Clotty => self.clot(frame, grid, parameters, random),
                WalkerKind::Mulligan => self.run_away(frame, grid, parameters, random),
                WalkerKind::Hopper => self.hop(grid, parameters, random),
                WalkerKind::FleshMound => self.crawl(grid, parameters, random),
            },
        }
    }

    /// Whether its death has played.
    pub fn death_finished(&self, parameters: &EnemyParameters) -> bool {
        matches!(self.state, WalkerState::Dying(frames) if frames >= self.kind.body(parameters).dying_frames)
    }
}

#[cfg(test)]
mod clotty_tests;
#[cfg(test)]
mod conversion_tests;
#[cfg(test)]
mod floor_tests;
#[cfg(test)]
mod headless_tests;
#[cfg(test)]
mod hopper_tests;
#[cfg(test)]
mod horf_tests;
#[cfg(test)]
mod mulligan_tests;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
