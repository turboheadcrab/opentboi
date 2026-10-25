//! Drawing what a room holds.

use crate::game::attacks::explosion::{Explosion, scorch_draw_item};
use crate::game::draw::{DrawItem, FLOOR_DECAL_DEPTH, Rgba, actor_depth};
use crate::game::room::contents::RoomContents;
use crate::game::room::grid_object::GridObject;
use crate::game::texture_id::{DECORATION_FRAME_COUNT, TextureId};

/// The broken floor is drawn into the floor bitmap over the rubble.
fn broken_floor_draw_item(center: glam::Vec2) -> DrawItem {
    DrawItem::textured(
        TextureId::BrokenFloor,
        center,
        1.0,
        BROKEN_FLOOR_FALLBACK,
        FLOOR_DECAL_DEPTH + BROKEN_FLOOR_DEPTH_ABOVE_RUBBLE,
    )
}

const BROKEN_FLOOR_FALLBACK: Rgba = Rgba::opaque(0.35, 0.3, 0.25);
const BROKEN_FLOOR_DEPTH_ABOVE_RUBBLE: f32 = 0.25;
const FIGURE_FALLBACK: Rgba = Rgba::opaque(0.2, 0.2, 0.2);

impl RoomContents {
    pub fn append_draw_items(&self, draw_items: &mut Vec<DrawItem>) {
        draw_items.extend(self.scorch_marks.iter().map(|center| scorch_draw_item(*center)));
        draw_items.extend(self.broken_floors.iter().map(|center| broken_floor_draw_item(*center)));
        // The rock that blocks the statue's cell is not drawn: the statue stands there.
        let under_figure = |object: &&GridObject| {
            self.decorations
                .iter()
                .any(|&(_, _, point)| object.tile_bounds().contains(point))
        };
        draw_items.extend(
            self.grid_objects
                .iter()
                .filter(|object| !under_figure(object))
                .filter_map(GridObject::draw_item),
        );
        let frame = self.age_frames.looped_frame(DECORATION_FRAME_COUNT);
        draw_items.extend(self.decorations.iter().map(|&(decoration, look, point)| {
            let texture = TextureId::Decoration {
                decoration,
                look,
                frame,
            };
            DrawItem::textured(texture, point, 1.0, FIGURE_FALLBACK, actor_depth(point.y))
        }));
        draw_items.extend(
            self.flies
                .iter()
                .map(|fly| fly.draw_item(self.grid_objects[fly.anchor_index].center())),
        );
        draw_items.extend(self.explosions.iter().map(Explosion::draw_item));
        self.creep.append_draw_items(&self.parameters.creep, draw_items);
        self.enemies.append_draw_items(draw_items);
        self.pickups.append_draw_items(draw_items);
        self.machines.append_draw_items(draw_items);
        self.wares.append_draw_items(draw_items);
    }
}
