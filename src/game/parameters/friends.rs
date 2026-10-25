//! Isaac's blue flies (`friends()`'s case for them, DoAction 11827–12210, and their creation,
//! 29725–29740): familiars that circle him, chase what was hurt last, hurt what they touch and die
//! half the time they do.

use crate::game::odds::Odds;
use crate::game::original_units::{Frames, Pixels, Retention};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlueFlyParameters {
    /// `sizes[3]`: the friends' collision size.
    pub size: Pixels,
    /// One is made each frame an owed one is waiting, 10 pixels above Isaac's floor point.
    pub spawn_lift: Pixels,
    /// A fly's place on the ring around Isaac turns by 0.03 half-turns a frame, and the flies share
    /// two half-turns between them; the ring is 35 wide and 30 high, 0.1 larger for each fly past 4.
    pub orbit_half_turns_per_frame: f32,
    pub orbit_shared_half_turns: f32,
    pub orbit_width: Pixels,
    pub orbit_height: Pixels,
    pub crowd_from: usize,
    pub crowd_growth: f32,
    /// Every other frame it is pushed towards its aim by 0.15 of the distance, at most 2, doubled.
    pub steer_every: Frames,
    pub steer_gain: f32,
    pub steer_limit: f32,
    pub steer_scale: f32,
    /// Each frame it keeps 80 % of its speed.
    pub velocity_retention: Retention,
    /// What it does to what it touches: 1.2 times Isaac's damage and 3 more.
    pub damage_scale: f32,
    pub damage_bonus: f32,
    /// It hurts the same enemy again only after this long.
    pub hit_memory: Frames,
    /// Having hurt something, it dies one time in two.
    pub death_odds: Odds,
    /// Circling Isaac, it goes for the enemy hurt last one frame in three.
    pub target_odds: Odds,
    /// A red chest's blue flies: 2 and a random 0–2 more.
    pub chest_base: u32,
    pub chest_range: u8,
    /// So many follow him into the next room.
    pub carried_between_rooms: usize,
}

pub const ORIGINAL_BLUE_FLIES: BlueFlyParameters = BlueFlyParameters {
    size: Pixels(13.0),
    spawn_lift: Pixels(10.0),
    orbit_half_turns_per_frame: 0.03,
    orbit_shared_half_turns: 2.0,
    orbit_width: Pixels(35.0),
    orbit_height: Pixels(30.0),
    crowd_from: 4,
    crowd_growth: 0.1,
    steer_every: Frames(2.0),
    steer_gain: 0.15,
    steer_limit: 2.0,
    steer_scale: 2.0,
    velocity_retention: Retention(0.8),
    damage_scale: 1.2,
    damage_bonus: 3.0,
    hit_memory: Frames(14.0),
    death_odds: Odds::EVEN,
    target_odds: Odds::one_in(3),
    chest_base: 2,
    chest_range: 3,
    carried_between_rooms: 10,
};
