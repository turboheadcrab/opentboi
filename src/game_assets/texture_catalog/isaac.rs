//! Isaac (`b1`, 5825): head clip 2261 sits at (-21.1, -44.4) from his floor point (5592 → 3069 →
//! 2261), and the unicorn's head 2803 (3069's frame 19, the look while riding) in the same place;
//! the body `bo` mirrors around x = 0 and its walk clip 4308 sits at (-1.05, -7.4), with walk
//! cycles 4307 and 1620 a further (-17.1, -9.7) and (-18.35, -9.7) in. Each texture is centered
//! where the game draws it: head 24 px and body 8 px above the floor point.

use swf::CharacterId;

use super::{TextureSource, centered_on, isaac_demonic};
use crate::game::{IsaacPart, RideColor};
use crate::game_assets::vector_art::{RenderOverrides, Tint};

const ISAAC_HEAD_CLIP: CharacterId = 2261;
const UNICORN_HEAD_CLIP: CharacterId = 2803;
const ISAAC_HEAD_CENTER: (f32, f32) = (21.1, 44.4 - 24.0);
const ISAAC_BODY_CLIP: CharacterId = 4308;
const ISAAC_BODY_STANDING_FRAME: u16 = 1;
const ISAAC_BODY_CENTER: (f32, f32) = (1.05, 7.4 - 8.0);
/// `bo` (5591), its frame per look: 3 for flying (with item 20); 4308 sits in it at
/// (-1.05, -0.05).
const ISAAC_BODY_OUTFITS_CLIP: CharacterId = 5591;
const ISAAC_FLYING_FRAME: u16 = 3;
const ISAAC_FLYING_BODY_CENTER: (f32, f32) = (0.0, 7.35 - 8.0);
const ISAAC_VERTICAL_WALK_CLIP: CharacterId = 4307;
const ISAAC_VERTICAL_WALK_CENTER: (f32, f32) = (1.05 + 17.1, 7.4 + 9.7 - 8.0);
const ISAAC_HORIZONTAL_WALK_CLIP: CharacterId = 1620;
const ISAAC_HORIZONTAL_WALK_CENTER: (f32, f32) = (1.05 + 18.35, 7.4 + 9.7 - 8.0);

/// The ride's flashes (`playcol()`): one or two channels kept, all three lifted by 50 (or 70 for
/// the two-channel colors).
const fn ride_tint(multiply_per_mille: [u16; 3], lift: i16) -> RenderOverrides {
    RenderOverrides {
        children: &[],
        texts: &[],
        hidden_characters: &[],
        character_frames: &[],
        tint: Some(Tint {
            multiply_per_mille,
            add: [lift; 3],
        }),
    }
}

/// By `RideColor`.
static RIDE_TINTS: [RenderOverrides; 5] = [
    ride_tint([1000, 0, 0], 50),
    ride_tint([0, 1000, 0], 50),
    ride_tint([0, 0, 1000], 50),
    ride_tint([1000, 1000, 0], 70),
    ride_tint([0, 1000, 1000], 70),
];

pub(super) fn source(part: IsaacPart, tint: Option<RideColor>) -> TextureSource {
    let source = untinted_source(part);
    let Some(color) = tint else {
        return source;
    };
    let index = RideColor::ALL.iter().position(|&each| each == color).unwrap_or(0);
    TextureSource {
        overrides: &RIDE_TINTS[index],
        ..source
    }
}

fn untinted_source(part: IsaacPart) -> TextureSource {
    match part {
        IsaacPart::Head(frame) => centered_on(ISAAC_HEAD_CLIP, 1 + u16::from(frame), ISAAC_HEAD_CENTER),
        IsaacPart::UnicornHead(frame) => centered_on(UNICORN_HEAD_CLIP, 1 + u16::from(frame), ISAAC_HEAD_CENTER),
        IsaacPart::BodyWalkVertical(frame) => centered_on(
            ISAAC_VERTICAL_WALK_CLIP,
            1 + u16::from(frame),
            ISAAC_VERTICAL_WALK_CENTER,
        ),
        IsaacPart::BodyWalkHorizontal(frame) => centered_on(
            ISAAC_HORIZONTAL_WALK_CLIP,
            1 + u16::from(frame),
            ISAAC_HORIZONTAL_WALK_CENTER,
        ),
        // Centered like the other body textures: `bo` sits 7.35 above `4308`'s placement.
        IsaacPart::FlyingBody => centered_on(ISAAC_BODY_OUTFITS_CLIP, ISAAC_FLYING_FRAME, ISAAC_FLYING_BODY_CENTER),
        IsaacPart::BodyStanding => centered_on(ISAAC_BODY_CLIP, ISAAC_BODY_STANDING_FRAME, ISAAC_BODY_CENTER),
        demonic => isaac_demonic::source(demonic),
    }
}
