//! The minimap (`mapd()`): one piece per known room on a 25×10 grid, centered on stage point
//! (130, 40) in the HUD's top bar, with a mark on rooms where something is left (`itmc()`).

use std::collections::BTreeMap;

use glam::Vec2;

use crate::game::draw::{DrawItem, Rgba};
use crate::game::floor::Floor;
use crate::game::floor::FloorCell;
use crate::game::room::{RoomAppearance, stage_to_world};
use crate::game::texture_id::TextureId;

/// Rooms sit 25 apart across and 10 apart down, offset by (15, 7): DoAction 1252, DoAction 1253,
/// DoAction 1294, DoAction 1295. The starting room's piece is the one at cell 35 (DoAction 1230).
const PIECE_SPACING: Vec2 = Vec2::new(25.0, 10.0);
const PIECE_OFFSET: Vec2 = Vec2::new(15.0, 7.0);
/// The map's middle is put at this point of the HUD.
const MAP_CENTER_ON_STAGE: Vec2 = Vec2::new(130.0, 40.0);
/// Above the HUD, below the view masks. Each piece is drawn with its own pickup mark, one after
/// the other, so a later piece covers what it overlaps of an earlier one (the pieces are larger
/// than the grid cells).
const FIRST_PIECE_DEPTH: f32 = 810.0;
const DEPTH_PER_DRAWN_ITEM: f32 = 0.001;
/// `mpiece.itmc` sits at (-4.4, -3.35) in the piece.
const PICKUP_MARK_OFFSET: Vec2 = Vec2::new(-4.4, -3.35);
const NO_FALLBACK: Rgba = Rgba {
    red: 0.0,
    green: 0.0,
    blue: 0.0,
    alpha: 0.0,
};

pub fn append_draw_items(floor: &Floor, appearance: RoomAppearance, draw_items: &mut Vec<DrawItem>) {
    if appearance != RoomAppearance::Artwork {
        return;
    }
    let pieces = floor.map_pieces();
    let points = pieces.iter().map(|&(cell, _)| piece_point(cell));
    let Some(minimum) = points.clone().reduce(Vec2::min) else {
        return;
    };
    let maximum = points.fold(minimum, Vec2::max);
    let offset = MAP_CENTER_ON_STAGE - (minimum + maximum) / 2.0;
    let marks: BTreeMap<FloorCell, u8> = floor.pickup_marks().into_iter().collect();
    let mut next_depth = FIRST_PIECE_DEPTH;
    let mut take_depth = || {
        next_depth += DEPTH_PER_DRAWN_ITEM;
        next_depth
    };
    // `mapd()` draws the piece clip once per room and once more per icon, and the clip carries the
    // room's pickup mark (`itmc()`) every time: the mark lands on top of its piece.
    for (cell, piece) in pieces {
        draw_items.push(DrawItem::textured(
            TextureId::MapPiece(piece),
            stage_to_world(piece_point(cell) + offset),
            1.0,
            NO_FALLBACK,
            take_depth(),
        ));
        if let Some(&mark) = marks.get(&cell) {
            draw_items.push(DrawItem::textured(
                TextureId::MapPickupMark(mark),
                stage_to_world(piece_point(cell) + offset + PICKUP_MARK_OFFSET),
                1.0,
                NO_FALLBACK,
                take_depth(),
            ));
        }
    }
}

fn piece_point(cell: FloorCell) -> Vec2 {
    let column = f32::from(cell.column());
    let row = f32::from(cell.row());
    Vec2::new(column, row) * PIECE_SPACING + PIECE_OFFSET
}
