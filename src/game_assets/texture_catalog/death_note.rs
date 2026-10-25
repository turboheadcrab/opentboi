//! The death note: the pause clip `hud.paus` (1422) on frame 3, a diary page ("Dear diary, today I
//! died") with the killer's portrait `kill` (1168, on the killer's frame), the character's name
//! `nam` (1071, on the character's frame), the floor `loco` (1063, on the chapter's frame) and the
//! items left to Guppy (`i0`–`i17`, 1052: only items collected show, and there are none yet), and
//! the "Replay?" / "Back to main menu" note.

use std::sync::OnceLock;

use swf::CharacterId;

use super::TextureSource;
use crate::game::{Killer, TextureId};
use crate::game_assets::vector_art::{ChildState, FrameSelection, RenderOverrides};

const PAUSE_CLIP: CharacterId = 1422;
const DEATH_NOTE_FRAME: u16 = 3;
/// The item slots, hidden while no item is collected (`itit()`).
const ITEM_SLOT_CLIP: CharacterId = 1052;
/// `hud.paus` sits at (89.2, 140.65) in the HUD, which sits at the stage's origin; the texture is
/// centered on the stage's center (320, 240).
const STAGE_CENTER_IN_CLIP: (f32, f32) = (320.0 - 89.2, 240.0 - 140.65);

/// The portrait is the frame of the killer's type number, as the original sets it (`kill` goes to
/// the last thing that hurt Isaac), so the overrides are made from the killers themselves, once.
fn overrides_for(killer: Killer) -> &'static RenderOverrides {
    static NOTES: OnceLock<Vec<RenderOverrides>> = OnceLock::new();
    let notes = NOTES.get_or_init(|| {
        Killer::ALL
            .into_iter()
            .map(|each| RenderOverrides {
                // Isaac (`skiner` 1) in the Basement (`chaps` 1).
                children: Box::leak(Box::new([
                    ("kill", ChildState::Frame(each.portrait_frame())),
                    ("nam", ChildState::Frame(1)),
                    ("loco", ChildState::Frame(1)),
                ])),
                texts: &[],
                hidden_characters: &[ITEM_SLOT_CLIP],
                character_frames: &[],
                tint: None,
            })
            .collect()
    });
    let index = Killer::ALL.iter().position(|&each| each == killer).unwrap_or(0);
    &notes[index]
}

/// The death note, or the beam over dying Isaac (part of his animations).
pub(super) fn source_for(texture_id: TextureId) -> TextureSource {
    match texture_id {
        TextureId::DeathSpotlight(step) => super::isaac_animations::spotlight_source(step),
        TextureId::DeathNote(killer) => source(killer),
        _ => source(Killer::Bomb),
    }
}

fn source(killer: Killer) -> TextureSource {
    TextureSource {
        character_id: PAUSE_CLIP,
        selection: FrameSelection::start_of(DEATH_NOTE_FRAME),
        origin_x: STAGE_CENTER_IN_CLIP.0,
        origin_y: STAGE_CENTER_IN_CLIP.1,
        overrides: overrides_for(killer),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_killer_shows_its_portrait() {
        for killer in Killer::ALL {
            assert_eq!(
                overrides_for(killer).children[0],
                ("kill", ChildState::Frame(killer.portrait_frame()))
            );
        }
    }
}
