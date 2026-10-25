//! Gameplay values and formula coefficients, kept as data so their source can change: the
//! original's values are written out here; a later source may read them from the player's SWF.
//!
//! Units are the original's: Flash pixels and frames at 30 per second. Expressions in comments
//! are from the decompiled code (`DefineSprite_6765/frame_1/DoAction.as`).

mod provider;

pub use provider::{ParameterProvider, TranscribedOriginal};

mod cards;
mod clottys;
mod creep;
mod death;
mod enemies;
mod flesh_mounds;
mod friends;
mod gapers;
mod headless;
mod hoppers;
mod horfs;
mod mulligans;
mod music;
mod pickups;
mod pills;
mod pooters;
mod screen_fade;
mod walls;
use crate::game::odds::Odds;
use crate::game::original_units::{
    Damage, Frames, Hearts, Mass, Pixels, PixelsPerFrame, PixelsPerFrameSquared, Retention,
};
mod sections;

pub use cards::CardParameters;
pub use clottys::ClottyParameters;
pub use creep::CreepParameters;
pub use death::DeathParameters;
pub use enemies::{FlyGoal, FlyKindParameters, FlyParameters};
pub use flesh_mounds::FleshMoundParameters;
pub use friends::BlueFlyParameters;
pub use gapers::GaperParameters;
pub use headless::HeadlessParameters;
pub use hoppers::HopperParameters;
pub use horfs::HorfParameters;
pub use mulligans::MulliganParameters;
pub use music::MusicParameters;
pub use pickups::{FireParameters, InventoryParameters, PickupParameters, SpikeParameters};
pub use pills::PillParameters;
pub use pooters::{EnemyShotParameters, PooterParameters};
pub use screen_fade::ScreenFadeParameters;
pub use walls::WallParameters;

pub use sections::{
    BombParameters, BreakableParameters, ExplosionParameters, FiringParameters, KeyParameters, MovementParameters,
    RoomEntryParameters, TearParameters,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GameParameters {
    pub movement: MovementParameters,
    pub firing: FiringParameters,
    pub tear: TearParameters,
    pub breakables: BreakableParameters,
    pub explosion: ExplosionParameters,
    pub keys: KeyParameters,
    pub bombs: BombParameters,
    pub room_entry: RoomEntryParameters,
    pub pickups: PickupParameters,
    pub inventory: InventoryParameters,
    pub fire: FireParameters,
    pub spikes: SpikeParameters,
    pub walls: WallParameters,
    pub cards: CardParameters,
    pub flies: FlyParameters,
    pub blue_flies: BlueFlyParameters,
    pub creep: CreepParameters,
    pub music: MusicParameters,
    pub gapers: GaperParameters,
    pub headless: HeadlessParameters,
    pub horfs: HorfParameters,
    pub clottys: ClottyParameters,
    pub hoppers: HopperParameters,
    pub flesh_mounds: FleshMoundParameters,
    pub mulligans: MulliganParameters,
    pub death: DeathParameters,
    pub pills: PillParameters,
    pub screen_fade: ScreenFadeParameters,
    pub pooters: PooterParameters,
    pub enemy_shots: EnemyShotParameters,
}

impl GameParameters {
    pub const ORIGINAL: Self = Self {
        movement: MovementParameters {
            push_per_frame: PixelsPerFrame(3.0),
            push_scale: 0.8,
            base_velocity_retention: Retention(0.7),
            velocity_retention_per_speed: 0.075,
            speed_stat: 1.0,
            walking_minimum_speed_per_frame: PixelsPerFrame(2.0),
            web_slow_frames: Frames(4.0),
            web_velocity_retention: Retention(0.7),
        },
        firing: FiringParameters {
            delay_base: Frames(16.0),
            delay_per_root: Frames(6.0),
            delay_bonus_scale: 1.3,
            delay_minimum: Frames(5.0),
            delay_bonus: 0.0,
            mouse_aim_height: Pixels(10.0),
            face_timer_base: Frames(7.0),
            face_timer_per_delay: 0.45,
            shooting_face_above: Frames(8.0),
        },
        tear: TearParameters {
            speed_per_frame: PixelsPerFrame(10.0),
            inherited_velocity_ratio: 0.6,
            start_height: Pixels(23.0),
            landing_height: Pixels(5.0),
            fall_speed_retention: Retention(0.9),
            fall_acceleration: PixelsPerFrameSquared(0.1),
            initial_fall_speed_spread: PixelsPerFrame(0.2),
            initial_rise: PixelsPerFrame(0.0),
            speed_multiplier: 1.0,
            side_offset_minimum: 0.3,
            side_offset_random_range: 0.2,
            base_damage: Damage(3.5),
            damage_bonus_scale: 1.2,
            size_base: 0.7,
            size_per_damage: 0.04,
            size_per_root_damage: 0.15,
            dark_body_scale: 0.9,
            web_slowdown: 0.64,
        },
        breakables: BreakableParameters {
            poop_durability: 1.5,
            red_poop_durability: 1.5,
            corny_poop_durability: 1.495,
            golden_poop_durability: 1.4955,
            fireplace_durability: 1.4,
            tnt_durability: 1.3,
            solid_threshold: 1.0,
            durability_per_step: 0.13,
            extra_step_chance: 1.0 / 3.0,
            golden_step_chance: 1.0 / 3.0,
            destroyed_durability: 0.9,
            red_poop_regrow_frames: Frames(120.0),
        },
        explosion: ExplosionParameters {
            radius: Pixels(60.0),
            player_push_reach: Pixels(95.0),
            pickup_push_reach: Pixels(97.0),
            floor_break_reach: Pixels(40.0),
            velocity_retention: Retention(0.5),
            maximum_push_per_frame: PixelsPerFrame(15.0),
        },
        keys: KeyParameters {
            reach: Pixels(40.0),
            push_frames_to_open: Frames(2.8),
            push_decay_per_frame: 0.2,
            infinite_keys: false,
        },
        bombs: BombParameters {
            exploding_fuse_frame: Frames(59.0),
            frames_between_placements: Frames(30.0),
            infinite_bombs: false,
            player_overlap_reach: Pixels(26.0),
            bomb_overlap_reach: Pixels(32.0),
            mass: Mass(6.0),
            separation_scale: 0.5,
        },
        room_entry: RoomEntryParameters {
            movement_frozen_frames: Frames(20.0),
            doors_closed_frames: Frames(30.0),
        },
        pickups: PickupParameters {
            collect_reach: Pixels(22.0),
            frames_until_collectable: Frames(25.0),
            spawn_drop: Pixels(10.0),
            troll_bomb_fail_limit: Frames(60.0),
            chase_reach: Pixels(270.0),
            chase_pull_per_frame: PixelsPerFrameSquared(0.5),
            mega_troll_pull_factor: 3.0,
            chase_velocity_retention: Retention(0.8),
            mass: Mass(3.0),
            player_mass: Mass(5.0),
            throw_speed_per_frame: PixelsPerFrame(5.0),
            room_clear_reward_frames: Frames(10.0),
        },
        inventory: InventoryParameters {
            starting_hearts: Hearts(3.0),
            starting_bombs: 1,
            maximum_count: 99,
            invulnerable_frames_per_heart: Frames(60.0),
            blink_period_frames: Frames(3.0),
            blink_until_frames_left: Frames(6.0),
            blast_damage: Hearts::ONE,
            late_chapter_after: 6,
            blood_donations_before_unlock: 20,
        },
        fire: FireParameters {
            probe_offset: Pixels(20.0),
            reach: Pixels(38.0),
            damage: Hearts::HALF,
            push_per_frame: PixelsPerFrameSquared(4.0),
            hiss_interval_frames: Frames(10.0),
            enemy_damage: Damage(8.0),
            enemy_interval: Frames(10.0),
        },
        spikes: SpikeParameters {
            reach_handicap: Pixels(20.0),
            damage: Hearts::ONE,
            curse_door_damage: Hearts::HALF,
            sacrifice_chest_odds: Odds::one_in(5),
            sacrifice_low_health: Hearts(2.0),
            sacrifice_chest_stage_point: (320.0, 340.0),
        },
        walls: walls::ORIGINAL_WALLS,
        cards: cards::ORIGINAL_CARDS,
        flies: enemies::ORIGINAL_FLIES,
        blue_flies: friends::ORIGINAL_BLUE_FLIES,
        creep: creep::ORIGINAL_CREEP,
        music: music::ORIGINAL_MUSIC,
        gapers: gapers::ORIGINAL_GAPERS,
        headless: headless::ORIGINAL_HEADLESS,
        horfs: horfs::ORIGINAL_HORFS,
        clottys: clottys::ORIGINAL_CLOTTYS,
        hoppers: hoppers::ORIGINAL_HOPPERS,
        flesh_mounds: flesh_mounds::ORIGINAL_FLESH_MOUNDS,
        mulligans: mulligans::ORIGINAL_MULLIGANS,
        death: death::ORIGINAL_DEATH,
        pills: pills::ORIGINAL_PILLS,
        screen_fade: screen_fade::ORIGINAL_SCREEN_FADE,
        pooters: pooters::ORIGINAL_POOTERS,
        enemy_shots: pooters::ORIGINAL_ENEMY_SHOTS,
    };
}

#[cfg(test)]
mod tests;
