//! Breaking a machine open (`busted`, `chestopen = trg`): by a spin gone wrong or by a blast.

use crate::game::machines::MachineKind;
use crate::game::machines::clip;
use crate::game::machines::events::MachineEvent;
use crate::game::machines::machine::Machine;
use crate::game::pickups::rewards::RewardSource;
use crate::game::sound_event::SoundEvent;

impl Machine {
    /// Broken by a spin: an explosion, and it spills like a chest.
    pub(super) fn burst(&mut self, events: &mut Vec<MachineEvent>) {
        events.push(MachineEvent::Sound(SoundEvent::Explosion));
        self.break_open(clip::SLOT_BROKEN, events);
    }

    /// `busted`, showing `broken_frame`; what it held spills out once (the devil beggar's spiders
    /// do not exist yet).
    pub(super) fn break_open(&mut self, broken_frame: u8, events: &mut Vec<MachineEvent>) {
        self.show(broken_frame);
        self.spinning = false;
        self.broken = true;
        if std::mem::replace(&mut self.empty, true) {
            return;
        }
        let source = match self.kind {
            MachineKind::DevilBeggar => return,
            MachineKind::BloodDonation => RewardSource::BloodDonationMachine,
            _ => RewardSource::BrownChest,
        };
        events.push(MachineEvent::Contents(source));
    }

    /// Caught in a blast (`bloww` on pickup frame 8): slot and fortune machines and the blood
    /// bank break, beggars and the shell game's host are blown away and can be walked through.
    pub fn blow_up(&mut self, events: &mut Vec<MachineEvent>) {
        if self.broken || self.passable {
            return;
        }
        let broken_frame = blown_up_frame(self.kind);
        self.passable = broken_frame == clip::BEGGAR_BLOWN;
        self.break_open(broken_frame, events);
    }
}

/// What a blast leaves of a machine of `kind`.
pub(super) fn blown_up_frame(kind: MachineKind) -> u8 {
    match kind {
        MachineKind::SlotMachine | MachineKind::FortuneTeller => clip::SLOT_BROKEN,
        MachineKind::BloodDonation => clip::BLOOD_BROKEN,
        MachineKind::Beggar | MachineKind::DevilBeggar | MachineKind::ShellGame => clip::BEGGAR_BLOWN,
    }
}
