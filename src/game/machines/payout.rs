//! What a machine gives once a result has played for a moment (`sloty()` and the payout block
//! after it), and breaking one open (`chestopen`). Items get an empty pedestal; pills, trinkets,
//! and flies do not exist yet, so nothing appears for them.

use crate::game::hud::name_banner::FORTUNE_COUNT;
use crate::game::machines::MachineKind;
use crate::game::machines::clip;
use crate::game::machines::events::{MachineEvent, Throw};
use crate::game::machines::machine::{Machine, SHELL_ITEM_PRIZE};
use crate::game::odds::Odds;
use crate::game::pickups::kind::{
    BOMB_FRAME, CARD_MARKER_FRAME, COIN_FRAME, HEART_FRAME, ITEM_PEDESTAL_FRAME, KEY_FRAME, PILL_FRAME,
    TRINKET_MARKER_FRAME,
};
use crate::game::pickups::rewards::Drop;
use crate::game::random::Random;
use crate::game::sound_event::SoundEvent;
use crate::game::texture_id::FlyKind;

/// The original's `f2` codes, by what they become here.
enum Gift {
    Pickups(Drop),
    Item,
    /// A fly the machine lets out: a harmless one from the slot machine (`f2 = 13`), an attack fly (`18`)
    /// for a lost shell game.
    Fly(FlyKind),
}

fn pickups(frame: u8, count: u8) -> Gift {
    Gift::Pickups(Drop {
        frame,
        count,
        soul_heart: false,
    })
}

impl Machine {
    pub(super) fn pay_out(&mut self, random: &mut Random, lucky_foot: bool, events: &mut Vec<MachineEvent>) {
        let frame = self.clip_frame;
        let gift = match self.kind {
            MachineKind::FortuneTeller if frame < clip::LAST_RESULT => self.fortune(random, lucky_foot, events),
            MachineKind::SlotMachine | MachineKind::FortuneTeller => self.slot(random, events),
            MachineKind::BloodDonation => Some(blood(random, events)),
            MachineKind::Beggar | MachineKind::DevilBeggar if frame == clip::BEGGAR_GIVES => {
                Some(self.beggar_gift(random))
            }
            MachineKind::ShellGame if frame >= clip::LEFT_SHELL => Some(self.shell(random, lucky_foot, events)),
            MachineKind::Beggar | MachineKind::DevilBeggar | MachineKind::ShellGame => None,
        };
        let Some(gift) = gift else { return };
        let throw = match self.kind {
            MachineKind::ShellGame => Throw::Shell(frame),
            _ => Throw::Spray,
        };
        if frame < clip::DONATING {
            events.push(MachineEvent::Sound(SoundEvent::MachinePaidOut));
        } else if frame != clip::DONATING && !matches!(gift, Gift::Fly(_)) {
            events.push(MachineEvent::Sound(SoundEvent::ThumbsUp));
        }
        let gives_pedestal_below = matches!(
            self.kind,
            MachineKind::Beggar | MachineKind::DevilBeggar | MachineKind::ShellGame
        );
        match gift {
            Gift::Item if gives_pedestal_below => {
                events.push(MachineEvent::PedestalBelow);
                self.empty = true;
                self.passable = true;
                self.spinning = false;
                self.show(clip::BEGGAR_LEAVES);
            }
            Gift::Item => {
                events.push(MachineEvent::Prize(pedestal(), throw));
                self.gone = true;
            }
            Gift::Pickups(drop) => events.push(MachineEvent::Prize(drop, throw)),
            Gift::Fly(kind) => events.push(MachineEvent::Fly(kind)),
        }
    }

    /// `sloty()`.
    fn slot(&mut self, random: &mut Random, events: &mut Vec<MachineEvent>) -> Option<Gift> {
        const PRETTY_FLY_ODDS: Odds = Odds::one_in(3);
        let gift = match self.clip_frame {
            // "Pretty Fly" one time in three (one more orbiting fly), else a fly (`f2 = 13`).
            3 if random.happens(PRETTY_FLY_ODDS) => {
                events.push(MachineEvent::Sound(SoundEvent::ThumbsUp));
                events.push(MachineEvent::PrettyFly);
                return None;
            }
            3 => {
                events.push(MachineEvent::Sound(SoundEvent::ThumbsDown));
                Gift::Fly(FlyKind::Black)
            }
            4 => pickups(BOMB_FRAME, 1),
            5 | 6 => pickups(HEART_FRAME, 1),
            7 => pickups(KEY_FRAME, 1),
            8 => pickups(PILL_FRAME, 1),
            9 => {
                events.push(MachineEvent::Sound(SoundEvent::Explosion));
                Gift::Item
            }
            10..=12 => pickups(COIN_FRAME, random.below_u8(3).max(1)),
            _ => {
                events.push(MachineEvent::Sound(SoundEvent::ThumbsDown));
                return None;
            }
        };
        Some(gift)
    }

    /// The fortune machine: a fortune (65 %), else it breaks (1 in 20), an item (1 in 30), a card,
    /// a soul heart or a trinket.
    /// With the Lucky Foot no fortune is told: every play rolls for a prize.
    fn fortune(&mut self, random: &mut Random, lucky_foot: bool, events: &mut Vec<MachineEvent>) -> Option<Gift> {
        const FORTUNE_CHANCE: f32 = 0.65;
        const LUCKY_FOOT_FORTUNE_CUT: f32 = 1.0;
        const BREAK_ODDS: Odds = Odds::one_in(20);
        const ITEM_ODDS: Odds = Odds::one_in(30);
        const CARD_ODDS: Odds = Odds::one_in(3);
        const SOUL_HEART_ODDS: Odds = Odds::one_in(3);
        let fortune_chance = FORTUNE_CHANCE - if lucky_foot { LUCKY_FOOT_FORTUNE_CUT } else { 0.0 };
        if random.next_unit() < fortune_chance {
            events.push(MachineEvent::Fortune(random.below_u8(FORTUNE_COUNT)));
            return None;
        }
        if random.happens(BREAK_ODDS) {
            self.burst(events);
            return None;
        }
        let gift = if random.happens(ITEM_ODDS) {
            Gift::Item
        } else if random.happens(CARD_ODDS) {
            pickups(CARD_MARKER_FRAME, 1)
        } else if random.happens(SOUL_HEART_ODDS) {
            Gift::Pickups(Drop {
                frame: HEART_FRAME,
                count: 1,
                soul_heart: true,
            })
        } else {
            pickups(TRINKET_MARKER_FRAME, 1)
        };
        Some(gift)
    }

    /// `case 38`.
    #[allow(
        clippy::same_functions_in_if_condition,
        reason = "each condition is a roll of its own"
    )]
    fn beggar_gift(&mut self, random: &mut Random) -> Gift {
        const HEART_ODDS: Odds = Odds::one_in(3);
        self.spinning = false;
        self.show(clip::BEGGAR_IDLE);
        let coin_flip = |random: &mut Random| random.happens(Odds::EVEN);
        if self.kind == MachineKind::DevilBeggar {
            const TRINKET_ODDS: Odds = Odds::one_in(4);
            return if coin_flip(random) {
                // A pill or a card, even odds.
                if coin_flip(random) {
                    pickups(PILL_FRAME, 1)
                } else {
                    pickups(CARD_MARKER_FRAME, 1)
                }
            } else if random.happens(TRINKET_ODDS) {
                pickups(TRINKET_MARKER_FRAME, 1)
            } else {
                Gift::Item
            };
        }
        // One of the beggar's five items.
        random.below(5);
        if coin_flip(random) {
            if random.happens(HEART_ODDS) {
                pickups(HEART_FRAME, 1)
            } else {
                pickups(if coin_flip(random) { KEY_FRAME } else { BOMB_FRAME }, 1)
            }
        } else if coin_flip(random) {
            if coin_flip(random) {
                random.below(5);
                Gift::Item
            } else {
                pickups(CARD_MARKER_FRAME, 1)
            }
        } else {
            Gift::Item
        }
    }

    /// `case 43–45`: a win one time in three gives two of the prize (coins: 2–3), or the item.
    /// A loss rolls again, and with the Lucky Foot a win on that second roll counts (the original
    /// rolls it either way).
    fn shell(&mut self, random: &mut Random, lucky_foot: bool, events: &mut Vec<MachineEvent>) -> Gift {
        const WIN_ODDS: Odds = Odds::one_in(3);
        let won = random.happens(WIN_ODDS) || (random.happens(WIN_ODDS) && lucky_foot);
        if !won {
            events.push(MachineEvent::Sound(SoundEvent::ShellGameLost));
            return Gift::Fly(FlyKind::Attack);
        }
        match self.shell_prize {
            SHELL_ITEM_PRIZE => Gift::Item,
            prize => {
                let frame = HEART_FRAME + prize;
                let count = if frame == COIN_FRAME {
                    2 + random.below_u8(2)
                } else {
                    random.below_u8(3).max(2)
                };
                pickups(frame, count)
            }
        }
    }
}

fn pedestal() -> Drop {
    Drop {
        frame: ITEM_PEDESTAL_FRAME,
        count: 1,
        soul_heart: false,
    }
}

/// `case 33`: one time in 15 an item and the machine is gone, else 1–2 coins.
fn blood(random: &mut Random, events: &mut Vec<MachineEvent>) -> Gift {
    const ITEM_ODDS: Odds = Odds::one_in(15);
    const MOST_COINS: i32 = 3;
    if random.happens(ITEM_ODDS) {
        events.push(MachineEvent::Sound(SoundEvent::Explosion));
        return Gift::Item;
    }
    events.push(MachineEvent::Sound(SoundEvent::BloodBankPaidOut));
    let coins = random.below(MOST_COINS).max(1);
    pickups(COIN_FRAME, u8::try_from(coins).unwrap_or(1))
}
