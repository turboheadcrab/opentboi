//! Which frame of its clip the Clotty and the Mulligan show.

use crate::game::original_units::Frames;
use crate::game::room::enemies::{Walker, WalkerClip, WalkerState};
use crate::game::texture_id::{ClottyAnimation, MulliganAnimation, TextureId};

/// The loops play over and over; the rest hold their last frame.
fn frame_of(frames: Frames, count: u8, loops: bool) -> u8 {
    if loops {
        frames.looped_frame(count)
    } else {
        frames.held_frame(count)
    }
}

pub(super) fn clotty_texture(walker: &Walker) -> TextureId {
    let (animation, frames) = match (walker.state, walker.clip) {
        (WalkerState::Appearing, _) => (ClottyAnimation::Appearing, walker.age),
        (WalkerState::Active, WalkerClip::Standing) => (ClottyAnimation::Idle, walker.clip_age),
        (WalkerState::Active, WalkerClip::Walking { .. }) => (ClottyAnimation::Walking, walker.clip_age),
        (WalkerState::Active, WalkerClip::Shooting { since }) => (ClottyAnimation::Shooting, since),
        (WalkerState::Dying(frames), _) => (ClottyAnimation::Dying, frames),
    };
    let loops = matches!(animation, ClottyAnimation::Idle | ClottyAnimation::Walking);
    TextureId::Clotty {
        animation,
        frame: frame_of(frames, animation.frame_count(), loops),
        flashing: walker.is_flashing(),
    }
}

pub(super) fn mulligan_texture(walker: &Walker) -> TextureId {
    let (animation, frames) = match (walker.state, walker.clip) {
        (WalkerState::Appearing, _) => (MulliganAnimation::Appearing, walker.age),
        (WalkerState::Active, WalkerClip::Standing) => (MulliganAnimation::Standing, walker.clip_age),
        (WalkerState::Active, WalkerClip::Walking { .. }) => (MulliganAnimation::Walking, walker.clip_age),
        (WalkerState::Active, WalkerClip::Shooting { since }) => (MulliganAnimation::Bursting, since),
        (WalkerState::Dying(frames), _) => (MulliganAnimation::Dying, frames),
    };
    let loops = matches!(animation, MulliganAnimation::Standing | MulliganAnimation::Walking);
    TextureId::Mulligan {
        animation,
        frame: frame_of(frames, animation.frame_count(), loops),
        scared: walker.is_scared(),
        flashing: walker.is_flashing(),
    }
}
