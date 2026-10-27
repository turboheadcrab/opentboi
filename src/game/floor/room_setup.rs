//! Setting up a floor's rooms: which layout each gets, the tinted rock, and the super secret
//! room's hearts.

use crate::game::floor::FloorCell;
use crate::game::odds::Odds;
use crate::game::room::{ORIGINAL_INTERIOR_FIRST_COLUMN, ORIGINAL_INTERIOR_FIRST_ROW};
use std::collections::BTreeMap;

use crate::game::floor::plan::{FloorPlan, RoomRole};
use crate::game::machines::MachineKind;
use crate::game::pickups::item::Item;
use crate::game::pickups::kind::{CoinKind, HeartKind, PickupKind};
use crate::game::random::Random;
use crate::game::room::contents::RoomContents;
use crate::game::room::layout::{RoomLayout, RoomLayouts};

/// Chapter 1 rooms use frames 2–116 of the room sprite, so layouts 1..=115 counting from 0.
/// The mini-boss layouts a floor may use.
const MINIBOSS_LAYOUTS: [usize; 6] = [0, 1, 2, 3, 5, 6];
const FIRST_CHAPTER_LAYOUTS: std::ops::Range<usize> = 1..116;
const ORIGINAL_GRID_ROW_LENGTH: i32 = 16;

/// A random layout for a room of `role` from the chapter's lists (`None` for rooms without one).
pub(super) fn choose_layout<'layouts>(
    role: RoomRole,
    layouts: &'layouts RoomLayouts,
    super_secret_style: u8,
    random: &mut Random,
) -> Option<&'layouts RoomLayout> {
    let pick = |list: &'layouts [RoomLayout], random: &mut Random| list.get(random.index(list.len()));
    match role {
        RoomRole::Normal => {
            let index = FIRST_CHAPTER_LAYOUTS.start + random.index(FIRST_CHAPTER_LAYOUTS.len());
            layouts.normal.get(index)
        }
        RoomRole::Treasure => pick(&layouts.treasure, random),
        RoomRole::Secret => pick(&layouts.secret, random),
        RoomRole::Shop => pick(&layouts.shop, random),
        RoomRole::Boss => pick(&layouts.boss, random),
        RoomRole::SuperSecret => layouts.super_secret.get(usize::from(super_secret_style)),
        RoomRole::Sacrifice => layouts.sacrifice.first(),
        RoomRole::Library => layouts.library.first(),
        RoomRole::MiniBoss => layouts
            .miniboss
            .get(MINIBOSS_LAYOUTS[random.index(MINIBOSS_LAYOUTS.len())]),
        RoomRole::Devil => layouts.devil.first(),
        RoomRole::Curse => layouts.curse.get(curse_layout_index(random)),
        RoomRole::Start => None,
    }
}

/// The curse room's layout is `c`, but one time in three `c0` or `c1`, and of those one in four
/// `c2`; the curse list holds `c`, `c0`, `c1`, `c2`.
fn curse_layout_index(random: &mut Random) -> usize {
    const VARIANT_ODDS: Odds = Odds::one_in(3);
    const NUMBERED_VARIANTS: usize = 2;
    const LAST_VARIANT_ODDS: Odds = Odds::one_in(4);
    const LAST_VARIANT_INDEX: usize = 3;
    if !random.happens(VARIANT_ODDS) {
        return 0;
    }
    let numbered = 1 + random.index(NUMBERED_VARIANTS);
    if random.happens(LAST_VARIANT_ODDS) {
        LAST_VARIANT_INDEX
    } else {
        numbered
    }
}

/// What a room of `role` holds beyond its layout: the devil's deals, the treasure room's item (the
/// Deck of Cards, until the item pools exist) and the super secret room's hearts.
pub(super) fn stock_room(room: &mut RoomContents, role: RoomRole, super_secret_style: u8, random: &mut Random) {
    match role {
        RoomRole::Devil => room.wares.make_devil_deals(random),
        RoomRole::Treasure => room.pickups.stock_pedestals(Item::DeckOfCards),
        RoomRole::SuperSecret => {
            if let Some(heart) = super_secret_hearts(super_secret_style) {
                room.pickups.set_heart_kind(heart);
            }
        }
        _ => {}
    }
}

/// `create()` in the super secret room: hearts are full red ones in style 0 (`col = 1`) and
/// eternal in style 1 (`col = 4`).
pub(super) fn super_secret_hearts(style: u8) -> Option<HeartKind> {
    match style {
        0 => Some(HeartKind::Full),
        1 => Some(HeartKind::Eternal),
        _ => None,
    }
}

/// Converts the original's grid index to an interior cell, if it is inside the room.
pub(super) fn tinted_rock_cell(index: i32) -> Option<(u32, u32)> {
    let column = index % ORIGINAL_GRID_ROW_LENGTH - ORIGINAL_INTERIOR_FIRST_COLUMN;
    let row = index / ORIGINAL_GRID_ROW_LENGTH - ORIGINAL_INTERIOR_FIRST_ROW;
    Some((u32::try_from(column).ok()?, u32::try_from(row).ok()?))
}

/// Where the start room's items stand (stage pixels): either side of the middle, above the
/// controls drawing, and Mom's Purse above them.
const START_ROOM_ITEMS: [(Item, glam::Vec2); 4] = [
    (Item::LuckyFoot, glam::Vec2::new(270.0, 190.0)),
    (Item::MyLittleUnicorn, glam::Vec2::new(370.0, 190.0)),
    (Item::MomsPurse, glam::Vec2::new(320.0, 140.0)),
    (Item::MomsBottleOfPills, glam::Vec2::new(440.0, 140.0)),
];
/// A fortune teller in the lower left corner, and a dime beside it to play with.
const START_ROOM_FORTUNE_TELLER: glam::Vec2 = glam::Vec2::new(130.0, 370.0);
const START_ROOM_DIME: glam::Vec2 = glam::Vec2::new(195.0, 390.0);

/// The start room offers the Lucky Foot and My Little Unicorn on pedestals, and a fortune teller
/// to try the foot's luck on (not in the original: a way to try items while the treasure rooms
/// cannot give them).
pub(super) fn add_start_room_items(rooms: &mut BTreeMap<FloorCell, RoomContents>) {
    use crate::game::room::stage_to_world;
    if let Some(start) = rooms.get_mut(&FloorCell::START) {
        for (item, stage_point) in START_ROOM_ITEMS {
            start.pickups.place(PickupKind::Item(item), stage_to_world(stage_point));
        }
        start
            .machines
            .place(MachineKind::FortuneTeller, stage_to_world(START_ROOM_FORTUNE_TELLER));
        start
            .pickups
            .place(PickupKind::Coin(CoinKind::Dime), stage_to_world(START_ROOM_DIME));
    }
}

/// Temporary, for checking art and rules: one pickup of every kind (and two eternal hearts) waits
/// in the boss room, one machine of each kind in the shop, every card and trinket in the treasure room.
pub(super) fn add_showcases(plan: &FloorPlan, rooms: &mut BTreeMap<FloorCell, RoomContents>) {
    let cell_of = |wanted: RoomRole| {
        plan.roles_by_cell
            .iter()
            .find(|(_, role)| **role == wanted)
            .map(|(cell, _)| *cell)
    };
    if let Some(boss_room) = cell_of(RoomRole::Boss).and_then(|cell| rooms.get_mut(&cell)) {
        boss_room.pickups.add_one_of_each();
        boss_room.pickups.add_eternal_hearts();
    }
    if let Some(shop) = cell_of(RoomRole::Shop).and_then(|cell| rooms.get_mut(&cell)) {
        shop.machines.add_one_of_each();
    }
    if let Some(treasure_room) = cell_of(RoomRole::Treasure).and_then(|cell| rooms.get_mut(&cell)) {
        treasure_room.pickups.add_every_card();
        treasure_room.pickups.add_every_trinket();
    }
}
