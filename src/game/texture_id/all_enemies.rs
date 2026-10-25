//! The enemies' textures for the list of every texture.

use super::{ClottyAnimation, GaperAnimation, HeadlessAnimation, HorfAnimation, MulliganAnimation, TextureId};

impl TextureId {
    /// Every frame of an enemy's animations; only the living flash red from a hit.
    fn enemy_frames<Animation: Copy>(
        animations: impl IntoIterator<Item = Animation>,
        frame_count: fn(Animation) -> u8,
        is_dying: fn(Animation) -> bool,
        texture: impl Fn(Animation, u8, bool) -> Self + Copy,
    ) -> impl Iterator<Item = Self> {
        animations.into_iter().flat_map(move |animation| {
            let flashes: &[bool] = if is_dying(animation) { &[false] } else { &[false, true] };
            (0..frame_count(animation))
                .flat_map(move |frame| flashes.iter().map(move |&flashing| texture(animation, frame, flashing)))
        })
    }

    /// Every ground enemy's frames.
    pub(super) fn walkers() -> impl Iterator<Item = Self> {
        Self::gapers()
            .chain(Self::headless())
            .chain(Self::horfs())
            .chain(Self::clottys())
            .chain(Self::mulligans())
            .chain(super::EnemyTexture::all().map(Self::Enemy))
    }

    fn gapers() -> impl Iterator<Item = Self> {
        Self::enemy_frames(
            GaperAnimation::ALL,
            GaperAnimation::frame_count,
            |animation| animation == GaperAnimation::Dying,
            |animation, frame, flashing| Self::Gaper {
                animation,
                frame,
                flashing,
            },
        )
    }

    fn headless() -> impl Iterator<Item = Self> {
        Self::enemy_frames(
            HeadlessAnimation::ALL,
            HeadlessAnimation::frame_count,
            |animation| animation == HeadlessAnimation::Dying,
            |animation, frame, flashing| Self::Headless {
                animation,
                frame,
                flashing,
            },
        )
    }

    fn horfs() -> impl Iterator<Item = Self> {
        Self::enemy_frames(
            HorfAnimation::ALL,
            HorfAnimation::frame_count,
            |animation| animation == HorfAnimation::Dying,
            |animation, frame, flashing| Self::Horf {
                animation,
                frame,
                flashing,
            },
        )
    }

    fn clottys() -> impl Iterator<Item = Self> {
        Self::enemy_frames(
            ClottyAnimation::ALL,
            ClottyAnimation::frame_count,
            |animation| animation == ClottyAnimation::Dying,
            |animation, frame, flashing| Self::Clotty {
                animation,
                frame,
                flashing,
            },
        )
    }

    fn mulligans() -> impl Iterator<Item = Self> {
        [false, true].into_iter().flat_map(|scared| {
            Self::enemy_frames(
                MulliganAnimation::ALL,
                MulliganAnimation::frame_count,
                |animation| animation == MulliganAnimation::Dying,
                move |animation, frame, flashing| Self::Mulligan {
                    animation,
                    frame,
                    scared,
                    flashing,
                },
            )
        })
    }
}
