//! The kinds of walker, and the values each is made of.

use crate::game::original_units::{Damage, Frames, Hearts, HitPoints, Mass, Pixels, Retention};
use crate::game::parameters::{
    ClottyParameters, FleshMoundParameters, GaperParameters, HeadlessParameters, HopperParameters, HorfParameters,
    MulliganParameters,
};
use crate::game::player::death::Killer;
use crate::game::room::enemies::EnemyParameters;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WalkerKind {
    Gaper,
    Headless,
    Horf,
    Clotty,
    Mulligan,
    Hopper,
    FleshMound,
}

/// A kind's own values, gathered from the parameters.
#[derive(Clone, Copy, Debug)]
pub struct Body {
    pub hit_points: HitPoints,
    pub size: Pixels,
    pub mass: Mass,
    pub scale: f32,
    pub appearing_frames: Frames,
    pub appearing_retention: Retention,
    pub dying_frames: Frames,
    pub contact_damage: Hearts,
    pub trample_damage: Damage,
}

/// Every kind's parameters hold the same body values under the same names.
macro_rules! body_from {
    ($parameters:ty) => {
        impl From<$parameters> for Body {
            fn from(parameters: $parameters) -> Self {
                Self {
                    hit_points: parameters.hit_points,
                    size: parameters.size,
                    mass: parameters.mass,
                    scale: parameters.scale,
                    appearing_frames: parameters.appearing_frames,
                    appearing_retention: parameters.appearing_retention,
                    dying_frames: parameters.dying_frames,
                    contact_damage: parameters.contact_damage,
                    trample_damage: parameters.trample_damage,
                }
            }
        }
    };
}

body_from!(GaperParameters);
body_from!(HeadlessParameters);
body_from!(HorfParameters);
body_from!(ClottyParameters);
body_from!(MulliganParameters);
body_from!(HopperParameters);
body_from!(FleshMoundParameters);

impl WalkerKind {
    pub fn body(self, parameters: &EnemyParameters) -> Body {
        match self {
            Self::Gaper => parameters.gapers.into(),
            Self::Headless => parameters.headless.into(),
            Self::Horf => parameters.horfs.into(),
            Self::Clotty => parameters.clottys.into(),
            Self::Mulligan => parameters.mulligans.into(),
            Self::Hopper => parameters.hoppers.into(),
            Self::FleshMound => parameters.flesh_mounds.into(),
        }
    }

    /// Who the death note names when it hurts the player.
    pub fn killer(self) -> Killer {
        match self {
            Self::Gaper => Killer::Gaper,
            Self::Headless => Killer::Headless,
            Self::Horf => Killer::Horf,
            Self::Clotty => Killer::Clotty,
            Self::Mulligan => Killer::Mulligan,
            Self::Hopper => Killer::Hopper,
            Self::FleshMound => Killer::FleshMound,
        }
    }
}
