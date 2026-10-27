//! Engine-independent game rules.
//!
//! This module is the boundary a frontend (SDL3 or Bevy) programs against. Its parts:
//! `player` (Isaac, his look and inventory), `pickups`, `room` (a room's grid, contents, doors and
//! changes between rooms), `floor` (generation and traversal), `attacks` (tears and bombs), `hud`
//! (HUD and minimap), and shared basics (units, ticks, drawing, input, sounds, textures).
//!
//! A frontend only has to:
//! 1. translate devices into an [`InputState`] each frame,
//! 2. call [`Game::update`] with the elapsed time,
//! 3. draw every [`DrawItem`] from [`Game::fill_draw_items`], fitting the [`VIEW_SIZE`] area
//!    around [`VIEW_CENTER`] into the window (world origin is the room center, y points up); items
//!    may name a
//!    [`TextureId`] that the asset pipeline supplies,
//! 4. play a sound for every [`SoundEvent`] from [`Game::drain_sound_events`], and keep each
//!    [`LoopingSound`] from [`Game::looping_sounds`] playing.
//!
//! Nothing in here may depend on an engine crate.

mod attacks;
mod ball_contact;
mod bounds;
mod draw;
mod floor;
mod friends;
mod hud;
mod input_state;
mod interpolation;
mod machines;
mod music;
mod odds;
mod original_units;
mod parameters;
mod pickups;
mod player;
mod progress;
mod random;
mod restart_hold;
mod room;
mod scene;
mod settings;
mod shop;
mod sound_event;
mod texture_id;
mod tick_clock;
mod wall_contact;

use glam::Vec2;

pub use draw::{DrawItem, Rgba};
pub use hud::message::{PickupMessage, texture_center_in_clip};
pub use hud::name_banner::{BannerName, FORTUNE_COUNT};
pub use hud::pill_text::{HUD_NAME_COUNT, MESSAGES_BEFORE_PILLS, PillText};
pub use hud::{HudHeart, MAXIMUM_COUNTER_VALUE, held_trinket::SlotView, stats::HUD_GLYPHS};
pub use input_state::{
    GamepadKind, InputDevice, InputState, LEFT_STICK_DEADZONE, RIGHT_STICK_DEADZONE, apply_radial_deadzone,
};
pub use music::{MusicEvent, MusicStart, MusicTrack};
pub use parameters::{GameParameters, ParameterProvider, TranscribedOriginal};
pub use pickups::card::{Card as PickupCard, TAROT_COUNT};
pub use pickups::kind::{ChestKind, PickupKind};
pub use pickups::pill::{Pill, PillLook};
pub use pickups::{item::Item, phase::PickupPhase, trinket::Trinket};
pub use player::animation::{Held, PlayerAnimation};
pub use player::death::Killer;
pub use room::grid_kind::{BreakableKind, TileKind};
#[cfg(test)]
pub use room::layout::kind_for_value;
pub use room::layout::{Decoration, LayoutCell, RoomLayout, RoomLayouts};
pub use room::{ORIGINAL_INTERIOR_FIRST_COLUMN, ORIGINAL_INTERIOR_FIRST_ROW, RoomAppearance};
pub use sound_event::{LoopingSound, SoundEvent};
pub use texture_id::{
    CARD_NAME_TEXTURE_CENTER_X, ClottyAnimation, DoorStyle, DoorTexture, EnemyTexture, FleshMoundAnimation,
    FlyAnimation, FlyKind, GaperAnimation, HeadlessAnimation, HopperAnimation, HorfAnimation, IsaacPart,
    MulliganAnimation, RideColor, RoomArt, TextureId,
};

use crate::game::original_units::Frames;
use attacks::bombs::Bombs;
use attacks::splash::Splash;
use attacks::tear::Tear;
use floor::Floor;
pub use floor::{MapPiece, StartRoomControl};
use hud::overlay::Overlay;
use original_units::DEFAULT_TICKS_PER_SECOND;
pub use original_units::DEFAULT_TICKS_PER_SECOND as DEFAULT_TICK_RATE;
use parameters::RoomEntryParameters;
use player::Player;
use player::inventory::Inventory;
use random::Random;
use restart_hold::RestartHold;
use room::transition::RoomTransition;
pub use settings::{GameSettings, InputSampling, Style};
use tick_clock::TickClock;

/// World-space area a frontend must keep visible.
pub const VIEW_SIZE: Vec2 = room::STAGE_SIZE;
/// World position at the middle of the view: the stage's center, above the room's (the HUD strip
/// takes the top of the stage).
pub const VIEW_CENTER: Vec2 = room::STAGE_CENTER;

/// How many output pixels one world unit covers when [`VIEW_SIZE`] is fitted into an output of
/// `output_pixels` (letterboxed). Textures rendered at this density are drawn 1:1.
pub fn pixels_per_world_unit(output_pixels: Vec2) -> f32 {
    (output_pixels / VIEW_SIZE).min_element().max(f32::EPSILON)
}
pub(crate) const PLAYER_START_POSITION: Vec2 = Vec2::new(0.0, -40.0);

pub struct Game {
    room_appearance: RoomAppearance,
    player: Player,
    tears: Vec<Tear>,
    floor: Floor,
    room_transition: Option<RoomTransition>,
    /// Off with `--instant-room-changes` (handy while debugging).
    slide_between_rooms: bool,
    room_entry: RoomEntryParameters,
    /// Kept to start a new run after dying.
    parameters: GameParameters,
    layouts: RoomLayouts,
    settings: GameSettings,
    inventory: Inventory,
    clock: TickClock,
    overlay: Overlay,
    restart_hold: RestartHold,
    /// What the floor was generated from (shown in the stats panel).
    seed: u32,
    /// A card's teleport under way, and The Tower's falling bombs.
    cards: player::cards::CardEffects,
    /// World area the frontend shows (wider than the stage in widescreen), for the stats panel.
    visible_size: Vec2,
    /// Time since the last fire hiss (`firs`).
    frames_since_hiss: Frames,
    /// Time since entering the current room (the original's `fra`).
    frames_in_room: Frames,
    bombs: Bombs,
    blue_flies: friends::BlueFlies,
    /// Dying, and the death note.
    death: Option<player::death::Death>,
    splashes: Vec<Splash>,
    pending_sound_events: Vec<SoundEvent>,
    random: Random,
    /// Picks the button names the on-screen hints show.
    last_input_device: InputDevice,
    /// What lasts beyond this run.
    progress: progress::Progress,
    music: music::MusicDirector,
}

impl Game {
    /// A new run on a freshly generated floor; `layouts` are the chapter's room layouts from the
    /// assets (rooms are empty without them), `seed` picks the floor.
    pub fn new(room_appearance: RoomAppearance, parameters: &GameParameters, layouts: &RoomLayouts, seed: u32) -> Self {
        let mut random = Random::with_seed(seed);
        let floor = Floor::generate(layouts, parameters, &mut random);
        let inventory = Inventory::new_run(parameters.inventory, &mut random);
        Self {
            room_appearance,
            player: Player::new(PLAYER_START_POSITION, parameters),
            tears: Vec::new(),
            floor,
            room_transition: None,
            slide_between_rooms: true,
            room_entry: parameters.room_entry,
            parameters: *parameters,
            layouts: layouts.clone(),
            settings: GameSettings::default(),
            inventory,
            clock: TickClock::new(DEFAULT_TICKS_PER_SECOND, settings::InputSampling::Original),
            overlay: Overlay::default(),
            restart_hold: RestartHold::default(),
            seed,
            cards: player::cards::CardEffects::default(),
            visible_size: VIEW_SIZE,
            frames_since_hiss: Frames(f32::INFINITY),
            frames_in_room: Frames::ZERO,
            bombs: Bombs::new(parameters.bombs, parameters.walls),
            blue_flies: friends::BlueFlies::default(),
            death: None,
            splashes: Vec::new(),
            pending_sound_events: Vec::new(),
            random,
            last_input_device: InputDevice::default(),
            progress: progress::Progress::default(),
            music: music::MusicDirector::new(parameters.music),
        }
    }

    /// Advances the world by `elapsed` original frames.
    fn advance(&mut self, input: &InputState, elapsed: Frames) {
        if self.restart_hold.update(input.restart_held, elapsed) {
            self.restart();
            return;
        }
        self.update_music(elapsed);
        let Some(input) = self.advance_death(input, elapsed) else {
            return;
        };
        self.remember_positions();
        self.frames_in_room += elapsed;
        let input = &self.input_after_entry_freeze(&input);
        self.move_player(input, elapsed);
        let has_key = self.has_key();
        let opened_key_block = self.floor.current_room_mut().push_against_key_blocks(
            self.player.position(),
            self.player.velocity(),
            has_key,
            elapsed,
            &mut self.pending_sound_events,
        );
        if opened_key_block {
            self.spend_key();
        }
        self.trample(elapsed);
        self.update_items(input, elapsed);
        self.floor.current_room_mut().advance(
            elapsed,
            self.player.sighting(),
            &mut self.random,
            &mut self.pending_sound_events,
        );
        self.advance_blue_flies(elapsed);
        self.splashes.iter_mut().for_each(|splash| splash.advance(elapsed));
        self.splashes.retain(|splash| !splash.is_finished());
        self.advance_tears(elapsed);
        self.overlay.advance(elapsed);
        self.overlay.item_box.advance(self.inventory.active.charge(), elapsed);
        self.update_pickups(elapsed);
        self.touch_hazards(elapsed);
        self.apply_blasts();
        self.update_doors(elapsed);
        if let Some(transition) = &mut self.room_transition
            && transition.advance(elapsed)
        {
            self.room_transition = None;
        }
    }

    /// Sounds triggered since the last call, oldest first.
    pub fn drain_sound_events(&mut self) -> impl Iterator<Item = SoundEvent> {
        self.music.hear(&self.pending_sound_events);
        self.pending_sound_events.drain(..)
    }
}

#[cfg(test)]
mod hint_tests;
#[cfg(test)]
mod tests;
