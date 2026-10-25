//! What the floor does to enemies: the original calls `firecheck()` for them as well (`alive()`,
//! DoAction 13844–13851). A burning fireplace under one of the four corners 20 px from an enemy and
//! within 38 pushes it away and, like spikes under a walker, hurts it for 8 every 10 frames (flies
//! are not hurt by spikes, and never pushed by them).

use glam::Vec2;

use crate::game::original_units::{Damage, Frames, Pixels, distance};
use crate::game::random::Random;
use crate::game::room::enemies::contact::push;
use crate::game::room::enemies::{Enemies, EnemyParameters, Hurt};
use crate::game::room::grid_kind::{GridObjectKind, TileKind};
use crate::game::room::grid_object::GridObject;
use crate::game::room::stage_to_world;
use crate::game::sound_event::SoundEvent;

/// What an enemy stands against.
#[derive(Clone, Copy, Debug, PartialEq)]
struct FloorContact {
    /// From the hazard's center to the enemy.
    away: Vec2,
    from_spikes: bool,
}

/// The first of the enemy's four corners (as `firecheck()` takes them) that meets a hazard in reach.
fn floor_contact(
    position: Vec2,
    objects: &[GridObject],
    parameters: &EnemyParameters,
    flying: bool,
) -> Option<FloorContact> {
    let (fire, spikes) = (parameters.fire, parameters.spikes);
    [Vec2::ONE, Vec2::new(-1.0, 1.0), Vec2::new(1.0, -1.0), Vec2::NEG_ONE]
        .into_iter()
        .map(|corner| position + corner * fire.probe_offset.get())
        .find_map(|corner| {
            objects
                .iter()
                .filter(|object| object.tile_bounds().contains(corner))
                .find_map(|object| {
                    let from_spikes = object.kind() == GridObjectKind::Tile(TileKind::Spikes);
                    let hurts = if from_spikes {
                        !flying
                    } else {
                        object.is_burning_fireplace()
                    };
                    let handicap = if from_spikes {
                        spikes.reach_handicap
                    } else {
                        Pixels::ZERO
                    };
                    (hurts && distance(position, object.center()) + handicap < fire.reach).then_some(FloorContact {
                        away: position - object.center(),
                        from_spikes,
                    })
                })
        })
}

impl Enemies {
    /// One frame of the floor's hurt to every live enemy.
    pub(super) fn feel_the_floor(
        &mut self,
        frame: Frames,
        objects: &[GridObject],
        random: &mut Random,
        sounds: &mut Vec<SoundEvent>,
    ) {
        let parameters = self.parameters;
        let fire = parameters.fire;
        let due = |last: Option<Frames>| last.is_none_or(|last| frame - last >= fire.enemy_interval);
        let damage: Damage = fire.enemy_damage;
        for fly in self.flies.iter_mut().filter(|fly| fly.is_alive()) {
            let Some(contact) = floor_contact(stage_to_world(fly.position), objects, &parameters, true) else {
                continue;
            };
            if !contact.from_spikes {
                push(
                    &mut fly.position,
                    &mut fly.velocity,
                    Vec2::ZERO,
                    contact.away.normalize_or_zero() * fire.push_per_frame.get(),
                );
            }
            if due(fly.last_burned) {
                fly.last_burned = Some(frame);
                sounds.push(SoundEvent::FireHiss);
                if fly.hurt(damage, &parameters) {
                    sounds.push(SoundEvent::EnemyDied);
                }
            }
        }
        for walker in self.walkers.iter_mut().filter(|walker| walker.is_alive()) {
            let Some(contact) = floor_contact(stage_to_world(walker.position), objects, &parameters, false) else {
                continue;
            };
            if !contact.from_spikes {
                push(
                    &mut walker.position,
                    &mut walker.velocity,
                    Vec2::ZERO,
                    contact.away.normalize_or_zero() * fire.push_per_frame.get(),
                );
            }
            if due(walker.last_burned) {
                walker.last_burned = Some(frame);
                if !contact.from_spikes {
                    sounds.push(SoundEvent::FireHiss);
                }
                if walker.hurt(damage, &parameters, random) == Hurt::Died {
                    sounds.push(SoundEvent::EnemyDied);
                }
            }
        }
        self.release_flies(random);
    }
}
