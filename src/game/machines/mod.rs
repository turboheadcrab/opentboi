//! Gambling: slot and fortune machines, the blood donation machine, beggars, the devil beggar and
//! the shell game. In the original they are all pickup frame 8 (clip 2246), told apart by `col`.
//! They stand where they were placed until broken; touching one plays it.

mod breaking;
pub mod clip;
mod events;
use clip::animation_length;
mod machine;
mod payout;
pub(crate) mod rolls;
mod showcase;

use glam::Vec2;

use crate::game::ball_contact::{Body, Contact, ball_hit};
use crate::game::draw::{DrawItem, Rgba, actor_depth};
use crate::game::odds::Odds;
use crate::game::original_units::{Frames, Pixels, distance};
use crate::game::parameters::PickupParameters;
use crate::game::pickups::markers;
use crate::game::random::Random;
use crate::game::room::layout::RoomLayout;
use crate::game::sound_event::SoundEvent;
use crate::game::texture_id::TextureId;
pub use events::{MachineEvent, Throw};
pub use machine::{Cost, Machine};

/// The shell game is wider than the others by this much.
const SHELL_GAME_EXTRA_REACH: Pixels = Pixels(28.0);
/// How far a blast reaches a machine: 85 plus a pickup's size.
const BLAST_REACH: Pixels = Pixels(97.0);
/// How many frames of a playing animation are rendered ahead.
const LOOKAHEAD_FRAMES: u8 = 8;
const FALLBACK_COLOR: Rgba = Rgba::opaque(0.5, 0.45, 0.4);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MachineKind {
    /// No `col`.
    SlotMachine,
    /// `col = 41`.
    FortuneTeller,
    /// `col = 1`.
    BloodDonation,
    /// `col` 2–9: the coins it has been given.
    Beggar,
    /// `col = 31`: takes hearts instead of coins.
    DevilBeggar,
    /// `col = 10`.
    ShellGame,
}

impl MachineKind {
    pub const ALL: [Self; 6] = [
        Self::SlotMachine,
        Self::FortuneTeller,
        Self::BloodDonation,
        Self::Beggar,
        Self::DevilBeggar,
        Self::ShellGame,
    ];

    /// What a layout marker for `b5` frame `marker_frame` places, as `create()` sets it up on the
    /// first visit: frame 8 is a slot machine (a fortune machine one time in three), 38 a fortune
    /// machine, 31 the blood bank, 32 a beggar (a devil beggar one time in three), 37 a devil
    /// beggar, 33 the shell game.
    pub fn for_marker(marker_frame: u8, random: &mut Random) -> Option<Self> {
        const VARIANT_ODDS: Odds = Odds::one_in(3);
        let kind = match marker_frame {
            8 if random.happens(VARIANT_ODDS) => Self::FortuneTeller,
            8 => Self::SlotMachine,
            38 => Self::FortuneTeller,
            31 => Self::BloodDonation,
            32 if random.happens(VARIANT_ODDS) => Self::DevilBeggar,
            32 => Self::Beggar,
            37 => Self::DevilBeggar,
            33 => Self::ShellGame,
            _ => return None,
        };
        Some(kind)
    }
}

#[derive(Clone, Debug)]
pub struct Machines {
    parameters: PickupParameters,
    standing: Vec<Machine>,
}

impl Machines {
    pub fn new(parameters: PickupParameters) -> Self {
        Self {
            parameters,
            standing: Vec::new(),
        }
    }

    /// The machines a room's markers create on the first visit.
    pub fn from_layout(layout: &RoomLayout, parameters: PickupParameters, random: &mut Random) -> Self {
        let mut machines = Self::new(parameters);
        for (frame, center) in markers(layout, parameters) {
            if let Some(kind) = MachineKind::for_marker(frame, random) {
                machines.place(kind, center);
            }
        }
        machines
    }

    /// Machines that can still give something (not `empty`).
    pub fn with_something_to_give(&self) -> impl Iterator<Item = MachineKind> {
        self.standing
            .iter()
            .filter(|machine| !machine.empty)
            .map(|machine| machine.kind)
    }

    /// Where the machines stand.
    pub fn positions(&self) -> impl Iterator<Item = Vec2> {
        self.standing.iter().map(|machine| machine.center)
    }

    pub fn place(&mut self, kind: MachineKind, center: Vec2) {
        self.standing.push(Machine::new(kind, center));
    }

    /// `ballhit()` with the player, who is pushed back (machines stay put, `xp = xpp`), and touching
    /// them: returns the contact and what the first game started costs.
    pub fn bump(
        &mut self,
        player: Body,
        coins: u8,
        elapsed: Frames,
        random: &mut Random,
        sounds: &mut Vec<SoundEvent>,
    ) -> (Contact, Option<Cost>) {
        let mut total = Contact::default();
        let mut cost = None;
        for machine in self.standing.iter_mut().filter(|machine| !machine.passable) {
            let body = Body {
                position: machine.center,
                velocity: Vec2::ZERO,
                mass: self.parameters.mass,
            };
            let mut reach = self.parameters.collect_reach;
            if machine.kind == MachineKind::ShellGame {
                reach += SHELL_GAME_EXTRA_REACH;
            }
            let Some(contact) = ball_hit(player, body, reach, 1.0, elapsed) else {
                continue;
            };
            total.first_offset += contact.first_offset;
            total.first_velocity_change += contact.first_velocity_change;
            let coins_left = if cost == Some(Cost::Coin) { 0 } else { coins };
            cost = cost.or(machine.touch(player.position, coins_left, random, sounds));
        }
        (total, cost)
    }

    /// Runs the machines; returns what they did, and where.
    /// A game running or just paid out keeps the doors shut (`lastspin`).
    pub fn doors_locked(&self) -> bool {
        self.standing.iter().any(|machine| machine.door_lock > Frames::ZERO)
    }

    pub fn advance(&mut self, elapsed: Frames, random: &mut Random, lucky_foot: bool) -> Vec<(Vec2, MachineEvent)> {
        let mut happened = Vec::new();
        for machine in &mut self.standing {
            let mut events = Vec::new();
            machine.advance(elapsed, random, lucky_foot, &mut events);
            happened.extend(events.into_iter().map(|event| (machine.center, event)));
        }
        self.standing.retain(|machine| !machine.gone);
        happened
    }

    /// A blast at `center` breaks the machines in reach.
    pub fn blow_up_near(&mut self, center: Vec2) -> Vec<(Vec2, MachineEvent)> {
        let mut happened = Vec::new();
        for machine in &mut self.standing {
            if distance(machine.center, center) < BLAST_REACH {
                let mut events = Vec::new();
                machine.blow_up(&mut events);
                happened.extend(events.into_iter().map(|event| (machine.center, event)));
            }
        }
        happened
    }

    pub fn append_draw_items(&self, draw_items: &mut Vec<DrawItem>) {
        draw_items.extend(self.standing.iter().map(|machine| {
            let texture = texture(machine, machine.clip_frame, machine.animation_frame() - 1);
            DrawItem::textured(
                texture,
                machine.center,
                1.0,
                FALLBACK_COLOR,
                actor_depth(machine.center.y),
            )
        }));
    }

    /// Textures soon needed, to render ahead of time: the next frames of what each machine is
    /// playing, then its whole break animation (a blast can come at any moment).
    pub fn upcoming_textures(&self, textures: &mut Vec<TextureId>) {
        for machine in &self.standing {
            let frame = machine.clip_frame;
            let next =
                machine.animation_frame()..animation_length(frame).min(machine.animation_frame() + LOOKAHEAD_FRAMES);
            textures.extend(next.map(|animation| texture(machine, frame, animation)));
        }
        for machine in self
            .standing
            .iter()
            .filter(|machine| !machine.broken && !machine.passable)
        {
            let broken = breaking::blown_up_frame(machine.kind);
            textures.extend((0..animation_length(broken)).map(|animation| texture(machine, broken, animation)));
        }
    }
}

/// `machine` showing animation frame `animation` (from 0) of clip frame `frame`.
fn texture(machine: &Machine, frame: u8, animation: u8) -> TextureId {
    TextureId::Machine {
        alternate: matches!(machine.kind, MachineKind::FortuneTeller | MachineKind::DevilBeggar),
        frame,
        prize: if frame == clip::SHUFFLING {
            machine.shell_prize
        } else {
            0
        },
        animation,
    }
}

#[cfg(test)]
mod tests;
