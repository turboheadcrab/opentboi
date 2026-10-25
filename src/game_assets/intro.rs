//! The original's opening (root frame label `intro`): sprite 7317, cards for the authors,
//! Newgrounds, the composer and the title, over its own streamed music. Space skips it (on its
//! sixth frame, where it waits).

use std::sync::Arc;

use swf::{CharacterId, Tag};

use super::vector_art::{
    Canvas, CharacterLibrary, FrameSelection, NO_OVERRIDES, RenderError, RgbaImage, UnsupportedFeatureCounts,
    render_frame,
};

const INTRO_SPRITE: CharacterId = 7317;
/// Its last frame moves the main timeline on.
#[cfg_attr(
    not(feature = "sdl3-frontend"),
    allow(
        dead_code,
        reason = "only the SDL3 frontend plays the intro; tests use it either way"
    )
)]
pub const INTRO_FRAME_COUNT: u16 = 373;
/// The Flash stage the intro was laid out on.
#[cfg_attr(
    not(feature = "sdl3-frontend"),
    allow(
        dead_code,
        reason = "only the SDL3 frontend plays the intro; tests use it either way"
    )
)]
pub const STAGE_WIDTH: f32 = 800.0;
#[cfg_attr(
    not(feature = "sdl3-frontend"),
    allow(
        dead_code,
        reason = "only the SDL3 frontend plays the intro; tests use it either way"
    )
)]
pub const STAGE_HEIGHT: f32 = 600.0;
/// Root frame 6 places the sprite at (-84.6, -74.9) on the stage.
const PLACEMENT_X: f32 = -84.6;
const PLACEMENT_Y: f32 = -74.9;
/// MP3 `SoundStreamBlock`s start with a sample count and a seek field (16 bits each).
const MP3_STREAM_BLOCK_HEADER_SIZE: usize = 4;
/// Frames are drawn every 33 ms while the game loads, so the intro uses MEDIUM quality (2×2).
const INTRO_SUPERSAMPLING: u32 = 2;

/// Renders intro frame `frame_index` (0-based) covering the whole stage at `output_width` pixels.
#[cfg_attr(
    not(feature = "sdl3-frontend"),
    allow(
        dead_code,
        reason = "only the SDL3 frontend plays the intro; tests use it either way"
    )
)]
pub fn render_intro_frame(
    library: &CharacterLibrary,
    frame_index: u16,
    output_width: u32,
    unsupported: &mut UnsupportedFeatureCounts,
) -> Result<RgbaImage, RenderError> {
    let canvas = Canvas {
        center_x: STAGE_WIDTH / 2.0 - PLACEMENT_X,
        center_y: STAGE_HEIGHT / 2.0 - PLACEMENT_Y,
        width: STAGE_WIDTH,
        height: STAGE_HEIGHT,
        output_pixels_per_unit: output_width as f32 / STAGE_WIDTH,
        maximum_supersampling: INTRO_SUPERSAMPLING,
    };
    render_frame(
        library,
        INTRO_SPRITE,
        FrameSelection::start_of(frame_index + 1),
        &canvas,
        &NO_OVERRIDES,
        unsupported,
    )
}

/// The intro's streamed soundtrack joined into one standalone MP3, if it is there.
pub fn intro_music(movie: &swf::Swf) -> Option<Arc<[u8]>> {
    let sprite_tags = movie.tags.iter().find_map(|tag| match tag {
        Tag::DefineSprite(sprite) if sprite.id == INTRO_SPRITE => Some(&sprite.tags),
        _ => None,
    })?;
    let is_mp3 = sprite_tags.iter().any(|tag| {
        matches!(tag, Tag::SoundStreamHead(head) | Tag::SoundStreamHead2(head)
            if head.stream_format.compression == swf::AudioCompression::Mp3)
    });
    if !is_mp3 {
        return None;
    }
    let mp3: Vec<u8> = sprite_tags
        .iter()
        .filter_map(|tag| match tag {
            Tag::SoundStreamBlock(block) => block.get(MP3_STREAM_BLOCK_HEADER_SIZE..),
            _ => None,
        })
        .flatten()
        .copied()
        .collect();
    (!mp3.is_empty()).then(|| Arc::from(mp3))
}

#[cfg(test)]
mod tests;
