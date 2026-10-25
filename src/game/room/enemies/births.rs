//! What enemies leave behind when they die: a Mulligan's flies (`flya()`, DoAction 10509–10535), a
//! red fly's burst of shots, and a Moter's two attack flies (`deathscripts()`, DoAction 13106 and
//! 13564).

use glam::Vec2;

use crate::game::player::death::Killer;
use crate::game::random::Random;
use crate::game::room::enemies::fly::Legacy;
use crate::game::room::enemies::{Enemies, Fly};
use crate::game::texture_id::FlyKind;

impl Enemies {
    /// Releases the flies that dying or bursting Mulligans owe the room: each at a random spot near
    /// where it was.
    pub(super) fn release_flies(&mut self, random: &mut Random) {
        self.release_mulligan_flies(random);
        let parameters = self.parameters;
        let owing: Vec<(Legacy, Vec2)> = self
            .flies
            .iter_mut()
            .filter_map(|fly| fly.take_legacy().map(|legacy| (legacy, fly.position)))
            .collect();
        for (legacy, position) in owing {
            match legacy {
                Legacy::Burst => {
                    self.shoot_cross(position, parameters.flies.red_burst_speed, Killer::Enemy(FlyKind::Red));
                }
                Legacy::Split => self.split_moter(position, random),
            }
        }
    }

    fn release_mulligan_flies(&mut self, random: &mut Random) {
        let parameters = self.parameters;
        let mulligans = parameters.mulligans;
        let owing: Vec<Vec2> = self
            .walkers
            .iter_mut()
            .filter_map(|walker| walker.take_owed_flies().then_some(walker.position))
            .collect();
        for position in owing {
            for _ in 0..mulligans.fly_count {
                let kind = mulligans.fly_kinds[random.index(mulligans.fly_kinds.len())];
                let spread = random.next_unit() * mulligans.fly_spread.get();
                let spot = position + random.direction(spread);
                let phase = u32::try_from(self.flies.len() + self.walkers.len()).unwrap_or(u32::MAX);
                self.flies.push(Fly::new(kind, spot, phase, &parameters, random));
            }
        }
    }

    /// A dying Moter's two attack flies: either side of it across the line to the player (a step
    /// of 10 px at right angles to it), each sent off that way.
    fn split_moter(&mut self, position: Vec2, random: &mut Random) {
        let parameters = self.parameters;
        let player = self.player;
        let across = Vec2::new(player.y - position.y, position.x - player.x);
        let step = if across.length() > 0.0 {
            across.normalize() * parameters.flies.moter_split_offset.get()
        } else {
            random.direction(parameters.flies.moter_split_offset.get())
        };
        for side in [step, -step] {
            let phase = u32::try_from(self.flies.len() + self.walkers.len()).unwrap_or(u32::MAX);
            let mut fly = Fly::new(FlyKind::Attack, position + side, phase, &parameters, random);
            fly.velocity = side;
            self.flies.push(fly);
        }
    }
}
