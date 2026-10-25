//! The flies: the fly (type 13, black) appears, wanders, drifts towards breakables, and dies in a
//! puff; the attack fly (type 18), the red fly (type 61) and the Moter (type 80) go for the player
//! instead; the Pooter (type 14) wanders like the fly and shoots (see `pooter.rs`). Their state is
//! kept in the original's units and stage coordinates (y down), so the rules read as written
//! (`alive()`, DoAction 13907–14118).

use glam::Vec2;

use crate::game::original_units::{Damage, Frames, HitPoints};
use crate::game::parameters::{FlyGoal, FlyKindParameters};
use crate::game::random::Random;
use crate::game::room::enemies::EnemyParameters;
use crate::game::room::enemies::shooter::Shooter;
use crate::game::texture_id::{FlyAnimation, FlyKind};

/// What a fly wants done this frame beyond moving itself.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FlyTurn {
    /// The breakable cell it pecks (`killshit()`, stage coordinates).
    pub peck: Option<Vec2>,
    /// A Pooter's shot leaves, aimed at this point (the player).
    pub shoots_at: Option<Vec2>,
}

/// What the fly sees of the room, in stage coordinates.
pub trait FlySurroundings {
    /// Something solid (pits are not).
    fn is_solid(&self, stage_point: Vec2) -> bool;
    /// The center of the cell at `stage_point` if a breakable stands there.
    fn breakable_cell(&self, stage_point: Vec2) -> Option<Vec2>;
    /// Where the player is (`player.xp`, `player.yp`).
    fn player(&self) -> Vec2;
    /// `linechecky()`: nothing solid on the line between the points.
    fn in_sight(&self, from: Vec2, target: Vec2) -> bool;
}

/// What a fly leaves behind when it dies, for the room to make.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Legacy {
    /// A red fly's shot along each axis (`quadf()`, DoAction 13106–13121).
    Burst,
    /// A Moter's two attack flies (`deathscripts()`, DoAction 13564–13600).
    Split,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FlyState {
    /// `d` frame 4: the appearing clip.
    Appearing,
    Flying,
    /// `dones`: `d` frame 3 plays this far.
    Dying(Frames),
}

#[derive(Clone, Debug)]
pub struct Fly {
    pub kind: FlyKind,
    /// `xp`, `yp`.
    pub position: Vec2,
    pub previous_position: Vec2,
    /// Pixels per frame.
    pub velocity: Vec2,
    /// `xbb`, `ybb`: the wandering drift.
    pub(super) drift: Vec2,
    hit_points: HitPoints,
    pub state: FlyState,
    /// Which wing frame it starts on, at random.
    pub wing_phase: u8,
    /// `goshit`: the breakable cell it heads for; the attack fly's is the player (`goshit = 1`).
    goal: Option<Goal>,
    /// Frames since it appeared.
    pub age: Frames,
    /// A Pooter's shooting.
    pub shooter: Shooter,
    /// Time left flashing red from a hit.
    flash_left: Frames,
    /// The frame the floor last hurt it (`lastfir`).
    pub(super) last_burned: Option<Frames>,
    /// What its death still owes the room.
    owed: Option<Legacy>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Goal {
    Cell(Vec2),
    Player,
}

impl Fly {
    /// `phase` is its place among the room's enemies (`e`).
    pub fn new(kind: FlyKind, position: Vec2, phase: u32, parameters: &EnemyParameters, random: &mut Random) -> Self {
        let mut hit_points = kind_parameters(kind, parameters).hit_points;
        if kind == FlyKind::SuperPooter {
            hit_points = hit_points * parameters.pooters.variant_hit_point_factor;
        }
        Self {
            kind,
            position,
            previous_position: position,
            velocity: Vec2::ZERO,
            drift: Vec2::ZERO,
            hit_points,
            state: FlyState::Appearing,
            wing_phase: random.below_u8(2),
            goal: None,
            age: Frames::ZERO,
            shooter: Shooter::with_phase(phase),
            flash_left: Frames::ZERO,
            last_burned: None,
            owed: None,
        }
    }

    /// What its death owes the room; asking clears it.
    pub(super) fn take_legacy(&mut self) -> Option<Legacy> {
        self.owed.take()
    }

    /// A landed peck turns it into a Pooter (the two-shot one one time in three), flying at once
    /// and keeping its hit points.
    pub fn become_pooter(&mut self, parameters: &EnemyParameters, random: &mut Random) {
        self.kind = if random.happens(parameters.pooters.transformed_variant_odds) {
            FlyKind::SuperPooter
        } else {
            FlyKind::Pooter
        };
        self.state = FlyState::Flying;
        self.age = kind_parameters(self.kind, parameters).appearing_frames;
        self.shooter = Shooter::with_phase(self.shooter.phase);
    }

    pub fn hit_points(&self) -> HitPoints {
        self.hit_points
    }

    pub fn is_alive(&self) -> bool {
        !matches!(self.state, FlyState::Dying(_))
    }

    /// `hurt()`; `true` if it dies (below zero hit points).
    pub fn hurt(&mut self, damage: Damage, parameters: &EnemyParameters) -> bool {
        if !self.is_alive() {
            return false;
        }
        self.hit_points -= damage;
        if self.hit_points < HitPoints::ZERO {
            self.state = FlyState::Dying(Frames::ZERO);
            self.velocity = Vec2::ZERO;
            self.owed = match self.kind {
                FlyKind::Red => Some(Legacy::Burst),
                FlyKind::Moter if self.hit_points > parameters.flies.moter_split_floor => Some(Legacy::Split),
                _ => None,
            };
            return true;
        }
        self.flash_left = parameters.flies.flash_after_hit(damage);
        false
    }

    /// Flashing red from a hit (`hurtcol`).
    pub fn is_flashing(&self) -> bool {
        self.flash_left > Frames::ZERO && self.is_alive()
    }

    /// One original frame of its AI, `frame` being the room's `fra`.
    pub fn think(
        &mut self,
        frame: Frames,
        surroundings: &impl FlySurroundings,
        parameters: &EnemyParameters,
        random: &mut Random,
    ) -> FlyTurn {
        let mut turn = FlyTurn::default();
        match self.state {
            FlyState::Appearing => {
                self.velocity *= parameters.flies.appearing_retention.0;
                if self.age >= kind_parameters(self.kind, parameters).appearing_frames {
                    self.state = FlyState::Flying;
                }
            }
            FlyState::Dying(_) => {}
            FlyState::Flying => {
                if self.kind.is_pooter() {
                    turn.shoots_at = self.pooter_turn(frame, surroundings, parameters.pooters);
                }
                // A goal every third frame (DoAction 13977), the player for the chasers (DoAction
                // 13993).
                if frame.is_on_cycle(parameters.flies.goal_every, Frames::ZERO) {
                    match kind_parameters(self.kind, parameters).goal {
                        FlyGoal::Player(_) => self.goal = Some(Goal::Player),
                        FlyGoal::Breakables => self.look_for_goal(surroundings, parameters, random),
                    }
                }
                if frame.is_on_cycle(parameters.flies.approach_every, Frames::ZERO) {
                    turn.peck = self.approach_goal(surroundings.player(), parameters, random);
                }
                if frame.is_on_cycle(parameters.flies.jitter_every, parameters.flies.jitter_phase) {
                    self.jitter(surroundings, parameters, random);
                }
                if frame.is_on_cycle(parameters.flies.drift_every, parameters.flies.drift_phase) {
                    self.drift_within_room(parameters);
                }
            }
        }
        turn
    }

    /// Whether its death has played.
    pub fn death_finished(&self, parameters: &EnemyParameters) -> bool {
        matches!(self.state, FlyState::Dying(frames) if frames >= kind_parameters(self.kind, parameters).dying_frames)
    }

    /// Moves and ages.
    pub fn advance(&mut self, elapsed: Frames) {
        self.flash_left = (self.flash_left - elapsed).max(Frames::ZERO);
        self.previous_position = self.position;
        self.position += self.velocity * elapsed.get();
        self.age += elapsed;
        if let Some(age) = &mut self.shooter.animation_age {
            *age += elapsed;
            if *age >= Frames::from(self.kind.frame_count(FlyAnimation::Shooting)) {
                self.shooter.animation_age = None;
            }
        }
        if let FlyState::Dying(frames) = &mut self.state {
            *frames += elapsed;
        }
    }
}

pub(super) fn kind_parameters(kind: FlyKind, parameters: &EnemyParameters) -> FlyKindParameters {
    match kind {
        FlyKind::Black => parameters.flies.black,
        FlyKind::Attack => parameters.flies.attack,
        FlyKind::Pooter | FlyKind::SuperPooter => parameters.flies.pooter,
        FlyKind::Red => parameters.flies.red,
        FlyKind::Moter => parameters.flies.moter,
    }
}

mod steering;
