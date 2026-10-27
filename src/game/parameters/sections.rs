//! Parameter groups and the formulas they drive.

use crate::game::original_units::{Damage, Frames, Mass, Pixels, PixelsPerFrame, PixelsPerFrameSquared, Retention};

/// Per frame the input pushes the velocity by (3 + momentum) × 0.8, then the velocity keeps
/// 0.7 + speed × 0.075 of itself (the small momentum bonus is not modeled).

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MovementParameters {
    pub push_per_frame: PixelsPerFrame,
    pub push_scale: f32,
    pub base_velocity_retention: Retention,
    pub velocity_retention_per_speed: f32,
    /// `playsp`: 1 for Isaac without items.
    pub speed_stat: f32,
    /// The body plays its walk cycle at this speed or more; below it Isaac stands.
    pub walking_minimum_speed_per_frame: PixelsPerFrame,
    /// Standing on a spider web sets `playslow = 4`; while it counts down, velocity is also
    /// multiplied by 0.7 every frame.
    pub web_slow_frames: Frames,
    pub web_velocity_retention: Retention,
}

impl MovementParameters {
    pub fn acceleration_per_frame(&self) -> PixelsPerFrame {
        self.push_per_frame * self.push_scale
    }

    pub fn velocity_retention_per_frame(&self) -> Retention {
        Retention(self.base_velocity_retention.0 + self.speed_stat * self.velocity_retention_per_speed)
    }

    /// Where the per-frame rule settles: about 8.3 px per frame for Isaac.
    pub fn top_speed_per_frame(&self) -> PixelsPerFrame {
        let Retention(retention) = self.velocity_retention_per_frame();
        self.acceleration_per_frame() * (retention / (1.0 - retention))
    }
}

/// `firr()` (DoAction 11051, DoAction 11052): the delay is 16 − 6 × √(1 + 1.3 × bonus), at least 5;
/// the counter drops by one per frame and a tear fires once it is below zero, so shots are one frame
/// further apart than the counter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FiringParameters {
    pub delay_base: Frames,
    pub delay_per_root: Frames,
    pub delay_bonus_scale: f32,
    pub delay_minimum: Frames,
    /// Sum of fire-rate item bonuses; 0 without items.
    pub delay_bonus: f32,
    /// Frames added to the delay after its minimum is applied (Cancer takes two off).
    pub delay_offset: Frames,
    /// Mouse aim is measured from this far above the player's floor point (`_ymouse - yp + 10`).
    pub mouse_aim_height: Pixels,
    /// After a shot the face timer `plox` is `7 + delay * 0.45`; it drops by one per frame.
    pub face_timer_base: Frames,
    pub face_timer_per_delay: f32,
    /// The squinting shooting face shows while the timer is above this; the head turns back down
    /// once it is below zero.
    pub shooting_face_above: Frames,
}

impl FiringParameters {
    pub fn delay_counter(&self) -> Frames {
        let root = (1.0 + self.delay_bonus * self.delay_bonus_scale).max(0.0).sqrt();
        (self.delay_base - self.delay_per_root * root).max(self.delay_minimum) + self.delay_offset
    }

    pub fn frames_between_shots(&self) -> Frames {
        self.delay_counter() + Frames(1.0)
    }

    pub fn face_timer_after_shot(&self) -> Frames {
        self.face_timer_base + self.delay_counter() * self.face_timer_per_delay
    }
}

/// `plff()` and tear type 2 in `create()`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TearParameters {
    pub speed_per_frame: PixelsPerFrame,
    pub inherited_velocity_ratio: f32,
    /// `dy = -23`; the tear lands when `dy > -5`.
    pub start_height: Pixels,
    pub landing_height: Pixels,
    /// Per frame the fall speed keeps 90 % of itself and gains 0.1, and the height drops by it.
    pub fall_speed_retention: Retention,
    pub fall_acceleration: PixelsPerFrameSquared,
    /// Initial fall speed is `rand() * 0.2`, with `rand()` centred on zero: within ±0.1.
    pub initial_fall_speed_spread: PixelsPerFrame,
    /// What range makes of it: the first fall speed starts this much lower (`dm`, a rise).
    pub initial_rise: PixelsPerFrame,
    /// What shot speed multiplies the velocity by.
    pub speed_multiplier: f32,
    /// Alternating side offset of `0.3 + random * 0.2` of the per-frame velocity.
    pub side_offset_minimum: f32,
    pub side_offset_random_range: f32,
    /// `dmgdo()`: 3.5 × √(1 + 1.2 × bonus), the bonus summed from items and cards.
    pub base_damage: Damage,
    pub damage_bonus_scale: f32,
    /// Drawn size: 0.7 + 0.04 × damage + 0.15 × √damage.
    pub size_base: f32,
    pub size_per_damage: f32,
    pub size_per_root_damage: f32,
    /// The dark tear picture (the devil's tears) is drawn at 0.9 of that size.
    pub dark_body_scale: f32,
    /// Tears crossing a spider web are slowed once by this factor.
    pub web_slowdown: f32,
}

impl TearParameters {
    pub fn damage(&self, bonus: f32) -> Damage {
        self.base_damage * (1.0 + bonus * self.damage_bonus_scale).sqrt()
    }

    pub fn scale_for_damage(&self, damage: Damage) -> f32 {
        self.size_base + damage.get() * self.size_per_damage + damage.get().sqrt() * self.size_per_root_damage
    }
}

/// Breakable grid objects (`turd()`, `killshit()`). Starting values are the grid values `levz`
/// the level data uses for each object.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BreakableParameters {
    pub poop_durability: f32,
    /// Level value 1.49 marks red poop; `turd()` resets it to 1.5.
    pub red_poop_durability: f32,
    pub corny_poop_durability: f32,
    pub golden_poop_durability: f32,
    pub fireplace_durability: f32,
    /// The TNT barrel (`egg`).
    pub tnt_durability: f32,
    /// Grid values from this up block movement and tears (`mhit`); objects break at or below it.
    pub solid_threshold: f32,
    pub durability_per_step: f32,
    /// Chance of each extra damage step per hit (one in three).
    pub extra_step_chance: f32,
    /// Golden poop only takes a step with this chance (one in three).
    pub golden_step_chance: f32,
    /// Value a destroyed object leaves in the grid (walkable).
    pub destroyed_durability: f32,
    /// Broken red poop grows back after this many frames.
    pub red_poop_regrow_frames: Frames,
}

/// Explosions (`gosplash()` for bombs and the TNT barrel).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExplosionParameters {
    /// Grid tiles within this distance of the blast get a strong hit.
    pub radius: Pixels,
    /// Isaac and pickups are pushed when within 85 plus their size of the blast (95 for Isaac, 97
    /// for pickups) and in its line of sight: their speed halves, then they are pushed away by the
    /// reach left over, at most 15 px per frame.
    pub player_push_reach: Pixels,
    pub pickup_push_reach: Pixels,
    /// A cleared rock or web within this distance of the blast is broken floor when pits lie on
    /// both sides of it.
    pub floor_break_reach: Pixels,
    pub velocity_retention: Retention,
    pub maximum_push_per_frame: PixelsPerFrame,
}

/// Entering a room (the room's frame counter starts again).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RoomEntryParameters {
    /// Movement input is ignored while the room slides in.
    pub movement_frozen_frames: Frames,
    /// Doors work only after this.
    pub doors_closed_frames: Frames,
}

/// Bombs Isaac places (`bomb()`, object type 4, clip `b4`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BombParameters {
    /// The fuse clip (sprite 2039) explodes from a script on its frame 59, which moves the bomb on
    /// to its blast; it starts on frame 1 when the bomb appears.
    pub exploding_fuse_frame: Frames,
    /// At most one bomb per second.
    pub frames_between_placements: Frames,
    /// Debugging: bombs never run out.
    pub infinite_bombs: bool,
    /// The player and a bomb bump apart (`ballhit()`) inside their two sizes (10 + 16), by mass (5
    /// and 6) and at half strength.
    pub player_overlap_reach: Pixels,
    /// Two lit bombs bump apart inside twice a bomb's size (16 + 16).
    pub bomb_overlap_reach: Pixels,
    pub mass: Mass,
    pub separation_scale: f32,
}

/// Opening key blocks by walking into them (`laps()`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KeyParameters {
    /// The tile checked is this far ahead along the player's movement (`roxx = 40`).
    pub reach: Pixels,
    /// The push counter rises by one per frame against a key block and opens it above 2.8; it
    /// falls by 0.2 per frame otherwise.
    pub push_frames_to_open: Frames,
    pub push_decay_per_frame: f32,
    /// Debugging: keys never run out.
    pub infinite_keys: bool,
}
