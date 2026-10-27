//! What a blast sees of the current room (`linecheckxx()` over `golev()`'s grid): its objects, a door
//! in each doorway, and a hidden wall (1.85, which a blast reaches into) where a secret room is still
//! walled off; and the hidden walls a blast opens (`bloww()` on such a cell).

use glam::Vec2;

use crate::game::floor::Floor;
use crate::game::floor::passages::passage;
use crate::game::floor::plan::RoomRole;
use crate::game::original_units::{Pixels, distance};
use crate::game::room::cell_bounds;
use crate::game::room::doors::DoorSide;
use crate::game::room::sight::{HIDDEN_WALL_VALUE, SightGrid};
use crate::game::sound_event::SoundEvent;

impl Floor {
    /// The current room's grid as a blast sees it.
    pub fn blast_sight(&self) -> SightGrid {
        let mut sight = self.current_room().sight_grid();
        for side in DoorSide::ALL {
            if self.is_hidden_wall(side) {
                sight.set_doorway(side, HIDDEN_WALL_VALUE);
            }
        }
        sight
    }

    /// A wall into or out of a secret room that no blast has opened yet.
    fn is_hidden_wall(&self, side: DoorSide) -> bool {
        self.neighbor(side).is_some_and(|neighbor| {
            self.is_secret_passage(neighbor) && !self.opened_passages.contains(&passage(self.current_cell, neighbor))
        })
    }

    /// A blast at `center` reaching `radius` opens any hidden wall into a secret room whose cell it
    /// touches and whose center it sees (`sight`); the first find on a floor plays the room's jingle
    /// (`hider()`, `hider2()`).
    pub fn blast_walls(&mut self, center: Vec2, radius: Pixels, sight: &SightGrid) -> Option<SoundEvent> {
        let mut jingle = None;
        for side in DoorSide::ALL {
            let Some(neighbor) = self.neighbor(side) else { continue };
            let (column, row) = side.doorway_cell();
            let wall = cell_bounds(column, row);
            let in_reach = distance(wall.clamp(center), center) <= radius && sight.blast_reaches(center, wall.center());
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
}
