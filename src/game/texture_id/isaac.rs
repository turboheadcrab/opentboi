//! Isaac's own textures: his head (or the unicorn's, while riding) and his body, each also in the
//! colors he flashes on the unicorn, and in the demonic look the Empress gives him.

use glam::Vec2;

pub use crate::game::player::unicorn_ride::RideColor;

/// Isaac's head clip (2261): down, right, up, left, then the same with the shooting face. The
/// unicorn's head (2803) and the demonic one (2912) have the same eight.
pub const ISAAC_HEAD_FRAME_COUNT: u8 = 8;
/// Walk cycles inside the body clip 4308: frame 2 holds 4307 (vertical), frame 3 holds 1620. The
/// demonic body's (5389) hold 5388 and 1650 with the same lengths.
pub const ISAAC_VERTICAL_WALK_FRAME_COUNT: u8 = 24;
pub const ISAAC_HORIZONTAL_WALK_FRAME_COUNT: u8 = 22;
const ISAAC_HEAD_SIZE: Vec2 = Vec2::new(56.0, 56.0);
/// Taller for the horn.
const UNICORN_HEAD_SIZE: Vec2 = Vec2::new(60.0, 72.0);
/// Horns and wings.
const DEMONIC_HEAD_SIZE: Vec2 = Vec2::new(84.0, 92.0);
const ISAAC_BODY_SIZE: Vec2 = Vec2::new(48.0, 48.0);
const DEMONIC_BODY_SIZE: Vec2 = Vec2::new(96.0, 80.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IsaacPart {
    /// Frame `0..ISAAC_HEAD_FRAME_COUNT`.
    Head(u8),
    /// The unicorn's head, frame `0..ISAAC_HEAD_FRAME_COUNT`.
    UnicornHead(u8),
    BodyStanding,
    /// Flying (The Hanged Man, item 20): the body clip `bo` on frame 3 is only a large shadow;
    /// the head floats above it.
    FlyingBody,
    /// Frame `0..ISAAC_VERTICAL_WALK_FRAME_COUNT`.
    BodyWalkVertical(u8),
    /// Frame `0..ISAAC_HORIZONTAL_WALK_FRAME_COUNT`.
    BodyWalkHorizontal(u8),
    /// The same in the demonic look (`bo` on frame 8, or 9 flying; the head mode 30).
    DemonicHead(u8),
    DemonicBodyStanding,
    DemonicFlyingBody,
    DemonicBodyWalkVertical(u8),
    DemonicBodyWalkHorizontal(u8),
}

impl IsaacPart {
    pub fn all() -> impl Iterator<Item = Self> + Clone {
        (0..ISAAC_HEAD_FRAME_COUNT)
            .flat_map(|frame| [Self::Head(frame), Self::UnicornHead(frame), Self::DemonicHead(frame)])
            .chain([
                Self::BodyStanding,
                Self::FlyingBody,
                Self::DemonicBodyStanding,
                Self::DemonicFlyingBody,
            ])
            .chain(
                (0..ISAAC_VERTICAL_WALK_FRAME_COUNT)
                    .flat_map(|frame| [Self::BodyWalkVertical(frame), Self::DemonicBodyWalkVertical(frame)]),
            )
            .chain(
                (0..ISAAC_HORIZONTAL_WALK_FRAME_COUNT)
                    .flat_map(|frame| [Self::BodyWalkHorizontal(frame), Self::DemonicBodyWalkHorizontal(frame)]),
            )
    }

    /// The same part in the demonic look (the unicorn's head is the same for everyone).
    pub fn demonic(self) -> Self {
        match self {
            Self::Head(frame) => Self::DemonicHead(frame),
            Self::BodyStanding => Self::DemonicBodyStanding,
            Self::FlyingBody => Self::DemonicFlyingBody,
            Self::BodyWalkVertical(frame) => Self::DemonicBodyWalkVertical(frame),
            Self::BodyWalkHorizontal(frame) => Self::DemonicBodyWalkHorizontal(frame),
            other => other,
        }
    }

    #[cfg_attr(
        not(test),
        allow(
            dead_code,
            reason = "the list of on-demand textures, which tests use, filters with it"
        )
    )]
    pub fn is_demonic(self) -> bool {
        matches!(
            self,
            Self::DemonicHead(_)
                | Self::DemonicBodyStanding
                | Self::DemonicFlyingBody
                | Self::DemonicBodyWalkVertical(_)
                | Self::DemonicBodyWalkHorizontal(_)
        )
    }

    pub fn size(self) -> Vec2 {
        match self {
            Self::Head(_) => ISAAC_HEAD_SIZE,
            Self::UnicornHead(_) => UNICORN_HEAD_SIZE,
            Self::DemonicHead(_) => DEMONIC_HEAD_SIZE,
            Self::BodyStanding | Self::FlyingBody | Self::BodyWalkVertical(_) | Self::BodyWalkHorizontal(_) => {
                ISAAC_BODY_SIZE
            }
            Self::DemonicBodyStanding
            | Self::DemonicFlyingBody
            | Self::DemonicBodyWalkVertical(_)
            | Self::DemonicBodyWalkHorizontal(_) => DEMONIC_BODY_SIZE,
        }
    }
}
