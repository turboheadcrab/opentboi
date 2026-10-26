//! The headless body: a Gaper that lost its head (`hurt()`, DoAction 4495–4515), or a layout's own.
//! It wanders (`randrun()`, 8614–8660): with no goal it picks a spot 40 pixels off in a random
//! direction, if the tile there is free, and pulls towards it by half a pixel per frame, giving it
//! up on arriving or when nearly stopped. One that spurts (`bnuts`) shoots a blood ball ahead of it
//! every 40 to 60 frames (`firewalk()`, 9712–9750). Its clip is chosen as the Gaper's is
//! (`walkframe(2)`, 8528).

use glam::Vec2;

use super::{Walker, WalkerClip, WalkerKind, WalkerShot, WalkerState, WalkerTurn};
use crate::game::original_units::{Frames, PixelsPerFrame};
use crate::game::parameters::HeadlessParameters;
use crate::game::random::Random;
use crate::game::room::blood::Splatter;
use crate::game::room::enemies::EnemyParameters;
use crate::game::room::enemies::surroundings::Grid;

/// `enfget()`: a standstill's speed, so that dividing by it is safe.
pub(super) const STANDSTILL_SPEED: PixelsPerFrame = PixelsPerFrame(0.001);

impl Walker {
    /// The Gaper's head is gone: it carries on from where it stood, at full health, turned about by
    /// nothing, with the blood it already had.
    pub(super) fn lose_head(&mut self, parameters: &EnemyParameters) {
        self.hit_points = self.kind.body(parameters).hit_points;
        self.kind = WalkerKind::Headless;
        self.state = WalkerState::Active;
        self.clip = WalkerClip::Standing;
        self.clip_age = Frames::ZERO;
        self.goal = None;
        self.spurt_cooldown = Frames::ZERO;
        self.flash_left = Frames::ZERO;
    }

    /// One frame of the headless body's case of `smarts()`; one frame in ten it bleeds on the floor
    /// (`splater()`).
    pub(super) fn wander(&mut self, grid: &Grid, parameters: &EnemyParameters, random: &mut Random) -> WalkerTurn {
        let headless = parameters.headless;
        let splats = &parameters.splats;
        let shot = if self.spurts {
            self.spurt(parameters, random)
        } else {
            None
        };
        self.velocity *= headless.velocity_retention;
        let _ = self.wander_step(grid, &headless, random);
        let spilled = random
            .happens(splats.headless_odds)
            .then(|| Splatter::any_look_or_last(self.position, splats.headless, splats, random));
        self.choose_wander_clip(&headless);
        self.face(self.velocity.x);
        WalkerTurn {
            shot,
            spilled,
            ..WalkerTurn::default()
        }
    }

    /// `randrun()`: one frame of wandering, shared with the Clotty and the Mulligan.
    /// Returns whether it gave up its goal this frame (arrived, or nearly stopped).
    pub(super) fn wander_step(&mut self, grid: &Grid, headless: &HeadlessParameters, random: &mut Random) -> bool {
        if self.goal.is_none() {
            self.goal = pick_goal(self.position, grid, headless, random);
        }
        self.head_for_goal(headless, random)
    }

    /// `firewalk()`: a blood ball ahead of where it is going (DoAction 9734, DoAction 9750, DoAction 9751, DoAction 9752), once
    /// the cooldown has run out (DoAction 9749).
    fn spurt(&mut self, parameters: &EnemyParameters, random: &mut Random) -> Option<WalkerShot> {
        let mut shot = None;
        if self.spurt_cooldown <= Frames::ZERO {
            let speed = PixelsPerFrame(self.velocity.length()).max(STANDSTILL_SPEED);
            shot = Some(WalkerShot::Spurted(
                self.velocity * (parameters.shots.spurt_speed / speed),
            ));
            self.spurt_cooldown =
                parameters.headless.spurt_cooldown_base + random.frames_below(parameters.headless.spurt_cooldown_range);
        }
        self.spurt_cooldown -= Frames::ONE;
        shot
    }

    /// The pull towards its goal, which it gives up on arriving or when nearly stopped (the pull of
    /// this frame is still made).
    fn head_for_goal(&mut self, headless: &HeadlessParameters, random: &mut Random) -> bool {
        let Some(goal) = self.goal else { return false };
        let away = self.position - goal;
        let distance = away.length();
        let gave_up = distance < headless.arrival_distance.get()
            || (PixelsPerFrame(self.velocity.length()) < headless.stalled_speed
                && random.happens(headless.stalled_odds));
        if gave_up {
            self.goal = None;
        }
        self.velocity -= away * (headless.pull.get() / distance.max(STANDSTILL_SPEED.get()));
        self.face(-away.x);
        gave_up
    }

    /// `walkframe(2)` (DoAction 8535, 8537 and 8539): walking while twice its speed is above 2 pixels
    /// per frame; changing clip starts the new one over.
    pub(super) fn choose_wander_clip(&mut self, headless: &HeadlessParameters) {
        let walking = PixelsPerFrame(self.velocity.length() * headless.walking_speed_scale) > headless.walking_speed;
        let clip = match (self.clip, walking) {
            (WalkerClip::Standing, true) => Some(WalkerClip::Walking { since: Frames::ZERO }),
            (WalkerClip::Walking { .. }, false) => Some(WalkerClip::Standing),
            _ => None,
        };
        if let Some(clip) = clip {
            self.clip = clip;
            self.clip_age = Frames::ZERO;
        }
    }
}

/// `randrun()`'s pick: a point 40 pixels away in a random direction (kept in the room's middle one
/// time in two), the center of the tile there if it is free.
fn pick_goal(from: Vec2, grid: &Grid, headless: &HeadlessParameters, random: &mut Random) -> Option<Vec2> {
    let mut point = from + random.direction(headless.goal_distance.get());
    if !random.happens(headless.goal_middle_odds) {
        point = point.clamp(headless.middle_minimum, headless.middle_maximum);
    }
    point = point.clamp(headless.room_minimum, headless.room_maximum);
    grid.walk_map.free_spot(point)
}
