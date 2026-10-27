//! What the player knows of the floor, for the minimap (`mapd()` in the original).

use crate::game::floor::DEVIL_CELL;
use crate::game::floor::DoorSide;
use crate::game::floor::Floor;
use crate::game::floor::FloorCell;
use crate::game::floor::plan::RoomRole;

/// One piece of the minimap: a room or a special room's icon (frames of clip `mpiece`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MapPiece {
    /// Seen through a door but never entered.
    Seen,
    Visited,
    /// The room the player is in.
    Current,
    ShopIcon,
    TreasureIcon,
    BossIcon,
    SecretIcon,
    SuperSecretIcon,
    CurseIcon,
    SacrificeIcon,
    LibraryIcon,
    MiniBossIcon,
}

impl MapPiece {
    pub const ALL: [Self; 12] = [
        Self::Seen,
        Self::Visited,
        Self::Current,
        Self::ShopIcon,
        Self::TreasureIcon,
        Self::BossIcon,
        Self::SecretIcon,
        Self::SuperSecretIcon,
        Self::CurseIcon,
        Self::SacrificeIcon,
        Self::LibraryIcon,
        Self::MiniBossIcon,
    ];
}

impl Floor {
    /// On entering a room: it and every room next to it count as seen (a wall into or out of a
    /// secret room counts like a door, though it shows none, so a secret room shows what lies around
    /// it; a secret room itself is drawn only once visited); a door to the
    /// shop, treasure, boss, curse room or library reveals its icon; the sacrifice and mini-boss
    /// rooms' show once entered (a door reveals them only with item 91).
    pub(super) fn reveal_surroundings(&mut self) {
        let neighbors: Vec<FloorCell> = DoorSide::ALL
            .into_iter()
            .filter_map(|side| self.neighbor(side))
            .collect();
        for cell in neighbors.into_iter().chain([self.current_cell]) {
            self.seen.insert(cell);
            let role = self.role_of(cell);
            if matches!(
                role,
                RoomRole::Shop | RoomRole::Treasure | RoomRole::Boss | RoomRole::Curse | RoomRole::Library
            ) {
                self.revealed_roles.insert(role);
            }
        }
        let current_role = self.role_of(self.current_cell);
        if matches!(current_role, RoomRole::Sacrifice | RoomRole::MiniBoss) {
            self.revealed_roles.insert(current_role);
        }
    }

    /// Visited rooms with something left in them, and their minimap mark.
    pub fn pickup_marks(&self) -> Vec<(FloorCell, u8)> {
        self.visited
            .iter()
            .filter(|&&cell| cell != DEVIL_CELL)
            .filter_map(|&cell| Some((cell, self.rooms.get(&cell)?.map_mark()?)))
            .collect()
    }

    /// Rooms then icons, in drawing order, by cell. Secret rooms only show as rooms once visited;
    /// their icons show once found.
    pub fn map_pieces(&self) -> Vec<(FloorCell, MapPiece)> {
        let is_secret = |cell| matches!(self.role_of(cell), RoomRole::Secret | RoomRole::SuperSecret);
        let rooms = self
            .seen
            .iter()
            .filter(|&&cell| cell != DEVIL_CELL && (!is_secret(cell) || self.visited.contains(&cell)))
            .map(|&cell| {
                let piece = if cell == self.current_cell {
                    MapPiece::Current
                } else if self.visited.contains(&cell) {
                    MapPiece::Visited
                } else {
                    MapPiece::Seen
                };
                (cell, piece)
            });
        let icons = self.plan.roles_by_cell.iter().filter_map(|(&cell, &role)| {
            let icon = match role {
                RoomRole::Shop => MapPiece::ShopIcon,
                RoomRole::Treasure => MapPiece::TreasureIcon,
                RoomRole::Boss => MapPiece::BossIcon,
                RoomRole::Secret => MapPiece::SecretIcon,
                RoomRole::SuperSecret => MapPiece::SuperSecretIcon,
                RoomRole::Curse => MapPiece::CurseIcon,
                RoomRole::Sacrifice => MapPiece::SacrificeIcon,
                RoomRole::Library => MapPiece::LibraryIcon,
                RoomRole::MiniBoss => MapPiece::MiniBossIcon,
                RoomRole::Start
                | RoomRole::Normal
                | RoomRole::Devil
                | RoomRole::SecondTreasure
                | RoomRole::Challenge
                | RoomRole::BossRush
                | RoomRole::Arcade => return None,
            };
            self.revealed_roles.contains(&role).then_some((cell, icon))
        });
        rooms.chain(icons).collect()
    }
}

#[cfg(test)]
mod tests;
