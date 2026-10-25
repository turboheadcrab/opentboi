//! Reads room layouts from a chapter's room sprite: each frame is one room, built from marker
//! clips whose frame scripts pass a value to `spawnb()`. We do not run ActionScript, so the
//! markers' values are transcribed here, and `spawnb()`'s rules are applied to them.

use crate::game::{ORIGINAL_INTERIOR_FIRST_COLUMN, ORIGINAL_INTERIOR_FIRST_ROW};
use crate::game_assets::vector_art::TWIPS_PER_PIXEL;
use glam::Vec2;

use swf::CharacterId;

use super::vector_art::{Character, CharacterLibrary, Placement};
use crate::game::{Decoration, LayoutCell, RoomLayout, RoomLayouts};

/// `levs1` (Basement and Cellar, chapters 1–2): sprite 6622, placed at the stage origin.
pub const BASEMENT_ROOM_SPRITE: CharacterId = 6622;
/// `levs6`: special rooms (sprite 6732). Treasure rooms are labels `t0`–`t5`.
const SPECIAL_ROOM_SPRITE: CharacterId = 6732;
/// Frames of labels `t0`, `t1` … `t5`.
const TREASURE_ROOM_FRAMES: [usize; 6] = [130, 125, 126, 127, 128, 129];
/// Frames of labels `hide0` … `hide4`.
const SECRET_ROOM_FRAMES: [usize; 5] = [115, 116, 117, 118, 119];
/// Frames of labels `shop1` … `shop6`.
const SHOP_FRAMES: [usize; 6] = [109, 110, 111, 112, 113, 114];
/// Frames of labels `b1-0` … `b1-4`, the Basement's boss rooms (`bosss = chaps`).
const BASEMENT_BOSS_ROOM_FRAMES: [usize; 5] = [20, 21, 22, 23, 24];
/// Frames of labels `sac`, and `c`, `c0`, `c1`, `c2`.
const SACRIFICE_ROOM_FRAMES: [usize; 1] = [169];
/// Frame of label `lib`.
const LIBRARY_FRAMES: [usize; 1] = [173];
const CURSE_ROOM_FRAMES: [usize; 4] = [178, 179, 180, 181];
/// Frames of labels `min0` … `min6`.
const MINIBOSS_ROOM_FRAMES: [usize; 7] = [4, 5, 6, 7, 8, 9, 10];
/// Frame of label `satan` (`satan1`, `krampus` and `sata` belong to later floors and variants).
const DEVIL_ROOM_FRAMES: [usize; 1] = [132];
/// Frames of labels `hi0` … `hi4`.
const SUPER_SECRET_ROOM_FRAMES: [usize; 5] = [183, 184, 185, 186, 187];
/// `ingrid()`: 40-pixel cells; the 13×7 interior starts at cell (2, 4).
const CELL_SIZE: f32 = 40.0;
/// `spawnb`: faint markers (`_alpha` < 30) lose 0.2, half-transparent ones (< 75) lose 0.1, except
/// that a half-transparent rock (1) becomes a block (1.9). Values in (-0.5, 0] are left alone.
const FAINT_ALPHA: f32 = 0.30;
const HALF_TRANSPARENT_ALPHA: f32 = 0.75;
const FAINT_REDUCTION: f32 = 0.2;
const HALF_TRANSPARENT_REDUCTION: f32 = 0.1;
const UNCHANGED_RANGE: std::ops::RangeInclusive<f32> = -0.5..=0.0;
const ROCK_VALUE: f32 = 1.0;
const BLOCK_VALUE: f32 = 1.9;

/// The pickup marker (sprite 6605) reports a value from its instance name.
const PICKUP_MARKER: CharacterId = 6605;
/// Figures the layouts place (all named `shop`, `inl.shop`): the devil statue, the shopkeepers.
const DECORATIONS: [(CharacterId, Decoration); 3] = [
    (3729, Decoration::DevilStatue),
    (6711, Decoration::Shopkeeper),
    (6718, Decoration::SecretShopkeeper),
];
const PICKUP_MARKER_NAME_PREFIX: &str = "a";
const PICKUP_MARKER_NAMED_VARIANTS: u8 = 50;
const PICKUP_MARKER_DEFAULT_VARIANT: u8 = 1;
const PICKUP_MARKER_VALUE_STEP: f32 = 0.01;
/// Marker clips placing grid objects and enemies, with the value their script passes to `spawnb()`
/// (an enemy marker's value `v` creates type 9 − `v`). Only the enemies the game has are
/// transcribed: sprite 6597 (-1), the Gaper (type 10), sprite 6610 (-2), the headless body
/// (type 11), sprite 6599 (-3), the Horf (type 12), sprite 6601 (-4), the black fly (type 13),
/// sprite 6589 (-9), the attack fly (type 18), sprite 6603 (-5), the Pooter (type 14), sprite 6618
/// (-6), the Clotty (type 15), and sprite 6615 (-7), the Mulligan (type 16). See `docs/enemies.md`
/// for the rest.
const TILE_MARKERS: [(CharacterId, f32); 20] = [
    (6597, -1.0),
    (6610, -2.0),
    (6599, -3.0),
    (6601, -4.0),
    (6589, -9.0),
    (6603, -5.0),
    (6618, -6.0),
    (6615, -7.0),
    (6593, 1.0),
    (6595, 1.5),
    (6611, 1.4),
    (6630, 1.3),
    (6650, 4.0),
    // Pits and spikes named `a<n>` report `-n / 100` instead; those variants are not handled yet.
    (6628, 3.0),
    (6643, 1.93),
    (6736, 1.94),
    (6738, 1.41),
    (6743, 1.495),
    (6759, 1.49),
    (6762, 1.4955),
];

/// The Basement's normal and treasure room layouts.
pub fn basement_layouts(library: &CharacterLibrary) -> RoomLayouts {
    let special = room_layouts(library, SPECIAL_ROOM_SPRITE);
    RoomLayouts {
        normal: room_layouts(library, BASEMENT_ROOM_SPRITE),
        treasure: frames(&special, &TREASURE_ROOM_FRAMES),
        secret: frames(&special, &SECRET_ROOM_FRAMES),
        shop: frames(&special, &SHOP_FRAMES),
        boss: frames(&special, &BASEMENT_BOSS_ROOM_FRAMES),
        super_secret: frames(&special, &SUPER_SECRET_ROOM_FRAMES),
        sacrifice: frames(&special, &SACRIFICE_ROOM_FRAMES),
        library: frames(&special, &LIBRARY_FRAMES),
        curse: frames(&special, &CURSE_ROOM_FRAMES),
        miniboss: frames(&special, &MINIBOSS_ROOM_FRAMES),
        devil: frames(&special, &DEVIL_ROOM_FRAMES),
    }
}

fn frames(layouts: &[RoomLayout], frame_numbers: &[usize]) -> Vec<RoomLayout> {
    frame_numbers
        .iter()
        .filter_map(|&frame_number| layouts.get(frame_number - 1).cloned())
        .collect()
}

/// Every frame of `sprite` as a layout, frame 1 first. Empty if the sprite is missing.
pub fn room_layouts(library: &CharacterLibrary, sprite: CharacterId) -> Vec<RoomLayout> {
    let Some(Character::Sprite(timeline)) = library.character(sprite) else {
        return Vec::new();
    };
    (1..=timeline.frame_count())
        .map(|frame_number| {
            let placements = timeline.display_list(frame_number);
            RoomLayout {
                cells: placements.iter().filter_map(layout_cell).collect(),
                decorations: placements
                    .iter()
                    .filter_map(|placement| {
                        let (_, decoration) = DECORATIONS
                            .iter()
                            .find(|(character_id, _)| *character_id == placement.character_id)?;
                        Some((*decoration, placement_point(placement)))
                    })
                    .collect(),
            }
        })
        .collect()
}

fn placement_point(placement: &Placement) -> Vec2 {
    Vec2::new(
        placement.matrix.tx.get() as f32 / TWIPS_PER_PIXEL,
        placement.matrix.ty.get() as f32 / TWIPS_PER_PIXEL,
    )
}

fn layout_cell(placement: &Placement) -> Option<LayoutCell> {
    let marker_value = if placement.character_id == PICKUP_MARKER {
        pickup_marker_value(placement)
    } else {
        TILE_MARKERS
            .iter()
            .find(|(character_id, _)| *character_id == placement.character_id)?
            .1
    };
    let x = placement.matrix.tx.get() as f32 / TWIPS_PER_PIXEL;
    let y = placement.matrix.ty.get() as f32 / TWIPS_PER_PIXEL;
    let column = crate::whole_numbers::clamped_i32((x / CELL_SIZE).round()) - ORIGINAL_INTERIOR_FIRST_COLUMN;
    let row = crate::whole_numbers::clamped_i32((y / CELL_SIZE).round()) - ORIGINAL_INTERIOR_FIRST_ROW;
    Some(LayoutCell {
        column: u32::try_from(column).ok()?,
        row: u32::try_from(row).ok()?,
        value: value_after_alpha(marker_value, placement.color_effect.alpha_multiplier()),
    })
}

/// The pickup marker's value: minus `e` hundredths when the marker is named `a<e>` (e < 50), else
/// minus one hundredth.
fn pickup_marker_value(placement: &Placement) -> f32 {
    let name = placement
        .name
        .map(|name| name.to_str_lossy(swf::UTF_8).into_owned())
        .unwrap_or_default();
    let named_variant = name
        .strip_prefix(PICKUP_MARKER_NAME_PREFIX)
        .and_then(|number| number.parse::<u8>().ok())
        .filter(|&number| number < PICKUP_MARKER_NAMED_VARIANTS);
    -f32::from(named_variant.unwrap_or(PICKUP_MARKER_DEFAULT_VARIANT)) * PICKUP_MARKER_VALUE_STEP
}

/// `spawnb()`'s rules for a marker's transparency.
fn value_after_alpha(value: f32, alpha: f32) -> f32 {
    if UNCHANGED_RANGE.contains(&value) {
        return value;
    }
    if alpha < FAINT_ALPHA {
        value - FAINT_REDUCTION
    } else if alpha < HALF_TRANSPARENT_ALPHA {
        if value == ROCK_VALUE {
            BLOCK_VALUE
        } else {
            value - HALF_TRANSPARENT_REDUCTION
        }
    } else {
        value
    }
}

#[cfg(test)]
mod tests;
