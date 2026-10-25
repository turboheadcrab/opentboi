//! The four doors of a room (`d1`–`d4`, clip 6550) and where walking through each one leads.
//! Positions are the original's stage coordinates (`x - 320`, `280 - y` in our world units).

use std::f32::consts::{FRAC_PI_2, PI};

use glam::Vec2;

use crate::game::draw::{DrawItem, Rgba};
use crate::game::original_units::Pixels;
use crate::game::texture_id::TextureId;

/// Doors sit on the walls, above the room art and below everything standing in the room.
const DOOR_DEPTH: f32 = 1.5;
/// The player goes through within this of the door's point.
pub const DOOR_REACH: Pixels = Pixels(25.0);
/// A door becomes usable again once the player is this far from it.
pub const DOOR_REARM_DISTANCE: Pixels = Pixels(35.0);
const FALLBACK_COLOR: Rgba = Rgba::opaque(0.35, 0.2, 0.15);

/// Door art (clip 6550): frame 4 for plain doors (`golev()`); doors into or out of a treasure room
/// use frame 32, the open gold door (the first floor's treasure room is never locked).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DoorStyle {
    Plain,
    Treasure,
    /// A secret room's wall after a bomb (frame 17, the blown hole).
    BlownOpen,
    /// Doors into and out of the boss room (frame 16).
    Boss,
    /// Doors into and out of the curse room, spiked (frame 36).
    Curse,
    /// The shop door before a key opens it (end of frame 9) and after (end of frame 8).
    ShopLocked,
    ShopOpen,
    /// Into and out of the devil room (frame 19, open).
    Devil,
}

impl DoorStyle {
    pub const ALL: [Self; 8] = [
        Self::Devil,
        Self::Plain,
        Self::Treasure,
        Self::Boss,
        Self::Curse,
        Self::BlownOpen,
        Self::ShopLocked,
        Self::ShopOpen,
    ];
}

/// The point that sends the player through lies this far out in the doorway, past the door art.
const TRIGGER_OFFSET: Pixels = Pixels(28.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DoorSide {
    Left,
    Bottom,
    Right,
    Top,
}

impl DoorSide {
    pub const ALL: [Self; 4] = [Self::Left, Self::Bottom, Self::Right, Self::Top];

    /// Placement of `d1` (51, 280), `d2` (320, 431), `d3` (589.95, 280) and `d4` (320, 128).
    pub fn position(self) -> Vec2 {
        match self {
            Self::Left => Vec2::new(-269.0, 0.0),
            Self::Bottom => Vec2::new(0.0, -151.0),
            Self::Right => Vec2::new(269.95, 0.0),
            Self::Top => Vec2::new(0.0, 152.0),
        }
    }

    /// Straight out of the room through this door.
    pub fn outward(self) -> Vec2 {
        match self {
            Self::Left => Vec2::NEG_X,
            Self::Bottom => Vec2::NEG_Y,
            Self::Right => Vec2::X,
            Self::Top => Vec2::Y,
        }
    }

    /// Where the player must get within [`DOOR_REACH`] of to go through.
    pub fn trigger_point(self) -> Vec2 {
        self.position() + self.outward() * TRIGGER_OFFSET
    }

    /// The wall cell the door sits in (`blo`), as (column, row) of the interior grid; the
    /// ring of walls is column -1 and 13, row -1 and 7.
    pub fn doorway_cell(self) -> (i32, i32) {
        match self {
            Self::Left => (-1, 3),
            Self::Bottom => (6, 7),
            Self::Right => (13, 3),
            Self::Top => (6, -1),
        }
    }

    /// The door art faces right; the others are the same clip turned (`d1` is flipped both ways,
    /// `d2` and `d4` are rotated a quarter turn).
    fn rotation_radians(self) -> f32 {
        match self {
            Self::Left => PI,
            Self::Bottom => -FRAC_PI_2,
            Self::Right => 0.0,
            Self::Top => FRAC_PI_2,
        }
    }

    /// Where the player appears after leaving through this door: by the opposite door of the next
    /// room (`playerx = 610 - door._x * 0.9`, or `playery = 530 - door._y * 0.9`).
    pub fn entry_position(self) -> Vec2 {
        match self {
            Self::Left => Vec2::new(244.1, 0.0),
            Self::Bottom => Vec2::new(0.0, 137.9),
            Self::Right => Vec2::new(-241.0, 0.0),
            Self::Top => Vec2::new(0.0, -134.8),
        }
    }

    pub fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Bottom => Self::Top,
            Self::Right => Self::Left,
            Self::Top => Self::Bottom,
        }
    }

    /// `texture_id` is door art drawn on the right wall ([`TextureId::Door`] or an animation).
    pub fn draw_item(self, texture_id: TextureId) -> DrawItem {
        DrawItem {
            rotation_radians: self.rotation_radians(),
            ..DrawItem::textured(texture_id, self.position(), 1.0, FALLBACK_COLOR, DOOR_DEPTH)
        }
    }
}
