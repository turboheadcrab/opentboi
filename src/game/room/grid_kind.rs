//! The kinds of objects that occupy room grid tiles, and the art facts the game needs about them
//! (frame counts and drawn sizes). Collision and damage rules live in `grid_object.rs`.

use glam::Vec2;

/// Static tiles drawn from the original's tile sheet (`tiles`, sprite 6011).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TileKind {
    /// Level value 1: solid, only bombs break it.
    Rock,
    /// The level's one marked rock (`rarer`, also level value 1): breaks like a rock; in the
    /// original it also gives a chest.
    TintedRock,
    /// Level value 1.9 (stored as 1.8): a dark rock that is solid and indestructible.
    Block,
    /// Level value 3: blocks walking; tears fly over it.
    Pit,
    /// Level value 1.93 (stored as 0.99): walkable; hurts on contact once health exists.
    Spikes,
    /// Level value 1.94 (stored as 0.99): walkable; slows tears passing through.
    SpiderWeb,
}

impl TileKind {
    pub const ALL: [Self; 6] = [
        Self::Rock,
        Self::TintedRock,
        Self::Block,
        Self::Pit,
        Self::Spikes,
        Self::SpiderWeb,
    ];
    /// Tiles an explosion clears (`bloww`: grid value 1 or a web).
    pub const BLOWABLE: [Self; 3] = [Self::Rock, Self::TintedRock, Self::SpiderWeb];
    /// Tiles that can end up cleared: blown away, or a pit bridged by a blown rock.
    pub const CLEARABLE: [Self; 4] = [Self::Rock, Self::TintedRock, Self::SpiderWeb, Self::Pit];

    pub fn is_rock(self) -> bool {
        matches!(self, Self::Rock | Self::TintedRock)
    }
}

/// Objects tears can break (`killshit()` accepts grid values strictly between 1 and 1.8).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BreakableKind {
    /// `breakblock`.
    Poop,
    /// `breakblock2`.
    RedPoop,
    /// `breakblock3`; spawns flies when broken, once enemies exist.
    CornyPoop,
    /// `breakblock4` with `gold = true`.
    GoldenPoop,
    /// `fireblock`.
    Fireplace,
    /// `fireblock2`.
    RedFireplace,
    /// `egg`, drawn as a TNT barrel; explodes when broken, once explosions exist.
    TntBarrel,
}

/// Fireplace flames loop over the fire clip (4 frames) and the glow (14 frames): 28 frames.
const FIREPLACE_IDLE_FRAME_COUNT: u8 = 28;
/// Golden poop's sparkle clip (5237) has 31 frames.
const GOLDEN_POOP_IDLE_FRAME_COUNT: u8 = 31;
const POOP_BURST_FRAME_COUNT: u8 = 23;
/// Red poop's grow-back animation (frame 6, clip 5835) has 19 frames.
const RED_POOP_REGROW_FRAME_COUNT: u8 = 19;
const FIREPLACE_SMOKE_FRAME_COUNT: u8 = 41;
const POOP_SIZE: Vec2 = Vec2::new(56.0, 56.0);
const FIREPLACE_SIZE: Vec2 = Vec2::new(80.0, 112.0);
const TNT_SIZE: Vec2 = Vec2::new(56.0, 64.0);
/// Measured with the `texture_extents` test: splatter reaches 169×115, smoke 63×191.
const POOP_BURST_SIZE: Vec2 = Vec2::new(176.0, 120.0);
const FIREPLACE_SMOKE_SIZE: Vec2 = Vec2::new(64.0, 192.0);

impl BreakableKind {
    pub const ALL: [Self; 7] = [
        Self::Poop,
        Self::RedPoop,
        Self::CornyPoop,
        Self::GoldenPoop,
        Self::Fireplace,
        Self::RedFireplace,
        Self::TntBarrel,
    ];

    pub fn is_fireplace(self) -> bool {
        matches!(self, Self::Fireplace | Self::RedFireplace)
    }

    /// Intact and damaged looks (clip frames 1 to N); the TNT barrel has three.
    pub fn stage_count(self) -> u8 {
        if self == Self::TntBarrel { 3 } else { 4 }
    }

    /// Frames of the looping idle animation for a damage stage; 1 means a still image. The fire
    /// (sprite 460) stays on every fireplace stage, shrinking as it is hit; only the intact golden
    /// poop sparkles.
    pub fn idle_frame_count(self, stage: u8) -> u8 {
        match (self, stage) {
            (Self::Fireplace | Self::RedFireplace, _) => FIREPLACE_IDLE_FRAME_COUNT,
            (Self::GoldenPoop, 0) => GOLDEN_POOP_IDLE_FRAME_COUNT,
            _ => 1,
        }
    }

    /// The broken animation; it holds on its last frame (stain, smoke-out or wreck).
    pub fn destroyed_frame_count(self) -> u8 {
        match self {
            Self::TntBarrel => 1,
            kind if kind.is_fireplace() => FIREPLACE_SMOKE_FRAME_COUNT,
            _ => POOP_BURST_FRAME_COUNT,
        }
    }

    /// Frames of the grow-back animation; 0 for objects that stay broken.
    pub fn regrow_frame_count(self) -> u8 {
        if self == Self::RedPoop {
            RED_POOP_REGROW_FRAME_COUNT
        } else {
            0
        }
    }

    pub fn stage_size(self) -> Vec2 {
        match self {
            Self::TntBarrel => TNT_SIZE,
            kind if kind.is_fireplace() => FIREPLACE_SIZE,
            _ => POOP_SIZE,
        }
    }

    pub fn destroyed_size(self) -> Vec2 {
        match self {
            Self::TntBarrel => TNT_SIZE,
            kind if kind.is_fireplace() => FIREPLACE_SMOKE_SIZE,
            _ => POOP_BURST_SIZE,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GridObjectKind {
    Tile(TileKind),
    Breakable(BreakableKind),
    /// `locktile`: solid until opened with a key.
    KeyBlock,
}
