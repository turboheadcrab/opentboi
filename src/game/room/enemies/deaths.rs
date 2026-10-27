//! Counting the enemies that died, for what a kill sets off (`hurt()`, DoAction 4411–4419).

use crate::game::room::enemies::{Enemies, Fly, Walker};

impl Enemies {
    /// How many enemies died since the last call, however they were hurt. An enemy counts when it was
    /// alive at the last call and is not any more; the ones that appear in between are noticed at
    /// the next call.
    pub fn take_deaths(&mut self) -> usize {
        let living: Vec<u32> = self
            .flies
            .iter()
            .filter(|fly| Fly::is_alive(fly))
            .map(|fly| fly.shooter.phase)
            .chain(
                self.walkers
                    .iter()
                    .filter(|walker| Walker::is_alive(walker))
                    .map(Walker::phase),
            )
            .collect();
        let seen = std::mem::replace(&mut self.living_seen, living);
        seen.iter().filter(|number| !self.living_seen.contains(number)).count()
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use crate::game::original_units::Damage;
    use crate::game::random::Random;
    use crate::game::room::enemies::tests::{ONE_FRAME, room_with_flies};
    use crate::game::room::stage_to_world;

    /// The ordinary fly's marker value.
    const FLY_MARKER: f32 = -52.0;

    #[test]
    fn a_death_is_counted_once() {
        let mut enemies = room_with_flies(&[FLY_MARKER, FLY_MARKER]);
        assert_eq!(enemies.take_deaths(), 0, "the first call only takes note of who lives");
        let point = stage_to_world(enemies.flies[0].position);
        let mut sounds = Vec::new();
        enemies.hit_by_tear(
            point,
            Vec2::X,
            Damage(1000.0),
            ONE_FRAME,
            &mut Random::with_seed(1),
            &mut sounds,
        );
        assert_eq!(enemies.take_deaths(), 1);
        assert_eq!(enemies.take_deaths(), 0);
    }
}
