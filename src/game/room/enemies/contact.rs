//! Enemies touching things: tears and blasts hurt them, the player bumps them (and is hurt by
//! the attack fly and by shots).

use glam::Vec2;

use crate::game::ball_contact::{Body, ball_hit};
use crate::game::original_units::{Damage, Frames, Hearts, Pixels, distance};
use crate::game::player::death::Killer;
use crate::game::random::Random;
use crate::game::room::enemies::{
    Bump, Enemies, EnemyParameters, Fly, Hurt, PLAYER_SIZE, TEAR_MASS, TEAR_SIZE, Walker, fly,
};
use crate::game::room::stage_to_world;
use crate::game::sound_event::SoundEvent;

impl Enemies {
    /// A tear at `floor_point` (world) hitting a live enemy: it is hurt and knocked back; returns
    /// whether the tear hit.
    pub fn hit_by_tear(
        &mut self,
        floor_point: Vec2,
        velocity: Vec2,
        damage: Damage,
        elapsed: Frames,
        random: &mut Random,
        sounds: &mut Vec<SoundEvent>,
    ) -> bool {
        let tear = Body {
            position: floor_point,
            velocity,
            mass: TEAR_MASS,
        };
        let parameters = self.parameters;
        let reach_of = |fly: &Fly| TEAR_SIZE + fly::kind_parameters(fly.kind, &parameters).size;
        let Some(fly) = self
            .flies
            .iter_mut()
            .find(|fly| fly.is_alive() && distance(world_position(fly), floor_point) < reach_of(fly))
        else {
            let hit = self.walker_hit_by_tear(tear, damage, elapsed, random, sounds);
            self.release_flies(random);
            return hit;
        };
        if let Some(contact) = ball_hit(tear, fly_body(fly, &parameters), reach_of(fly), 1.0, elapsed) {
            push(
                &mut fly.position,
                &mut fly.velocity,
                contact.second_offset,
                contact.second_velocity_change,
            );
        }
        if fly.hurt(damage, &parameters) {
            sounds.push(SoundEvent::EnemyDied);
        }
        true
    }

    /// The tear, if it reaches a live walker: hurts and pushes it.
    fn walker_hit_by_tear(
        &mut self,
        tear: Body,
        damage: Damage,
        elapsed: Frames,
        random: &mut Random,
        sounds: &mut Vec<SoundEvent>,
    ) -> bool {
        let parameters = self.parameters;
        let Some(walker) = self.walkers.iter_mut().find(|walker| {
            walker.is_alive()
                && distance(walker_world_position(walker), tear.position)
                    < TEAR_SIZE + walker.kind.body(&parameters).size
        }) else {
            return false;
        };
        let reach = TEAR_SIZE + walker.kind.body(&parameters).size;
        if let Some(contact) = ball_hit(tear, walker_body(walker, &parameters), reach, 1.0, elapsed) {
            push(
                &mut walker.position,
                &mut walker.velocity,
                contact.second_offset,
                contact.second_velocity_change,
            );
        }
        if walker.hurt(damage, &parameters, random) == Hurt::Died {
            sounds.push(SoundEvent::EnemyDied);
        }
        true
    }

    /// `ballhit()` between the player and the live enemies: the attack fly hurts the player; while
    /// `riding_unicorn`, every fly touched takes the unicorn's damage. Shots reaching the player
    /// hurt too.
    pub fn bump(
        &mut self,
        player: Body,
        riding_unicorn: bool,
        elapsed: Frames,
        random: &mut Random,
        sounds: &mut Vec<SoundEvent>,
    ) -> Bump {
        let parameters = self.parameters;
        let mut total = Bump {
            hurt: self.shot_hitting(player.position, random),
            ..Bump::default()
        };
        for fly in self.flies.iter_mut().filter(|fly| fly.is_alive()) {
            let reach = PLAYER_SIZE + fly::kind_parameters(fly.kind, &parameters).size;
            let Some(contact) = ball_hit(player, fly_body(fly, &parameters), reach, 1.0, elapsed) else {
                continue;
            };
            push(
                &mut fly.position,
                &mut fly.velocity,
                contact.second_offset,
                contact.second_velocity_change,
            );
            total.contact.first_offset += contact.first_offset;
            total.contact.first_velocity_change += contact.first_velocity_change;
            if fly::kind_parameters(fly.kind, &parameters).hurts_on_touch {
                total.hurt = Some((parameters.flies.attack_damage, Killer::Enemy(fly.kind)));
            }
            if riding_unicorn && fly.hurt(parameters.flies.contact_damage, &parameters) {
                sounds.push(SoundEvent::EnemyDied);
            }
        }
        self.bump_walkers(player, riding_unicorn, elapsed, random, sounds, &mut total);
        self.release_flies(random);
        total
    }

    /// The walkers touching the player hurt, and are pushed; the unicorn tramples them.
    fn bump_walkers(
        &mut self,
        player: Body,
        riding_unicorn: bool,
        elapsed: Frames,
        random: &mut Random,
        sounds: &mut Vec<SoundEvent>,
        total: &mut Bump,
    ) {
        let parameters = self.parameters;
        for walker in self.walkers.iter_mut().filter(|walker| walker.is_alive()) {
            let body = walker.kind.body(&parameters);
            let reach = PLAYER_SIZE + body.size;
            let Some(contact) = ball_hit(player, walker_body(walker, &parameters), reach, 1.0, elapsed) else {
                continue;
            };
            push(
                &mut walker.position,
                &mut walker.velocity,
                contact.second_offset,
                contact.second_velocity_change,
            );
            total.contact.first_offset += contact.first_offset;
            total.contact.first_velocity_change += contact.first_velocity_change;
            if body.contact_damage > Hearts::ZERO {
                total.hurt = Some((body.contact_damage, walker.kind.killer()));
            }
            if riding_unicorn && walker.hurt(body.trample_damage, &parameters, random) == Hurt::Died {
                sounds.push(SoundEvent::EnemyDied);
            }
        }
    }

    /// Hurts every live enemy in the room by `damage` (the Death card).
    pub fn hurt_all(&mut self, damage: Damage, random: &mut Random, sounds: &mut Vec<SoundEvent>) {
        let parameters = self.parameters;
        for fly in &mut self.flies {
            if fly.hurt(damage, &parameters) {
                sounds.push(SoundEvent::EnemyDied);
            }
        }
        for walker in &mut self.walkers {
            if walker.hurt(damage, &parameters, random) == Hurt::Died {
                sounds.push(SoundEvent::EnemyDied);
            }
        }
        self.release_flies(random);
    }

    /// A blast at `center` (world) reaching `radius` hurts every live enemy in reach.
    pub fn blast(&mut self, center: Vec2, radius: Pixels, random: &mut Random, sounds: &mut Vec<SoundEvent>) {
        let parameters = self.parameters;
        for fly in &mut self.flies {
            if distance(world_position(fly), center) <= radius && fly.hurt(parameters.flies.contact_damage, &parameters)
            {
                sounds.push(SoundEvent::EnemyDied);
            }
        }
        for walker in &mut self.walkers {
            if distance(walker_world_position(walker), center) <= radius
                && walker.hurt(walker.kind.body(&parameters).trample_damage, &parameters, random) == Hurt::Died
            {
                sounds.push(SoundEvent::EnemyDied);
            }
        }
        self.release_flies(random);
    }
}

fn world_position(fly: &Fly) -> Vec2 {
    stage_to_world(fly.position)
}

/// The fly as a body in the world (y up).
fn fly_body(fly: &Fly, parameters: &EnemyParameters) -> Body {
    Body {
        position: world_position(fly),
        velocity: Vec2::new(fly.velocity.x, -fly.velocity.y),
        mass: fly::kind_parameters(fly.kind, parameters).mass,
    }
}

fn walker_world_position(gaper: &Walker) -> Vec2 {
    stage_to_world(gaper.position)
}

/// The walker as a body in the world (y up).
pub(super) fn walker_body(walker: &Walker, parameters: &EnemyParameters) -> Body {
    Body {
        position: walker_world_position(walker),
        velocity: Vec2::new(walker.velocity.x, -walker.velocity.y),
        mass: walker.kind.body(parameters).mass,
    }
}

/// Applies a world-space push to an enemy's stage-space position and velocity.
pub(super) fn push(position: &mut Vec2, velocity: &mut Vec2, offset: Vec2, velocity_change: Vec2) {
    *position += Vec2::new(offset.x, -offset.y);
    *velocity += Vec2::new(velocity_change.x, -velocity_change.y);
}
