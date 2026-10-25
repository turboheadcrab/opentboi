//! The Mulligan (type 16): the case for it in `smarts()` (DoAction 24276–24372). It keeps away from
//! the player (`angstfind()`, 9766–9921): every 14th frame it looks for them, and one in view within
//! 250 pixels (170 if hidden) scares it for three looks. Scared, it is pushed away and looks for a
//! refuge out of the player's view to run to; calm, it wanders as the headless body does. Pressed
//! against a wall or a rock it bursts one frame in 300, firing four shots and leaving flies; killed
//! it leaves the flies too, unless it was blown far past death.

use super::{Walker, WalkerClip, WalkerShot, WalkerState, WalkerTurn};
use crate::game::original_units::{Frames, Pixels, distance};
use crate::game::parameters::MulliganParameters;
use crate::game::random::Random;
use crate::game::room::enemies::EnemyParameters;
use crate::game::room::enemies::fly::FlySurroundings;
use crate::game::room::enemies::surroundings::Grid;

impl Walker {
    /// One frame of the Mulligan's case of `smarts()`.
    pub(super) fn run_away(
        &mut self,
        frame: Frames,
        grid: &Grid,
        parameters: &EnemyParameters,
        random: &mut Random,
    ) -> WalkerTurn {
        let mulligan = parameters.mulligans;
        let mut turn = WalkerTurn::default();
        self.velocity *= mulligan.velocity_retention;
        if let WalkerClip::Shooting { since } = self.clip {
            if since >= mulligan.burst_after && !self.blob.shot_done {
                self.blob.shot_done = true;
                self.blob.owes_flies = true;
                self.state = WalkerState::Dying(Frames::ZERO);
                turn.shot = Some(WalkerShot::Cross(mulligan.shot_speed));
            }
            return turn;
        }
        self.look_out_for_the_player(frame, grid, parameters, random, &mut turn);
        self.face(self.velocity.x);
        self.choose_wander_clip(&parameters.headless);
        if self.blob.cornered && random.happens(mulligan.burst_odds) {
            self.start_clip(WalkerClip::Shooting { since: Frames::ZERO });
            self.blob.shot_done = false;
        }
        turn
    }

    /// `angstfind()` (DoAction 9779, DoAction 9801, DoAction 9822, DoAction 9826, DoAction 9829,
    /// DoAction 9900, DoAction 9902, DoAction 9903, DoAction 9908, DoAction 9913, DoAction 9917,
    /// DoAction 9918). Its cells already tried (`checked`, DoAction 9808, DoAction 9858) and the
    /// count of refuges that were too close (`noco`, DoAction 9873) are not kept.
    fn look_out_for_the_player(
        &mut self,
        frame: Frames,
        grid: &Grid,
        parameters: &EnemyParameters,
        random: &mut Random,
        turn: &mut WalkerTurn,
    ) {
        let mulligan = parameters.mulligans;
        let number = Frames::from_number(self.phase);
        if (frame + number).is_on_cycle(mulligan.look_every, mulligan.look_phase) {
            self.look_for_the_player(frame, grid, &mulligan, random, turn);
        }
        if self.blob.fear_looks <= 0 {
            self.blob.search_failures *= mulligan.failed_search_decay;
            let _ = self.wander_step(grid, &parameters.headless, random);
            self.velocity *= mulligan.wandering_retention;
            return;
        }
        let away = self.position - grid.player;
        let gap = Pixels(away.length());
        let near = if gap < mulligan.fear_reach { gap.get() } else { 0.0 };
        if self.goal.is_none() || self.blob.search_failures > mulligan.failed_search_limit {
            if near > 0.0 {
                self.velocity += away * (mulligan.flee_push / near);
            }
            let anchor = if near > 0.0 {
                self.position + away * (mulligan.flee_anchor_distance.get() / near)
            } else {
                self.position
            };
            self.look_for_a_refuge(anchor, grid, &mulligan, random);
        }
        let Some(refuge) = self.goal else { return };
        let checking = (frame + number).is_on_cycle(mulligan.refuge_check_every, mulligan.refuge_check_phase);
        let seen = grid.in_sight(grid.player, refuge) && self.blob.search_failures < mulligan.failed_search_limit;
        let too_close = distance(grid.player, refuge) < mulligan.refuge_danger_reach;
        if checking && (seen || too_close) {
            self.goal = None;
            return;
        }
        self.pull_toward(refuge, mulligan.flee_strength, parameters);
    }

    /// Every 14th frame: whether the player is in view, a whimper if so, and whether they are close
    /// enough to be scared of.
    /// (DoAction 9785 for the whimper.)
    fn look_for_the_player(
        &mut self,
        frame: Frames,
        grid: &Grid,
        mulligan: &MulliganParameters,
        random: &mut Random,
        turn: &mut WalkerTurn,
    ) {
        let in_view = grid.in_sight(self.position, grid.player);
        let reach = if in_view {
            mulligan.sighted_fear_reach
        } else {
            mulligan.hidden_fear_reach
        };
        if in_view && self.may_whimper(frame, mulligan, random) {
            self.blob.last_whimper = Some(frame);
            turn.whimpered = true;
        }
        if distance(self.position, grid.player) < reach {
            if self.blob.fear_looks <= 0 {
                self.goal = None;
            }
            self.blob.fear_looks = mulligan.fear_looks;
        } else {
            self.blob.fear_looks -= 1;
        }
    }

    fn may_whimper(&self, frame: Frames, mulligan: &MulliganParameters, random: &mut Random) -> bool {
        let pause = mulligan.whimper_pause_base + random.frames_below(mulligan.whimper_pause_range);
        self.blob.last_whimper.is_none_or(|last| last + pause <= frame) && random.happens(mulligan.whimper_odds)
    }

    /// Up to a few tries at a random place beyond the player's reach, in view of it and out of the
    /// player's.
    /// (DoAction 9777, DoAction 9839, DoAction 9854, DoAction 9869, DoAction 9881, DoAction 9884.)
    fn look_for_a_refuge(
        &mut self,
        anchor: glam::Vec2,
        grid: &Grid,
        mulligan: &MulliganParameters,
        random: &mut Random,
    ) {
        let mut tries = 0.0;
        while self.goal.is_none() && tries < mulligan.flee_tries - self.blob.search_failures * mulligan.flee_try_penalty
        {
            let spread = mulligan.flee_spread_minimum.get() + mulligan.flee_spread_range.get() * random.next_unit();
            let offset = random.direction(spread);
            let candidate = anchor + glam::Vec2::new(offset.x, offset.y * mulligan.flee_height_scale);
            if let Some(spot) = grid.walk_map.free_spot(candidate)
                && grid.in_sight(self.position, spot)
            {
                let hidden = !grid.in_sight(grid.player, spot);
                if hidden || self.blob.search_failures >= mulligan.failed_search_limit {
                    self.goal = Some(spot);
                    if hidden {
                        self.blob.search_failures /= 2.0;
                    }
                } else {
                    self.blob.search_failures += mulligan.failed_search_cost;
                }
            }
            tries += 1.0;
        }
    }
}
