//! Textures of things in motion: doors in their states and the flies' animations.

use crate::game::room::doors::DoorStyle;

/// The black fly's clip `b13` (1517): appearing (1516) for 45 frames, a 16-frame bob (1515) while
/// flying, dying (555) for 16.
pub const BLACK_FLY_APPEARING_FRAME_COUNT: u8 = 45;
pub const BLACK_FLY_BOB_FRAME_COUNT: u8 = 16;
pub const BLACK_FLY_DYING_FRAME_COUNT: u8 = 16;
/// The attack fly's clip `b18` (5053): appearing (5052) for 26 frames, a 4-frame flap (578)
/// while flying, dying (555) for 16.
pub const ATTACK_FLY_APPEARING_FRAME_COUNT: u8 = 26;
pub const ATTACK_FLY_FLAP_FRAME_COUNT: u8 = 4;
pub const ATTACK_FLY_DYING_FRAME_COUNT: u8 = 16;
/// The red fly's clip `b61` (4872): appearing (4871) for 25 frames, a 4-frame flap (`hx.d`)
/// while flying, dying (555) for 16.
pub const RED_FLY_APPEARING_FRAME_COUNT: u8 = 25;
pub const RED_FLY_FLAP_FRAME_COUNT: u8 = 4;
pub const RED_FLY_DYING_FRAME_COUNT: u8 = 16;
/// The Moter's clip `b80` (4904): appearing (4903) for 26 frames, a 4-frame flap (two 578s) while
/// flying, dying (566) for 36.
pub const MOTER_APPEARING_FRAME_COUNT: u8 = 26;
pub const MOTER_FLAP_FRAME_COUNT: u8 = 4;
pub const MOTER_DYING_FRAME_COUNT: u8 = 36;
/// `bloodsplash` (1488): what an enemy shot leaves, 17 frames.
pub const BLOOD_SPLASH_FRAME_COUNT: u8 = 17;
/// Steps of `bull`, the enemy shots' pulsing brightness.
pub const ENEMY_SHOT_BRIGHTNESS_STEPS: u8 = 3;

/// The Pooter's clip `b14` (4990): appearing (4949) for 25 frames, a 2-frame flap (`hx` 4945, on
/// the variant's frame) while flying, dying (4948) for 15, shooting (`hx` 4989: 14 frames, 19 for
/// the two-shot Pooter).
pub const POOTER_APPEARING_FRAME_COUNT: u8 = 25;
pub const POOTER_FLAP_FRAME_COUNT: u8 = 2;
pub const POOTER_DYING_FRAME_COUNT: u8 = 15;
pub const POOTER_SHOOTING_FRAME_COUNT: u8 = 14;
pub const SUPER_POOTER_SHOOTING_FRAME_COUNT: u8 = 19;

/// The doors that shut on a room's enemies, each with its own clips.
pub const MOVING_DOOR_STYLES: [DoorStyle; 4] =
    [DoorStyle::Plain, DoorStyle::Boss, DoorStyle::Treasure, DoorStyle::Curse];

/// Frames a door's closing clip plays until its `stop()`: plain 6472 (23), boss 6494 (11),
/// treasure 6541 (23); the curse door just stands shut on frame 36.
pub fn door_closing_frame_count(style: DoorStyle) -> u8 {
    match style {
        DoorStyle::Boss => 11,
        DoorStyle::Curse => 1,
        _ => 23,
    }
}

/// Frames a door's opening clip `p` plays until its `stop()`: plain 6471, treasure 6542 and curse
/// 6545 (9 each), boss 6493 (19, its red glow fading in).
pub fn door_opening_frame_count(style: DoorStyle) -> u8 {
    match style {
        DoorStyle::Boss => 19,
        _ => 9,
    }
}

/// A plain door blown open while shut (door frame 5: its smoke clip 6477 plays 8 frames).
pub const BLOWN_DOOR_FRAME_COUNT: u8 = 8;

/// A door in one of its states.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DoorTexture {
    Open(DoorStyle),
    /// Frame `0..SHOP_DOOR_UNLOCKING_FRAME_COUNT` of the shop door's key turning.
    ShopUnlocking(u8),
    /// A door shutting (frame 0 still open, the last shut) or opening, one of
    /// [`MOVING_DOOR_STYLES`].
    Closing(DoorStyle, u8),
    Opening(DoorStyle, u8),
    /// A plain door blown open in smoke, frame `0..BLOWN_DOOR_FRAME_COUNT`.
    Blown(u8),
}

/// The fly (type 13, black and harmless), the attack fly (type 18), the red fly (type 61, which
/// bursts), the Moter (type 80, which splits in two), or the Pooter (type 14), which shoots;
/// `SuperPooter` is its two-shot variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FlyKind {
    Black,
    Attack,
    Pooter,
    SuperPooter,
    Red,
    Moter,
}

impl FlyKind {
    pub const ALL: [Self; 6] = [
        Self::Black,
        Self::Attack,
        Self::Pooter,
        Self::SuperPooter,
        Self::Red,
        Self::Moter,
    ];

    pub fn is_pooter(self) -> bool {
        matches!(self, Self::Pooter | Self::SuperPooter)
    }

    /// The animations its clip has (only Pooters shoot).
    pub fn animations(self) -> &'static [FlyAnimation] {
        if self.is_pooter() {
            &FlyAnimation::ALL
        } else {
            &FlyAnimation::ALL[..FlyAnimation::ALL.len() - 1]
        }
    }

    pub fn frame_count(self, animation: FlyAnimation) -> u8 {
        match (self, animation) {
            (Self::Black, FlyAnimation::Appearing) => BLACK_FLY_APPEARING_FRAME_COUNT,
            (Self::Black, FlyAnimation::Flying) => BLACK_FLY_BOB_FRAME_COUNT,
            (Self::Black, FlyAnimation::Dying) => BLACK_FLY_DYING_FRAME_COUNT,
            (Self::Attack, FlyAnimation::Appearing) => ATTACK_FLY_APPEARING_FRAME_COUNT,
            (Self::Attack, FlyAnimation::Flying) => ATTACK_FLY_FLAP_FRAME_COUNT,
            (Self::Attack, FlyAnimation::Dying) => ATTACK_FLY_DYING_FRAME_COUNT,
            (Self::Red, FlyAnimation::Appearing) => RED_FLY_APPEARING_FRAME_COUNT,
            (Self::Red, FlyAnimation::Flying) => RED_FLY_FLAP_FRAME_COUNT,
            (Self::Red, FlyAnimation::Dying) => RED_FLY_DYING_FRAME_COUNT,
            (Self::Moter, FlyAnimation::Appearing) => MOTER_APPEARING_FRAME_COUNT,
            (Self::Moter, FlyAnimation::Flying) => MOTER_FLAP_FRAME_COUNT,
            (Self::Moter, FlyAnimation::Dying) => MOTER_DYING_FRAME_COUNT,
            (Self::Black | Self::Attack | Self::Red | Self::Moter, FlyAnimation::Shooting) => 1,
            (_, FlyAnimation::Appearing) => POOTER_APPEARING_FRAME_COUNT,
            (_, FlyAnimation::Flying) => POOTER_FLAP_FRAME_COUNT,
            (_, FlyAnimation::Dying) => POOTER_DYING_FRAME_COUNT,
            (Self::Pooter, FlyAnimation::Shooting) => POOTER_SHOOTING_FRAME_COUNT,
            (Self::SuperPooter, FlyAnimation::Shooting) => SUPER_POOTER_SHOOTING_FRAME_COUNT,
        }
    }

    /// Frames of the black fly's separate wings (`d.d`); the attack fly's flap includes its own.
    pub fn wing_frame_count(self) -> u8 {
        match self {
            Self::Black => 2,
            Self::Attack | Self::Pooter | Self::SuperPooter | Self::Red | Self::Moter => 1,
        }
    }
}

/// Which animation of a fly's clip a texture shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FlyAnimation {
    Appearing,
    Flying,
    Dying,
    /// A Pooter's shot (`d` frame 5), back to flying once its `hx.d` has played.
    Shooting,
}

impl FlyAnimation {
    pub const ALL: [Self; 4] = [Self::Appearing, Self::Flying, Self::Dying, Self::Shooting];
}
