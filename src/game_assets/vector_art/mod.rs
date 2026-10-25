//! Converts the SWF's vector art into ordinary RGBA images on the player's machine.

mod bitmap_cache;
mod bitmap_decoder;
mod character_library;
mod color_effect;
mod painter;
mod render_pass;
mod shape_paths;
mod supersampling;
mod text;
mod timeline;

use swf::CharacterId;
use tiny_skia::{Pixmap, Transform};

pub use bitmap_decoder::decode_bitmap;
pub use character_library::{Character, CharacterLibrary};
pub use color_effect::ColorEffect;
pub use painter::UnsupportedFeatureCounts;
pub use render_pass::{PlacedShape, ShapeSource};
use render_pass::{RenderPass, Target};
pub use supersampling::SUPERSAMPLING_FACTOR;
pub use text::GlyphKey;
pub use timeline::Placement;

/// SWF coordinates are twips, 20 to a pixel.
pub const TWIPS_PER_PIXEL: f32 = 20.0;

/// Straight (non-premultiplied) sRGB RGBA, 4 bytes per pixel, rows top to bottom.
pub struct RgbaImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl RgbaImage {
    pub const BYTES_PER_PIXEL: usize = 4;
    /// Offset of the alpha byte within a pixel.
    #[cfg_attr(
        not(any(test, feature = "sdl3-frontend")),
        expect(dead_code, reason = "read by the SDL3 frontend and the tests")
    )]
    pub const ALPHA_CHANNEL: usize = 3;
}

/// Which part of a character to render, in its own Flash pixel coordinates (y down), and how sharply.
pub struct Canvas {
    pub center_x: f32,
    pub center_y: f32,
    pub width: f32,
    pub height: f32,
    pub output_pixels_per_unit: f32,
    /// Samples per output pixel along each axis, at most; see `supersampling` (Flash's HIGH quality
    /// is 4, MEDIUM 2).
    pub maximum_supersampling: u32,
}

#[derive(Debug)]
pub enum RenderError {
    NotDrawable(CharacterId),
    InvalidCanvasSize,
    /// An input glyph's SVG is not the kind of file `input_glyphs` reads.
    InvalidGlyph,
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotDrawable(character_id) => write!(formatter, "character {character_id} is not a shape or sprite"),
            Self::InvalidCanvasSize => write!(formatter, "canvas size is zero or too large"),
            Self::InvalidGlyph => write!(formatter, "an input glyph's SVG could not be read"),
        }
    }
}

/// Which moment of a clip to render: a timeline frame (1-based; ignored for shapes), and how many
/// frames its child clips have played while the clip stays on that frame. Clips stopped on a
/// frame keep animating their children, which is how idle loops and burst effects play.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameSelection {
    pub frame_number: u16,
    pub frames_since_frame_started: u16,
}

impl FrameSelection {
    pub const fn start_of(frame_number: u16) -> Self {
        Self {
            frame_number,
            frames_since_frame_started: 0,
        }
    }
}

/// What the original's scripts change before a clip is shown, which replaying the timeline alone
/// cannot know.
pub struct RenderOverrides {
    /// Descendants by dotted instance path from the rendered character (`lif.h3`).
    pub children: &'static [(&'static str, ChildState)],
    /// Text fields by variable name, and the text a script put in them.
    pub texts: &'static [(&'static str, &'static str)],
    /// Clips hidden wherever they appear, for scripts that hide their own clip.
    pub hidden_characters: &'static [CharacterId],
    /// Clips stopped on a frame wherever they appear, for scripts that pick their own frame
    /// (`sloter()`).
    pub character_frames: &'static [(CharacterId, u16)],
    /// A color transform a script set on the clip itself (`colorit()`).
    pub tint: Option<Tint>,
}

/// Red, green and blue multipliers in thousandths and offsets in Flash's 0–255 units; alpha is
/// kept.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tint {
    pub multiply_per_mille: [u16; 3],
    pub add: [i16; 3],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChildState {
    /// Stopped on this frame.
    Frame(u16),
    /// `child._visible = false`.
    Hidden,
    /// `child._xscale = per_mille / 10` (percent), keeping its rotation and position.
    HorizontalScale { per_mille: u16 },
    /// A color transform multiplying red, green and blue by `per_mille / 1000`
    /// (`redMultiplier = greenMultiplier = blueMultiplier`).
    Brightness { per_mille: u16 },
}

pub const NO_OVERRIDES: RenderOverrides = RenderOverrides {
    children: &[],
    texts: &[],
    hidden_characters: &[],
    character_frames: &[],
    tint: None,
};

/// Renders the selected moment of `character_id` into the given canvas.
pub fn render_frame(
    library: &CharacterLibrary,
    character_id: CharacterId,
    selection: FrameSelection,
    canvas: &Canvas,
    overrides: &RenderOverrides,
    unsupported: &mut UnsupportedFeatureCounts,
) -> Result<RgbaImage, RenderError> {
    let output_width = crate::whole_numbers::clamped_u32((canvas.width * canvas.output_pixels_per_unit).round());
    let output_height = crate::whole_numbers::clamped_u32((canvas.height * canvas.output_pixels_per_unit).round());
    let factor = supersampling::factor_for(output_width, output_height).min(canvas.maximum_supersampling.max(1));
    let mut pixmap =
        Pixmap::new(output_width * factor, output_height * factor).ok_or(RenderError::InvalidCanvasSize)?;
    if library.character(character_id).is_none() {
        return Err(RenderError::NotDrawable(character_id));
    }

    let pixels_per_unit = canvas.output_pixels_per_unit * factor as f32;
    let output_pixels_per_twip = pixels_per_unit / TWIPS_PER_PIXEL;
    let canvas_left = canvas.center_x - canvas.width / 2.0;
    let canvas_top = canvas.center_y - canvas.height / 2.0;
    let twips_to_output = Transform::from_row(
        output_pixels_per_twip,
        0.0,
        0.0,
        output_pixels_per_twip,
        -canvas_left * pixels_per_unit,
        -canvas_top * pixels_per_unit,
    );
    let mut render_pass = RenderPass::new(library, unsupported, factor as f32, overrides);
    render_pass.render_character(
        character_id,
        selection,
        twips_to_output,
        overrides.tint.map_or(ColorEffect::IDENTITY, ColorEffect::from_tint),
        &mut Target::Pixmap(&mut pixmap),
    );

    let output = supersampling::downsample(&pixmap, factor).ok_or(RenderError::InvalidCanvasSize)?;
    Ok(RgbaImage {
        width: output_width,
        height: output_height,
        pixels: output.take_demultiplied(),
    })
}

/// The shapes one frame of `character_id` places, each with the transform from its twips to Flash
/// pixels around `center`; `None` if the frame holds anything meshes cannot show yet.
pub fn place_frame(
    library: &CharacterLibrary,
    character_id: CharacterId,
    selection: FrameSelection,
    center: (f32, f32),
    overrides: &RenderOverrides,
) -> Option<Vec<PlacedShape>> {
    library.character(character_id)?;
    let pixels_per_twip = 1.0 / TWIPS_PER_PIXEL;
    let twips_to_pixels = Transform::from_row(pixels_per_twip, 0.0, 0.0, pixels_per_twip, -center.0, -center.1);
    let mut unsupported = UnsupportedFeatureCounts::default();
    let mut placements = Vec::new();
    RenderPass::new(library, &mut unsupported, 1.0, overrides).render_character(
        character_id,
        selection,
        twips_to_pixels,
        overrides.tint.map_or(ColorEffect::IDENTITY, ColorEffect::from_tint),
        &mut Target::Placements(&mut placements),
    );
    (unsupported == UnsupportedFeatureCounts::default()).then_some(placements)
}
