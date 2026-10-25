//! Arriving in a room, through a door or by teleport (cards, `teller()`), and knowing the whole
//! map (`mapd()` after The World or The Sun).

use glam::Vec2;

use crate::game::floor::FloorCell;
use crate::game::floor::passages::passage;
use crate::game::floor::plan::RoomRole;
use crate::game::floor::{DEVIL_CELL, Floor, RoomEntry};
use crate::game::room::doors::DoorSide;

/// Before any door, a teleport lands at the bottom middle of the room.
pub(super) const FIRST_ARRIVAL_POSITION: Vec2 = Vec2::new(0.0, -120.0);
/// The order the room's doors are set up in: left, top, right, bottom, by where each puts the
/// player in the next room.
const WAY_OUT_ORDER: [DoorSide; 4] = [DoorSide::Left, DoorSide::Top, DoorSide::Right, DoorSide::Bottom];

impl Floor {
    /// What arriving in the current room at `position` does: rooms around it are seen, the
    /// shop and library unlock by being inside, and entering a secret room finds it.
    pub(super) fn arrive(&mut self, position: Vec2) -> RoomEntry {
        self.open_way_out_of_secret_room();
        self.set_doors_for_arrival();
        self.reveal_surroundings();
        let role = self.role_of(self.current_cell);
        if matches!(role, RoomRole::Shop | RoomRole::Library) {
            self.unlocked_rooms.insert(role);
        }
        let jingle = matches!(role, RoomRole::Secret | RoomRole::SuperSecret)
            .then(|| self.first_find(self.current_cell))
            .flatten();
        RoomEntry {
            position,
            role,
            first_visit: self.visited.insert(self.current_cell),
            jingle,
        }
    }

    /// The floor's room with `role`, if it has one.
    pub fn cell_with(&self, role: RoomRole) -> Option<FloorCell> {
        self.plan
            .roles_by_cell
            .iter()
            .find(|(_, each)| **each == role)
            .map(|(cell, _)| *cell)
    }

    /// Every room on the map, for picking somewhere to teleport to.
    pub fn cells(&self) -> Vec<FloorCell> {
        self.plan
            .roles_by_cell
            .keys()
            .copied()
            .filter(|&cell| cell != DEVIL_CELL)
            .collect()
    }

    /// Where a teleport lands: where the player last came in through a door.
    pub fn teleport_arrival(&self) -> Vec2 {
        self.last_entry_position
    }

    /// The player appears at `position` in `cell`, through no door.
    pub fn teleport_to(&mut self, cell: FloorCell, position: Vec2) -> RoomEntry {
        self.unlocking_door = None;
        self.current_cell = cell;
        self.resting_door = None;
        // No last room.
        self.last_room = None;
        self.arrive(position)
    }

    /// In the secret room without a last room (after a teleport, DoAction 5561), the first
    /// neighbor in door order that is not the boss room becomes it; the wall to the last room
    /// shows as the blown hole it came through (frame 17).
    fn open_way_out_of_secret_room(&mut self) {
        if self.role_of(self.current_cell) != RoomRole::Secret {
            return;
        }
        if self.last_room.is_none_or(|room| room == self.current_cell) {
            self.last_room = WAY_OUT_ORDER.into_iter().find_map(|side| {
                let neighbor = self.neighbor(side)?;
                (self.role_of(neighbor) != RoomRole::Boss).then_some(neighbor)
            });
        }
        let current = self.current_cell;
        let last_room_next_door = DoorSide::ALL
            .into_iter()
            .filter_map(|side| self.neighbor(side))
            .find(|&cell| Some(cell) == self.last_room);
        if let Some(neighbor) = last_room_next_door {
            self.opened_passages.insert(passage(current, neighbor));
        }
    }

    /// As with items 21 and 54 (`mapd()`): every room and every special room's icon shows (secret
    /// rooms still only once found).
    pub fn reveal_map(&mut self) {
        self.seen.extend(self.plan.roles_by_cell.keys().copied());
        let roles: Vec<RoomRole> = self.plan.roles_by_cell.values().copied().collect();
        self.revealed_roles.extend(
            roles
                .into_iter()
                .filter(|role| !matches!(role, RoomRole::Secret | RoomRole::SuperSecret)),
        );
    }

    #[cfg(test)]
    pub fn open_doors_count(&self) -> usize {
        self.open_doors().count()
    }
}
