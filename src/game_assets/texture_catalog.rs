//! Which SWF character becomes which game texture, and how it is framed.

mod cards;
mod clotty;
mod death_note;
mod decorations;
mod effects;
mod enemies;
mod friends;
mod gaper;
mod glyphs;
mod grid_objects;
mod ground_enemies;
mod headless;
mod horf;
mod hud;
mod hud_values;
mod isaac;
mod isaac_animations;
mod isaac_demonic;
mod machines;
mod mulligan;
mod pickups;
mod pills;
mod placement;
mod room_art;
mod shared_clips;
mod start_room_controls;
mod tears;
mod wares;

pub(super) use placement::place_texture;

use swf::CharacterId;

use super::vector_art::{
    Canvas, CharacterLibrary, FrameSelection, NO_OVERRIDES, RenderError, RenderOverrides, RgbaImage,
    SUPERSAMPLING_FACTOR, UnsupportedFeatureCounts, render_frame,
};
use crate::game::{RoomArt, TextureId};
use crate::game_assets::input_glyphs;

struct TextureSource {
    character_id: CharacterId,
    selection: FrameSelection,
    /// Point in the character's own coordinates (Flash pixels, y down) that lands on the center of
    /// the texture, i.e. on the `DrawItem` center.
    origin_x: f32,
    origin_y: f32,
    /// What a script would have changed in the clip (see `RenderOverrides`).
    overrides: &'static RenderOverrides,
}

/// The minimap piece's mark clip `itmc` (6562).
const MAP_PICKUP_MARK_CLIP: CharacterId = 6562;
const ROOM_CLIP: CharacterId = 6468;
/// Room clip point at the center of the walkable floor (see `RoomBackground`).
const ROOM_CENTER: (f32, f32) = (320.75, 195.25);

#[allow(clippy::too_many_lines, reason = "one arm for each family of textures")]
fn source_for(texture_id: TextureId) -> TextureSource {
    match texture_id {
        TextureId::RoomBackground(art) => room_source_for(art),
        TextureId::StartRoomControls | TextureId::StartRoomHint(..) => start_room_controls::source_for(texture_id),
        TextureId::ActiveItemLabel(_) => hud::active_item_label_source(),
        TextureId::HeldCardLabel(_) => cards::label_source(),
        TextureId::Pickup { kind, phase, frame } => pickups::source(kind, phase, frame),
        TextureId::ShopWare(frame) => wares::ware_source(frame),
        TextureId::ShopConsumableWare { color, on_sale } => wares::consumable_ware_source(color, on_sale),
        TextureId::HudGlyph(glyph) => glyphs::source(glyph),
        TextureId::Decoration { .. } => decorations::source_for(texture_id),
        TextureId::Gaper { .. } => gaper::source_for(texture_id),
        TextureId::Headless { .. } => headless::source_for(texture_id),
        TextureId::Horf { .. } => horf::source_for(texture_id),
        TextureId::BlueFly(frame) => friends::blue_fly_source(frame),
        TextureId::BloodCreep(look) => friends::blood_creep_source(look),
        TextureId::Clotty { .. } => clotty::source_for(texture_id),
        TextureId::Mulligan { .. } => mulligan::source_for(texture_id),
        TextureId::Enemy(texture) => ground_enemies::source_for(texture),
        TextureId::Fly { .. } | TextureId::EnemyShot(_) | TextureId::BloodSplash(_) => enemies::source_for(texture_id),
        TextureId::NameBanner { .. }
        | TextureId::HeldCard(_)
        | TextureId::HeldCardName(_)
        | TextureId::HeldPill(_)
        | TextureId::HeldPillName(_)
        | TextureId::CardSlotAnimation { .. }
        | TextureId::CardAnimation { .. } => cards::source_for(texture_id),
        TextureId::Machine { .. } => machines::source_for(texture_id),
        TextureId::DeathNote(_) | TextureId::DeathSpotlight(_) => death_note::source_for(texture_id),
        TextureId::Hud
        | TextureId::StageSurround
        | TextureId::MapPiece(_)
        | TextureId::MapPickupMark(_)
        | TextureId::HudCounter(_)
        | TextureId::HudHeart(_)
        | TextureId::HudKeyIcon { .. }
        | TextureId::ActiveItemBox { .. }
        | TextureId::EternalHeartUnion(_)
        | TextureId::PickupMessage { .. } => hud_source_for(texture_id),
        TextureId::Tile(_)
        | TextureId::KeyBlock
        | TextureId::BreakableStage { .. }
        | TextureId::BreakableDestroyed { .. }
        | TextureId::BreakableRegrow { .. }
        | TextureId::KeyBlockOpening(_)
        | TextureId::BrokenFloor
        | TextureId::ClearedTile(_) => grid_objects::source_for(texture_id),
        TextureId::Door(_)
        | TextureId::OrbitingFly(_)
        | TextureId::Explosion(_)
        | TextureId::ExplosionScorch
        | TextureId::BombFuse(_) => effects::source_for(texture_id),
        TextureId::Tear
        | TextureId::DarkTear
        | TextureId::TearShadow
        | TextureId::TearHitSplash(_)
        | TextureId::TearGroundSplash(_) => tears::source_for(texture_id),
        TextureId::IsaacAnimation { animation, frame } => isaac_animations::source(animation, frame),
        TextureId::Isaac { part, tint } => isaac::source(part, tint),
    }
}

/// Room art: the room clip (floor clip `bg` (6314) plus wall clips `w1`–`w4` (6400, 6466), one style
/// per chapter, centered on the walkable floor, halfway between the walls' inner edges (x 61.7–579.8,
/// y 54.9–335.6), with the start room's drawing (see `start_room_controls`) separate.
fn room_source_for(art: RoomArt) -> TextureSource {
    TextureSource {
        character_id: ROOM_CLIP,
        selection: FrameSelection::start_of(1),
        origin_x: ROOM_CENTER.0,
        origin_y: ROOM_CENTER.1,
        overrides: room_art::overrides(art),
    }
}

/// The HUD and what is drawn over and around it.
fn hud_source_for(texture_id: TextureId) -> TextureSource {
    match texture_id {
        TextureId::MapPiece(piece) => hud::map_piece_source(piece),
        TextureId::MapPickupMark(mark) => framed(MAP_PICKUP_MARK_CLIP, u16::from(mark)),
        TextureId::HudCounter(value) => hud_values::counter_source(value),
        TextureId::HudHeart(heart) => hud_values::heart_source(heart),
        TextureId::HudKeyIcon { golden } => hud_values::key_icon_source(golden),
        TextureId::ActiveItemBox { item, frame } => hud::item_box_source(item, frame),
        TextureId::EternalHeartUnion(frame) => hud_values::eternal_heart_union_source(frame),
        TextureId::PickupMessage { message, frame } => hud_values::message_source(message, frame),
        TextureId::StageSurround => hud::stage_surround_source(),
        _ => hud::source(),
    }
}

/// A small sprite whose texture center is `center` in the character's own coordinates.
fn centered_on(character_id: CharacterId, frame_number: u16, center: (f32, f32)) -> TextureSource {
    TextureSource {
        character_id,
        selection: FrameSelection::start_of(frame_number),
        origin_x: center.0,
        origin_y: center.1,
        overrides: &NO_OVERRIDES,
    }
}

/// A character drawn around its own origin.
fn framed(character_id: CharacterId, frame_number: u16) -> TextureSource {
    TextureSource {
        character_id,
        selection: FrameSelection::start_of(frame_number),
        origin_x: 0.0,
        origin_y: 0.0,
        overrides: &NO_OVERRIDES,
    }
}

/// Renders `texture_id` covering exactly its `world_size`, centered on its origin, with
/// `pixels_per_world_unit` pixels per world unit (the screen's density, so the art is never
/// shrunk or stretched).
pub fn render_texture(
    library: &CharacterLibrary,
    texture_id: TextureId,
    pixels_per_world_unit: f32,
    unsupported: &mut UnsupportedFeatureCounts,
) -> Result<RgbaImage, RenderError> {
    if let Some(layers) = input_glyphs::glyph_layers(texture_id) {
        return input_glyphs::render_layers(&layers?, texture_id.world_size(), pixels_per_world_unit);
    }
    cards::read_names(library);
    hud_values::read_messages(library);
    let source = source_for(texture_id);
    let world_size = texture_id.world_size();
    let pixels_per_world_unit = match texture_id {
        // A bitmap gains nothing from more pixels than it has, and the soft full-stage overlay
        // would take hundreds of megabytes at screen density; the frontend scales both up.
        TextureId::StageSurround | TextureId::EternalHeartUnion(_) => {
            pixels_per_world_unit.min(hud::STAGE_SURROUND_PIXELS_PER_UNIT)
        }
        _ => pixels_per_world_unit,
    };
    let canvas = Canvas {
        center_x: source.origin_x,
        center_y: source.origin_y,
        width: world_size.x,
        height: world_size.y,
        output_pixels_per_unit: pixels_per_world_unit,
        maximum_supersampling: SUPERSAMPLING_FACTOR,
    };
    let mut image = render_frame(
        library,
        source.character_id,
        source.selection,
        &canvas,
        source.overrides,
        unsupported,
    )?;
    if texture_id == TextureId::StartRoomControls {
        start_room_controls::blank_hint_areas(&mut image, pixels_per_world_unit);
    }
    Ok(image)
}

#[cfg(test)]
mod tests;
