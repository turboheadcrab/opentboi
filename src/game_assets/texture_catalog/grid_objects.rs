//! Art sources for room grid objects. Every grid object's origin is its tile center.

use swf::CharacterId;

use super::{TextureSource, framed};
use crate::game::{BreakableKind, TextureId, TileKind};
use crate::game_assets::vector_art::{FrameSelection, NO_OVERRIDES};

/// The tile sheet (`tiles`, sprite 6011), drawn into the floor bitmap one frame per tile.
const TILE_SHEET: CharacterId = 6011;
/// Rocks use frames 1–10 at random; frame 1 is shown.
const ROCK_FRAME: u16 = 1;
/// The level's marked rock (`rarer`).
const TINTED_ROCK_FRAME: u16 = 15;
/// A pit filled by a blown rock.
const PIT_BRIDGE_FRAME: u16 = 66;
/// Rubble cleared between two pits is covered by this (`breakall()`).
const BROKEN_FLOOR_FRAME: u16 = 67;
/// Blocks (level value 1.9) use frames 11–14 at random.
const BLOCK_FRAME: u16 = 11;
const SPIKES_FRAME: u16 = 21;
/// A pit with no pit neighbors uses frame 22 or 23, at random.
const LONE_PIT_FRAME: u16 = 22;
/// Webs use frames 159–161 at random.
const SPIDER_WEB_FRAME: u16 = 159;
/// `bloww`: rocks leave rubble from frames 60–64 at random (frame 60 shown), and a web on frame
/// `158 + n` leaves torn strands on frame `161 + n`.
const ROCK_RUBBLE_FRAME: u16 = 60;
const TORN_SPIDER_WEB_FRAME: u16 = SPIDER_WEB_FRAME + 3;
/// `locktile`: frame 1 locked, frame 2 opening.
const KEY_BLOCK_CLIP: CharacterId = 4615;

/// Damaged objects advance one clip frame per damage step; `killshit()` then jumps to the destroyed
/// frame (5), except the TNT barrel, whose fourth frame is already its wreck.
const DESTROYED_FRAME: u16 = 5;
const TNT_DESTROYED_FRAME: u16 = 4;
/// Red poop's grow-back animation (frame 6, sprite 5835).
const REGROW_FRAME: u16 = 6;
const KEY_BLOCK_OPENING_FRAME: u16 = 2;

fn breakable_clip(kind: BreakableKind) -> CharacterId {
    match kind {
        BreakableKind::Poop => 1471,
        BreakableKind::RedPoop => 5836,
        BreakableKind::CornyPoop => 5844,
        BreakableKind::GoldenPoop => 5243,
        BreakableKind::Fireplace => 1455,
        // `fireblock2` (5846) wraps the actual object `d` (5845) and adds smoke puffs.
        BreakableKind::RedFireplace => 5845,
        BreakableKind::TntBarrel => 4839,
    }
}

/// `fireblock2` places its object 0.25 px right and 1 px up of the tile center.
fn origin_in_clip(kind: BreakableKind) -> (f32, f32) {
    match kind {
        BreakableKind::RedFireplace => (-0.25, 1.0),
        _ => (0.0, 0.0),
    }
}

pub(super) fn source_for(texture_id: TextureId) -> TextureSource {
    match texture_id {
        TextureId::Tile(tile_kind) => framed(TILE_SHEET, tile_frame(tile_kind, false)),
        TextureId::BrokenFloor => framed(TILE_SHEET, BROKEN_FLOOR_FRAME),
        TextureId::ClearedTile(tile_kind) => framed(TILE_SHEET, tile_frame(tile_kind, true)),
        TextureId::BreakableStage {
            kind,
            stage,
            idle_frame,
        } => breakable_source(kind, 1 + u16::from(stage), idle_frame),
        TextureId::BreakableDestroyed { kind, frame } => {
            let destroyed_frame = if kind == BreakableKind::TntBarrel {
                TNT_DESTROYED_FRAME
            } else {
                DESTROYED_FRAME
            };
            breakable_source(kind, destroyed_frame, frame)
        }
        TextureId::BreakableRegrow { kind, frame } => breakable_source(kind, REGROW_FRAME, frame),
        TextureId::KeyBlockOpening(frame) => TextureSource {
            selection: FrameSelection {
                frame_number: KEY_BLOCK_OPENING_FRAME,
                frames_since_frame_started: u16::from(frame),
            },
            ..framed(KEY_BLOCK_CLIP, 1)
        },
        _ => framed(KEY_BLOCK_CLIP, 1),
    }
}

/// The tile sheet frame for a tile, intact or after a blast cleared it.
fn tile_frame(tile_kind: TileKind, cleared: bool) -> u16 {
    match (tile_kind, cleared) {
        (TileKind::Rock, false) => ROCK_FRAME,
        (TileKind::Rock | TileKind::TintedRock, true) => ROCK_RUBBLE_FRAME,
        (TileKind::TintedRock, false) => TINTED_ROCK_FRAME,
        (TileKind::Pit, true) => PIT_BRIDGE_FRAME,
        (TileKind::SpiderWeb, false) => SPIDER_WEB_FRAME,
        (TileKind::SpiderWeb, true) => TORN_SPIDER_WEB_FRAME,
        (TileKind::Block, _) => BLOCK_FRAME,
        (TileKind::Pit, false) => LONE_PIT_FRAME,
        (TileKind::Spikes, _) => SPIKES_FRAME,
    }
}

fn breakable_source(kind: BreakableKind, frame_number: u16, frames_since_frame_started: u8) -> TextureSource {
    let (origin_x, origin_y) = origin_in_clip(kind);
    TextureSource {
        character_id: breakable_clip(kind),
        selection: FrameSelection {
            frame_number,
            frames_since_frame_started: u16::from(frames_since_frame_started),
        },
        origin_x,
        origin_y,
        overrides: &NO_OVERRIDES,
    }
}
