//! A generated floor: every room's contents, kept as the player leaves them, and moving between
//! rooms through doors.

pub mod cell;
pub mod plan;

mod blast_sight;
mod devil_room;
mod door_motion;
mod map_knowledge;
mod passages;
mod room_setup;
mod start_room_hint;
mod teleport;

pub use cell::FloorCell;
pub use devil_room::DEVIL_CELL;
pub use map_knowledge::MapPiece;
pub use start_room_hint::StartRoomControl;

use std::collections::{BTreeMap, BTreeSet};

use glam::Vec2;

use crate::game::draw::DrawItem;
use crate::game::floor::plan::{FloorPlan, RoomRole};
use crate::game::input_state::InputDevice;
use crate::game::original_units::{Frames, distance};
use crate::game::parameters::GameParameters;
use crate::game::pickups::red_chest::RedChestChance;
use crate::game::random::Random;
use crate::game::room::contents::RoomContents;
use crate::game::room::doors::{DOOR_REACH, DOOR_REARM_DISTANCE, DoorSide};
use crate::game::room::layout::{RoomLayout, RoomLayouts};
use crate::game::settings::Style;
use crate::game::sound_event::SoundEvent;
use crate::game::texture_id::RoomArt;
use room_setup::{choose_layout, super_secret_hearts, tinted_rock_cell};

/// One random grid index per room, from 65 to 179 (16 per row; the interior starts at column 2,
/// row 4), whose rock, if any, is the tinted rock.
const TINTED_ROCK_INDEX_BASE: i32 = 65;
const TINTED_ROCK_INDEX_RANGE: i32 = 115;
const SUPER_SECRET_STYLES: u8 = 5;
/// Only the first chapter (`chaps` 1, the Basement) is played so far.
pub(crate) const PLAYED_CHAPTER: u8 = 1;

pub struct Floor {
    plan: FloorPlan,
    rooms: BTreeMap<FloorCell, RoomContents>,
    current_cell: FloorCell,
    /// `seenlev`: rooms entered or seen through a door, shown on the minimap.
    seen: BTreeSet<FloorCell>,
    /// The door just come through stays inactive until the player steps away from it (`nod`).
    resting_door: Option<DoorSide>,
    visited: BTreeSet<FloorCell>,
    /// Walls between a secret room and its neighbor that a bomb has opened (lower cell first).
    opened_passages: BTreeSet<(FloorCell, FloorCell)>,
    /// The boss room's side with the door to the devil room.
    devil_door: Option<DoorSide>,
    /// `lastl`: the room last left through a door (`None` after a teleport).
    last_room: Option<FloorCell>,
    /// Where the player last came into a room through a door (`playerx`, `playery`); a
    /// teleport lands there too.
    last_entry_position: Vec2,
    secret_entrances: Style,
    curse_secret_door: Style,
    /// `shopa`, `liba`: the shop's and library's doors are locked on every floor until a key
    /// opens them.
    unlocked_rooms: BTreeSet<RoomRole>,
    /// Time since a key turned in the shop door of the current room, while its animation plays.
    unlocking_door: Option<(DoorSide, Frames)>,
    /// Shut while enemies live (see `door_motion`).
    door_motion: door_motion::RoomDoors,
    /// `sacer`: the sacrifice room gave its chest on this floor.
    pub sacrifice_paid: bool,
    /// `hid2`: which of the five super secret rooms this floor has (art and layout).
    super_secret_style: u8,
    /// Special rooms the minimap marks: shop, treasure and boss once a door to them was seen
    /// (`shopaz`, `treasd`, `bossd`), secret rooms once found (`hider`, `hider2`; their jingle plays
    /// once).
    revealed_roles: BTreeSet<RoomRole>,
}

/// What happened when the player went through a door.
pub struct RoomEntry {
    /// Where the player appears.
    pub position: Vec2,
    pub role: RoomRole,
    pub first_visit: bool,
    /// Entering a secret room finds it (`hider()` at room setup): its jingle on the first time.
    pub jingle: Option<SoundEvent>,
}

impl Floor {
    pub fn generate(layouts: &RoomLayouts, parameters: &GameParameters, random: &mut Random) -> Self {
        // The super secret room's style, rolled before the floor is laid out.
        let super_secret_style = random.below_u8(SUPER_SECRET_STYLES);
        let mut plan = FloorPlan::generate(PLAYED_CHAPTER, random);
        let devil_door = devil_room::add_devil_room(&mut plan);
        let empty = RoomLayout::default();
        let mut rooms: BTreeMap<FloorCell, RoomContents> = plan
            .roles_by_cell
            .iter()
            .map(|(&cell, &role)| {
                let layout = choose_layout(role, layouts, super_secret_style, random).unwrap_or(&empty);
                let tinted_rock = tinted_rock_cell(random.below(TINTED_ROCK_INDEX_RANGE) + TINTED_ROCK_INDEX_BASE);
                let mut room = RoomContents::from_layout(layout, parameters, tinted_rock, random);
                let red_chests = RedChestChance {
                    always: matches!(role, RoomRole::Curse | RoomRole::SuperSecret),
                    first_visit: true,
                };
                room.pickups.set_red_chest_chance(red_chests, random);
                if role == RoomRole::Devil {
                    room.wares.make_devil_deals(random);
                }
                if role == RoomRole::SuperSecret
                    && let Some(heart) = super_secret_hearts(super_secret_style)
                {
                    room.pickups.set_heart_kind(heart);
                }
                (cell, room)
            })
            .collect();
        room_setup::add_start_room_items(&mut rooms);
        let mut floor = Self {
            plan,
            rooms,
            current_cell: FloorCell::START,
            seen: BTreeSet::new(),
            resting_door: None,
            visited: BTreeSet::from([FloorCell::START]),
            opened_passages: BTreeSet::new(),
            last_entry_position: teleport::FIRST_ARRIVAL_POSITION,
            secret_entrances: Style::Original,
            curse_secret_door: Style::Original,
            unlocked_rooms: BTreeSet::new(),
            unlocking_door: None,
            door_motion: door_motion::RoomDoors::default(),
            super_secret_style,
            sacrifice_paid: false,
            revealed_roles: BTreeSet::new(),
            devil_door,
            last_room: None,
        };
        floor.reveal_surroundings();
        floor
    }

    pub fn current_cell(&self) -> FloorCell {
        self.current_cell
    }

    #[cfg(test)]
    pub fn visible_doors(&self) -> Vec<DoorSide> {
        self.doors().collect()
    }

    /// The current room's wall and floor style.
    pub fn current_art(&self) -> RoomArt {
        match self.role_of(self.current_cell) {
            RoomRole::Shop => RoomArt::Shop,
            RoomRole::Secret => RoomArt::Secret,
            RoomRole::SuperSecret => RoomArt::SuperSecret(self.super_secret_style),
            RoomRole::Boss => RoomArt::Boss,
            // The devil room is drawn with the curse room's walls.
            RoomRole::Curse | RoomRole::Devil => RoomArt::Curse,
            RoomRole::Library => RoomArt::Library,
            RoomRole::Sacrifice => RoomArt::Sacrifice,
            RoomRole::Start | RoomRole::Normal | RoomRole::Treasure | RoomRole::MiniBoss => RoomArt::Basement,
        }
    }

    pub fn current_role(&self) -> RoomRole {
        self.role_of(self.current_cell)
    }

    pub fn current_room(&self) -> &RoomContents {
        &self.rooms[&self.current_cell]
    }

    pub fn current_room_mut(&mut self) -> &mut RoomContents {
        self.rooms.get_mut(&self.current_cell).expect("the current room exists")
    }

    /// Call every frame, even while doors are not working yet: the door just come through works
    /// again once the player has been far enough from it.
    pub fn track_player(&mut self, player_position: Vec2) {
        if let Some(side) = self.resting_door
            && distance(player_position, side.trigger_point()) >= DOOR_REARM_DISTANCE
        {
            self.resting_door = None;
        }
    }

    /// The door the player is touching, if it leads somewhere and was not just come through.
    pub fn door_reached(&self, player_position: Vec2) -> Option<DoorSide> {
        self.open_doors()
            .filter(|side| Some(*side) != self.resting_door)
            .find(|side| distance(player_position, side.trigger_point()) < DOOR_REACH)
    }

    /// Puts the developer showcases into the floor's rooms (`--showcase`): every pickup kind in
    /// the boss room, every machine in the shop, every card in the treasure room.
    pub fn add_showcases(&mut self) {
        room_setup::add_showcases(&self.plan, &mut self.rooms);
    }

    /// Moves to the room behind `side`.
    pub fn set_door_rules(&mut self, secret_entrances: Style, curse_secret_door: Style) {
        self.secret_entrances = secret_entrances;
        self.curse_secret_door = curse_secret_door;
    }

    pub fn go_through(&mut self, side: DoorSide) -> RoomEntry {
        self.close_entrance_if_leaving_secret_room(side);
        self.unlocking_door = None;
        if let Some(next_cell) = self.neighbor(side) {
            self.last_room = Some(self.current_cell);
            self.current_cell = next_cell;
            self.resting_door = Some(side.opposite());
        }
        self.last_entry_position = side.entry_position();
        self.arrive(side.entry_position())
    }

    pub(crate) fn role_of(&self, cell: FloorCell) -> RoomRole {
        self.plan.roles_by_cell.get(&cell).copied().unwrap_or(RoomRole::Normal)
    }

    pub fn append_draw_items(&self, input_device: InputDevice, draw_items: &mut Vec<DrawItem>) {
        if self.role_of(self.current_cell) == RoomRole::Start {
            start_room_hint::append_draw_items(input_device, draw_items);
        }
        draw_items.extend(self.doors().map(|side| side.draw_item(self.door_texture(side))));
        self.current_room().append_draw_items(draw_items);
    }
}

#[cfg(test)]
mod curse_tests;
#[cfg(test)]
mod tests;
