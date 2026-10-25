//! Enemies' values: the flies (`alive()`'s fly branch, DoAction 13907–14118). The fly (type 13,
//! black, `b13`) wanders and drifts towards breakables and never hurts; the attack fly (type 18,
//! `b18`), the red fly (type 61, `b61`) and the Moter (type 80, `b80`) go for the player and hurt
//! on contact.

use glam::Vec2;

use crate::game::original_units::{Damage, Frames, Hearts, HitPoints, Mass, Pixels, PixelsPerFrame, Retention};

/// Where a fly heads for (`goshit`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FlyGoal {
    /// A breakable it can see within reach, which it rests beside and may peck.
    Breakables,
    /// The player, whom it rests this far from.
    Player(Pixels),
}

/// What differs between the kinds of fly.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlyKindParameters {
    /// 17 for types 13, 14 and 18, 13 for types 61 and 80 (`sizes`).
    pub size: Pixels,
    pub goal: FlyGoal,
    /// Touching the player hurts (types 18, 61 and 80).
    pub hurts_on_touch: bool,
    /// Hit points (3 for type 13, 5 for type 18); a fly dies once below zero.
    pub hit_points: HitPoints,
    /// The appearing clip (`d` frame 4: 1516, 45 frames; 5052, 26 frames) plays out before it flies.
    pub appearing_frames: Frames,
    /// 0.17 (0.3 for type 18): the drift gains twice this in a random direction.
    pub jitter: f32,
    /// Dying plays `d` frame 3 one frame each frame: 555 (16 frames, the last empty) for the
    /// flies, 4948 (15) for the Pooter.
    pub dying_frames: Frames,
    /// 3 for types 13 and 18, 4 for type 14.
    pub mass: Mass,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlyParameters {
    pub black: FlyKindParameters,
    pub attack: FlyKindParameters,
    pub pooter: FlyKindParameters,
    pub red: FlyKindParameters,
    pub moter: FlyKindParameters,
    /// While appearing the fly keeps this much of its speed each frame.
    pub appearing_retention: Retention,
    /// Every third frame: a random spot up to 240 px away (the distance the product of two random
    /// fractions) becomes its goal if a breakable stands there.
    pub goal_every: Frames,
    pub goal_search_reach: Pixels,
    /// Every other frame while it has a goal: towards it, with the vertical gap halved, by 0.04 per
    /// pixel of gap beyond 30 (at most 0.6), doubled, until within 30 px.
    pub approach_every: Frames,
    pub goal_rest_distance: Pixels,
    pub approach_per_pixel: f32,
    pub approach_maximum: f32,
    pub approach_scale: f32,
    /// Flies other than type 18 keep this much of their drift while approaching.
    pub drift_retention_approaching: Retention,
    /// Every fourth frame: the drift gains its jitter, keeps 0.8 of itself, and halves when only one
    /// of its own cell and the cell 16 drifts ahead is solid.
    pub jitter_every: Frames,
    pub jitter_phase: Frames,
    pub jitter_retention: Retention,
    pub edge_retention: Retention,
    /// How many frames of drift ahead it looks for a solid cell.
    pub edge_lookahead: Frames,
    /// Every third frame: outside stage x 80–540 or y 210–420 the drift is turned back by 0.06; then
    /// the speed gains twice the drift and keeps 60 % of itself.
    pub drift_every: Frames,
    pub drift_phase: Frames,
    pub boundary_push: PixelsPerFrame,
    pub stage_minimum: Vec2,
    pub stage_maximum: Vec2,
    pub drift_gain: f32,
    pub velocity_retention: Retention,
    /// The attack fly touching the player hurts by half a heart.
    pub attack_damage: Hearts,
    /// Touching any fly while riding the unicorn does 40 damage to it, as does a blast.
    pub contact_damage: Damage,
    /// A hit it survives flashes it red for the damage divided by this many frames, at most the
    /// maximum, rounded.
    pub flash_frames_maximum: Frames,
    pub flash_damage_per_frame: Damage,
    /// Every live fly (types 13, 14, 18, 61, 80) makes the swarm loop louder: 30 % plus 30 % per fly,
    /// at most 150 %.
    pub swarm_volume_base: f32,
    pub swarm_volume_per_fly: f32,
    pub swarm_volume_maximum: f32,
    /// A red fly's death shoots along each axis at this speed (`quadf()`).
    pub red_burst_speed: PixelsPerFrame,
    /// A Moter killed with more than this many hit points left (not blown far past zero) splits
    /// into two attack flies, each this far to the side and sent that way.
    pub moter_split_floor: HitPoints,
    pub moter_split_offset: Pixels,
}

impl FlyParameters {
    /// How long a hit it survives flashes it: the hit's own frame, then the damage divided by
    /// [`Self::flash_damage_per_frame`] frames, rounded and at most the maximum.
    pub fn flash_after_hit(&self, damage: Damage) -> Frames {
        let frames_after_this = (damage / self.flash_damage_per_frame).min(self.flash_frames_maximum.get());
        Frames(frames_after_this.round() + 1.0)
    }
}

pub const ORIGINAL_FLIES: FlyParameters = FlyParameters {
    black: FlyKindParameters {
        size: Pixels(17.0),
        goal: FlyGoal::Breakables,
        hurts_on_touch: false,
        hit_points: HitPoints(3.0),
        appearing_frames: Frames(44.0),
        jitter: 0.34,
        dying_frames: Frames(16.0),
        mass: Mass(3.0),
    },
    attack: FlyKindParameters {
        size: Pixels(17.0),
        goal: FlyGoal::Player(Pixels(25.0)),
        hurts_on_touch: true,
        hit_points: HitPoints(5.0),
        appearing_frames: Frames(25.0),
        jitter: 0.6,
        dying_frames: Frames(16.0),
        mass: Mass(3.0),
    },
    // 8 hit points; appearing clip 4949 (25 frames); jitter 0.17 like the fly.
    pooter: FlyKindParameters {
        size: Pixels(17.0),
        goal: FlyGoal::Breakables,
        hurts_on_touch: false,
        hit_points: HitPoints(8.0),
        appearing_frames: Frames(24.0),
        jitter: 0.34,
        dying_frames: Frames(15.0),
        mass: Mass(4.0),
    },
    // 10 hit points; appearing clip 4871 (25 frames); rests 15 px from the player; bursts into a
    // shot along each axis at 8 px a frame when it dies.
    red: FlyKindParameters {
        size: Pixels(13.0),
        goal: FlyGoal::Player(Pixels(15.0)),
        hurts_on_touch: true,
        hit_points: HitPoints(10.0),
        appearing_frames: Frames(24.0),
        jitter: 0.34,
        dying_frames: Frames(16.0),
        mass: Mass(7.0),
    },
    // 10 hit points; appearing clip 4903 (26 frames); dying clip 566 (36 frames); on dying it
    // splits into two attack flies.
    moter: FlyKindParameters {
        size: Pixels(13.0),
        goal: FlyGoal::Player(Pixels(25.0)),
        hurts_on_touch: true,
        hit_points: HitPoints(10.0),
        appearing_frames: Frames(25.0),
        jitter: 0.34,
        dying_frames: Frames(36.0),
        mass: Mass(3.0),
    },
    appearing_retention: Retention(0.45),
    goal_every: Frames(3.0),
    goal_search_reach: Pixels(240.0),
    approach_every: Frames(2.0),
    goal_rest_distance: Pixels(30.0),
    approach_per_pixel: 0.04,
    approach_maximum: 0.6,
    approach_scale: 2.0,
    drift_retention_approaching: Retention(0.7),
    jitter_every: Frames(4.0),
    jitter_phase: Frames(1.0),
    jitter_retention: Retention(0.8),
    edge_retention: Retention(0.5),
    edge_lookahead: Frames(16.0),
    drift_every: Frames(3.0),
    drift_phase: Frames(2.0),
    boundary_push: PixelsPerFrame(0.06),
    stage_minimum: Vec2::new(80.0, 210.0),
    stage_maximum: Vec2::new(540.0, 420.0),
    drift_gain: 2.0,
    velocity_retention: Retention(0.6),
    attack_damage: Hearts::HALF,
    contact_damage: Damage(40.0),
    flash_frames_maximum: Frames(6.0),
    flash_damage_per_frame: Damage(1.5),
    swarm_volume_base: 30.0,
    swarm_volume_per_fly: 30.0,
    swarm_volume_maximum: 150.0,
    red_burst_speed: PixelsPerFrame(8.0),
    moter_split_floor: HitPoints(-20.0),
    moter_split_offset: Pixels(10.0),
};
