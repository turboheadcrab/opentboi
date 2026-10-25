//! Room geometry in world units, which equal the original's Flash pixels.

pub mod change;
pub mod contents;
pub mod creep;
pub mod doors;
pub mod enemies;
pub mod grid_cells;
pub mod grid_kind;
pub mod grid_object;
pub mod layout;
pub mod orbiting_fly;
pub mod stage_area;
pub mod transition;

use glam::Vec2;

use crate::game::bounds::Bounds;
use crate::game::draw::{DrawItem, FLOOR_DEPTH, ROOM_BACKGROUND_DEPTH, Rgba};
use crate::game::original_units::Pixels;
use crate::game::texture_id::{RoomArt, TextureId};

/// The original's grid: 13×7 tiles of 40 px (measured from the walls' inner edges in sprite 6468).
pub const TILE_SIZE: Pixels = Pixels(40.0);
/// The original's stage, 640×480: the room plus the HUD strip above.
/// Its point (320, 280) is the room's center, so the stage center sits 40 units above it.
pub const STAGE_SIZE: Vec2 = Vec2::new(640.0, 480.0);
pub const STAGE_CENTER: Vec2 = Vec2::new(0.0, 40.0);
/// View masks are this many views wide and tall, enough to cover any letterbox bar.
const VIEW_MASK_SCALE: f32 = 4.0;
const VIEW_MASK_COLOR: Rgba = Rgba::opaque(0.0, 0.0, 0.0);
/// Above everything the game draws, and inside Bevy's default 2D camera range (±1000).
const VIEW_MASK_DEPTH: f32 = 900.0;
pub const COLUMN_COUNT: u32 = 13;
pub const ROW_COUNT: u32 = 7;
pub const INTERIOR_SIZE: Vec2 = Vec2::new(
    COLUMN_COUNT as f32 * TILE_SIZE.get(),
    ROW_COUNT as f32 * TILE_SIZE.get(),
);
/// Side walls are thinner than top and bottom walls in the original art.
pub const WALL_THICKNESS: Vec2 = Vec2::new(64.0, 70.0);
pub const ROOM_SIZE: Vec2 = Vec2::new(
    INTERIOR_SIZE.x + 2.0 * WALL_THICKNESS.x,
    INTERIOR_SIZE.y + 2.0 * WALL_THICKNESS.y,
);

/// Whether the room is drawn from the original artwork or from built-in placeholder shapes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RoomAppearance {
    /// Draws [`TextureId::RoomBackground`], covering [`ROOM_SIZE`] centered on the interior.
    Artwork,
    #[default]
    Placeholder,
}

const LIGHT_FLOOR_COLOR: Rgba = Rgba::opaque(0.36, 0.27, 0.21);
const DARK_FLOOR_COLOR: Rgba = Rgba::opaque(0.33, 0.25, 0.19);
const WALL_COLOR: Rgba = Rgba::opaque(0.20, 0.14, 0.11);
const DOOR_COLOR: Rgba = Rgba::opaque(0.08, 0.05, 0.04);
const DOOR_LENGTH: f32 = TILE_SIZE.get() * 1.5;
const DOOR_DEPTH_RATIO: f32 = 0.8;

/// Walkable area, centered on the world origin.
pub fn interior_bounds() -> Bounds {
    Bounds::centered(Vec2::ZERO, INTERIOR_SIZE)
}

/// Bounds of a grid cell, including the wall ring around the interior (column -1 and 13, row -1
/// and 7), which `tile_center` cannot express.
pub fn cell_bounds(column: i32, row: i32) -> Bounds {
    let center = Vec2::new(
        -INTERIOR_SIZE.x / 2.0 + (column as f32 + 0.5) * TILE_SIZE.get(),
        INTERIOR_SIZE.y / 2.0 - (row as f32 + 0.5) * TILE_SIZE.get(),
    );
    Bounds::centered(center, Vec2::splat(TILE_SIZE.get()))
}

/// Center of a grid tile; column 0 is leftmost, row 0 is the top row (`outgrid()`, DoAction 1378: the
/// column is the cell's remainder over the row length, the row its quotient rounded down). Grid objects
/// in the original are placed with their origin on this point.
pub fn tile_center(column: u32, row: u32) -> Vec2 {
    Vec2::new(
        -INTERIOR_SIZE.x / 2.0 + (column as f32 + 0.5) * TILE_SIZE.get(),
        INTERIOR_SIZE.y / 2.0 - (row as f32 + 0.5) * TILE_SIZE.get(),
    )
}

pub fn append_draw_items(appearance: RoomAppearance, art: RoomArt, draw_items: &mut Vec<DrawItem>) {
    match appearance {
        RoomAppearance::Artwork => draw_items.push(DrawItem::textured(
            TextureId::RoomBackground(art),
            Vec2::ZERO,
            1.0,
            WALL_COLOR,
            ROOM_BACKGROUND_DEPTH,
        )),
        RoomAppearance::Placeholder => {
            draw_items.push(DrawItem::solid(
                Vec2::ZERO,
                ROOM_SIZE,
                WALL_COLOR,
                ROOM_BACKGROUND_DEPTH,
            ));
            append_floor_tiles(draw_items);
            append_doors(draw_items);
        }
    }
}

fn append_floor_tiles(draw_items: &mut Vec<DrawItem>) {
    let first_tile_center = -INTERIOR_SIZE / 2.0 + Vec2::splat(TILE_SIZE.get() / 2.0);
    for row in 0..ROW_COUNT {
        for column in 0..COLUMN_COUNT {
            let is_light_tile = (row + column) % 2 == 0;
            let color = if is_light_tile {
                LIGHT_FLOOR_COLOR
            } else {
                DARK_FLOOR_COLOR
            };
            let center = first_tile_center + Vec2::new(column as f32, row as f32) * TILE_SIZE.get();
            draw_items.push(DrawItem::solid(
                center,
                Vec2::splat(TILE_SIZE.get()),
                color,
                FLOOR_DEPTH,
            ));
        }
    }
}

/// Closed doors in the middle of each wall; placeholders until room transitions exist.
fn append_doors(draw_items: &mut Vec<DrawItem>) {
    let wall_center_offset = INTERIOR_SIZE / 2.0 + WALL_THICKNESS / 2.0;
    let horizontal_door = Vec2::new(DOOR_LENGTH, WALL_THICKNESS.y * DOOR_DEPTH_RATIO);
    let vertical_door = Vec2::new(WALL_THICKNESS.x * DOOR_DEPTH_RATIO, DOOR_LENGTH);
    let doors = [
        (Vec2::new(0.0, wall_center_offset.y), horizontal_door),
        (Vec2::new(0.0, -wall_center_offset.y), horizontal_door),
        (Vec2::new(wall_center_offset.x, 0.0), vertical_door),
        (Vec2::new(-wall_center_offset.x, 0.0), vertical_door),
    ];
    for (center, size) in doors {
        draw_items.push(DrawItem::solid(center, size, DOOR_COLOR, FLOOR_DEPTH));
    }
}

/// Stage points are y-down from the top left; the world is y-up around the stage center.
pub fn stage_to_world(stage_point: Vec2) -> Vec2 {
    let stage_middle = STAGE_SIZE / 2.0;
    Vec2::new(
        stage_point.x - stage_middle.x + STAGE_CENTER.x,
        stage_middle.y - stage_point.y + STAGE_CENTER.y,
    )
}

/// The inverse of [`stage_to_world`].
pub fn world_to_stage(world_point: Vec2) -> Vec2 {
    let stage_middle = STAGE_SIZE / 2.0;
    Vec2::new(
        world_point.x + stage_middle.x - STAGE_CENTER.x,
        stage_middle.y - (world_point.y - STAGE_CENTER.y),
    )
}

/// Black panels around `opening` (centered on the stage), drawn over everything, so nothing
/// outside the view (a room sliding in or out) shows in a frontend's letterbox bars.
pub fn append_view_masks(opening: Vec2, draw_items: &mut Vec<DrawItem>) {
    let mask_size = opening * VIEW_MASK_SCALE;
    let offsets = [
        Vec2::new(-opening.x / 2.0 - mask_size.x / 2.0, 0.0),
        Vec2::new(opening.x / 2.0 + mask_size.x / 2.0, 0.0),
        Vec2::new(0.0, -opening.y / 2.0 - mask_size.y / 2.0),
        Vec2::new(0.0, opening.y / 2.0 + mask_size.y / 2.0),
    ];
    draw_items.extend(
        offsets
            .into_iter()
            .map(|offset| DrawItem::solid(STAGE_CENTER + offset, mask_size, VIEW_MASK_COLOR, VIEW_MASK_DEPTH)),
    );
}

/// The original's grid (`ingrid()`, 16 cells a row) puts the 13×7 interior's first cell at (2, 4).
pub const ORIGINAL_INTERIOR_FIRST_COLUMN: i32 = 2;
pub const ORIGINAL_INTERIOR_FIRST_ROW: i32 = 4;

/// Where the player is and how fast he moves (world coordinates), as the enemies see him.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PlayerSighting {
    pub position: Vec2,
    pub velocity: Vec2,
}

impl From<Vec2> for PlayerSighting {
    /// A player standing still.
    fn from(position: Vec2) -> Self {
        Self {
            position,
            velocity: Vec2::ZERO,
        }
    }
}
