//! What one blue fly does in a frame: where it heads, and what it hurts where it is.

use std::f32::consts::PI;

use glam::Vec2;

use super::{Aim, BlueFly, ENEMY_SHOT_SIZE, Surroundings};
use crate::game::original_units::{Damage, Frames, distance};
use crate::game::parameters::BlueFlyParameters;
use crate::game::random::Random;
use crate::game::room::enemies::FriendTarget;
use crate::game::sound_event::SoundEvent;

/// Where it heads and the push towards it.
#[allow(clippy::too_many_arguments, reason = "everything one fly's frame looks at")]
pub(super) fn steer(
    fly: &mut BlueFly,
    index: usize,
    crowd: usize,
    frame: Frames,
    parameters: &BlueFlyParameters,
    player: Vec2,
    targets: &[FriendTarget],
    recent: Option<u32>,
    random: &mut Random,
) {
    let every_other = frame.is_on_cycle(parameters.steer_every, Frames::ZERO);
    if let Aim::Enemy(number) = fly.aim {
        match targets.iter().find(|target| target.number == number) {
            Some(target) => fly.goal = target.position,
            None => fly.aim = Aim::Player,
        }
    }
    if fly.aim == Aim::Player {
        if every_other {
            fly.goal = player + ring_offset(index, crowd, frame, parameters);
        }
        if let Some(number) = recent
            && random.happens(parameters.target_odds)
        {
            fly.aim = Aim::Enemy(number);
        }
    }
    if every_other {
        let gap = fly.goal - fly.position;
        let length = gap.length();
        if length > 0.0 {
            let pull = (length * parameters.steer_gain).min(parameters.steer_limit) / length * parameters.steer_scale;
            fly.velocity += gap * pull;
        }
    }
    fly.velocity *= parameters.velocity_retention;
}

/// Its place on the ring around Isaac (world, y up).
fn ring_offset(index: usize, crowd: usize, frame: Frames, parameters: &BlueFlyParameters) -> Vec2 {
    let share = index as f32 / crowd.max(1) as f32;
    let angle = (frame.get() * parameters.orbit_half_turns_per_frame + share * parameters.orbit_shared_half_turns) * PI;
    let growth = 1.0 + crowd.saturating_sub(parameters.crowd_from) as f32 * parameters.crowd_growth;
    Vec2::new(
        angle.sin() * parameters.orbit_width.get() * growth,
        -angle.cos() * parameters.orbit_height.get() * growth,
    )
}

/// What it hurts where it is: enemies (once in a while each) and shots; whether it dies of it.
pub(super) fn touches(
    fly: &mut BlueFly,
    damage: Damage,
    parameters: &BlueFlyParameters,
    targets: &[FriendTarget],
    world: &mut Surroundings,
    random: &mut Random,
    sounds: &mut Vec<SoundEvent>,
) -> bool {
    for target in targets {
        if fly.hurt.contains(&target.number) || distance(fly.position, target.position) >= parameters.size + target.size
        {
            continue;
        }
        fly.hurt.push(target.number);
        world.enemies.hurt_by_friend(target.number, damage, random, sounds);
        if random.happens(parameters.death_odds) {
            return true;
        }
    }
    world
        .enemies
        .absorb_shot_near(fly.position, parameters.size + ENEMY_SHOT_SIZE)
        && random.happens(parameters.death_odds)
}
