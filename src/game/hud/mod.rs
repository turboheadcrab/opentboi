//! The original's in-game HUD (clip 1441, attached to the stage's top left): counters, the
//! weapon and item boxes, hearts and the floor's name. The clip itself is one static texture; the
//! counters and hearts are drawn over it from the inventory, as the HUD update does each frame.
//! Around the stage lies the projector's `overlay` (sprite 6221), seen on screens wider or taller
//! than 4:3.

pub mod active_item_label;
pub mod held_card;
pub mod held_trinket;
pub mod item_box;
pub mod message;
pub mod minimap;
pub mod name_banner;
pub mod overlay;
pub mod pill_text;
pub mod stats;

use glam::Vec2;

use crate::game::draw::{DrawItem, Rgba};
use crate::game::input_state::InputDevice;
use crate::game::original_units::Hearts;
use crate::game::player::inventory::Inventory;
use crate::game::room::{RoomAppearance, STAGE_CENTER, STAGE_SIZE, stage_to_world};
use crate::game::settings::Style;
use crate::game::texture_id::TextureId;

/// Above everything in the room, below the view masks; counters and hearts over the HUD.
const HUD_DEPTH: f32 = 800.0;
const HUD_VALUE_DEPTH: f32 = 801.0;
/// The overlay covers whatever lies outside the stage, above the HUD and minimap.
const SURROUND_DEPTH: f32 = 850.0;
/// Without artwork nothing is drawn.
const NO_FALLBACK: Rgba = Rgba {
    red: 0.0,
    green: 0.0,
    blue: 0.0,
    alpha: 0.0,
};
/// Counters show two digits.
pub const MAXIMUM_COUNTER_VALUE: u8 = 99;
/// Stage centers of the `coins`, `bombs` and `keys` fields (795–797 at (220.35, 13.2 / 32.85 /
/// 52.5), plus the center of their bounds, (72, 10.05)).
const COINS_CENTER: Vec2 = Vec2::new(292.35, 23.25);
const BOMBS_CENTER: Vec2 = Vec2::new(292.35, 42.9);
const KEYS_CENTER: Vec2 = Vec2::new(292.35, 62.55);
/// `kep` (802) at (252.2, 50.65): frame 1 the key, frame 2 the golden key once held. Its 94 %
/// vertical scale is left out.
const KEY_ICON_POSITION: Vec2 = Vec2::new(252.2, 50.65);
/// `lif` (610) sits at (459.4, 25.9); its first frame holds 12 hearts `h0`–`h11`, 27 apart, in
/// rows at y 0.5 and 23. More hearts than that switch it to smaller layouts (not supported yet).
const LIFE_BAR_ORIGIN: Vec2 = Vec2::new(459.4, 25.9);
const FIRST_HEART_OFFSET: Vec2 = Vec2::new(-0.1, 0.5);
const HEART_SPACING: Vec2 = Vec2::new(27.0, 22.5);
const HEARTS_PER_ROW: u8 = 6;
const HEART_SLOTS: usize = 12;

/// A frame of the heart clip (609).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HudHeart {
    Full,
    Half,
    Empty,
    SoulFull,
    SoulHalf,
    Eternal,
}

impl HudHeart {
    pub const ALL: [Self; 6] = [
        Self::Full,
        Self::Half,
        Self::Empty,
        Self::SoulFull,
        Self::SoulHalf,
        Self::Eternal,
    ];
}

pub fn append_draw_items(
    appearance: RoomAppearance,
    style: Style,
    input_device: InputDevice,
    inventory: &Inventory,
    overlay: &overlay::Overlay,
    draw_items: &mut Vec<DrawItem>,
) {
    if appearance != RoomAppearance::Artwork {
        return;
    }
    // Only the original's HUD exists so far; Rebirth's is still to be made.
    match style {
        Style::Original | Style::Rebirth => {}
    }
    let textured = |texture_id, stage_point, depth| {
        DrawItem::textured(texture_id, stage_to_world(stage_point), 1.0, NO_FALLBACK, depth)
    };
    draw_items.push(DrawItem::textured(
        TextureId::Hud,
        STAGE_CENTER,
        1.0,
        NO_FALLBACK,
        HUD_DEPTH,
    ));
    for (count, center) in [
        (inventory.coins, COINS_CENTER),
        (inventory.bombs, BOMBS_CENTER),
        (inventory.keys, KEYS_CENTER),
    ] {
        let value = count.min(MAXIMUM_COUNTER_VALUE);
        draw_items.push(textured(TextureId::HudCounter(value), center, HUD_VALUE_DEPTH));
    }
    let key_icon = TextureId::HudKeyIcon {
        golden: inventory.golden_key,
    };
    draw_items.push(textured(key_icon, KEY_ICON_POSITION, HUD_VALUE_DEPTH));
    draw_items.push(active_item_label::draw_item(input_device));
    draw_items.push(overlay.item_box.draw_item(inventory.active.held()));
    append_held(inventory, overlay, input_device, draw_items);
    for (slot, heart) in (0_u8..).zip(life_bar(inventory)) {
        let Some(heart) = heart else { continue };
        let grid = Vec2::new(f32::from(slot % HEARTS_PER_ROW), f32::from(slot / HEARTS_PER_ROW));
        let center = LIFE_BAR_ORIGIN + FIRST_HEART_OFFSET + grid * HEART_SPACING;
        draw_items.push(textured(TextureId::HudHeart(heart), center, HUD_VALUE_DEPTH));
    }
    draw_items.push(DrawItem::textured(
        TextureId::StageSurround,
        STAGE_CENTER,
        1.0,
        NO_FALLBACK,
        SURROUND_DEPTH,
    ));
}

/// The card or pill held, and the trinket.
fn append_held(
    inventory: &Inventory,
    overlay: &overlay::Overlay,
    input_device: InputDevice,
    draw_items: &mut Vec<DrawItem>,
) {
    held_card::append_draw_items(
        inventory.held_textures(),
        overlay.slot_animation(),
        input_device,
        draw_items,
    );
    held_trinket::append_draw_items(inventory.trinkets(), overlay.trinket_slot_animation(), draw_items);
}

/// The HUD update's heart loop: red heart containers first (full, half or empty from `hp`), then
/// the eternal heart and soul hearts (`armor`) in the slots after them.
pub fn life_bar(inventory: &Inventory) -> [Option<HudHeart>; HEART_SLOTS] {
    let mut slots = [None; HEART_SLOTS];
    let red_containers = crate::whole_numbers::clamped_i32(inventory.maximum_red_hearts.get().ceil());
    let eternal_halves = if inventory.eternal_heart {
        Hearts::ONE.halves()
    } else {
        0
    };
    let mut extra_halves = inventory.soul_hearts.halves() + eternal_halves;
    let mut eternal_pending = inventory.eternal_heart;
    for (slot, heart) in (0..).zip(slots.iter_mut()) {
        *heart = if slot < red_containers {
            Some(match inventory.red_hearts.halves() - slot * Hearts::ONE.halves() {
                2.. => HudHeart::Full,
                1 => HudHeart::Half,
                _ => HudHeart::Empty,
            })
        } else if extra_halves > 0 {
            extra_halves -= Hearts::ONE.halves();
            if std::mem::take(&mut eternal_pending) {
                Some(HudHeart::Eternal)
            } else if extra_halves >= 0 {
                Some(HudHeart::SoulFull)
            } else {
                Some(HudHeart::SoulHalf)
            }
        } else {
            None
        };
    }
    slots
}

/// What the view masks leave open: the stage, or with artwork the stage and its surround.
pub fn view_opening(appearance: RoomAppearance) -> Vec2 {
    match appearance {
        RoomAppearance::Artwork => TextureId::StageSurround.world_size(),
        RoomAppearance::Placeholder => STAGE_SIZE,
    }
}
