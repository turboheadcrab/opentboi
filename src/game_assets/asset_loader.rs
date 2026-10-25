//! Reads the extracted SWF once and keeps it for the whole run, so sounds are collected up front
//! and textures can be rendered whenever the game first needs them.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use super::intro::{intro_music, render_intro_frame};
use super::room_layouts::basement_layouts;
use super::sound_bank::{MissingSounds, SoundBank, sound_bank_from_movie};
use super::texture_catalog::render_texture;
use super::vector_art::{CharacterLibrary, RenderError, RgbaImage, UnsupportedFeatureCounts};
use super::vector_mesh::ShapeMesh;
use super::vector_picture::MeshKey;
use crate::game::{PlayerAnimation, RoomLayouts, TextureId};

/// Renders game textures from the SWF on request. Lives as long as the process and is shared
/// between threads (rendering only reads the parsed movie).
pub struct TextureRenderer {
    pub(super) library: CharacterLibrary<'static>,
    /// Shapes meshed so far for vector drawing; `None` for shapes with fills meshes cannot show.
    pub(super) meshes: Mutex<HashMap<MeshKey, Option<Arc<ShapeMesh>>>>,
    /// Whether all frames of each of Isaac's animations can be drawn as meshes.
    pub(super) animations_drawable: Mutex<HashMap<PlayerAnimation, bool>>,
}

impl TextureRenderer {
    /// The image at `pixels_per_world_unit`, plus any art features it had to skip.
    pub fn render(
        &self,
        texture_id: TextureId,
        pixels_per_world_unit: f32,
    ) -> Result<(RgbaImage, UnsupportedFeatureCounts), RenderError> {
        let mut unsupported = UnsupportedFeatureCounts::default();
        let image = render_texture(&self.library, texture_id, pixels_per_world_unit, &mut unsupported)?;
        Ok((image, unsupported))
    }

    /// The Basement's normal and treasure room layouts.
    pub fn basement_room_layouts(&self) -> RoomLayouts {
        basement_layouts(&self.library)
    }

    /// One frame of the original's intro, covering its 800×600 stage at `output_width` pixels.
    #[cfg_attr(
        not(feature = "sdl3-frontend"),
        expect(dead_code, reason = "only the SDL3 frontend plays the intro")
    )]
    pub fn render_intro_frame(&self, frame_index: u16, output_width: u32) -> Result<RgbaImage, RenderError> {
        render_intro_frame(
            &self.library,
            frame_index,
            output_width,
            &mut UnsupportedFeatureCounts::default(),
        )
    }
}

/// What the frontend draws from: images rendered up front plus the renderer for everything else.
#[cfg_attr(
    not(feature = "bevy-frontend"),
    expect(dead_code, reason = "only the Bevy frontend loads up front")
)]
#[derive(Default)]
pub struct TextureSupply {
    pub prerendered: Vec<(TextureId, RgbaImage)>,
    pub renderer: Option<&'static TextureRenderer>,
    /// Density for textures rendered on first use, matching the prerendered ones.
    pub pixels_per_world_unit: f32,
}

/// Each part loads independently so one failure does not cost the other.
pub struct LoadedAssets {
    pub sound_bank: Result<SoundBank, MissingSounds>,
    pub texture_renderer: &'static TextureRenderer,
    /// The intro's soundtrack as one MP3, if the SWF has it.
    pub intro_music: Option<Arc<[u8]>>,
}

#[derive(Debug)]
pub enum SwfReadError {
    FileAccess(std::io::Error),
    InvalidSwf(String),
}

impl std::fmt::Display for SwfReadError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileAccess(io_error) => write!(formatter, "cannot read the SWF: {io_error}"),
            Self::InvalidSwf(reason) => write!(formatter, "cannot parse the SWF: {reason}"),
        }
    }
}

pub fn load_assets(swf_path: &Path) -> Result<LoadedAssets, SwfReadError> {
    let swf_bytes = std::fs::read(swf_path).map_err(SwfReadError::FileAccess)?;
    let swf_buffer =
        swf::decompress_swf(&swf_bytes[..]).map_err(|swf_error| SwfReadError::InvalidSwf(swf_error.to_string()))?;
    drop(swf_bytes);
    // The parsed movie borrows its buffer, and textures are rendered from it until the game exits,
    // so both are intentionally kept for the rest of the process.
    let swf_buffer: &'static swf::SwfBuf = Box::leak(Box::new(swf_buffer));
    let movie = swf::parse_swf(swf_buffer).map_err(|swf_error| SwfReadError::InvalidSwf(swf_error.to_string()))?;
    let movie: &'static swf::Swf<'static> = Box::leak(Box::new(movie));

    let texture_renderer = Box::leak(Box::new(TextureRenderer {
        library: CharacterLibrary::new(movie),
        meshes: Mutex::default(),
        animations_drawable: Mutex::default(),
    }));
    Ok(LoadedAssets {
        sound_bank: sound_bank_from_movie(movie),
        texture_renderer,
        intro_music: intro_music(movie),
    })
}

#[cfg(test)]
mod tests;
