//! Items: what an item pedestal holds and Isaac keeps for the run. Each has the original's number,
//! which is also its frame in the item pictures and its place in the list of item names.

/// The original's number for any item, including the ones the game does not have yet (what the
/// item rolls and the save data count by).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ItemNumber(pub u16);

impl From<Item> for ItemNumber {
    fn from(item: Item) -> Self {
        Self(item.number())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Item {
    /// Better odds at the machines and at room rewards (see where `Inventory::has` asks for it).
    LuckyFoot,
    /// My Little Unicorn, an active item: a unicorn ride (see `player::unicorn_ride`).
    MyLittleUnicorn,
    /// Mom's Purse: room for a second trinket (`ups[139]`, see `Inventory::trinket_slots`).
    MomsPurse,
}

impl Item {
    pub const ALL: [Self; 3] = [Self::LuckyFoot, Self::MyLittleUnicorn, Self::MomsPurse];
    /// The active ones, used with their own button.
    pub const ACTIVE: [Self; 1] = [Self::MyLittleUnicorn];

    /// The original's number for it.
    pub const fn number(self) -> u16 {
        match self {
            Self::LuckyFoot => 46,
            Self::MyLittleUnicorn => 77,
            Self::MomsPurse => 139,
        }
    }

    /// For an active item, the rooms to clear between uses; `None` for a passive one.
    pub const fn charges(self) -> Option<u8> {
        match self {
            Self::LuckyFoot | Self::MomsPurse => None,
            Self::MyLittleUnicorn => Some(6),
        }
    }

    /// An active item's place in the HUD's item pictures: the number less 32, or 40 from 55 on
    /// (`itn()`).
    pub const fn active_picture(self) -> u16 {
        const LATER_ITEMS_FROM: u16 = 55;
        const EARLY_OFFSET: u16 = 32;
        const LATER_OFFSET: u16 = 40;
        let number = self.number();
        if number < LATER_ITEMS_FROM {
            number - EARLY_OFFSET
        } else {
            number - LATER_OFFSET
        }
    }
}
