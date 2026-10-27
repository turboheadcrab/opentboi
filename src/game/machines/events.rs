//! What a machine does that the room has to carry out.

use crate::game::pickups::rewards::Drop;
use crate::game::sound_event::SoundEvent;
use crate::game::texture_id::FlyKind;

/// What a machine did that the room has to carry out.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MachineEvent {
    Sound(SoundEvent),
    /// Pickups thrown out of it, the way `throw` says.
    Prize(Drop, Throw),
    /// An item pedestal set down below a beggar, who leaves (`spaw()`).
    PedestalBelow,
    /// A fortune told, by place in the machine's list; the banner shows it.
    Fortune(u8),
    /// "Pretty Fly": one more orbiting fly, its message and Isaac's thumbs up.
    PrettyFly,
    /// A fly let out of the machine, and Isaac's thumbs down.
    Fly(FlyKind),
    /// Broken open: it spills like a chest.
    Contents(crate::game::pickups::rewards::RewardSource),
}

/// How prizes fly: sprayed downwards, or out of the lifted shell (see `gambling`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Throw {
    Spray,
    Shell(u8),
}
