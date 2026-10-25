//! Pickups (object type 5) and what the player carries.

/// Hearts, coins, keys and bombs lying in rooms.
use crate::game::odds::Odds;
use crate::game::original_units::{
    Damage, Frames, Hearts, Mass, Pixels, PixelsPerFrame, PixelsPerFrameSquared, Retention,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PickupParameters {
    /// Collected when the player comes this close: the player's size plus a pickup's (10 + 12).
    pub collect_reach: Pixels,
    /// `aicol()`: a new pickup cannot be taken, bumped or blasted until this long after it
    /// appears.
    pub frames_until_collectable: Frames,
    /// `create()`: placed this far below its marker's cell center.
    pub spawn_drop: Pixels,
    /// A troll bomb explodes once it has lain this long (counted from when it appears).
    pub troll_bomb_fail_limit: Frames,
    /// Mega troll bombs chase the player within this reach once settled: each frame they speed up
    /// towards the player by 1.5 px/frame², then keep 80 % of their speed.
    pub chase_reach: Pixels,
    pub chase_pull_per_frame: PixelsPerFrameSquared,
    pub mega_troll_pull_factor: f32,
    pub chase_velocity_retention: Retention,
    /// The masses of a pickup and of the player: how they share the push when they bump.
    pub mass: Mass,
    pub player_mass: Mass,
    /// `cspawn()`: chest contents fly out at 5 px per frame in a random direction.
    pub throw_speed_per_frame: PixelsPerFrame,
    /// Stand-in for the doors opening after a fight: rooms whose layout has enemies give their
    /// reward this long after the first entry.
    pub room_clear_reward_frames: Frames,
}

/// Burning fireplaces (`firecheck()`, `pffy()`): the player's four corners 20 px out are checked
/// for a fireplace cell whose center is within 38; it hurts by half a heart, pushes away by 4 px per
/// frame and hisses (`Firedeath_hiss.wav`) at most every 10 frames.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FireParameters {
    pub probe_offset: Pixels,
    pub reach: Pixels,
    pub damage: Hearts,
    pub push_per_frame: PixelsPerFrameSquared,
    pub hiss_interval_frames: Frames,
    /// An enemy on a burning fireplace (or a walker on spikes) takes 8 damage every 10 frames
    /// (`alive()`, DoAction 13844–13851).
    pub enemy_damage: Damage,
    pub enemy_interval: Frames,
}

/// What a run starts with and how getting hurt works.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InventoryParameters {
    /// Isaac, the first character, starts with three heart containers, full.
    pub starting_hearts: Hearts,
    /// A run starts with no coins, bombs or keys; Isaac then gets one bomb.
    pub starting_bombs: u8,
    /// Keys, coins and bombs stop at 99.
    pub maximum_count: u8,
    /// After a hit: invulnerable for 60 frames per heart of damage.
    pub invulnerable_frames_per_heart: Frames,
    /// While more than 6 frames remain, Isaac blinks: every 3rd frame he hides (even count left)
    /// or shows (odd), so he toggles every 3 frames.
    pub blink_period_frames: Frames,
    pub blink_until_frames_left: Frames,
    /// A blast hurts Isaac by a full heart.
    pub blast_damage: Hearts,
    /// `ader()`: after the chapter number passes 6, a blood donation first borrows half a soul
    /// heart to make a lone half red heart whole.
    pub late_chapter_after: u8,
    /// More than 20 blood donations unlock an item for later runs.
    pub blood_donations_before_unlock: u32,
}

/// Spikes (grid value 0.99, `firecheck()`): a spike cell counts 20 farther than it is, so it hurts
/// within 18 of its center: a full heart, or half a heart on a curse door's doorway. In the sacrifice
/// room a hit that lands while not invulnerable may first give a chest, once per floor: one time in
/// five, or always below two red hearts with no soul hearts; a brown or golden chest at (320, 340).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpikeParameters {
    pub reach_handicap: Pixels,
    pub damage: Hearts,
    pub curse_door_damage: Hearts,
    pub sacrifice_chest_odds: Odds,
    pub sacrifice_low_health: Hearts,
    pub sacrifice_chest_stage_point: (f32, f32),
}
