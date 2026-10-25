//! What Isaac's familiars (the blue flies) ask of the room's enemies: who is there to chase, hurting
//! one, blocking a shot, and who was hurt last (`blufer`, DoAction 4806 and 6984).

use glam::Vec2;

use crate::game::original_units::{Damage, HitPoints, Pixels, distance};
use crate::game::random::Random;
use crate::game::room::enemies::shot::EnemyShot;
use crate::game::room::enemies::{Enemies, FlyState, Hurt, WalkerState};
use crate::game::room::stage_to_world;
use crate::game::sound_event::SoundEvent;

/// An enemy a familiar can go for: its number in the room (`e`), where it is (world) and how big.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FriendTarget {
    pub number: u32,
    pub position: Vec2,
    pub size: Pixels,
}

impl Enemies {
    /// The live enemies that are out of their appearing clip.
    pub fn friend_targets(&self) -> Vec<FriendTarget> {
        let flies = self
            .flies
            .iter()
            .filter(|fly| fly.state == FlyState::Flying)
            .map(|fly| FriendTarget {
                number: fly.shooter.phase,
                position: stage_to_world(fly.position),
                size: super::fly::kind_parameters(fly.kind, &self.parameters).size,
            });
        let walkers = self
            .walkers
            .iter()
            .filter(|walker| walker.state == WalkerState::Active)
            .map(|walker| FriendTarget {
                number: walker.phase(),
                position: stage_to_world(walker.position),
                size: walker.kind.body(&self.parameters).size,
            });
        flies.chain(walkers).collect()
    }

    /// A familiar hurts the enemy with number `number`.
    pub fn hurt_by_friend(&mut self, number: u32, damage: Damage, random: &mut Random, sounds: &mut Vec<SoundEvent>) {
        let parameters = self.parameters;
        if let Some(fly) = self.flies.iter_mut().find(|fly| fly.shooter.phase == number)
            && fly.hurt(damage, &parameters)
        {
            sounds.push(SoundEvent::EnemyDied);
        }
        if let Some(walker) = self.walkers.iter_mut().find(|walker| walker.phase() == number)
            && walker.hurt(damage, &parameters, random) == Hurt::Died
        {
            sounds.push(SoundEvent::EnemyDied);
        }
        self.release_flies(random);
    }

    /// A familiar at `position` (world) meets a shot within `reach`: the shot is gone. Returns
    /// whether there was one.
    pub fn absorb_shot_near(&mut self, position: Vec2, reach: Pixels) -> bool {
        let found = self
            .shots
            .iter()
            .position(|shot: &EnemyShot| distance(stage_to_world(shot.position), position) < reach);
        found.map(|index| self.shots.swap_remove(index)).is_some()
    }

    /// The enemy hurt since the last call (the last one found, if several), by whatever hurt it.
    pub fn take_last_hurt(&mut self) -> Option<u32> {
        let seen = std::mem::take(&mut self.health_seen);
        let now: Vec<(u32, HitPoints)> = self
            .flies
            .iter()
            .map(|fly| (fly.shooter.phase, fly.hit_points()))
            .chain(self.walkers.iter().map(|walker| (walker.phase(), walker.hit_points())))
            .collect();
        let hurt = now
            .iter()
            .filter(|(number, health)| {
                seen.iter()
                    .any(|(seen_id, before)| seen_id == number && health < before)
            })
            .map(|(number, _)| *number)
            .next_back();
        self.health_seen = now;
        hurt
    }
}
