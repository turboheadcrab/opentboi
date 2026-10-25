//! Floor generation, ported from the original's level setup (`DoAction.as` ~32000): rooms grow
//! outward from the start cell on an 8×10 grid until the floor has enough of them; dead ends
//! become the boss room and special rooms.

use std::collections::BTreeMap;

use crate::game::floor::cell::{FloorCell, GRID_CELLS};
use crate::game::odds::Odds;
use crate::game::random::Random;
use crate::game::room::doors::DoorSide;

/// The floor's target room count: 5 plus 2.6 per chapter, plus 0 or 1, at most 20.
const ROOM_COUNT_LIMIT: f32 = 20.0;
const ROOM_COUNT_BASE: f32 = 5.0;
const ROOM_COUNT_PER_CHAPTER: f32 = 2.6;
/// Rooms per growth pass and the retry loop's bounds (`e < 300`, `fuckup++ < 1000`).
const GROWTH_PASSES: f32 = 300.0;
const MAXIMUM_ATTEMPTS: i32 = 1000;
/// Retried while the rooms times 1.25 fall short of the target, or while the boss room touches
/// the start room.
const MINIMUM_SIZE_FACTOR: f32 = 1.25;
/// Late passes of struggling attempts roll for every direction every time.
const EAGER_BRANCH_ODDS: Odds = Odds::one_in(1);
/// Treasure and shop rooms need this many spare dead ends (`endrooms.length >= 4`, or 3 in
/// chapter 1).
const SPECIAL_ROOM_DEAD_ENDS: usize = 4;
const FIRST_CHAPTER_SPECIAL_ROOM_DEAD_ENDS: usize = 3;
/// Chapters 1–2 always have a secret room: without one the floor is grown again.
const SECRET_ROOM_REQUIRED_BEFORE_CHAPTER: u8 = 3;
const SECRET_ROOM_TRIES: u32 = 3000;
const SECRET_ROOM_NEIGHBORS: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RoomRole {
    Start,
    Normal,
    Boss,
    Treasure,
    Shop,
    /// `hide`: an empty cell next to three or more rooms; its doors stay hidden until bombed.
    Secret,
    /// `hide2`: a dead end set aside, also hidden.
    SuperSecret,
    /// `cus`: behind spiked doors.
    Curse,
    /// `sac`: spikes that may give a chest.
    Sacrifice,
    /// `lib`: books on pedestals behind a locked door.
    Library,
    /// `minb`: a mini-boss (none exist yet, so the room stands empty).
    MiniBoss,
    /// Room 166, `satan`: reached from the boss room, off the map.
    Devil,
}

#[derive(Clone, Debug)]
pub struct FloorPlan {
    pub roles_by_cell: BTreeMap<FloorCell, RoomRole>,
}

impl FloorPlan {
    pub fn generate(chapter: u8, random: &mut Random) -> Self {
        let target_rooms = (random.below(2) as f32 + ROOM_COUNT_BASE + f32::from(chapter) * ROOM_COUNT_PER_CHAPTER)
            .min(ROOM_COUNT_LIMIT);
        let mut attempt = 0;
        loop {
            // `fuckup++ < 1000`: the counter is already 1 in the first attempt.
            attempt += 1;
            let mut growth = Growth::new(target_rooms);
            growth.grow(attempt, random);
            let plan = growth.assign_rooms(chapter, random);
            let big_enough = growth.room_count as f32 * MINIMUM_SIZE_FACTOR >= target_rooms;
            if let Some(plan) = plan.filter(|_| big_enough) {
                return plan;
            }
            if attempt >= MAXIMUM_ATTEMPTS {
                return Self::start_room_only();
            }
        }
    }

    pub fn has_room(&self, cell: FloorCell) -> bool {
        self.roles_by_cell.contains_key(&cell)
    }

    fn start_room_only() -> Self {
        Self {
            roles_by_cell: BTreeMap::from([(FloorCell::START, RoomRole::Start)]),
        }
    }
}

struct Growth {
    occupied: [bool; GRID_CELLS],
    target_rooms: f32,
    room_count: i32,
    dead_ends: Vec<FloorCell>,
    frontier: Vec<FloorCell>,
}

impl Growth {
    fn new(target_rooms: f32) -> Self {
        let mut growth = Self {
            occupied: [false; GRID_CELLS],
            target_rooms,
            room_count: 0,
            dead_ends: Vec::new(),
            frontier: Vec::new(),
        };
        growth.try_add(FloorCell::START);
        growth
    }

    /// `pffx()` (DoAction 39, DoAction 40, DoAction 45): a cell joins if it is empty, touches at most one room, and the floor is not full.
    /// A neighbour outside the grid has no value in the original, which makes the test fail.
    fn try_add(&mut self, cell: FloorCell) -> bool {
        let neighbors = cell
            .array_neighbors()
            .map(|neighbor| neighbor.map(|neighbor| self.occupied[neighbor.index()]));
        let neighbor_rooms = neighbors.iter().flatten().filter(|&&occupied| occupied).count();
        let all_defined = neighbors.iter().all(Option::is_some);
        let is_start = cell == FloorCell::START;
        let joins = is_start
            || (!self.occupied[cell.index()]
                && all_defined
                && neighbor_rooms < 2
                && (self.room_count as f32) < self.target_rooms);
        if joins {
            self.occupied[cell.index()] = true;
            self.frontier.push(cell);
            if !is_start {
                self.room_count += 1;
            }
        }
        joins
    }

    fn grow(&mut self, attempt: i32, random: &mut Random) {
        let mut pass = 0.0;
        while pass < GROWTH_PASSES {
            let active = std::mem::take(&mut self.frontier);
            if self.target_rooms > 15.0 {
                self.try_add(FloorCell::START);
            }
            for &cell in &active {
                self.grow_from(cell, pass, attempt, random);
            }
            if self.frontier.len() < 2 {
                pass -= 0.5;
            }
            pass += 1.0;
        }
    }

    fn grow_from(&mut self, cell: FloorCell, pass: f32, attempt: i32, random: &mut Random) {
        let mut added = 0;
        // Late passes of struggling attempts branch more eagerly.
        let eager = pass > (3 + random.below(3 - attempt)) as f32
            && ((self.room_count + random.below(2)) as f32) < self.target_rooms
            && attempt > 4 + random.below(4);
        let odds = if eager { EAGER_BRANCH_ODDS } else { Odds::EVEN };
        // The original rolls for every direction, even where the map has no door.
        for side in [DoorSide::Right, DoorSide::Left, DoorSide::Bottom, DoorSide::Top] {
            let rolled = random.happens(odds);
            if rolled
                && let Some(next) = cell.through(side)
                && self.try_add(next)
            {
                added += 1;
            }
        }
        if added == 0 && cell != FloorCell::START {
            self.dead_ends.push(cell);
        }
    }

    /// `ender()`: a random remaining dead end.
    fn take_dead_end(&mut self, random: &mut Random) -> Option<FloorCell> {
        if self.dead_ends.is_empty() {
            return None;
        }
        let index = random.index(self.dead_ends.len());
        Some(self.dead_ends.remove(index))
    }
}

mod special_rooms;

#[cfg(test)]
mod tests;
