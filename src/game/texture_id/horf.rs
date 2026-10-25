//! Horf's clip `b12` (5024): `d` frame 1 idles (4994, 3 frames), 3 dies (566, 36 frames), 4 appears
//! (5013, 60 frames) and 5 shoots (5023, 20 frames).

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HorfAnimation {
    Idle,
    Shooting,
    Dying,
    Appearing,
}

impl HorfAnimation {
    pub const ALL: [Self; 4] = [Self::Idle, Self::Shooting, Self::Dying, Self::Appearing];

    pub fn frame_count(self) -> u8 {
        match self {
            Self::Idle => HORF_IDLE_FRAME_COUNT,
            Self::Shooting => HORF_SHOOTING_FRAME_COUNT,
            Self::Dying => HORF_DYING_FRAME_COUNT,
            Self::Appearing => HORF_APPEARING_FRAME_COUNT,
        }
    }
}

pub const HORF_IDLE_FRAME_COUNT: u8 = 3;
pub const HORF_SHOOTING_FRAME_COUNT: u8 = 20;
pub const HORF_DYING_FRAME_COUNT: u8 = 36;
pub const HORF_APPEARING_FRAME_COUNT: u8 = 60;
