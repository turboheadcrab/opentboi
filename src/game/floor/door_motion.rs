//! Doors slamming shut on a room with live enemies, and opening once it is cleared. Every door
//! goes to its closing clip on entry (frame 4, DoAction 5505; boss doors 16, treasure doors 32,
//! curse doors 36: see `motions.rs`). The doors are usable again as soon as no enemy lives, and
//! their clips open them 10 frames later (a count that restarts while enemies live): at once in the
//! room's first second, with `Door_Heavy_Open.mp` after its first 20 frames. An empty room's doors
//! are open from the first frame. Only a bombed-open hole stays usable meanwhile.

use crate::game::floor::Floor;
use glam::Vec2;

use crate::game::floor::plan::RoomRole;
use crate::game::original_units::{Frames, Pixels, distance};
use crate::game::room::cell_bounds;
use crate::game::room::doors::{DoorSide, DoorStyle};
use crate::game::sound_event::SoundEvent;
use crate::game::texture_id::{
    BLOWN_DOOR_FRAME_COUNT, DoorTexture, MOVING_DOOR_STYLES, TextureId, door_closing_frame_count,
    door_opening_frame_count,
};

/// Frame 14 of the plain and treasure doors' closing clips (`Door_Heavy_Close.mp`), from 0; the
/// boss door's clip is silent.
const CLOSING_SOUND_FRAME: Frames = Frames(13.0);
const OPENING_SOUND_AFTER: Frames = Frames(20.0);
/// Frames without enemies before the doors open.
const CLEAR_FRAMES_BEFORE_OPENING: Frames = Frames(10.0);
/// The doors of a room cleared this early open at once.
const INSTANT_OPENING_BEFORE: Frames = Frames(30.0);

/// The current room's doors: how they move, and which a blast opened while they were shut.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RoomDoors {
    pub motion: DoorMotion,
    /// `blown`, with the time since, for the smoke clip.
    pub blown: Vec<(DoorSide, Frames)>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum DoorMotion {
    #[default]
    Open,
    /// Shutting (then shut once the clip ends), this far in; `clear` counts the frames since the
    /// last enemy died.
    Closing {
        frames: Frames,
        clear: Frames,
    },
    Opening(Frames),
}

impl Floor {
    /// On arrival: the doors shut behind the player if enemies live here.
    pub(super) fn set_doors_for_arrival(&mut self) {
        self.door_motion.blown.clear();
        self.door_motion.motion = if self.current_room().enemies.any_alive() {
            DoorMotion::Closing {
                frames: Frames::ZERO,
                clear: Frames::ZERO,
            }
        } else {
            DoorMotion::Open
        };
    }

    /// Plays the doors' clips, and opens them once the room's enemies are all dead.
    pub fn update_doors(&mut self, elapsed: Frames, frames_in_room: Frames, sounds: &mut Vec<SoundEvent>) {
        for (_, frames) in &mut self.door_motion.blown {
            *frames += elapsed;
        }
        self.door_motion.motion = match self.door_motion.motion {
            DoorMotion::Closing { frames, clear } => self.close(frames, clear, elapsed, frames_in_room, sounds),
            DoorMotion::Opening(frames) if frames + elapsed >= longest_opening() => DoorMotion::Open,
            DoorMotion::Opening(frames) => DoorMotion::Opening(frames + elapsed),
            DoorMotion::Open => DoorMotion::Open,
        };
    }

    fn close(
        &self,
        frames: Frames,
        clear: Frames,
        elapsed: Frames,
        frames_in_room: Frames,
        sounds: &mut Vec<SoundEvent>,
    ) -> DoorMotion {
        let clear = if self.current_room().enemies.any_alive() {
            Frames::ZERO
        } else {
            clear + elapsed
        };
        if clear >= CLEAR_FRAMES_BEFORE_OPENING {
            if frames_in_room > OPENING_SOUND_AFTER {
                sounds.push(SoundEvent::DoorsOpened);
            }
            return if frames_in_room < INSTANT_OPENING_BEFORE {
                DoorMotion::Open
            } else {
                DoorMotion::Opening(Frames::ZERO)
            };
        }
        let next = frames + elapsed;
        let slams = self
            .doors()
            .any(|side| matches!(self.door_style(side), DoorStyle::Plain | DoorStyle::Treasure));
        if slams && frames < CLOSING_SOUND_FRAME && next >= CLOSING_SOUND_FRAME {
            sounds.push(SoundEvent::DoorsClosed);
        }
        DoorMotion::Closing { frames: next, clear }
    }

    /// Enemies keep the doors shut (`ashut > 0`).
    pub(super) fn doors_shut(&self) -> bool {
        matches!(self.door_motion.motion, DoorMotion::Closing { .. }) && self.current_room().enemies.any_alive()
    }

    /// A blast at `center` reaching a shut plain door's doorway blows it open for good (door frame
    /// 5); other doors' cells are too strong (only plain doors' doorways can be blown), and in the
    /// mini-boss room the doorway closes again.
    pub fn blow_doors(&mut self, center: Vec2, radius: Pixels) {
        let shut =
            matches!(self.door_motion.motion, DoorMotion::Closing { clear, .. } if clear < CLEAR_FRAMES_BEFORE_OPENING);
        if !shut || self.current_role() == RoomRole::MiniBoss {
            return;
        }
        let reached: Vec<DoorSide> = self
            .doors()
            .filter(|&side| self.door_style(side) == DoorStyle::Plain && !self.is_blown(side))
            .filter(|&side| {
                let (column, row) = side.doorway_cell();
                let doorway = cell_bounds(column, row);
                distance(doorway.clamp(center), center) <= radius
            })
            .collect();
        self.door_motion
            .blown
            .extend(reached.into_iter().map(|side| (side, Frames::ZERO)));
    }

    pub(super) fn is_blown(&self, side: DoorSide) -> bool {
        self.door_motion.blown.iter().any(|&(blown, _)| blown == side)
    }

    /// A blown door's smoke clip, this far in.
    pub(super) fn blown_door_texture(&self, side: DoorSide) -> Option<TextureId> {
        let &(_, frames) = self.door_motion.blown.iter().find(|&&(blown, _)| blown == side)?;
        Some(TextureId::Door(DoorTexture::Blown(
            frames.held_frame(BLOWN_DOOR_FRAME_COUNT),
        )))
    }

    /// The door's look while shutting, shut or opening; `None` when it simply stands open or has
    /// no clips for it.
    pub(super) fn moving_door_texture(&self, style: DoorStyle) -> Option<TextureId> {
        if !MOVING_DOOR_STYLES.contains(&style) {
            return None;
        }
        let texture = match self.door_motion.motion {
            DoorMotion::Open => return None,
            DoorMotion::Closing { frames, .. } => {
                DoorTexture::Closing(style, frames.held_frame(door_closing_frame_count(style)))
            }
            DoorMotion::Opening(frames) => {
                DoorTexture::Opening(style, frames.held_frame(door_opening_frame_count(style)))
            }
        };
        Some(TextureId::Door(texture))
    }
}

fn longest_opening() -> Frames {
    let longest = MOVING_DOOR_STYLES
        .map(door_opening_frame_count)
        .into_iter()
        .max()
        .unwrap_or(1);
    Frames::from(longest)
}

#[cfg(test)]
mod tests;
