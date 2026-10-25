//! Clotty's clip `b15` (4902): `d` frame 1 idles (4877, 24 frames), 2 walks (4894, 20), 3 dies (566,
//! 36), 4 appears (4895, 27) and 5 shoots (4900, 24).

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ClottyAnimation {
    Idle,
    Walking,
    Shooting,
    Dying,
    Appearing,
}

impl ClottyAnimation {
    pub const ALL: [Self; 5] = [Self::Idle, Self::Walking, Self::Shooting, Self::Dying, Self::Appearing];

    pub fn frame_count(self) -> u8 {
        match self {
            Self::Idle => CLOTTY_IDLE_FRAME_COUNT,
            Self::Walking => CLOTTY_WALKING_FRAME_COUNT,
            Self::Shooting => CLOTTY_SHOOTING_FRAME_COUNT,
            Self::Dying => CLOTTY_DYING_FRAME_COUNT,
            Self::Appearing => CLOTTY_APPEARING_FRAME_COUNT,
        }
    }
}

pub const CLOTTY_IDLE_FRAME_COUNT: u8 = 24;
pub const CLOTTY_WALKING_FRAME_COUNT: u8 = 20;
pub const CLOTTY_SHOOTING_FRAME_COUNT: u8 = 24;
pub const CLOTTY_DYING_FRAME_COUNT: u8 = 36;
pub const CLOTTY_APPEARING_FRAME_COUNT: u8 = 27;
