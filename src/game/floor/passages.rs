//! Which doors a room shows and which can be walked through, the walls around it, bombing open
//! the way into secret rooms, and unlocking the shop.
//!
//! A door between a room and a secret room is invisible (clip frame 10) and its wall cell is
//! bombable; a blast clears it (`bloww()`), after which the door shows as a blown hole (frame 17).
//! The shop's door is locked on every floor (frame 9) until the player walks into it with a key
//! (frame 8, `Unlock00.wav`).

use super::devil_room::DevilPassage;
use crate::game::floor::FloorCell;
use crate::game::texture_id::DoorTexture;
use glam::Vec2;

use crate::game::bounds::Bounds;
use crate::game::floor::Floor;
use crate::game::floor::plan::RoomRole;
use crate::game::original_units::{Frames, Pixels, PixelsPerFrame, distance};
use crate::game::room::doors::{DoorSide, DoorStyle};
use crate::game::room::{COLUMN_COUNT, ROW_COUNT, cell_bounds};
use crate::game::settings::Style;
use crate::game::sound_event::SoundEvent;
use crate::game::texture_id::{SHOP_DOOR_UNLOCKING_FRAME_COUNT, TextureId};

/// The door unlocks when the player, moving faster than 1 px per frame, will be within this of it
/// two frames on, moving into it: the grid cell this far ahead along the movement is the door's.
const UNLOCK_REACH: Pixels = Pixels(40.0);
const KEYHOLE_LOOKAHEAD: Pixels = Pixels(40.0);
const UNLOCK_LOOKAHEAD: Frames = Frames(2.0);
const UNLOCK_MINIMUM_SPEED: PixelsPerFrame = PixelsPerFrame(1.0);

impl Floor {
    /// Doors the room shows: to every neighbor, except that walls into or out of a secret room
    /// stay closed until bombed.
    pub(super) fn doors(&self) -> impl Iterator<Item = DoorSide> + '_ {
        DoorSide::ALL.into_iter().filter(|&side| {
            let Some(neighbor) = self.neighbor(side) else {
                return false;
            };
            !self.is_secret_passage(neighbor) || self.opened_passages.contains(&passage(self.current_cell, neighbor))
        })
    }

    /// Doors that can be walked through (a locked shop door is shown but shut, and enemies shut
    /// every door but a bombed-open hole).
    pub(super) fn open_doors(&self) -> impl Iterator<Item = DoorSide> + '_ {
        self.doors().filter(|&side| {
            let blown = self.neighbor(side).is_some_and(|cell| self.is_secret_passage(cell)) || self.is_blown(side);
            !self.is_locked(side) && (!self.doors_shut() || blown)
        })
    }

    /// The shop's and library's doors are locked until a key opens them (`shopa`, `liba`); a hole
    /// bombed in from a secret room is not a door and is always open.
    fn is_locked(&self, side: DoorSide) -> bool {
        self.neighbor(side).is_some_and(|cell| {
            let role = self.role_of(cell);
            is_lockable(role) && !self.unlocked_rooms.contains(&role) && !self.is_secret_passage(cell)
        })
    }

    pub(super) fn neighbor(&self, side: DoorSide) -> Option<FloorCell> {
        match self.devil_passage(side) {
            Some(DevilPassage::Leads(cell)) => return Some(cell),
            Some(DevilPassage::Closed) => return None,
            None => {}
        }
        self.current_cell.through(side).filter(|&cell| self.plan.has_room(cell))
    }

    fn is_secret_passage(&self, neighbor: FloorCell) -> bool {
        let is_secret = |cell| matches!(self.role_of(cell), RoomRole::Secret | RoomRole::SuperSecret);
        let curse_door = self.curse_secret_door == Style::Original
            && (self.role_of(self.current_cell) == RoomRole::Curse || self.role_of(neighbor) == RoomRole::Curse);
        (is_secret(self.current_cell) || is_secret(neighbor)) && !curse_door
    }

    /// Doors into or out of the curse room: spiked once open.
    fn is_curse_door(&self, side: DoorSide) -> bool {
        let curse = |cell| self.role_of(cell) == RoomRole::Curse;
        let Some(neighbor) = self.neighbor(side) else {
            return false;
        };
        (curse(self.current_cell) || curse(neighbor)) && !self.is_secret_passage(neighbor)
    }

    /// Centers of the spiked doorway cells of the current room's curse doors.
    pub fn curse_doorways(&self) -> Vec<Vec2> {
        self.open_doors()
            .filter(|&side| self.is_curse_door(side))
            .map(|side| {
                let (column, row) = side.doorway_cell();
                cell_bounds(column, row).center()
            })
            .collect()
    }

    pub(super) fn door_style(&self, side: DoorSide) -> DoorStyle {
        let neighbor = self.neighbor(side);
        let neighbor_role = neighbor.map(|cell| self.role_of(cell));
        let devil = |role| role == Some(RoomRole::Devil);
        if neighbor.is_some_and(|cell| self.is_secret_passage(cell)) {
            DoorStyle::BlownOpen
        } else if devil(neighbor_role) || devil(Some(self.role_of(self.current_cell))) {
            DoorStyle::Devil
        } else if self.is_curse_door(side) {
            DoorStyle::Curse
        } else if self.role_of(self.current_cell) == RoomRole::Boss || neighbor_role == Some(RoomRole::Boss) {
            DoorStyle::Boss
        } else if neighbor_role.is_some_and(is_lockable) {
            if self.is_locked(side) {
                DoorStyle::ShopLocked
            } else {
                DoorStyle::ShopOpen
            }
        } else if self.role_of(self.current_cell) == RoomRole::Treasure || neighbor_role == Some(RoomRole::Treasure) {
            DoorStyle::Treasure
        } else {
            DoorStyle::Plain
        }
    }

    /// A blast at `center` reaching `radius` opens any hidden wall into a secret room whose cell it
    /// touches; the first find on a floor plays the room's jingle (`hider()`, `hider2()`).
    pub fn blast_walls(&mut self, center: Vec2, radius: Pixels) -> Option<SoundEvent> {
        let mut jingle = None;
        for side in DoorSide::ALL {
            let Some(neighbor) = self.neighbor(side) else { continue };
            let (column, row) = side.doorway_cell();
            let wall = cell_bounds(column, row);
            let in_reach = distance(wall.clamp(center), center) <= radius;
            // Walls between a boss room and a secret room cannot be blown.
            let touches_boss =
                self.role_of(self.current_cell) == RoomRole::Boss || self.role_of(neighbor) == RoomRole::Boss;
            if !self.is_secret_passage(neighbor) || !in_reach || touches_boss {
                continue;
            }
            if self.opened_passages.insert(passage(self.current_cell, neighbor)) {
                let is_secret = |cell| matches!(self.role_of(cell), RoomRole::Secret | RoomRole::SuperSecret);
                let secret_room = if is_secret(neighbor) {
                    neighbor
                } else {
                    self.current_cell
                };
                jingle = jingle.or(self.first_find(secret_room));
            }
        }
        jingle
    }

    pub(super) fn first_find(&mut self, secret_room: FloorCell) -> Option<SoundEvent> {
        let role = self.role_of(secret_room);
        self.revealed_roles.insert(role).then_some(match role {
            RoomRole::SuperSecret => SoundEvent::SuperSecretRoomFound,
            _ => SoundEvent::SecretRoomFound,
        })
    }

    /// Leaving a secret room through `side` closes the wall behind, unless entrances stay open.
    pub(super) fn close_entrance_if_leaving_secret_room(&mut self, side: DoorSide) {
        let leaving_secret = matches!(
            self.role_of(self.current_cell),
            RoomRole::Secret | RoomRole::SuperSecret
        );
        if leaving_secret
            && self.secret_entrances == Style::Original
            && let Some(neighbor) = self.neighbor(side)
        {
            self.opened_passages.remove(&passage(self.current_cell, neighbor));
        }
    }

    /// Walking into the locked shop door with a key opens it for the floor; returns whether it did.
    pub fn unlock_doors(&mut self, player_position: Vec2, player_velocity: Vec2) -> bool {
        let moving = player_velocity.length() > UNLOCK_MINIMUM_SPEED.get();
        let probe = player_position + player_velocity * UNLOCK_LOOKAHEAD;
        let keyhole = player_position + player_velocity.normalize_or_zero() * KEYHOLE_LOOKAHEAD;
        let reached = self.doors().find(|&side| {
            let (column, row) = side.doorway_cell();
            self.is_locked(side)
                && distance(side.position(), probe) < UNLOCK_REACH
                && cell_bounds(column, row).contains(keyhole)
        });
        let Some(side) = reached.filter(|_| moving) else {
            return false;
        };
        if let Some(neighbor) = self.neighbor(side) {
            self.unlocked_rooms.insert(self.role_of(neighbor));
        }
        self.unlocking_door = Some((side, Frames::ZERO));
        true
    }

    /// Plays an unlocked door's key-turning animation (frame 8) once.
    pub fn advance_door_animations(&mut self, elapsed: Frames) {
        self.unlocking_door = self
            .unlocking_door
            .map(|(side, frames)| (side, frames + elapsed))
            .filter(|&(_, frames)| frames < Frames::from(SHOP_DOOR_UNLOCKING_FRAME_COUNT));
    }

    pub(super) fn door_texture(&self, side: DoorSide) -> TextureId {
        match self.unlocking_door {
            Some((unlocking_side, frames)) if unlocking_side == side => {
                TextureId::Door(DoorTexture::ShopUnlocking(frames.frame_number()))
            }
            _ => {
                let style = self.door_style(side);
                self.blown_door_texture(side)
                    .or_else(|| self.moving_door_texture(style))
                    .unwrap_or(TextureId::Door(DoorTexture::Open(style)))
            }
        }
    }

    /// The ring of wall cells around the interior, open where a door can be used.
    pub fn wall_tiles(&self) -> Vec<Bounds> {
        let open: Vec<(i32, i32)> = self.open_doors().map(DoorSide::doorway_cell).collect();
        let (last_column, last_row) = (
            i32::try_from(COLUMN_COUNT).unwrap_or(i32::MAX),
            i32::try_from(ROW_COUNT).unwrap_or(i32::MAX),
        );
        let horizontal = (-1..=last_column).flat_map(|column| [(column, -1), (column, last_row)]);
        let vertical = (0..last_row).flat_map(|row| [(-1, row), (last_column, row)]);
        horizontal
            .chain(vertical)
            .filter(|cell| !open.contains(cell))
            .map(|(column, row)| cell_bounds(column, row))
            .collect()
    }
}

/// Rooms behind a key-locked door (frame 9, then 8 when a key turns).
fn is_lockable(role: RoomRole) -> bool {
    matches!(role, RoomRole::Shop | RoomRole::Library)
}

pub(super) fn passage(first: FloorCell, second: FloorCell) -> (FloorCell, FloorCell) {
    (first.min(second), first.max(second))
}
