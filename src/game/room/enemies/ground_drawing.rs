//! Which frame of its clip the Hopper and the flesh mound show.

use crate::game::original_units::Frames;
use crate::game::room::enemies::{Walker, WalkerState};
use crate::game::texture_id::{
    EnemyTexture, FleshMoundAnimation, HOPPER_HOPPING_FRAME_COUNT, HopperAnimation, TextureId,
};

/// The Hopper: the hop clip's frame while it plays (from 1), else the sitting clip.
pub(super) fn hopper_texture(walker: &Walker) -> TextureId {
    let (animation, frame) = match walker.state {
        WalkerState::Appearing => (
            HopperAnimation::Appearing,
            walker.age.held_frame(HopperAnimation::Appearing.frame_count()),
        ),
        WalkerState::Active => match walker.hop_frame() {
            Some(clip_frame) => (
                HopperAnimation::Hopping,
                clip_frame.saturating_sub(1).min(HOPPER_HOPPING_FRAME_COUNT - 1),
            ),
            None => (HopperAnimation::Idle, 0),
        },
        WalkerState::Dying(frames) => (
            HopperAnimation::Dying,
            frames.held_frame(HopperAnimation::Dying.frame_count()),
        ),
    };
    TextureId::Enemy(EnemyTexture::Hopper {
        animation,
        frame,
        flashing: walker.is_flashing(),
    })
}

/// The flesh mound: appearing, then its walking clip around and around from when it was done.
pub(super) fn flesh_mound_texture(walker: &Walker, appearing_frames: Frames) -> TextureId {
    let (animation, frame) = match walker.state {
        WalkerState::Appearing => (
            FleshMoundAnimation::Appearing,
            walker.age.held_frame(FleshMoundAnimation::Appearing.frame_count()),
        ),
        WalkerState::Active => (
            FleshMoundAnimation::Walking,
            (walker.age - appearing_frames).looped_frame(FleshMoundAnimation::Walking.frame_count()),
        ),
        WalkerState::Dying(frames) => (
            FleshMoundAnimation::Dying,
            frames.held_frame(FleshMoundAnimation::Dying.frame_count()),
        ),
    };
    TextureId::Enemy(EnemyTexture::FleshMound {
        animation,
        frame,
        flashing: walker.is_flashing(),
    })
}
