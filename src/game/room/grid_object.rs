//! Objects on room grid tiles, following the original's `turd()`, `killshit()`, `mhit()` (walking)
//! and `mhity()` (tears).

mod appearance;
mod level_value;

use glam::Vec2;

use crate::game::bounds::Bounds;
use crate::game::original_units::Frames;
use crate::game::parameters::BreakableParameters;
use crate::game::random::Random;
use crate::game::room::grid_kind::{BreakableKind, GridObjectKind, TileKind};
use crate::game::room::stage_area::stops_flight;
use crate::game::room::{TILE_SIZE, tile_center};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitOutcome {
    Damaged,
    /// A golden poop ignoring the hit (two times in three; `killshit()` returns nothing).
    Resisted,
    Destroyed(BreakableKind),
    /// A rock or web cleared by an explosion (`bloww`).
    Cleared(TileKind),
}

/// `killshit()`'s strength: tears hit weakly unless their damage is above 5 (never for Isaac),
/// explosions strongly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitStrength {
    Tear,
    Explosion,
}

#[derive(Clone, Debug)]
struct BreakableState {
    durability: f32,
    stage: u8,
    frames_since_destroyed: Option<Frames>,
    /// Red poop only: counts down after destruction, then the object grows back.
    frames_until_regrow: Option<Frames>,
    frames_since_regrow: Option<Frames>,
}

#[derive(Clone, Debug)]
pub struct GridObject {
    kind: GridObjectKind,
    center: Vec2,
    parameters: BreakableParameters,
    breakable: Option<BreakableState>,
    /// Key blocks only: time since a key opened it.
    frames_since_opened: Option<Frames>,
    /// Rocks and webs blown away (leaving rubble or torn strands), or a pit bridged by rubble.
    cleared: bool,
    age_frames: Frames,
}

impl GridObject {
    pub fn on_tile(kind: GridObjectKind, column: u32, row: u32, parameters: BreakableParameters) -> Self {
        let breakable = match kind {
            GridObjectKind::Breakable(breakable_kind) => Some(BreakableState {
                durability: initial_durability(breakable_kind, &parameters),
                stage: 0,
                frames_since_destroyed: None,
                frames_until_regrow: None,
                frames_since_regrow: None,
            }),
            _ => None,
        };
        Self {
            kind,
            center: tile_center(column, row),
            parameters,
            breakable,
            frames_since_opened: None,
            cleared: false,
            age_frames: Frames::ZERO,
        }
    }

    pub fn kind(&self) -> GridObjectKind {
        self.kind
    }

    pub fn center(&self) -> Vec2 {
        self.center
    }

    pub fn tile_bounds(&self) -> Bounds {
        Bounds::centered(self.center, Vec2::splat(TILE_SIZE.get()))
    }

    /// Whether a blast cleared it (a rock or web gone, a pit bridged).
    pub fn is_cleared(&self) -> bool {
        self.cleared
    }

    pub fn is_intact_breakable(&self) -> bool {
        self.breakable
            .as_ref()
            .is_some_and(|state| state.durability >= self.parameters.solid_threshold)
    }

    pub fn is_closed_key_block(&self) -> bool {
        self.kind == GridObjectKind::KeyBlock && self.frames_since_opened.is_none()
    }

    /// `mhit`: grid values of 1 and above block walking, including pits.
    pub fn blocks_walking(&self) -> bool {
        match self.kind {
            GridObjectKind::Tile(TileKind::Rock | TileKind::TintedRock | TileKind::Pit) => !self.cleared,
            GridObjectKind::Tile(TileKind::Block) => true,
            GridObjectKind::KeyBlock => self.is_closed_key_block(),
            GridObjectKind::Tile(TileKind::Spikes | TileKind::SpiderWeb) => false,
            GridObjectKind::Breakable(_) => self.is_intact_breakable(),
        }
    }

    /// `mhitx` (DoAction 6015): flying passes over everything below 1.8 and over pits; metal blocks
    /// (1.9) and closed key blocks still stop it. The room's edge (DoAction 6010) is the walls'.
    pub fn blocks_flying(&self) -> bool {
        match self.kind {
            GridObjectKind::Tile(TileKind::Block) => true,
            GridObjectKind::KeyBlock => self.is_closed_key_block(),
            _ => false,
        }
    }

    /// `mhity`: like walking, except tears fly over pits (value 3).
    pub fn blocks_tears(&self) -> bool {
        stops_flight(self.level_value())
    }

    /// A fireplace still burning (`.fire`): it hurts on contact.
    pub fn is_burning_fireplace(&self) -> bool {
        matches!(self.kind, GridObjectKind::Breakable(kind) if kind.is_fireplace()) && self.is_intact_breakable()
    }

    pub fn is_spider_web(&self) -> bool {
        self.kind == GridObjectKind::Tile(TileKind::SpiderWeb) && !self.cleared
    }

    /// `bloww`: a rock blown towards a neighboring pit fills it, leaving a bridge (tile frame 66).
    pub fn bridge_if_pit(&mut self) -> bool {
        let is_open_pit = self.kind == GridObjectKind::Tile(TileKind::Pit) && !self.cleared;
        self.cleared |= is_open_pit;
        is_open_pit
    }

    /// Opens a closed key block (the key is spent by the caller); it stops blocking at once while
    /// its opening animation plays.
    pub fn open_with_key(&mut self) {
        if self.is_closed_key_block() {
            self.frames_since_opened = Some(Frames::ZERO);
        }
    }

    /// `killshit()`: a hit takes a step (the golden poop resists two in three: DoAction 10711; an
    /// extra step: DoAction 10727), a blast more (DoAction 10722, DoAction 10741), a poop one time
    /// in three (DoAction 10732). `None` if this object cannot be damaged or is already broken.
    pub fn take_hit(&mut self, strength: HitStrength, random: &mut Random) -> Option<HitOutcome> {
        let kind = match self.kind {
            GridObjectKind::Breakable(kind) => kind,
            GridObjectKind::Tile(tile_kind)
                if strength == HitStrength::Explosion && TileKind::BLOWABLE.contains(&tile_kind) && !self.cleared =>
            {
                self.cleared = true;
                return Some(HitOutcome::Cleared(tile_kind));
            }
            _ => return None,
        };
        if !self.is_intact_breakable() {
            return None;
        }
        let parameters = self.parameters;
        let state = self.breakable.as_mut()?;
        let take_step = |state: &mut BreakableState| {
            state.durability -= parameters.durability_per_step;
            state.stage = (state.stage + 1).min(kind.stage_count());
        };
        let is_golden = kind == BreakableKind::GoldenPoop;
        let is_explosion = strength == HitStrength::Explosion;
        // Golden poop only takes damage one hit in three, and skips the extra steps marked `!gold`.
        if is_golden && random.next_unit() >= parameters.golden_step_chance {
            return Some(HitOutcome::Resisted);
        }
        take_step(state);
        if is_explosion && state.durability > parameters.solid_threshold && !is_golden {
            take_step(state);
        }
        if (is_explosion || random.next_unit() < parameters.extra_step_chance) && !is_golden {
            take_step(state);
        }
        if is_explosion || (kind == BreakableKind::Poop && random.next_unit() < parameters.extra_step_chance) {
            take_step(state);
        }
        if state.durability > parameters.solid_threshold {
            return Some(HitOutcome::Damaged);
        }
        state.durability = parameters.destroyed_durability;
        state.frames_since_destroyed = Some(Frames::ZERO);
        if kind.regrow_frame_count() > 0 {
            state.frames_until_regrow = Some(parameters.red_poop_regrow_frames);
        }
        Some(HitOutcome::Destroyed(kind))
    }

    pub fn advance(&mut self, elapsed: Frames) {
        self.age_frames += elapsed;
        if let Some(frames) = &mut self.frames_since_opened {
            *frames += elapsed;
        }
        let GridObjectKind::Breakable(kind) = self.kind else {
            return;
        };
        let initial = initial_durability(kind, &self.parameters);
        let Some(state) = self.breakable.as_mut() else { return };
        for frames in [&mut state.frames_since_destroyed, &mut state.frames_since_regrow]
            .into_iter()
            .flatten()
        {
            *frames += elapsed;
        }
        if let Some(remaining) = &mut state.frames_until_regrow {
            *remaining -= elapsed;
            if *remaining <= Frames::ZERO {
                // It plays its grow-back animation and is solid again at once.
                state.durability = initial;
                state.stage = 0;
                state.frames_since_destroyed = None;
                state.frames_until_regrow = None;
                state.frames_since_regrow = Some(Frames::ZERO);
            }
        }
    }
}

fn initial_durability(kind: BreakableKind, parameters: &BreakableParameters) -> f32 {
    match kind {
        BreakableKind::Poop => parameters.poop_durability,
        BreakableKind::RedPoop => parameters.red_poop_durability,
        BreakableKind::CornyPoop => parameters.corny_poop_durability,
        BreakableKind::GoldenPoop => parameters.golden_poop_durability,
        BreakableKind::Fireplace | BreakableKind::RedFireplace => parameters.fireplace_durability,
        BreakableKind::TntBarrel => parameters.tnt_durability,
    }
}

#[cfg(test)]
mod tests;
