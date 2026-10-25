//! What the player carries: coins, bombs, keys and health. Health is in [`Hearts`], as the
//! original keeps it.

use crate::game::original_units::{Frames, Hearts};
use crate::game::parameters::InventoryParameters;
use crate::game::pickups::consumable::Consumable;
use crate::game::pickups::item::Item;
use crate::game::pickups::kind::{BombKind, CoinKind, HeartKind, KeyKind, PickupKind};
use crate::game::player::active_item::ActiveSlot;
use crate::game::player::pill_state::PillState;
use crate::game::player::unicorn_ride::UnicornRide;
use crate::game::sound_event::SoundEvent;

/// `coincol()` (DoAction 7194–7210): a penny is worth 1, a nickel (DoAction 7201, DoAction 7205) 5, a
/// dime (DoAction 7209) 10.
const NICKEL_VALUE: u8 = 5;
const DIME_VALUE: u8 = 10;

pub struct Inventory {
    parameters: InventoryParameters,
    pub coins: u8,
    pub bombs: u8,
    pub keys: u8,
    /// `kep`: the golden key.
    pub golden_key: bool,
    /// Red hearts, and the heart containers that hold them.
    pub red_hearts: Hearts,
    pub maximum_red_hearts: Hearts,
    /// Soul hearts.
    pub soul_hearts: Hearts,
    /// `eta`: an eternal heart.
    pub eternal_heart: bool,
    /// Two eternal halves just became a heart (the overlay's frame 16); taken by the game.
    pub eternal_hearts_united: bool,
    /// The passive items taken this run, in order.
    items: Vec<Item>,
    /// The active item and its charge.
    pub active: ActiveSlot,
    /// `pilc`: the card or pill held.
    pub held: Option<Consumable>,
    /// A card or pill just taken, and the one it replaced (`pillc()`), which the pickup becomes.
    pub taken: Option<Consumable>,
    pub dropped: Option<Consumable>,
    /// What the pills used this run have changed (`ups`), and their effects (`piller`).
    pub pills: PillState,
    /// Time left in which the player cannot be hurt.
    invulnerable_frames: Frames,
    /// The unicorn ride's invincibility, apart from the blink after a hit.
    pub ride: UnicornRide,
}

impl Inventory {
    pub fn new(parameters: InventoryParameters) -> Self {
        let hearts = parameters.starting_hearts;
        Self {
            parameters,
            coins: 0,
            bombs: parameters.starting_bombs,
            keys: 0,
            golden_key: false,
            red_hearts: hearts,
            maximum_red_hearts: hearts,
            soul_hearts: Hearts::ZERO,
            eternal_heart: false,
            eternal_hearts_united: false,
            items: Vec::new(),
            active: ActiveSlot::default(),
            held: None,
            taken: None,
            dropped: None,
            pills: PillState::default(),
            invulnerable_frames: Frames::ZERO,
            ride: UnicornRide::default(),
        }
    }

    /// Takes a pickup the player touched; `None` if it stays (a heart at full health, a troll bomb).
    pub fn take(&mut self, kind: PickupKind) -> Option<SoundEvent> {
        let maximum = self.parameters.maximum_count;
        let add = |count: &mut u8, amount: u8| *count = count.saturating_add(amount).min(maximum);
        let sound = match kind {
            // `balljunk()` case 1 (DoAction 7705–7742): an eternal heart (`eta()`), a soul heart, or
            // a red heart while one is missing.
            PickupKind::Heart(HeartKind::Soul) => {
                self.soul_hearts += Hearts::ONE;
                SoundEvent::SoulHeartPickedUp
            }
            PickupKind::Heart(HeartKind::Eternal) => self.add_eternal_heart(),
            PickupKind::Heart(heart) => self.take_red_heart(heart)?,
            // Case 2: coins, by `coincol()`.
            PickupKind::Coin(coin) => {
                let (value, sound) = match coin {
                    CoinKind::Penny => (1, SoundEvent::PennyPickedUp),
                    CoinKind::Nickel => (NICKEL_VALUE, SoundEvent::NickelPickedUp),
                    CoinKind::Dime => (DIME_VALUE, SoundEvent::DimePickedUp),
                };
                add(&mut self.coins, value);
                sound
            }
            // Case 3: the golden key (`goldenkey.wav`), or a key (`KeyPickup_Gauntlet.wav`).
            PickupKind::Key(KeyKind::Golden) => {
                self.golden_key = true;
                SoundEvent::GoldenKeyPickedUp
            }
            PickupKind::Card(card) => self.take_held(Consumable::Card(card)),
            PickupKind::Pill(pill) => self.take_held(Consumable::Pill(pill)),
            PickupKind::Item(item) => {
                self.take_item(item);
                SoundEvent::ItemTaken
            }
            PickupKind::Key(KeyKind::Normal) => {
                add(&mut self.keys, 1);
                SoundEvent::KeyPickedUp
            }
            // Case 4: troll bombs stay and go off; chests are opened, not taken.
            PickupKind::Bomb(BombKind::Troll | BombKind::MegaTroll)
            | PickupKind::Chest(_)
            | PickupKind::ItemPedestal => {
                return None;
            }
            // A bomb, or two for the double bomb (with its message), then one of two pickup sounds.
            PickupKind::Bomb(bomb) => {
                add(&mut self.bombs, if bomb == BombKind::Double { 2 } else { 1 });
                SoundEvent::BombPickedUp
            }
        };
        Some(sound)
    }

    /// A red heart, or half of one; `None` (it stays) at full health.
    fn take_red_heart(&mut self, heart: HeartKind) -> Option<SoundEvent> {
        if self.red_hearts >= self.maximum_red_hearts {
            return None;
        }
        let amount = if heart == HeartKind::Half {
            Hearts::HALF
        } else {
            Hearts::ONE
        };
        self.red_hearts = (self.red_hearts + amount).min(self.maximum_red_hearts);
        Some(SoundEvent::HeartPickedUp)
    }

    /// `eta()`: a second eternal heart turns both into a heart container.
    /// `eta()` (DoAction 7011–7023): the first eternal heart is kept, with its sound (DoAction 7013);
    /// the second joins it into a heart container (DoAction 7018) and a heart's worth of health
    /// (DoAction 7020), with the overlay on its frame 16 (DoAction 7021).
    fn add_eternal_heart(&mut self) -> SoundEvent {
        self.eternal_heart = !self.eternal_heart;
        if !self.eternal_heart {
            self.maximum_red_hearts += Hearts::ONE;
            self.red_hearts += Hearts::ONE;
            self.eternal_hearts_united = true;
        }
        SoundEvent::EternalHeartPickedUp
    }

    /// Whether Isaac has taken `item` (or holds it, if active).
    pub fn has(&self, item: Item) -> bool {
        self.items.contains(&item) || self.active.held() == Some(item)
    }

    pub fn has_key(&self) -> bool {
        self.golden_key || self.keys > 0
    }

    /// Spends a key (the golden key is never used up).
    pub fn use_key(&mut self) {
        if !self.golden_key {
            self.keys = self.keys.saturating_sub(1);
        }
    }

    /// Spends a bomb if there is one.
    pub fn use_bomb(&mut self) -> bool {
        let has_bomb = self.bombs > 0;
        self.bombs = self.bombs.saturating_sub(u8::from(has_bomb));
        has_bomb
    }

    /// `lasth = 0` before a hurt that must go through: no blink left from an earlier hit.
    pub fn end_invulnerability(&mut self) {
        self.invulnerable_frames = Frames::ZERO;
    }

    /// `playerhurt()`: soul hearts absorb the hit (the rest is lost), else the eternal heart softens
    /// it by half a heart and red hearts take it. Returns whether it hurt.
    pub fn hurt(&mut self, damage: Hearts) -> bool {
        // Not while invulnerable after a hit, nor on the unicorn.
        if self.invulnerable_frames > Frames::ZERO || self.ride.is_riding() {
            return false;
        }
        if self.soul_hearts > Hearts::ZERO {
            self.soul_hearts = (self.soul_hearts - damage).max(Hearts::ZERO);
        } else {
            if self.eternal_heart {
                self.eternal_heart = false;
                self.red_hearts += Hearts::HALF;
            }
            self.red_hearts = (self.red_hearts - damage).max(Hearts::ZERO);
        }
        self.invulnerable_frames = self.parameters.invulnerable_frames_per_heart * damage.get();
        true
    }

    pub fn is_dead(&self) -> bool {
        self.red_hearts <= Hearts::ZERO && self.soul_hearts <= Hearts::ZERO && !self.eternal_heart
    }

    pub fn rides_unicorn(&self) -> bool {
        self.ride.is_riding()
    }

    pub fn is_invulnerable(&self) -> bool {
        self.invulnerable_frames > Frames::ZERO
    }

    pub fn advance(&mut self, elapsed: Frames) {
        self.invulnerable_frames = (self.invulnerable_frames - elapsed).max(Frames::ZERO);
    }

    /// Whether the player is hidden this frame by the hurt blink.
    pub fn blinks_hidden(&self) -> bool {
        let frames_left = Frames(self.invulnerable_frames.get().ceil());
        let periods_left = (frames_left / self.parameters.blink_period_frames).ceil();
        frames_left > self.parameters.blink_until_frames_left && periods_left % 2.0 == 0.0
    }
}

mod blood;
mod gains;
mod pills;

#[cfg(test)]
mod tests;
