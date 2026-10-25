//! Red chests (`c2`): when a brown chest turns red, and what one holds (`chestopen`'s `c2`
//! branch, DoAction ~31271). Spiders and blue flies need enemies and familiars and items are empty
//! pedestals, so those outcomes give nothing or an empty pedestal for now.

use crate::game::odds::Odds;
use crate::game::original_units::Pixels;
use crate::game::pickups::kind::{BombKind, ChestKind, HeartKind, PickupKind};
use crate::game::random::Random;

/// One brown chest in 20 is red after the first visit, and every one in the curse room and the
/// super secret room.
const RED_CHEST_ODDS: Odds = Odds::one_in(20);
/// The devil item it may hold: one of items 81, 134, 133 and 145.
const DEVIL_ITEM_CHOICES: i32 = 4;
/// One in 10 that item (if not held already); one in 18 the devil room; then one in 5 each for
/// spiders, a mega troll bomb and blue flies.
const ITEM_ODDS: Odds = Odds::one_in(10);
const DEVIL_ROOM_ODDS: Odds = Odds::one_in(18);
const SPIDERS_ODDS: Odds = Odds::one_in(5);
const MEGA_TROLL_BOMB_ODDS: Odds = Odds::one_in(5);
const BLUE_FLIES_ODDS: Odds = Odds::one_in(5);
/// Otherwise two soul hearts, troll bombs or pills (`cspawn()`), and half the time just one.
const THROWN_CHOICES: i32 = 3;
const SINGLE_SOUL_HEART_ODDS: Odds = Odds::one_in(2);
const THROWN_COUNT: u8 = 2;
/// The mega troll bomb's search radius (`spaw()`).
pub const MEGA_TROLL_BOMB_RADIUS: Pixels = Pixels(20.0);

/// Where a brown chest can turn red: the curse and super secret rooms always; other rooms one
/// time in twenty while the player is in them for the first time (`firsttime = beenlev[lev]`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RedChestChance {
    pub always: bool,
    pub first_visit: bool,
}

impl RedChestChance {
    /// `kind`, or a red chest if a brown one turns.
    pub fn apply(self, kind: PickupKind, random: &mut Random) -> PickupKind {
        if kind != PickupKind::Chest(ChestKind::Brown) {
            return kind;
        }
        let rolled = random.happens(RED_CHEST_ODDS);
        if rolled && self.first_visit || self.always {
            PickupKind::Chest(ChestKind::Red)
        } else {
            kind
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RedChestContents {
    /// A devil item on a pedestal where the chest stands (empty until items exist).
    Item,
    /// Teleported to the devil room (`teller()`).
    DevilRoom,
    /// `boil()`: spiders burst out (no enemies yet).
    Spiders,
    /// A mega troll bomb set down near the chest.
    MegaTrollBomb,
    /// Blue flies (two to four).
    BlueFlies,
    /// `count` pickups thrown out like a chest's; pills, each of a look of its own, for `None`.
    Thrown { kind: Option<PickupKind>, count: u8 },
}

/// What opening a red chest gives.
pub fn red_chest_contents(random: &mut Random) -> RedChestContents {
    random.below(DEVIL_ITEM_CHOICES);
    if random.happens(ITEM_ODDS) {
        return RedChestContents::Item;
    }
    if random.happens(DEVIL_ROOM_ODDS) {
        return RedChestContents::DevilRoom;
    }
    if random.happens(SPIDERS_ODDS) {
        return RedChestContents::Spiders;
    }
    if random.happens(MEGA_TROLL_BOMB_ODDS) {
        return RedChestContents::MegaTrollBomb;
    }
    if random.happens(BLUE_FLIES_ODDS) {
        return RedChestContents::BlueFlies;
    }
    let kind = match random.below(THROWN_CHOICES) {
        0 => Some(PickupKind::Heart(HeartKind::Soul)),
        1 => Some(PickupKind::Bomb(BombKind::Troll)),
        _ => None,
    };
    let single = kind == Some(PickupKind::Heart(HeartKind::Soul)) && random.happens(SINGLE_SOUL_HEART_ODDS);
    RedChestContents::Thrown {
        kind,
        count: if single { 1 } else { THROWN_COUNT },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curse_rooms_turn_every_brown_chest_red_and_leave_others() {
        let chance = RedChestChance {
            always: true,
            first_visit: false,
        };
        let mut random = Random::with_seed(1);
        let brown = PickupKind::Chest(ChestKind::Brown);
        assert_eq!(chance.apply(brown, &mut random), PickupKind::Chest(ChestKind::Red));
        let golden = PickupKind::Chest(ChestKind::Golden);
        assert_eq!(chance.apply(golden, &mut random), golden);
    }

    #[test]
    fn revisited_rooms_never_turn_chests_red() {
        let mut random = Random::with_seed(1);
        let brown = PickupKind::Chest(ChestKind::Brown);
        assert!((0..200).all(|_| RedChestChance::default().apply(brown, &mut random) == brown));
    }

    #[test]
    fn red_chests_give_every_outcome() {
        let mut random = Random::with_seed(3);
        let outcomes: Vec<RedChestContents> = (0..2000).map(|_| red_chest_contents(&mut random)).collect();
        assert!(outcomes.contains(&RedChestContents::Item));
        assert!(outcomes.contains(&RedChestContents::DevilRoom));
        assert!(outcomes.contains(&RedChestContents::MegaTrollBomb));
        assert!(
            outcomes
                .iter()
                .any(|outcome| matches!(outcome, RedChestContents::Thrown { count: 1, .. }))
        );
    }
}
