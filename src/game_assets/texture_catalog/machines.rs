//! Machines and beggars: frame 8 of the pickup clip `b5` (3376) holds the machine clip `d` (2246),
//! whose frame picks the machine and what it is doing; that frame's own child `d` is the animation,
//! one frame per game frame. The fortune machine's parts (2168, 2172) show their second frame and
//! the slot machine's their first (`sloter()`); the devil beggar's parts `p`, `p0`–`p4` show their
//! second frame, a beggar's their first.

use super::shared_clips::{FUSE_SPARK_CLIP, PICKUP_CLIP};

use super::{TextureSource, framed};
use crate::game::TextureId;
use crate::game_assets::vector_art::{ChildState, FrameSelection, RenderOverrides};

const MACHINE_PICKUP_FRAME: u16 = 8;

macro_rules! machine_overrides {
    ($part_frame:literal; $($frame:literal)*) => {
        [$(RenderOverrides {
            children: &[("d", ChildState::Frame($frame))],
            texts: &[],
            hidden_characters: &[],
            character_frames: &[(2168, $part_frame), (2172, $part_frame), (2175, $part_frame), (2206, $part_frame)],
            tint: None,
        }),*]
    };
}

/// Beggar frames 36–40, with the parts of the animation clip on `$part_frame`.
macro_rules! beggar_overrides {
    ($part_frame:literal; $($frame:literal)*) => {
        [$(RenderOverrides {
            children: &[("d", ChildState::Frame($frame))],
            texts: &[],
            hidden_characters: &[],
            // The parts by character, as some are placed without their `p` names (2236).
            character_frames: &[
                (1705, $part_frame),
                (1708, $part_frame),
                (1712, $part_frame),
                (1715, $part_frame),
                (2225, $part_frame),
                (2229, $part_frame),
                (2232, $part_frame),
            ],
            tint: None,
        }),*]
    };
}

const FIRST_BEGGAR_FRAME: u8 = 36;
const BEGGAR_FRAME_COUNT: usize = 5;
static BEGGAR: [RenderOverrides; BEGGAR_FRAME_COUNT] = beggar_overrides!(1; 36 37 38 39 40);
static DEVIL_BEGGAR: [RenderOverrides; BEGGAR_FRAME_COUNT] = beggar_overrides!(2; 36 37 38 39 40);

/// The shuffle shows the prize being played (one frame per prize), its own
/// pickup clips on their first variant.
macro_rules! shell_overrides {
    ($($prize_frame:literal)*) => {
        [$(RenderOverrides {
            children: &[
                ("d", ChildState::Frame(SHUFFLING_FRAME)),
                ("d.d.it", ChildState::Frame($prize_frame)),
                ("d.d.it.p", ChildState::Frame(1)),
                ("d.d.it.d.p", ChildState::Frame(1)),
                ("d.d.it.d.d.p", ChildState::Frame(1)),
            ],
            texts: &[],
            hidden_characters: &[FUSE_SPARK_CLIP],
            character_frames: &[],
            tint: None,
        }),*]
    };
}

const SHUFFLING_FRAME: u16 = 42;
/// Prizes 0–3 and 5; the item's frame is past the clip's last, which stays on 5.
const PRIZE_FRAME_COUNT: usize = 5;
static SHUFFLING: [RenderOverrides; PRIZE_FRAME_COUNT] = shell_overrides!(1 2 3 4 5);

const FRAME_COUNT: u8 = 45;
static SLOT: [RenderOverrides; FRAME_COUNT as usize] = machine_overrides!(1;
    1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30
    31 32 33 34 35 36 37 38 39 40 41 42 43 44 45);
static FORTUNE: [RenderOverrides; FRAME_COUNT as usize] = machine_overrides!(2;
    1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30
    31 32 33 34 35 36 37 38 39 40 41 42 43 44 45);

/// Several result frames show an animation placed on an earlier frame; the renderer counts its
/// frames from there, while the game restarts it on entering the frame.
fn first_frame_showing_the_same_animation(frame: u8) -> u8 {
    match frame {
        6 => 5,
        11 | 12 => 10,
        14..=16 => 13,
        18..=22 => 17,
        24..=29 => 23,
        _ => frame,
    }
}

/// `alternate` is the fortune machine's or the devil beggar's look, `prize` the shell game's.
pub(super) fn source_for(texture_id: TextureId) -> TextureSource {
    let TextureId::Machine {
        alternate,
        frame,
        prize,
        animation,
    } = texture_id
    else {
        unreachable!("not a machine texture: {texture_id:?}")
    };
    source(alternate, frame, prize, animation)
}

fn source(alternate: bool, frame: u8, prize: u8, animation: u8) -> TextureSource {
    let beggar_index = usize::from(frame.wrapping_sub(FIRST_BEGGAR_FRAME));
    let overrides = match (alternate, beggar_index < BEGGAR_FRAME_COUNT) {
        _ if u16::from(frame) == SHUFFLING_FRAME => &SHUFFLING[usize::from(prize).min(PRIZE_FRAME_COUNT - 1)],
        (false, true) => &BEGGAR[beggar_index],
        (true, true) => &DEVIL_BEGGAR[beggar_index],
        (false, false) => &SLOT[usize::from(frame.clamp(1, FRAME_COUNT) - 1)],
        (true, false) => &FORTUNE[usize::from(frame.clamp(1, FRAME_COUNT) - 1)],
    };
    let frames_after_placement = frame - first_frame_showing_the_same_animation(frame);
    TextureSource {
        selection: FrameSelection {
            frame_number: MACHINE_PICKUP_FRAME,
            frames_since_frame_started: u16::from(animation.saturating_sub(frames_after_placement)),
        },
        overrides,
        ..framed(PICKUP_CLIP, MACHINE_PICKUP_FRAME)
    }
}
