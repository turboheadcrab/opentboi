//! A texture's frame as placed shapes, for drawing it with meshes instead of pixels.

use super::{cards, hud_values, source_for};
use crate::game::TextureId;
use crate::game_assets::vector_art::{CharacterLibrary, PlacedShape, place_frame};

/// The shapes `texture_id` shows, in Flash pixels around the texture's center; `None` if it holds
/// anything meshes cannot show yet.
pub(in crate::game_assets) fn place_texture(
    library: &CharacterLibrary,
    texture_id: TextureId,
) -> Option<Vec<PlacedShape>> {
    cards::read_names(library);
    hud_values::read_messages(library);
    let source = source_for(texture_id);
    place_frame(
        library,
        source.character_id,
        source.selection,
        (source.origin_x, source.origin_y),
        source.overrides,
    )
}
