//! The end of a floor attempt: the boss room, the hidden rooms and the shop and treasure room,
//! taken from the dead ends (`DoAction.as` ~32100–32250).

use std::collections::BTreeMap;

use crate::game::floor::cell::{FloorCell, GRID_CELLS};
use crate::game::floor::plan::{
    FIRST_CHAPTER_SPECIAL_ROOM_DEAD_ENDS, FloorPlan, Growth, RoomRole, SECRET_ROOM_NEIGHBORS,
    SECRET_ROOM_REQUIRED_BEFORE_CHAPTER, SECRET_ROOM_TRIES, SPECIAL_ROOM_DEAD_ENDS,
};
use crate::game::odds::Odds;
use crate::game::random::Random;

const SACRIFICE_ODDS: Odds = Odds::one_in(7);
const SACRIFICE_FULL_HEALTH_ODDS: Odds = Odds::one_in(4);
/// `fullhp` is only set once play has started; the first floor is laid out before.
const FULL_HEALTH: bool = false;
const LIBRARY_ODDS: Odds = Odds::one_in(20);
const CURSE_ODDS: Odds = Odds::one_in(2);
const MINIBOSS_ODDS: Odds = Odds::one_in(4);
const FIRST_CHAPTER_MINIBOSS_ODDS: Odds = Odds::one_in(3);
/// The secret room sits in these columns.
const SECRET_ROOM_COLUMNS: std::ops::RangeInclusive<u8> = 2..=8;

impl Growth {
    /// The rest of one attempt: boss room, hidden rooms and special rooms, or `None` if the
    /// original would throw this floor away and grow another.
    pub(super) fn assign_rooms(&mut self, chapter: u8, random: &mut Random) -> Option<FloorPlan> {
        let boss_cell = self.dead_ends.pop().filter(|&cell| !cell.touches(FloorCell::START))?;
        let super_secret_cell = self.take_dead_end(random);
        let secret_cell = (self.room_count as f32 * 2.0 > self.target_rooms)
            .then(|| self.find_secret_cell(boss_cell, super_secret_cell, random))
            .flatten();
        if secret_cell.is_none() && chapter < SECRET_ROOM_REQUIRED_BEFORE_CHAPTER {
            return None;
        }
        // Four dead ends left (three in chapter 1), else the floor is grown again.
        let needed = if chapter == 1 {
            FIRST_CHAPTER_SPECIAL_ROOM_DEAD_ENDS
        } else {
            SPECIAL_ROOM_DEAD_ENDS
        };
        if self.dead_ends.len() < needed {
            return None;
        }
        let shop_cell = self.take_dead_end(random)?;
        let treasure_cell = self.take_dead_end(random)?;
        let [sacrifice_cell, library_cell, curse_cell, miniboss_cell] = self.optional_rooms(chapter, random);
        let mut roles_by_cell: BTreeMap<FloorCell, RoomRole> = FloorCell::all()
            .filter(|cell| self.occupied[cell.index()])
            .map(|cell| (cell, RoomRole::Normal))
            .collect();
        let specials = [
            (Some(FloorCell::START), RoomRole::Start),
            (Some(boss_cell), RoomRole::Boss),
            (super_secret_cell, RoomRole::SuperSecret),
            (secret_cell, RoomRole::Secret),
            (Some(shop_cell), RoomRole::Shop),
            (Some(treasure_cell), RoomRole::Treasure),
            (sacrifice_cell, RoomRole::Sacrifice),
            (curse_cell, RoomRole::Curse),
            (library_cell, RoomRole::Library),
            (miniboss_cell, RoomRole::MiniBoss),
        ];
        for (cell, role) in specials {
            if let Some(cell) = cell {
                roles_by_cell.insert(cell, role);
            }
        }
        Some(FloorPlan { roles_by_cell })
    }

    /// The rooms each floor may have, in the original's order, each taking a dead end if one is
    /// left: sacrifice (one in 7, or one in 4 at full health), library (one in 20), curse (one in
    /// 2), miniboss (one in 4, or one in 3 in chapter 1). Returns the sacrifice room, library, curse
    /// room and mini-boss room. Full health never counts on the first floor.
    fn optional_rooms(&mut self, chapter: u8, random: &mut Random) -> [Option<FloorCell>; 4] {
        let mut roll = |growth: &mut Self, odds: &dyn Fn(&mut Random) -> bool| {
            (!growth.dead_ends.is_empty() && odds(random))
                .then(|| growth.take_dead_end(random))
                .flatten()
        };
        let sacrifice = roll(self, &|random| {
            random.happens(SACRIFICE_ODDS) || (random.happens(SACRIFICE_FULL_HEALTH_ODDS) && FULL_HEALTH)
        });
        let library = roll(self, &|random| random.happens(LIBRARY_ODDS));
        let curse = roll(self, &|random| random.happens(CURSE_ODDS));
        let miniboss = roll(self, &|random| {
            random.happens(MINIBOSS_ODDS) || (chapter == 1 && random.happens(FIRST_CHAPTER_MINIBOSS_ODDS))
        });
        [sacrifice, library, curse, miniboss]
    }

    /// `hide`: up to 3000 random empty cells in columns 2–8; the first touching three or more
    /// rooms, none of them the boss or super secret room, holds the secret room.
    fn find_secret_cell(
        &self,
        boss_cell: FloorCell,
        super_secret_cell: Option<FloorCell>,
        random: &mut Random,
    ) -> Option<FloorCell> {
        (0..SECRET_ROOM_TRIES).find_map(|_| {
            let cell = FloorCell::all().nth(random.index(GRID_CELLS))?;
            if self.occupied[cell.index()] || !SECRET_ROOM_COLUMNS.contains(&cell.column()) {
                return None;
            }
            let rooms: Vec<FloorCell> = cell
                .array_neighbors()
                .into_iter()
                .flatten()
                .filter(|next| self.occupied[next.index()])
                .collect();
            let next_to_hidden_or_boss = rooms
                .iter()
                .any(|&room| room == boss_cell || Some(room) == super_secret_cell);
            (rooms.len() >= SECRET_ROOM_NEIGHBORS && !next_to_hidden_or_boss).then_some(cell)
        })
    }
}
