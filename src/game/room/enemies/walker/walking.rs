//! How a Gaper walks: the walker case of `smarts()` (DoAction 24985–25140) picks its clip from its
//! speed (`walkframe()`, 8528), pulls towards where it is heading (`pathfind()`, 5777–5990) and turns
//! to face its way (`sideflip()`, 5757).
//!
//! It heads straight for the player while a clear line to them was seen (looked for every sixth
//! frame) and otherwise for the next tile on the shortest way round, from a flood fill over the
//! room's tiles that is redone as its timer runs out. The crowd avoidance that makes walkers
//! prefer tiles other enemies are not on is not ported.

use glam::Vec2;

use super::{Route, Walker, WalkerClip};
use crate::game::original_units::{Frames, PixelsPerFrame};
use crate::game::random::Random;
use crate::game::room::enemies::EnemyParameters;
use crate::game::room::enemies::surroundings::Grid;
use crate::game::room::enemies::walk_map::Tile;

use super::headless::STANDSTILL_SPEED;

impl Walker {
    /// Returns whether it growled at the sight of the player.
    pub(super) fn walk(
        &mut self,
        frame: Frames,
        grid: &mut Grid,
        parameters: &EnemyParameters,
        enemy_count: usize,
        random: &mut Random,
    ) -> bool {
        self.choose_clip(parameters);
        let growled = self.pull_towards_player(frame, grid, parameters, enemy_count, random);
        self.face(self.velocity.x);
        growled
    }

    /// `walkframe()`: only the standing and walking clips change (DoAction 8535), walking when the
    /// speed times the factor is above 2 pixels per frame (DoAction 8537), standing otherwise
    /// (DoAction 8539); the head `h` is its own clip that both frames share, so changing clip leaves it.
    fn choose_clip(&mut self, parameters: &EnemyParameters) {
        let speed = PixelsPerFrame(self.velocity.length()).max(STANDSTILL_SPEED);
        let walking = speed > parameters.gapers.walking_speed;
        self.clip = match (self.clip, walking) {
            (WalkerClip::Standing, true) => WalkerClip::Walking { since: Frames::ZERO },
            (WalkerClip::Walking { .. }, false) => WalkerClip::Standing,
            (unchanged, _) => unchanged,
        };
    }

    /// `pathfind()` towards the player: nothing in the room's first frames (DoAction 5789), its cell
    /// marked for the others' line checks, a look for a clear line from its cell's center to the
    /// player's every sixth frame (DoAction 5832, `linecheck()`) with a growl (DoAction 5842), the
    /// shouting head speeding it up (DoAction 5859, DoAction 5864) and a sighting costing the flood
    /// fill's timer (DoAction 5874).
    fn pull_towards_player(
        &mut self,
        frame: Frames,
        grid: &mut Grid,
        parameters: &EnemyParameters,
        enemy_count: usize,
        random: &mut Random,
    ) -> bool {
        let gapers = parameters.gapers;
        if frame <= gapers.idle_frames || self.position == grid.player {
            self.velocity *= gapers.arrived_retention;
            return false;
        }
        let mut growled = false;
        grid.sight.mark_walker(self.position);
        let here = Tile::containing(self.position);
        let player_tile = Tile::containing(grid.player);
        if (frame + Frames::from_number(self.phase)).is_on_cycle(gapers.sight_check_every, gapers.sight_check_phase) {
            self.route.line_clear = grid.sight.walk_line_clear(here.center(), player_tile.center());
            growled = self.route.line_clear
                && self
                    .route
                    .last_growl
                    .is_none_or(|last| last + gapers.growl_spacing <= frame)
                && random.happens(gapers.growl_odds);
            if growled {
                self.route.last_growl = Some(frame);
            }
        }
        let mut strength = gapers.chase_strength;
        if !self.frantic {
            strength *= gapers.calm_factor;
        }
        strength *= self.shout(parameters);
        let sighted = self.route.line_clear || here == player_tile;
        let target = if sighted {
            if self.frantic {
                self.keep_shouting(parameters);
                strength *= gapers.frantic_sight_factor;
            }
            self.route.refill_timer -= gapers.path_timer_sighted_cost;
            Some(grid.player)
        } else {
            self.route
                .follow_flood(here, player_tile, strength, grid, parameters, enemy_count)
        };
        let Some(target) = target else {
            self.velocity *= gapers.stuck_retention;
            return growled;
        };
        self.pull_toward(target, strength, parameters);
        growled
    }

    /// `pathfind()`'s pull towards `target`: a tenth of the gap at most two pixels per frame, times
    /// `strength`, once farther than a pixel (DoAction 5964, DoAction 5973, DoAction 5974,
    /// DoAction 5975); with nowhere to go it keeps half its speed (DoAction 5983, DoAction 5984),
    /// standing on its goal 85 % (DoAction 5990, DoAction 5991).
    pub(super) fn pull_toward(&mut self, target: Vec2, strength: f32, parameters: &EnemyParameters) {
        let gapers = parameters.gapers;
        let gap = target - self.position;
        let distance = gap.length();
        if distance > 1.0 {
            let pull = (distance * gapers.pull_per_pixel).min(gapers.pull_maximum) / distance * strength;
            self.velocity *= gapers.pulling_retention;
            self.velocity += gap * pull;
        }
    }

    /// Its head's shouting (past its second frame) speeds it up and goes on; a frantic head waits on
    /// its second frame. Returns the speed factor.
    fn shout(&mut self, parameters: &EnemyParameters) -> f32 {
        let last = parameters.gapers.head_frame_count - 1;
        if self.head >= 2 {
            self.head = (self.head + 1).min(last);
            parameters.gapers.shouting_factor
        } else {
            if self.frantic {
                self.head = 1;
            }
            1.0
        }
    }

    /// A frantic Gaper that sees the player moves its head on towards the shouting.
    fn keep_shouting(&mut self, parameters: &EnemyParameters) {
        self.head = (self.head + 1).min(parameters.gapers.head_frame_count - 1);
    }
}

impl Route {
    /// The center of the next tile on the way to the player, from a flood fill redone when its timer
    /// has run out (none if the player cannot be reached): DoAction 5891, DoAction 5921 for the
    /// unreachable, DoAction 5925 for the timer, DoAction 5927 and DoAction 5928 for its cost, and
    /// the tiles around weighed east, west, north, south (DoAction 5943, DoAction 5951).
    fn follow_flood(
        &mut self,
        here: Tile,
        player_tile: Tile,
        strength: f32,
        grid: &Grid,
        parameters: &EnemyParameters,
        enemy_count: usize,
    ) -> Option<Vec2> {
        let gapers = parameters.gapers;
        if self.distances.is_none() || self.refill_timer < Frames::ZERO {
            let distances = grid.walk_map.distances_to(player_tile, here);
            let steps = distances
                .steps_from(here)
                .map_or(gapers.path_timer_longest, |steps| Frames::from(steps) + Frames::ONE);
            let enemies = f32::from(u16::try_from(enemy_count).unwrap_or(u16::MAX));
            self.refill_timer =
                steps.min(gapers.path_timer_longest) + gapers.path_timer_per_enemy * enemies + gapers.path_timer_base;
            self.distances = Some(distances);
        }
        self.refill_timer -= Frames(strength * gapers.path_timer_per_step);
        self.distances.as_ref()?.next_step(here).map(Tile::center)
    }
}
