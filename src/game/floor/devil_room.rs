//! The devil room (room 166, layout `satan`): off the map, reached through a door in the boss room
//! (the devil door) on the side facing away from where the boss room is entered; its own door leads
//! back to the room last left. The original opens it after the boss, sometimes and never on the
//! first floor; for now it is always there and open.

use crate::game::floor::Floor;
use crate::game::floor::FloorCell;
use crate::game::floor::plan::{FloorPlan, RoomRole};
use crate::game::room::doors::DoorSide;

pub const DEVIL_CELL: FloorCell = FloorCell::DEVIL_ROOM;

/// What lies through a door when the devil room decides it.
pub(super) enum DevilPassage {
    Leads(FloorCell),
    /// The devil room has one door only.
    Closed,
}

/// Adds the devil room to `plan`; returns the side of the boss room its door is on.
pub(super) fn add_devil_room(plan: &mut FloorPlan) -> Option<DoorSide> {
    let boss = plan
        .roles_by_cell
        .iter()
        .find(|(_, role)| **role == RoomRole::Boss)
        .map(|(cell, _)| *cell)?;
    let entrance = DoorSide::ALL
        .into_iter()
        .find(|&side| boss.through(side).is_some_and(|cell| plan.has_room(cell)))?;
    plan.roles_by_cell.insert(DEVIL_CELL, RoomRole::Devil);
    Some(entrance.opposite())
}

impl Floor {
    /// The room last left, or the start room if there is none (`lastl`, DoAction 4880–4884).
    pub fn room_last_left(&self) -> FloorCell {
        self.last_room
            .filter(|&room| room != DEVIL_CELL)
            .unwrap_or(FloorCell::START)
    }

    /// `Some` when the devil room decides what lies through `side`: from the boss room its door,
    /// from the devil room only the way back.
    pub(super) fn devil_passage(&self, side: DoorSide) -> Option<DevilPassage> {
        let door = self.devil_door?;
        if self.current_cell == DEVIL_CELL {
            return Some(if side == door.opposite() {
                DevilPassage::Leads(self.room_last_left())
            } else {
                DevilPassage::Closed
            });
        }
        (self.role_of(self.current_cell) == RoomRole::Boss && side == door).then_some(DevilPassage::Leads(DEVIL_CELL))
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::*;
    use crate::game::parameters::GameParameters;
    use crate::game::random::Random;
    use crate::game::room::layout::RoomLayouts;

    #[test]
    fn the_boss_room_has_a_door_to_the_devil_room_and_back() {
        let mut random = Random::with_seed(7);
        let mut floor = Floor::generate(&RoomLayouts::default(), &GameParameters::ORIGINAL, &mut random);
        let boss = floor.cell_with(RoomRole::Boss).expect("a boss room");
        floor.teleport_to(boss, Vec2::ZERO);
        let devil_side = DoorSide::ALL
            .into_iter()
            .find(|&side| floor.neighbor(side) == Some(DEVIL_CELL))
            .expect("a devil door");
        floor.go_through(devil_side);
        assert_eq!(floor.current_cell(), DEVIL_CELL);
        floor.go_through(devil_side.opposite());
        assert_eq!(floor.current_cell(), boss);
        assert!(
            floor.map_pieces().iter().all(|(cell, _)| *cell != DEVIL_CELL),
            "off the map"
        );
    }
}
