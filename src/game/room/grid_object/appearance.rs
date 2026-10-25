//! Which texture a grid object shows, from its state and animation clocks.

use crate::game::draw::{DrawItem, FLOOR_DECAL_DEPTH, Rgba, actor_depth};
use crate::game::original_units::Frames;
use crate::game::room::grid_kind::{GridObjectKind, TileKind};
use crate::game::room::grid_object::{BreakableState, GridObject};
use crate::game::texture_id::{KEY_BLOCK_OPENING_FRAME_COUNT, TextureId};

const FALLBACK_COLOR: Rgba = Rgba::opaque(0.35, 0.28, 0.22);

#[derive(Clone, Copy)]
enum Playback {
    Loop,
    HoldLast,
}

impl GridObject {
    /// `None` once there is nothing left to draw (an opened key block).
    pub fn draw_item(&self) -> Option<DrawItem> {
        let depth = match self.kind {
            GridObjectKind::Tile(TileKind::Pit | TileKind::Spikes | TileKind::SpiderWeb) => FLOOR_DECAL_DEPTH,
            // Rubble and bridges are drawn into the floor bitmap.
            GridObjectKind::Tile(_) if self.cleared => FLOOR_DECAL_DEPTH,
            _ => actor_depth(self.center.y),
        };
        let texture = self.texture()?;
        Some(DrawItem::textured(texture, self.center, 1.0, FALLBACK_COLOR, depth))
    }

    fn texture(&self) -> Option<TextureId> {
        match (self.kind, &self.breakable) {
            (GridObjectKind::Breakable(kind), Some(state)) => Some(breakable_texture(kind, state, self.age_frames)),
            (GridObjectKind::Tile(tile_kind), _) if self.cleared => Some(TextureId::ClearedTile(tile_kind)),
            (GridObjectKind::Tile(tile_kind), _) => Some(TextureId::Tile(tile_kind)),
            _ => match self.frames_since_opened {
                None => Some(TextureId::KeyBlock),
                Some(frames) => {
                    let frame = frames.frame_number();
                    (frame < KEY_BLOCK_OPENING_FRAME_COUNT).then_some(TextureId::KeyBlockOpening(frame))
                }
            },
        }
    }
}

fn breakable_texture(kind: crate::game::BreakableKind, state: &BreakableState, age_frames: Frames) -> TextureId {
    if let Some(frames) = state.frames_since_destroyed {
        let frame = animation_frame(frames, kind.destroyed_frame_count(), Playback::HoldLast);
        return TextureId::BreakableDestroyed { kind, frame };
    }
    if let Some(frames) = state.frames_since_regrow {
        let elapsed = frames.frame_number();
        if elapsed < kind.regrow_frame_count() {
            return TextureId::BreakableRegrow { kind, frame: elapsed };
        }
    }
    let shown_stage = state.stage.min(kind.stage_count() - 1);
    let idle_frame = animation_frame(age_frames, kind.idle_frame_count(shown_stage), Playback::Loop);
    TextureId::BreakableStage {
        kind,
        stage: shown_stage,
        idle_frame,
    }
}

fn animation_frame(frames: Frames, frame_count: u8, playback: Playback) -> u8 {
    match playback {
        Playback::Loop => frames.looped_frame(frame_count),
        Playback::HoldLast => frames.held_frame(frame_count),
    }
}
