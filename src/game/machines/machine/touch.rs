//! Touching a machine: `balljunk()` case 8 (DoAction 7836–7910).

use glam::Vec2;

use super::{
    Cost, DOOR_LOCK_ON_SHELL_PICK, DOOR_LOCK_ON_START, Frames, MIDDLE_SHELL_REACH, Machine, MachineKind, Random,
    SHELL_ITEM_ODDS, SHELL_ITEM_PRIZE, SHELL_PRIZES, SoundEvent, clip,
};

impl Machine {
    /// The player bumped into it (`balljunk()` case 8, DoAction 7836–7910); returns what it costs
    /// if a game starts. Machines always bump back.
    pub fn touch(
        &mut self,
        player: Vec2,
        coins: u8,
        random: &mut Random,
        sounds: &mut Vec<SoundEvent>,
    ) -> Option<Cost> {
        // Not while running or broken.
        let idle = !self.spinning && !self.broken;
        match self.kind {
            MachineKind::DevilBeggar | MachineKind::BloodDonation if idle => {
                let busy_frame = if self.kind == MachineKind::BloodDonation {
                    clip::DONATING
                } else {
                    clip::BEGGAR_PAID
                };
                self.start(busy_frame);
                return Some(Cost::Blood);
            }
            MachineKind::DevilBeggar | MachineKind::BloodDonation => return None,
            MachineKind::ShellGame if self.spinning => {
                self.pick_shell(player);
                return None;
            }
            _ => {}
        }
        // A coin, an idle machine, and for the shell game a player below it.
        let in_front = self.kind != MachineKind::ShellGame || player.y < self.center.y;
        if coins == 0 || !idle || !in_front {
            return None;
        }
        let busy_frame = match self.kind {
            MachineKind::ShellGame => {
                // One of four prizes; the item one time in 13 (rolled below).
                self.shell_prize = random.below_u8(SHELL_PRIZES);
                if random.happens(SHELL_ITEM_ODDS) {
                    self.shell_prize = SHELL_ITEM_PRIZE;
                }
                clip::SHUFFLING
            }
            MachineKind::Beggar => clip::BEGGAR_PAID,
            // The coin drops in and the machine spins.
            _ => {
                sounds.push(SoundEvent::CoinInserted);
                clip::SPINNING
            }
        };
        self.start(busy_frame);
        Some(Cost::Coin)
    }

    fn start(&mut self, busy_frame: u8) {
        self.lock_doors(DOOR_LOCK_ON_START);
        self.show(busy_frame);
        self.spinning = true;
        self.paid_out = false;
    }

    /// Near the end of the shuffle, the shell on the player's side is lifted: the middle one within
    /// 13 px of the center, else the left or the right one.
    fn pick_shell(&mut self, player: Vec2) {
        if self.clip_frame != clip::SHUFFLING || self.animation_frame() <= clip::PICKABLE_AFTER_FRAME {
            return;
        }
        let across = player.x - self.center.x;
        let lifted = if across.abs() < MIDDLE_SHELL_REACH.get() {
            clip::MIDDLE_SHELL
        } else if across < 0.0 {
            clip::LEFT_SHELL
        } else {
            clip::RIGHT_SHELL
        };
        self.show(lifted);
        self.lock_doors(DOOR_LOCK_ON_SHELL_PICK);
        self.paid_out = false;
    }

    pub(super) fn lock_doors(&mut self, frames: Frames) {
        self.door_lock = self.door_lock.max(frames);
    }
}
