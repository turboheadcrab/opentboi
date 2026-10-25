//! One machine or beggar: what touching it does (case 8 of the touch handler) and how it runs its
//! animations and pays out (the per-frame block for a running machine).

use glam::Vec2;

use crate::game::machines::MachineKind;
use crate::game::machines::clip::{self, animation_length};
use crate::game::machines::events::MachineEvent;
use crate::game::machines::rolls::{BEGGAR_FIRST_COUNT, slot_result};
use crate::game::odds::Odds;
use crate::game::original_units::{Frames, Pixels};
use crate::game::random::Random;
use crate::game::sound_event::SoundEvent;

/// The middle shell is picked from this close to the center.
const MIDDLE_SHELL_REACH: Pixels = Pixels(13.0);
/// The shell game's prize: one of four, and one time in 13 the item.
const SHELL_PRIZES: u8 = 4;
const SHELL_ITEM_ODDS: Odds = Odds::one_in(13);
pub const SHELL_ITEM_PRIZE: u8 = 5;
/// One spin in 50 breaks the slot machine.
const SPIN_BREAK_ODDS: Odds = Odds::one_in(50);
/// No door works for this long after a game starts (`balljunk()`), 15 frames after a shell is
/// picked, 50 after the shells stop (`aicol()`) and 12 after a result pays out (DoAction 27712).
const DOOR_LOCK_ON_START: Frames = Frames(100.0);
const DOOR_LOCK_ON_SHELL_PICK: Frames = Frames(15.0);
const DOOR_LOCK_AFTER_SHUFFLE: Frames = Frames(50.0);
const DOOR_LOCK_ON_PAYOUT: Frames = Frames(12.0);
/// What touching a machine costs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cost {
    Coin,
    /// `ader()`: half a heart, taken from red hearts first.
    Blood,
}

#[derive(Clone, Debug)]
#[allow(clippy::struct_excessive_bools, reason = "independent states of one machine")]
pub struct Machine {
    pub kind: MachineKind,
    pub center: Vec2,
    pub(super) clip_frame: u8,
    /// Time since `clip_frame` was entered: its animation shows frame `1 + whole frames`.
    animation: Frames,
    /// `spin`: busy with a game; touching does nothing meanwhile.
    pub(super) spinning: bool,
    /// `busted`: blown up or broken by a spin.
    pub(super) broken: bool,
    /// `wtf`: the current result has paid out.
    paid_out: bool,
    /// A beggar's coin count (`col` 2–9).
    pub(super) beggar_count: u8,
    /// `itt`: the shell game's prize (0 heart, 1 coin, 2 key, 3 bomb, 5 item).
    pub(super) shell_prize: u8,
    /// `dones`: the player walks through it (a blown or leaving beggar).
    pub(super) passable: bool,
    /// `empty`: it has given its item or contents and gives no more.
    pub(super) empty: bool,
    /// `done`: removed from the room.
    pub(super) gone: bool,
    /// Time left in which this machine keeps the room's doors shut (`lastspin`).
    pub(super) door_lock: Frames,
}

impl Machine {
    pub fn new(kind: MachineKind, center: Vec2) -> Self {
        let clip_frame = match kind {
            MachineKind::SlotMachine | MachineKind::FortuneTeller => clip::SLOT_IDLE,
            MachineKind::BloodDonation => clip::BLOOD_IDLE,
            MachineKind::Beggar | MachineKind::DevilBeggar => clip::BEGGAR_IDLE,
            MachineKind::ShellGame => clip::SHELL_IDLE,
        };
        Self {
            kind,
            center,
            clip_frame,
            animation: Frames::ZERO,
            spinning: false,
            broken: false,
            paid_out: false,
            beggar_count: BEGGAR_FIRST_COUNT,
            shell_prize: 0,
            passable: false,
            empty: false,
            gone: false,
            door_lock: Frames::ZERO,
        }
    }

    /// Frame of the current animation, from 1, held on its last frame.
    pub fn animation_frame(&self) -> u8 {
        let length = animation_length(self.clip_frame);
        (self.animation + Frames(1.0)).held_frame(length.saturating_add(1))
    }

    pub(super) fn show(&mut self, clip_frame: u8) {
        self.clip_frame = clip_frame;
        self.animation = Frames::ZERO;
    }

    /// Plays the animation; a running game moves on when its animation ends and pays out part way
    /// through a result.
    pub fn advance(&mut self, elapsed: Frames, random: &mut Random, lucky_foot: bool, events: &mut Vec<MachineEvent>) {
        let before = self.animation_frame();
        self.animation += elapsed;
        self.door_lock = (self.door_lock - elapsed).max(Frames::ZERO);
        let reached = |frame: u8| before < frame && self.animation_frame() >= frame;
        let sound = match self.clip_frame {
            clip::SPINNING if self.spinning && reached(clip::REELS_STOP_FRAME) => Some(SoundEvent::ReelsStopped),
            clip::SHUFFLING if reached(clip::SHUFFLE_SOUND_FRAME) => Some(SoundEvent::ShellsShuffled),
            clip::SHUFFLING if reached(clip::SHELLS_LIFTED_FRAME) => Some(SoundEvent::ShellsLifted),
            clip::BEGGAR_LEAVES if reached(clip::BEGGAR_LEAVES_SOUND_FRAME) => Some(SoundEvent::BeggarLeft),
            _ => None,
        };
        events.extend(sound.map(MachineEvent::Sound));
        if self.clip_frame == clip::BEGGAR_LEAVES && reached(clip::BEGGAR_GONE_FRAME) {
            self.gone = true;
        }
        if !self.spinning {
            return;
        }
        let finished = self.animation_frame() >= animation_length(self.clip_frame);
        if finished && before < self.animation_frame() {
            self.finish_animation(random, lucky_foot, events);
        }
        let payout_after = if self.kind == MachineKind::BloodDonation {
            clip::BLOOD_PAYOUT_AFTER_FRAME
        } else {
            clip::PAYOUT_AFTER_FRAME
        };
        if self.spinning && !self.paid_out && self.clip_frame > clip::SPINNING && self.animation_frame() > payout_after
        {
            self.paid_out = true;
            self.lock_doors(DOOR_LOCK_ON_PAYOUT);
            self.pay_out(random, lucky_foot, events);
        }
    }

    /// `trgnextd()`: what follows an animation that reached its last frame.
    fn finish_animation(&mut self, random: &mut Random, lucky_foot: bool, events: &mut Vec<MachineEvent>) {
        match self.clip_frame {
            clip::LEFT_SHELL..=clip::RIGHT_SHELL => self.spinning = false,
            clip::SHUFFLING => {
                self.paid_out = false;
                self.lock_doors(DOOR_LOCK_AFTER_SHUFFLE);
            }
            clip::BEGGAR_PAID => {
                if self.beggar_gives(random, lucky_foot) {
                    self.show(clip::BEGGAR_GIVES);
                } else {
                    self.show(clip::BEGGAR_IDLE);
                    self.spinning = false;
                }
                self.paid_out = false;
            }
            clip::SPINNING if random.happens(SPIN_BREAK_ODDS) => self.burst(events),
            clip::SPINNING => {
                let result = slot_result(random, lucky_foot, false);
                self.show(result);
                self.paid_out = false;
            }
            _ if !matches!(
                self.kind,
                MachineKind::Beggar | MachineKind::DevilBeggar | MachineKind::ShellGame
            ) =>
            {
                self.spinning = false;
            }
            _ => {}
        }
    }
}

mod touch;
