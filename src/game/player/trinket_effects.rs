//! What the trinkets do when something happens: Isaac is hurt (`playerhurt()`, DoAction 4781–4885),
//! an enemy dies (`hurt()`, DoAction 4411–4419), a room is cleared (DoAction 30873–30885) or the boss
//! room is entered (DoAction 32842–32846).

use crate::game::Game;
use crate::game::floor::RoomEntry;
use crate::game::floor::plan::RoomRole;
use crate::game::original_units::Hearts;
use crate::game::pickups::item::Item;
use crate::game::pickups::luck::PickupLuck;
use crate::game::pickups::trinket::Trinket;
use crate::game::sound_event::SoundEvent;

/// The overlay frame A Missing Page plays (`over` frame 3).
pub const MISSING_PAGE_ANIMATION: u8 = 3;
/// Isaac's Fork heals this much.
const FORK_HEAL: Hearts = Hearts::HALF;

impl Game {
    /// What Isaac carries that changes what the room gives.
    pub(super) fn pickup_luck(&self) -> PickupLuck {
        PickupLuck {
            lucky_foot: self.inventory.has(Item::LuckyFoot),
            luck: self.player_stats().luck,
            lucky_toe: self.inventory.has_trinket(Trinket::LuckyToe),
            moms_pearl: self.inventory.has_trinket(Trinket::MomsPearl),
        }
    }

    /// Everything a hit sets off, fatal or not: the Fish Head's blue fly, the Red Patch's damage and
    /// A Missing Page's death.
    pub(super) fn trinkets_react_to_hurt(&mut self) {
        let amounts = self.parameters.trinkets;
        if self.inventory.has_trinket(Trinket::FishHead) {
            self.blue_flies.owe(1);
        }
        if self.inventory.has_trinket(Trinket::RedPatch) && self.random.happens(amounts.red_patch_odds) {
            self.cards.boosts.razor += amounts.red_patch_points;
        }
        if self.inventory.has_trinket(Trinket::AMissingPage) && self.random.happens(amounts.missing_page_odds) {
            self.play_card_animation(MISSING_PAGE_ANIMATION, SoundEvent::DeathCard);
            self.floor.current_room_mut().enemies.hurt_all(
                amounts.missing_page_damage,
                &mut self.random,
                &mut self.pending_sound_events,
            );
        }
    }

    /// A hit that Isaac survived with half a heart or less: the Polaroid's bubble, and the Cursed
    /// Skull's teleport back to the room last left (not out of a boss room).
    pub(super) fn trinkets_react_to_near_death(&mut self) {
        if !self.inventory.is_near_death() {
            return;
        }
        if self.inventory.has_trinket(Trinket::ThePolaroid) {
            self.inventory.shield_for(self.parameters.trinkets.polaroid_shield);
        }
        if self.inventory.has_trinket(Trinket::CursedSkull)
            && self.floor.current_role() != RoomRole::Boss
            && self.inventory.has_little_armor()
        {
            self.teleport(self.floor.room_last_left());
        }
    }

    /// Samson's Lock: now and then a kill makes Isaac hit harder for the rest of the room.
    pub(crate) fn trinkets_react_to_kills(&mut self) {
        let kills = self.floor.current_room_mut().enemies.take_deaths();
        self.rage_for_kills(kills);
    }

    pub(super) fn rage_for_kills(&mut self, kills: usize) {
        let amounts = self.parameters.trinkets;
        if !self.inventory.has_trinket(Trinket::SamsonsLock) {
            return;
        }
        for _ in 0..kills {
            if self.random.happens(amounts.samsons_lock_odds) {
                self.cards.boosts.rage = (self.cards.boosts.rage + amounts.rage_step).min(amounts.rage_cap);
            }
        }
    }

    /// The Tick gives a heart on first entering the boss room.
    pub(crate) fn trinkets_welcome(&mut self, entry: &RoomEntry) {
        if entry.role == RoomRole::Boss && entry.first_visit && self.inventory.has_trinket(Trinket::Tick) {
            self.inventory.heal(self.parameters.trinkets.tick_heal);
        }
    }

    /// Isaac's Fork: one time in ten, clearing a room heals half a heart.
    pub(crate) fn trinkets_react_to_clearing(&mut self) {
        if self.inventory.has_trinket(Trinket::IsaacsFork)
            && self.random.happens(self.parameters.trinkets.isaacs_fork_odds)
        {
            self.inventory.heal(FORK_HEAL);
            self.pending_sound_events.push(SoundEvent::HealthGulped);
        }
    }
}
