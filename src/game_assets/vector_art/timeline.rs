//! Replays a sprite's tags to find what is on stage at a given frame.
//!
//! Only the timeline itself is emulated, not ActionScript. The one script that matters for plain
//! animations, a frame action consisting only of `stop()`, is recognized so such clips hold.

use std::collections::BTreeMap;

use swf::{CharacterId, Depth, Matrix, PlaceObjectAction, Tag};

use super::color_effect::ColorEffect;

/// SWF action bytecode of a frame script that is exactly `stop();` (`ActionStop`, `ActionEndFlag`).
const STOP_ONLY_ACTION: [u8; 2] = [0x07, 0x00];
const FIRST_FRAME: u16 = 1;

#[derive(Clone, Debug)]
pub struct Placement<'movie> {
    pub character_id: CharacterId,
    /// Instance name (`_name`), which some clips' scripts read.
    pub name: Option<&'movie swf::SwfStr>,
    /// Maps the child's coordinates (twips) into the parent's coordinates (twips).
    pub matrix: Matrix,
    pub color_effect: ColorEffect,
    /// Parent frame on which this child appeared; child clips start playing there.
    pub placed_on_frame: u16,
}

pub struct SpriteTimeline<'movie> {
    tags: &'movie [Tag<'movie>],
    frame_count: u16,
    /// First frame whose only script is `stop()`; the clip holds there once it arrives.
    stop_frame: Option<u16>,
}

impl<'movie> SpriteTimeline<'movie> {
    pub fn new(tags: &'movie [Tag<'movie>], frame_count: u16) -> Self {
        let mut frame = FIRST_FRAME;
        let mut stop_frame = None;
        for tag in tags {
            match tag {
                Tag::ShowFrame => frame += 1,
                Tag::DoAction(actions) if *actions == STOP_ONLY_ACTION && stop_frame.is_none() => {
                    stop_frame = Some(frame);
                }
                _ => {}
            }
        }
        Self {
            tags,
            frame_count: frame_count.max(FIRST_FRAME),
            stop_frame,
        }
    }

    pub fn frame_count(&self) -> u16 {
        self.frame_count
    }

    /// Frame shown `frames_elapsed` frames after the clip was placed: plays from frame 1, then holds
    /// on a `stop()` frame or loops.
    pub fn frame_after(&self, frames_elapsed: u16) -> u16 {
        match self.stop_frame {
            Some(stop_frame) => FIRST_FRAME.saturating_add(frames_elapsed).min(stop_frame),
            None => FIRST_FRAME + frames_elapsed % self.frame_count,
        }
    }

    /// Children on stage at `frame_number` (1-based), back to front.
    pub fn display_list(&self, frame_number: u16) -> Vec<Placement<'movie>> {
        let mut placements_by_depth: BTreeMap<Depth, Placement<'movie>> = BTreeMap::new();
        let mut frame = FIRST_FRAME;
        for tag in self.tags {
            match tag {
                Tag::ShowFrame => {
                    if frame == frame_number {
                        break;
                    }
                    frame += 1;
                }
                Tag::PlaceObject(place_object) => apply_place_object(&mut placements_by_depth, place_object, frame),
                Tag::RemoveObject(remove_object) => {
                    placements_by_depth.remove(&remove_object.depth);
                }
                _ => {}
            }
        }
        placements_by_depth.into_values().collect()
    }
}

fn apply_place_object<'movie>(
    placements_by_depth: &mut BTreeMap<Depth, Placement<'movie>>,
    place_object: &swf::PlaceObject<'movie>,
    frame: u16,
) {
    let color_effect = place_object.color_transform.as_ref().map(ColorEffect::from_swf);
    match place_object.action {
        PlaceObjectAction::Place(character_id) | PlaceObjectAction::Replace(character_id) => {
            let previous = placements_by_depth.get(&place_object.depth);
            let placement = Placement {
                character_id,
                name: place_object.name.or(previous.and_then(|placement| placement.name)),
                matrix: place_object
                    .matrix
                    .or(previous.map(|placement| placement.matrix))
                    .unwrap_or(Matrix::IDENTITY),
                color_effect: color_effect
                    .or(previous.map(|placement| placement.color_effect))
                    .unwrap_or(ColorEffect::IDENTITY),
                placed_on_frame: frame,
            };
            placements_by_depth.insert(place_object.depth, placement);
        }
        PlaceObjectAction::Modify => {
            if let Some(placement) = placements_by_depth.get_mut(&place_object.depth) {
                if let Some(matrix) = place_object.matrix {
                    placement.matrix = matrix;
                }
                if let Some(color_effect) = color_effect {
                    placement.color_effect = color_effect;
                }
            }
        }
    }
}
