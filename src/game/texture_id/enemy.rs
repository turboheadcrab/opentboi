//! The Hopper (type 29, clip `b29` 4935) and the flesh mound (type 77, `b74` 1906).

use glam::Vec2;

/// The Hopper: `d` frame 1 sits (342 and its `hx`), 2 hops (4933, 26 frames), 3 dies (566, 36),
/// 4 appears (4934, 26).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HopperAnimation {
    Idle,
    Hopping,
    Dying,
    Appearing,
}

impl HopperAnimation {
    pub const ALL: [Self; 4] = [Self::Idle, Self::Hopping, Self::Dying, Self::Appearing];

    pub fn frame_count(self) -> u8 {
        match self {
            Self::Idle => 1,
            Self::Hopping => HOPPER_HOPPING_FRAME_COUNT,
            Self::Dying => HOPPER_DYING_FRAME_COUNT,
            Self::Appearing => HOPPER_APPEARING_FRAME_COUNT,
        }
    }
}

pub const HOPPER_HOPPING_FRAME_COUNT: u8 = 26;
pub const HOPPER_DYING_FRAME_COUNT: u8 = 36;
pub const HOPPER_APPEARING_FRAME_COUNT: u8 = 26;

/// The flesh mound: `d` frame 13 walks (1902, 24 frames), 14 dies (566, 36), 15 appears (1903, 25).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FleshMoundAnimation {
    Walking,
    Dying,
    Appearing,
}

impl FleshMoundAnimation {
    pub const ALL: [Self; 3] = [Self::Walking, Self::Dying, Self::Appearing];

    pub fn frame_count(self) -> u8 {
        match self {
            Self::Walking => FLESH_MOUND_WALKING_FRAME_COUNT,
            Self::Dying => FLESH_MOUND_DYING_FRAME_COUNT,
            Self::Appearing => FLESH_MOUND_APPEARING_FRAME_COUNT,
        }
    }
}

pub const FLESH_MOUND_WALKING_FRAME_COUNT: u8 = 24;
pub const FLESH_MOUND_DYING_FRAME_COUNT: u8 = 36;
pub const FLESH_MOUND_APPEARING_FRAME_COUNT: u8 = 25;

/// A ground enemy beyond the first five: `frame` of its `animation`, flashing red from a hit
/// (`hurtcol`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EnemyTexture {
    Hopper {
        animation: HopperAnimation,
        frame: u8,
        flashing: bool,
    },
    FleshMound {
        animation: FleshMoundAnimation,
        frame: u8,
        flashing: bool,
    },
}

/// Both rise out of the floor while appearing; their deaths burst as the Gapers' do.
const HOPPER_SIZE: Vec2 = Vec2::new(120.0, 128.0);
const HOPPER_APPEARING_SIZE: Vec2 = Vec2::new(84.0, 176.0);
const FLESH_MOUND_SIZE: Vec2 = Vec2::new(120.0, 128.0);
const FLESH_MOUND_APPEARING_SIZE: Vec2 = Vec2::new(84.0, 176.0);
const DYING_SIZE: Vec2 = Vec2::new(236.0, 276.0);

impl EnemyTexture {
    /// Every frame of both, the living also flashing.
    pub fn all() -> impl Iterator<Item = Self> {
        let flashes = |dying: bool| -> &'static [bool] { if dying { &[false] } else { &[false, true] } };
        let hoppers = HopperAnimation::ALL.into_iter().flat_map(move |animation| {
            (0..animation.frame_count()).flat_map(move |frame| {
                flashes(animation == HopperAnimation::Dying)
                    .iter()
                    .map(move |&flashing| Self::Hopper {
                        animation,
                        frame,
                        flashing,
                    })
            })
        });
        let mounds = FleshMoundAnimation::ALL.into_iter().flat_map(move |animation| {
            (0..animation.frame_count()).flat_map(move |frame| {
                flashes(animation == FleshMoundAnimation::Dying)
                    .iter()
                    .map(move |&flashing| Self::FleshMound {
                        animation,
                        frame,
                        flashing,
                    })
            })
        });
        hoppers.chain(mounds)
    }

    pub fn world_size(self) -> Vec2 {
        match self {
            Self::Hopper { animation, .. } => match animation {
                HopperAnimation::Dying => DYING_SIZE,
                HopperAnimation::Appearing => HOPPER_APPEARING_SIZE,
                _ => HOPPER_SIZE,
            },
            Self::FleshMound { animation, .. } => match animation {
                FleshMoundAnimation::Dying => DYING_SIZE,
                FleshMoundAnimation::Appearing => FLESH_MOUND_APPEARING_SIZE,
                FleshMoundAnimation::Walking => FLESH_MOUND_SIZE,
            },
        }
    }
}
