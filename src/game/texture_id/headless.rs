//! The headless body's clip `b11` (3483): `d` frame 1 stands (its stump's blood `bb`, 10 frames,
//! shown only while it spurts), 2 walks (3481, 20 frames), 3 dies (566, 36 frames) and 4 appears
//! (3482, 26 frames).

/// Which animation of the headless body's clip a texture shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HeadlessAnimation {
    /// `spurting`: the stump's blood shows, playing its 10 frames.
    Standing {
        spurting: bool,
    },
    Walking,
    Dying,
    /// `spurting`: its blood shows partway through the clip.
    Appearing {
        spurting: bool,
    },
}

impl HeadlessAnimation {
    pub const ALL: [Self; 6] = [
        Self::Standing { spurting: false },
        Self::Standing { spurting: true },
        Self::Walking,
        Self::Dying,
        Self::Appearing { spurting: false },
        Self::Appearing { spurting: true },
    ];

    pub fn frame_count(self) -> u8 {
        match self {
            Self::Standing { spurting: true } => HEADLESS_SPURT_FRAME_COUNT,
            Self::Standing { spurting: false } => 1,
            Self::Walking => HEADLESS_WALKING_FRAME_COUNT,
            Self::Dying => HEADLESS_DYING_FRAME_COUNT,
            Self::Appearing { .. } => HEADLESS_APPEARING_FRAME_COUNT,
        }
    }
}

pub const HEADLESS_SPURT_FRAME_COUNT: u8 = 10;
pub const HEADLESS_WALKING_FRAME_COUNT: u8 = 20;
pub const HEADLESS_DYING_FRAME_COUNT: u8 = 36;
pub const HEADLESS_APPEARING_FRAME_COUNT: u8 = 26;
