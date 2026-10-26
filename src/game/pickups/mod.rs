//! Pickups lying in a room: they land when they appear, are taken when the player touches them,
//! chests open, troll bombs go off on their own, and what cannot be taken gets bumped aside.

pub mod card;
mod collect;
pub mod consumable;
pub mod item;
pub mod kind;
mod markers;
pub mod phase;
pub mod pickup;
pub mod pill;
pub mod red_chest;
pub mod rewards;
pub mod showcase;
pub mod sounds;

use glam::Vec2;

use crate::game::attacks::explosion::blast_push;
use crate::game::ball_contact::{Body, Contact, ball_hit};
use crate::game::bounds::Bounds;
use crate::game::draw::DrawItem;
use crate::game::hud::message::PickupMessage;
use crate::game::original_units::{Frames, length};
use crate::game::parameters::{ExplosionParameters, PickupParameters, WallParameters};
use crate::game::pickups::consumable::Consumable;
use crate::game::pickups::kind::{BombKind, ChestKind, HeartKind, PickupKind};
use crate::game::pickups::pickup::Pickup;
use crate::game::pickups::red_chest::RedChestChance;
use crate::game::pickups::sounds::landing_sound;
use crate::game::random::Random;
use crate::game::room::layout::RoomLayout;
use crate::game::sound_event::SoundEvent;
pub use markers::markers;

/// What pickups move among this tick.
#[derive(Clone, Copy)]
pub struct PickupSurroundings<'tiles> {
    pub room_interior: Bounds,
    pub solid_tiles: &'tiles [Bounds],
    /// The original's `fra`, for sounds the first frames of a room keep quiet.
    pub frames_in_room: Frames,
    /// What mega troll bombs chase.
    pub player_position: Vec2,
    pub walls: WallParameters,
}

/// What touching the pickups did that the room has to follow up.
#[derive(Default)]
pub struct Collection {
    /// Chests opened: their contents spill out here.
    pub opened_chests: Vec<(Vec2, ChestKind)>,
    /// A golden chest took a key.
    pub used_key: bool,
    pub message: Option<PickupMessage>,
    /// A card or pill taken: Isaac holds it up (`pillc()`).
    pub card_taken: Option<crate::game::pickups::consumable::Consumable>,
    /// An item taken: Isaac holds it up and its name shows; the pedestal stays, empty.
    pub item_taken: Option<crate::game::pickups::item::Item>,
}

#[derive(Clone, Debug)]
pub struct Pickups {
    parameters: PickupParameters,
    lying: Vec<Pickup>,
    /// Whether brown chests made here turn red.
    pub red_chests: RedChestChance,
}

impl Pickups {
    /// The pickups a room's markers create on the first visit.
    pub fn from_layout(layout: &RoomLayout, parameters: PickupParameters, random: &mut Random) -> Self {
        let lying = markers(layout, parameters)
            .filter_map(|(frame, center)| Some(Pickup::placed(PickupKind::roll(frame, random)?, center)))
            .collect();
        Self {
            parameters,
            lying,
            red_chests: RedChestChance::default(),
        }
    }

    /// Sets the room's red chest chance; brown chests its layout put down may turn red.
    pub fn set_red_chest_chance(&mut self, chance: RedChestChance, random: &mut Random) {
        self.red_chests = chance;
        for pickup in &mut self.lying {
            pickup.kind = chance.apply(pickup.kind, random);
        }
    }

    /// A pickup placed where the room's layout would put it.
    pub fn place(&mut self, kind: PickupKind, center: Vec2) {
        self.lying.push(Pickup::placed(kind, center));
    }

    /// A card or pill that a purchase pushed out of the slot: it lies where the ware was, cannot be
    /// taken for a moment and is tossed up like a handed-back one.
    pub fn drop_swapped(&mut self, held: Consumable, center: Vec2, player_position: Vec2) {
        let mut pickup = Pickup::new(held.pickup_kind(), center, Vec2::ZERO);
        pickup.hand_back(held, player_position, collect::SWAP_BLOCK);
        self.lying.push(pickup);
    }

    /// A pickup thrown or dropped during play (from chests, poop, rewards).
    pub fn spawn(&mut self, kind: PickupKind, center: Vec2, velocity: Vec2) {
        self.lying.push(Pickup::new(kind, center, velocity));
    }

    /// Ages and moves the pickups, with their landing sounds; returns where troll bombs went off.
    pub fn advance(
        &mut self,
        elapsed: Frames,
        surroundings: PickupSurroundings,
        sounds: &mut Vec<SoundEvent>,
    ) -> Vec<Vec2> {
        let fail_limit = self.parameters.troll_bomb_fail_limit;
        self.chase(elapsed, surroundings.player_position);
        let mut explosions = Vec::new();
        self.lying.retain_mut(|pickup| {
            let age_before = pickup.age_frames;
            let still_here = pickup.advance(
                elapsed,
                surroundings.room_interior,
                surroundings.solid_tiles,
                surroundings.walls,
            );
            let sound = landing_sound(pickup.kind, age_before, pickup.age_frames, surroundings.frames_in_room);
            sounds.extend(sound);
            // The original compares its count before adding this frame, i.e. the age before it.
            let exploded = pickup.kind.is_troll_bomb() && pickup.age_frames - elapsed > fail_limit;
            if exploded {
                explosions.push(pickup.center);
            }
            still_here && !exploded
        });
        explosions
    }

    /// Mega troll bombs roll toward the player.
    fn chase(&mut self, elapsed: Frames, player_position: Vec2) {
        let parameters = self.parameters;
        let chasers = self.lying.iter_mut().filter(|pickup| {
            pickup.kind == PickupKind::Bomb(BombKind::MegaTroll)
                && pickup.is_lying()
                && pickup.age_frames > parameters.frames_until_collectable
        });
        for pickup in chasers {
            let toward = player_position - pickup.center;
            if length(toward) >= parameters.chase_reach {
                continue;
            }
            let pull = parameters.chase_pull_per_frame * parameters.mega_troll_pull_factor;
            pickup.accelerate(toward.normalize_or_zero() * pull * elapsed);
            pickup.slow_down(parameters.chase_velocity_retention.over(elapsed));
        }
    }

    /// Blasts push what is lying in reach (`trg2.bh`: not before it has settled, and not once
    /// taken) where `sees` says the blast sees it (`linecheckxx()`).
    pub fn push_from_blast(&mut self, center: Vec2, explosion: ExplosionParameters, sees: impl Fn(Vec2) -> bool) {
        let solid_after = self.parameters.frames_until_collectable;
        for pickup in self
            .lying
            .iter_mut()
            .filter(|pickup| pickup.is_lying() && pickup.age_frames > solid_after && sees(pickup.center))
        {
            if let Some(push) = blast_push(pickup.center - center, explosion.pickup_push_reach, explosion) {
                pickup.knock_back(explosion.velocity_retention, push);
            }
        }
    }

    /// `ballhit()` between the player and what was left lying (settled, not taken, or an opened
    /// chest); returns how the player is moved and slowed.
    /// A pickup wedged against a wall or rock cannot give way, so the player takes the rest of
    /// the push.
    pub fn bump(&mut self, player: Body, elapsed: Frames, room_interior: Bounds, solid_tiles: &[Bounds]) -> Contact {
        let parameters = self.parameters;
        let mut total = Contact::default();
        for pickup in &mut self.lying {
            // Troll bombs are not taken, so they bump like anything left lying; pickups collide with
            // the player only. A handed-back card or item still blocked is passed through.
            let solid = pickup.age_frames > parameters.frames_until_collectable
                && (pickup.is_lying() || pickup.kind.is_chest())
                && pickup.swap_blocked_frames == Frames::ZERO;
            let body = Body {
                position: pickup.center,
                velocity: pickup.velocity(),
                mass: parameters.mass,
            };
            let Some(contact) = solid
                .then(|| ball_hit(player, body, parameters.collect_reach, 1.0, elapsed))
                .flatten()
            else {
                continue;
            };
            let moved = pickup.move_to(pickup.center + contact.second_offset, room_interior, solid_tiles);
            pickup.accelerate(contact.second_velocity_change);
            total.first_offset += contact.first_offset - (contact.second_offset - moved);
            total.first_velocity_change += contact.first_velocity_change;
        }
        total
    }

    #[cfg(test)]
    pub fn lying_kinds(&self) -> Vec<PickupKind> {
        self.lying.iter().map(|pickup| pickup.kind).collect()
    }

    /// Kinds of what still lies here (not taken, chests not opened).
    pub fn lying_untaken(&self) -> impl Iterator<Item = PickupKind> {
        self.lying
            .iter()
            .filter(|pickup| pickup.is_lying())
            .map(|pickup| pickup.kind)
    }

    /// Where the pickups lie.
    pub fn positions(&self) -> impl Iterator<Item = Vec2> {
        self.lying.iter().map(|pickup| pickup.center)
    }

    pub fn remember_positions(&mut self) {
        for pickup in &mut self.lying {
            pickup.previous_center = pickup.center;
        }
    }

    pub fn append_draw_items(&self, draw_items: &mut Vec<DrawItem>) {
        draw_items.extend(self.lying.iter().map(Pickup::draw_item));
    }

    /// Every heart becomes `heart` (the super secret room's styles 0 and 1).
    pub fn set_heart_kind(&mut self, heart: HeartKind) {
        for pickup in &mut self.lying {
            if matches!(pickup.kind, PickupKind::Heart(_)) {
                pickup.kind = PickupKind::Heart(heart);
            }
        }
    }
}

#[cfg(test)]
mod tests;
