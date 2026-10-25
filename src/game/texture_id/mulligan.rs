//! The Mulligan's clip `b16` (4932): `d` frame 1 stands (its own body and a hand `hx` of two
//! frames, calm or scared), 2 walks (4922, 22 frames, with a hand of its own), 3 dies (566, 36), 4
//! appears (4923, 22) and 5 bursts (4930, 70).

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MulliganAnimation {
    Standing,
    Walking,
    Bursting,
    Dying,
    Appearing,
}

impl MulliganAnimation {
    pub const ALL: [Self; 5] = [
        Self::Standing,
        Self::Walking,
        Self::Bursting,
        Self::Dying,
        Self::Appearing,
    ];

    pub fn frame_count(self) -> u8 {
        match self {
            Self::Standing => MULLIGAN_STANDING_FRAME_COUNT,
            Self::Walking => MULLIGAN_WALKING_FRAME_COUNT,
            Self::Bursting => MULLIGAN_BURSTING_FRAME_COUNT,
            Self::Dying => MULLIGAN_DYING_FRAME_COUNT,
            Self::Appearing => MULLIGAN_APPEARING_FRAME_COUNT,
        }
    }
}

pub const MULLIGAN_STANDING_FRAME_COUNT: u8 = 1;
pub const MULLIGAN_WALKING_FRAME_COUNT: u8 = 22;
pub const MULLIGAN_BURSTING_FRAME_COUNT: u8 = 70;
pub const MULLIGAN_DYING_FRAME_COUNT: u8 = 36;
pub const MULLIGAN_APPEARING_FRAME_COUNT: u8 = 22;
