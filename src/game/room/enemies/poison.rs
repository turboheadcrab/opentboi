//! Poison (`poiss`, `poisd`): a poisoned enemy takes some damage every 20th frame while the poison
//! lasts, and is tinted green, less so as it wears off (DoAction 13826–13835, DoAction 1884–1892).

use glam::Vec2;

use crate::game::draw::Rgba;
use crate::game::original_units::{Damage, Frames, distance};
use crate::game::parameters::PillParameters;
use crate::game::random::Random;
use crate::game::room::enemies::{Enemies, Walker};
use crate::game::sound_event::SoundEvent;

/// A poisoned enemy looks at least this far recovered (DoAction 1886).
const LEAST_RECOVERY: f32 = 0.6;
const RECOVERY_OFFSET: f32 = 0.1;
/// The tint's red and blue share of its recovery, with the base, and the green's (DoAction 1892).
const RECOVERY_SHARE: f32 = 0.25;
const RECOVERY_BASE: f32 = 0.75;
const GREEN_BASE: f32 = 0.9;
const GREEN_SHARE: f32 = 0.2;

/// One enemy's poison.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Poison {
    number: u32,
    frames_left: Frames,
    total_frames: Frames,
    every: Frames,
    damage: Damage,
}

impl Poison {
    /// It hurts on the frame after each multiple of its period, counting down (DoAction 13829).
    fn hurts_now(&self) -> bool {
        self.frames_left > Frames::ZERO && self.frames_left.whole() % self.every.whole() == 1
    }
}

impl Enemies {
    /// Bad Gas (`fart()`, DoAction 26706–26718): every live enemy within `gas_reach` of `center`
    /// (world), plus its own size, is hurt and poisoned.
    pub fn poison_near(
        &mut self,
        center: Vec2,
        pills: &PillParameters,
        random: &mut Random,
        sounds: &mut Vec<SoundEvent>,
    ) {
        let reached: Vec<u32> = self
            .friend_targets()
            .into_iter()
            .filter(|target| distance(center, target.position) < pills.gas_reach + target.size)
            .map(|target| target.number)
            .collect();
        for number in reached {
            self.poisons.retain(|poison| poison.number != number);
            self.poisons.push(Poison {
                number,
                frames_left: pills.poison_frames,
                total_frames: pills.poison_frames,
                every: pills.poison_every,
                damage: pills.poison_damage,
            });
            self.hurt_by_friend(number, pills.gas_damage, random, sounds);
        }
    }

    /// Each frame the poison counts down and may hurt; the poison of the dead is gone.
    pub(super) fn advance_poison(&mut self, frames_crossed: u32, random: &mut Random, sounds: &mut Vec<SoundEvent>) {
        for _ in 0..frames_crossed {
            let mut hurts = Vec::new();
            for poison in &mut self.poisons {
                poison.frames_left = (poison.frames_left - Frames::ONE).max(Frames::ZERO);
                if poison.hurts_now() {
                    hurts.push((poison.number, poison.damage));
                }
            }
            for (number, damage) in hurts {
                self.hurt_by_friend(number, damage, random, sounds);
            }
        }
        let living = self.living_numbers();
        self.poisons
            .retain(|poison| poison.frames_left > Frames::ZERO && living.contains(&poison.number));
    }

    /// The tint of the poisoned enemy `number`, if it is (DoAction 1886–1892).
    pub fn poison_tint(&self, number: u32) -> Option<Rgba> {
        let poison = self.poisons.iter().find(|poison| poison.number == number)?;
        let recovered = (1.0 - poison.frames_left / poison.total_frames - RECOVERY_OFFSET).max(LEAST_RECOVERY);
        let red_and_blue = recovered * RECOVERY_SHARE + RECOVERY_BASE;
        Some(Rgba {
            red: red_and_blue,
            green: GREEN_BASE + GREEN_SHARE * (1.0 - recovered),
            blue: red_and_blue,
            alpha: 1.0,
        })
    }

    fn living_numbers(&self) -> Vec<u32> {
        let flies = self
            .flies
            .iter()
            .filter(|fly| fly.is_alive())
            .map(|fly| fly.shooter.phase);
        let walkers = self
            .walkers
            .iter()
            .filter(|walker| walker.is_alive())
            .map(Walker::phase);
        flies.chain(walkers).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::parameters::GameParameters;
    use crate::game::room::enemies::FriendTarget;
    use crate::game::room::enemies::tests::{FAR_PLAYER, room_with_flies, run_with_player};

    /// The black fly's marker value.
    const FLY_MARKER: f32 = -4.0;

    /// Lets the enemies appear, and returns the `count` that are out and about.
    fn settled(enemies: &mut Enemies, count: usize, random: &mut Random) -> Vec<FriendTarget> {
        for _ in 0..400 {
            if enemies.friend_targets().len() >= count {
                break;
            }
            run_with_player(enemies, 1, FAR_PLAYER, random);
        }
        let targets = enemies.friend_targets();
        assert_eq!(targets.len(), count, "{targets:?}");
        targets
    }

    #[test]
    /// DoAction 26706–26718 and 13826–13835: gas poisons the enemies near and not those far, and the
    /// poison goes on hurting every twentieth frame until the enemy is dead.
    fn gas_poisons_the_enemies_near_and_the_poison_hurts_on_its_own() {
        let pills = GameParameters::ORIGINAL.pills;
        let mut enemies = room_with_flies(&[FLY_MARKER, FLY_MARKER]);
        let mut random = Random::with_seed(2);
        let targets = settled(&mut enemies, 2, &mut random);
        let (near, far) = (targets[0], targets[1]);
        enemies.poison_near(near.position, &pills, &mut random, &mut Vec::new());
        assert!(enemies.poison_tint(near.number).is_some());
        assert!(enemies.poison_tint(far.number).is_none(), "out of reach");
        run_with_player(&mut enemies, 220, FAR_PLAYER, &mut random);
        let alive: Vec<u32> = enemies.friend_targets().iter().map(|target| target.number).collect();
        assert!(!alive.contains(&near.number), "the poison killed it");
        assert!(alive.contains(&far.number));
        assert!(enemies.poison_tint(near.number).is_none(), "and is gone with it");
    }

    #[test]
    /// DoAction 1886: the tint fades as the poison wears off.
    fn the_tint_is_greenest_at_the_start() {
        let pills = GameParameters::ORIGINAL.pills;
        let mut enemies = room_with_flies(&[FLY_MARKER]);
        let mut random = Random::with_seed(2);
        let target = settled(&mut enemies, 1, &mut random)[0];
        enemies.poison_near(target.position, &pills, &mut random, &mut Vec::new());
        let tint = enemies.poison_tint(target.number).expect("poisoned");
        assert!(tint.green > tint.red && tint.red < 1.0, "{tint:?}");
    }
}
