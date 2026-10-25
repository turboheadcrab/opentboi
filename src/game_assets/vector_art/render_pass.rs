//! Walks a character tree for one frame, composing transforms and color effects.

use swf::CharacterId;
use tiny_skia::{Pixmap, Transform};

use super::FrameSelection;
use super::bitmap_cache::BitmapCache;
use super::character_library::{Character, CharacterLibrary};
use super::color_effect::ColorEffect;
use super::painter::{self, UnsupportedFeatureCounts};
use super::text::GlyphKey;
use super::timeline::SpriteTimeline;
use super::{ChildState, RenderOverrides};

const PER_MILLE: f32 = 1000.0;

/// What a pass draws into: pixels, or a list of the shapes and how each is placed (for meshes).
pub enum Target<'target> {
    Pixmap(&'target mut Pixmap),
    Placements(&'target mut Vec<PlacedShape>),
}

/// A shape as one frame of a clip places it: `transform` maps its twips to the pass's output.
#[derive(Clone, Debug)]
pub struct PlacedShape {
    pub source: ShapeSource,
    pub transform: Transform,
    pub color_effect: ColorEffect,
}

/// Where a placed shape comes from: the movie's characters, or a font's glyph laid out as text.
#[derive(Clone, Debug)]
pub enum ShapeSource {
    Character(CharacterId),
    Glyph(GlyphKey, Box<swf::Shape>),
}

/// Guards against cyclic sprite references in a corrupt SWF.
const MAXIMUM_NESTING_DEPTH: usize = 64;

pub struct RenderPass<'pass, 'movie> {
    library: &'pass CharacterLibrary<'movie>,
    bitmaps: BitmapCache,
    unsupported: &'pass mut UnsupportedFeatureCounts,
    nesting_depth: usize,
    /// Device pixels per output pixel (the supersampling factor).
    device_pixels_per_output_pixel: f32,
    overrides: &'pass RenderOverrides,
    /// Instance names from the rendered character down to the clip being drawn.
    name_path: Vec<String>,
}

impl<'pass, 'movie> RenderPass<'pass, 'movie> {
    pub fn new(
        library: &'pass CharacterLibrary<'movie>,
        unsupported: &'pass mut UnsupportedFeatureCounts,
        device_pixels_per_output_pixel: f32,
        overrides: &'pass RenderOverrides,
    ) -> Self {
        Self {
            library,
            bitmaps: BitmapCache::default(),
            unsupported,
            nesting_depth: 0,
            device_pixels_per_output_pixel,
            overrides,
            name_path: Vec::new(),
        }
    }

    pub fn render_character(
        &mut self,
        character_id: CharacterId,
        selection: FrameSelection,
        transform: Transform,
        color_effect: ColorEffect,
        target: &mut Target,
    ) {
        if self.nesting_depth > MAXIMUM_NESTING_DEPTH {
            return;
        }
        match self.library.character(character_id) {
            Some(Character::Shape(shape)) => match target {
                Target::Pixmap(pixmap) => {
                    self.bitmaps.prepare_bitmaps_used_by(shape, color_effect, self.library);
                    let mut paint_target = painter::PaintTarget {
                        pixmap,
                        minimum_stroke_pixels: self.device_pixels_per_output_pixel,
                    };
                    painter::paint_shape(
                        shape,
                        transform,
                        color_effect,
                        &mut paint_target,
                        &self.bitmaps,
                        self.unsupported,
                    );
                }
                Target::Placements(placements) => placements.push(PlacedShape {
                    source: ShapeSource::Character(shape.id),
                    transform,
                    color_effect,
                }),
            },
            Some(Character::Sprite(timeline)) => {
                self.render_sprite_frame(timeline, selection, transform, color_effect, target);
            }
            Some(Character::Button(button)) => self.render_button(button, transform, color_effect, target),
            Some(Character::Text(edit_text)) => self.render_text(edit_text, transform, color_effect, target),
            Some(Character::StaticText(text)) => self.render_static_text(text, transform, color_effect, target),
            None => self.unsupported.other_characters += 1,
        }
    }

    fn render_sprite_frame(
        &mut self,
        timeline: &SpriteTimeline,
        selection: FrameSelection,
        transform: Transform,
        color_effect: ColorEffect,
        target: &mut Target,
    ) {
        self.nesting_depth += 1;
        for placement in timeline.display_list(selection.frame_number) {
            let frames_since_placed =
                selection.frame_number.saturating_sub(placement.placed_on_frame) + selection.frames_since_frame_started;
            let name = placement
                .name
                .map(|name| name.to_str_lossy(swf::UTF_8).into_owned())
                .unwrap_or_default();
            self.name_path.push(name);
            let state = self.overridden_state();
            if state == Some(ChildState::Hidden) || self.overrides.hidden_characters.contains(&placement.character_id) {
                self.name_path.pop();
                continue;
            }
            let chosen_frame = match state {
                Some(ChildState::Frame(frame)) => Some(frame),
                _ => self
                    .overrides
                    .character_frames
                    .iter()
                    .find(|(character_id, _)| *character_id == placement.character_id)
                    .map(|&(_, frame)| frame),
            };
            let child_selection = match (chosen_frame, self.library.character(placement.character_id)) {
                // A clip a script stopped on a frame keeps playing its own children meanwhile.
                (Some(frame), _) => FrameSelection {
                    frame_number: frame,
                    frames_since_frame_started: frames_since_placed,
                },
                (None, Some(Character::Sprite(child_timeline))) => {
                    FrameSelection::start_of(child_timeline.frame_after(frames_since_placed))
                }
                (None, _) => FrameSelection::start_of(selection.frame_number),
            };
            let mut placement_transform = painter::to_skia_transform(&placement.matrix);
            if let Some(ChildState::HorizontalScale { per_mille }) = state {
                let current_scale = placement_transform.sx.hypot(placement_transform.ky);
                let factor = f32::from(per_mille) / PER_MILLE / current_scale.max(f32::EPSILON);
                placement_transform.sx *= factor;
                placement_transform.ky *= factor;
            }
            self.render_character(
                placement.character_id,
                child_selection,
                transform.pre_concat(placement_transform),
                color_effect.then_child(child_color_effect(state, placement.color_effect)),
                target,
            );
            self.name_path.pop();
        }
        self.nesting_depth -= 1;
    }

    /// Buttons show their "up" state when nothing hovers them.
    fn render_button(
        &mut self,
        button: &swf::Button,
        transform: Transform,
        color_effect: ColorEffect,
        target: &mut Target,
    ) {
        self.nesting_depth += 1;
        let mut up_records: Vec<_> = button
            .records
            .iter()
            .filter(|record| record.states.contains(swf::ButtonState::UP))
            .collect();
        up_records.sort_by_key(|record| record.depth);
        for record in up_records {
            self.render_character(
                record.id,
                FrameSelection::start_of(1),
                transform.pre_concat(painter::to_skia_transform(&record.matrix)),
                color_effect.then_child(ColorEffect::from_swf(&record.color_transform)),
                target,
            );
        }
        self.nesting_depth -= 1;
    }

    /// The override for the clip at the current name path, if any.
    fn overridden_state(&self) -> Option<ChildState> {
        if self.overrides.children.is_empty() {
            return None;
        }
        let path = self.name_path.join(".");
        self.overrides
            .children
            .iter()
            .find(|(child_path, _)| *child_path == path)
            .map(|&(_, state)| state)
    }
}

/// A placement's own color effect, then a brightness a script set on it.
fn child_color_effect(state: Option<ChildState>, own: ColorEffect) -> ColorEffect {
    match state {
        Some(ChildState::Brightness { per_mille }) => {
            ColorEffect::brightness(f32::from(per_mille) / PER_MILLE).then_child(own)
        }
        _ => own,
    }
}

mod text_pass;
