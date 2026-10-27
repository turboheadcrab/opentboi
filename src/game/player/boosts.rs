//! What cards give Isaac until he leaves the room: The Devil's damage, Strength's mushroom
//! effect, The Hanged Man's flight (the original counts them as items held for the room).

use crate::game::Game;
use crate::game::bounds::Bounds;
use crate::game::original_units::{Damage, Frames, Hearts, Pixels, PixelsPerFrame};
use crate::game::parameters::{CardParameters, GameParameters};
use crate::game::pickups::trinket::Trinket;
use crate::game::player::pill_state::PillState;
use crate::game::player::unicorn_ride::RideColor;
use crate::game::room::grid_object::GridObject;

/// Card effects lasting for the current room.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RoomBoosts {
    /// The Devil (`doub`).
    pub devil: bool,
    /// Strength used this many times (counted as item 12).
    pub strength: u8,
    /// The Hanged Man (counted as item 20).
    pub flying: bool,
    /// The Empress (counted as 1.5 of item 122).
    pub empress: bool,
    /// The Red Patch's points of damage bonus for the room (`razor`).
    pub razor: f32,
    /// Samson's Lock's rage above the usual (`rage - 1`) for the room.
    pub rage: f32,
}

/// Isaac's stats with the room's boosts applied.
#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "independent facts about the room's boosts, each read on its own"
)]
pub struct PlayerStats {
    /// `playsp`.
    pub speed: f32,
    pub damage: Damage,
    /// Extra starting height of tears (range).
    pub tear_height_bonus: Pixels,
    /// How much sooner a tear starts rising (range points, `dm`).
    pub tear_rise_bonus: PixelsPerFrame,
    /// The sum the fire rate depends on (`firr()`).
    pub delay_bonus: f32,
    /// Frames added to the fire delay (trinkets).
    pub delay_offset: Frames,
    /// What a tear's velocity is multiplied by (shot speed).
    pub shot_speed_multiplier: f32,
    /// How lucky Isaac is (`luck`).
    pub luck: i32,
    /// Drawn size (`player._xscale / 100`).
    pub scale: f32,
    pub flying: bool,
    /// Tears drawn dark (the devil's boost, `doub`).
    pub dark_tears: bool,
    /// Isaac looks demonic (the Empress's item 122, `curs()`).
    pub demonic: bool,
    /// On the unicorn (`unic`): faster, and no tears.
    pub riding_unicorn: bool,
    /// The color Isaac flashes on it.
    pub ride_tint: Option<RideColor>,
}

impl RoomBoosts {
    pub fn stats(self, parameters: &GameParameters, pills: &PillState) -> PlayerStats {
        let cards: CardParameters = parameters.cards;
        let pill_parameters = parameters.pills;
        let strength = f32::from(self.strength);
        let empress = if self.empress { cards.empress_strength } else { 0.0 };
        let devil_bonus = if self.devil { cards.devil_damage_bonus } else { 0.0 };
        let trinkets = parameters.trinkets;
        let damage_bonus = devil_bonus
            + strength * cards.strength_damage_bonus
            + empress * empress * cards.empress_damage_scale
            + self.razor * trinkets.razor_damage_per_point
            + self.rage;
        let speed_bonus = ((strength + empress) * cards.strength_speed_bonus
            + pills.speed * pill_parameters.speed_per_point)
            .min(cards.maximum_speed_bonus);
        let range_height = pill_parameters.range_height_per_point
            + pill_parameters.range_rise_per_point * pill_parameters.range_height_per_rise;
        let shot_speed = (1.0 + pills.shot_speed * pill_parameters.shot_speed_per_point).clamp(
            pill_parameters.shot_speed_multiplier_minimum,
            pill_parameters.shot_speed_multiplier_maximum,
        );
        PlayerStats {
            speed: parameters.movement.speed_stat + speed_bonus,
            damage: parameters.tear.damage(damage_bonus),
            tear_height_bonus: cards.strength_range_bonus * strength + range_height * pills.range,
            tear_rise_bonus: pill_parameters.range_rise_per_point * pills.range,
            delay_bonus: pills.tears * pill_parameters.delay_bonus_per_tear,
            delay_offset: Frames::ZERO,
            shot_speed_multiplier: shot_speed,
            luck: pills.luck,
            scale: 1.0 + (strength + empress) * cards.strength_size_bonus,
            flying: self.flying,
            dark_tears: self.devil,
            demonic: self.empress,
            riding_unicorn: false,
            ride_tint: None,
        }
    }
}

impl Game {
    pub(crate) fn player_stats(&self) -> PlayerStats {
        let stats = self.stats_with_trinkets(self.cards.boosts.stats(&self.parameters, &self.inventory.pills));
        if !self.inventory.rides_unicorn() {
            return stats;
        }
        PlayerStats {
            speed: stats.speed + self.parameters.cards.unicorn_speed_bonus,
            riding_unicorn: true,
            ride_tint: self.inventory.ride.color(),
            ..stats
        }
    }

    /// What the trinkets held add: speed (the Goat Hoof), flat damage (the Curved Horn, the Game
    /// Squid) and a shorter fire delay (Cancer).
    fn stats_with_trinkets(&self, mut stats: PlayerStats) -> PlayerStats {
        let amounts = self.parameters.trinkets;
        let held = |trinket| self.inventory.has_trinket(trinket);
        if held(Trinket::GoatHoof) {
            stats.speed += amounts.goat_hoof_speed;
        }
        if held(Trinket::CurvedHorn) {
            stats.damage += amounts.curved_horn_damage;
        }
        if held(Trinket::GameSquid) {
            stats.damage += amounts.game_squid_damage;
        }
        if held(Trinket::Cancer) {
            stats.delay_offset += amounts.cancer_delay;
        }
        stats
    }

    /// What stops Isaac (walls and obstacles; flying, only metal blocks and walls) and the webs
    /// that slow him.
    pub(crate) fn player_obstacles(&self, flying: bool) -> (Vec<Bounds>, Vec<Bounds>) {
        let blocks: fn(&GridObject) -> bool = if flying {
            GridObject::blocks_flying
        } else {
            GridObject::blocks_walking
        };
        let room = self.floor.current_room();
        let mut solid_tiles = room.bounds_where(blocks);
        solid_tiles.extend(self.floor.wall_tiles());
        (solid_tiles, room.bounds_where(GridObject::is_spider_web))
    }

    /// Strength: a heart container for the room, and a heart.
    pub(crate) fn use_strength(&mut self) {
        self.cards.boosts.strength += 1;
        self.inventory.maximum_red_hearts += Hearts::ONE;
        self.inventory.heal(Hearts::ONE);
    }

    /// Leaving the room ends its boosts; Strength's containers go again.
    pub(crate) fn end_room_boosts(&mut self) {
        let containers = Hearts::ONE * f32::from(self.cards.boosts.strength);
        self.inventory.maximum_red_hearts = (self.inventory.maximum_red_hearts - containers).max(Hearts::ZERO);
        self.inventory.red_hearts = self.inventory.red_hearts.min(self.inventory.maximum_red_hearts);
        self.cards.boosts = RoomBoosts::default();
    }
}
