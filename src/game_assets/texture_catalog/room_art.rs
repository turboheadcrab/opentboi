//! Room art styles: `golev()` sets `b.bg` and `b.w1`–`b.w4` to frame 1 in the Basement, 2 in the
//! shop, 3 in its boss room, 20 in the secret room and 10, 26, 19, 23 or 21 (by its style) in the
//! super secret room; the curse room 19, the sacrifice room 24 and the library 25.

use crate::game::RoomArt;
use crate::game_assets::vector_art::{ChildState, NO_OVERRIDES, RenderOverrides};

macro_rules! room_frame {
    ($frame:literal) => {
        RenderOverrides {
            children: &[
                ("bg", ChildState::Frame($frame)),
                ("w1", ChildState::Frame($frame)),
                ("w2", ChildState::Frame($frame)),
                ("w3", ChildState::Frame($frame)),
                ("w4", ChildState::Frame($frame)),
            ],
            texts: &[],
            hidden_characters: &[],
            character_frames: &[],
            tint: None,
        }
    };
}

static SHOP_ROOM: RenderOverrides = room_frame!(2);
static BOSS_ROOM: RenderOverrides = room_frame!(3);
static SECRET_ROOM: RenderOverrides = room_frame!(20);
static CURSE_ROOM: RenderOverrides = room_frame!(19);
static SACRIFICE_ROOM: RenderOverrides = room_frame!(24);
static LIBRARY_ROOM: RenderOverrides = room_frame!(25);
static SUPER_SECRET_ROOMS: [RenderOverrides; 5] = [
    room_frame!(10),
    room_frame!(26),
    room_frame!(19),
    room_frame!(23),
    room_frame!(21),
];

pub(super) fn overrides(art: RoomArt) -> &'static RenderOverrides {
    match art {
        RoomArt::Basement => &NO_OVERRIDES,
        RoomArt::Shop => &SHOP_ROOM,
        RoomArt::Boss => &BOSS_ROOM,
        RoomArt::Secret => &SECRET_ROOM,
        RoomArt::Curse => &CURSE_ROOM,
        RoomArt::Sacrifice => &SACRIFICE_ROOM,
        RoomArt::Library => &LIBRARY_ROOM,
        RoomArt::SuperSecret(style) => &SUPER_SECRET_ROOMS[usize::from(style) % SUPER_SECRET_ROOMS.len()],
    }
}
