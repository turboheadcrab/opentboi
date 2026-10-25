//! The Gaper's clip `b10` (5051): `d` frame 1 stands (the body 4543 with the head `h`, 5043, whose
//! 22 frames are calm, frantic, then shouting), 2 walks (5048, a 22-frame loop that includes the
//! head), 3 dies (566, 36 frames), 4 appears (5050, 46 frames; its head is on the calm or the
//! frantic frame).

/// Which animation of the Gaper's clip a texture shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GaperAnimation {
    /// `frantic`: the head on its second frame (a plain marker's Gaper) rather than its first.
    Appearing {
        frantic: bool,
    },
    /// `frame` is the head's.
    Standing,
    /// The body alone (its head `h` is drawn apart as `Head`).
    Walking,
    /// The head `h` on its frame, drawn over the walking body.
    Head,
    Dying,
}

impl GaperAnimation {
    pub const ALL: [Self; 6] = [
        Self::Appearing { frantic: false },
        Self::Appearing { frantic: true },
        Self::Standing,
        Self::Walking,
        Self::Head,
        Self::Dying,
    ];

    pub fn frame_count(self) -> u8 {
        match self {
            Self::Appearing { .. } => GAPER_APPEARING_FRAME_COUNT,
            Self::Standing | Self::Head => GAPER_HEAD_FRAME_COUNT,
            Self::Walking => GAPER_WALKING_FRAME_COUNT,
            Self::Dying => GAPER_DYING_FRAME_COUNT,
        }
    }
}

pub const GAPER_APPEARING_FRAME_COUNT: u8 = 46;
pub const GAPER_HEAD_FRAME_COUNT: u8 = 22;
pub const GAPER_WALKING_FRAME_COUNT: u8 = 22;
pub const GAPER_DYING_FRAME_COUNT: u8 = 36;
