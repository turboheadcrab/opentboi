//! Enemies coming and going: what they leave behind when they die (their blood, `splater()` in
//! `physix()`; a Mulligan's flies, `flya()`, DoAction 10509–10535; a red fly's burst of shots and a
//! Moter's two attack flies, `deathscripts()`, DoAction 13106 and 13564), a pecking fly becoming a
//! Pooter, and the dead taken away.

use glam::Vec2;

use crate::game::original_units::Frames;
use crate::game::player::death::Killer;
use crate::game::random::Random;
use crate::game::room::blood::Splatter;
use crate::game::room::enemies::fly::Legacy;
use crate::game::room::enemies::{Enemies, Fly, FlyState, Walker, WalkerState};
use crate::game::texture_id::FlyKind;

impl Enemies {
    /// Removes the flies whose death has played (after the room resolved this frame's pecks).
    pub fn remove_finished(&mut self) {
        let parameters = self.parameters;
        self.flies.retain(|fly| !fly.death_finished(&parameters));
        self.walkers.retain(|walker| !walker.death_finished(&parameters));
    }

    /// The blood left since the room last asked: the enemies' and their shots', and what walkers
    /// spilled losing their heads.
    pub fn take_splatters(&mut self) -> Vec<Splatter> {
        self.splatters
            .extend(self.walkers.iter_mut().filter_map(Walker::take_spilled));
        std::mem::take(&mut self.splatters)
    }

    /// `physix()` for the dead: on each of its death clip's first three frames an enemy leaves a
    /// splat, a Pooter a smaller one; `into_turn` is how far into this turn the frame is.
    pub(super) fn bleed_dying(&mut self, into_turn: Frames, random: &mut Random) {
        let splats = self.parameters.splats;
        let bleeding = |state_frames: Frames| state_frames + into_turn < splats.dying_frames;
        for fly in &self.flies {
            if let FlyState::Dying(frames) = fly.state
                && bleeding(frames)
            {
                let splatter = Splatter::any_look(fly.position, splats.dying, &splats, random);
                // Both Pooters are type 14 (`create()` rounds the marker's type).
                self.splatters.push(if fly.kind.is_pooter() {
                    splatter.scaled(splats.pooter_dying_factor)
                } else {
                    splatter
                });
            }
        }
        for walker in &self.walkers {
            if let WalkerState::Dying(frames) = walker.state
                && bleeding(frames)
            {
                self.splatters
                    .push(Splatter::any_look(walker.position, splats.dying, &splats, random));
            }
        }
    }

    /// The fly at `index` landed a peck on its breakable and turns into a Pooter.
    pub fn peck_landed(&mut self, index: usize, random: &mut Random) {
        let parameters = self.parameters;
        if let Some(fly) = self.flies.get_mut(index) {
            fly.become_pooter(&parameters, random);
        }
    }

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
