//! One ware: what it is, whether it is on sale, its price and how it is chosen (`create()` for
//! pickup frame 15).

use crate::game::odds::Odds;
use crate::game::pickups::consumable::Consumable;
use crate::game::random::Random;
use crate::game::shop::Stock;
use crate::game::texture_id::TextureId;

/// A random one of these, each at most once per room; the item (index 4) is never drawn directly but
/// comes in instead of index 3 (see [`Ware::roll`]).
pub const CHOICES: usize = 7;
const ITEM_CHOICE: usize = 3;
const NEVER_DRAWN: usize = 4;
/// An item after two other wares, or one in three after one.
const ITEM_AFTER_ODDS: Odds = Odds::one_in(3);
/// The pill ware's `col` is one of this many, from 1.
const CONSUMABLE_COLORS: u8 = 15;
/// One ware per shop may be on sale, one in five (its picture four frames on).
const SALE_ODDS: Odds = Odds::one_in(5);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WareKind {
    Heart,
    Bomb,
    Key,
    SoulHeart,
    /// The pill ware (`d.d` frame 3): a pill, or a tarot card, by a random `col` of 1–15.
    Consumable(Consumable),
    /// The devil room's deals (`d.d` frame 9 or 10, one time in five 11): items for heart
    /// containers, or soul hearts for frame 11. Items do not exist yet, so they cannot be taken.
    DevilDeal(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Ware {
    pub kind: WareKind,
    pub on_sale: bool,
}

impl Ware {
    /// The next ware; `None` for an item, which does not exist yet.
    pub(super) fn roll(stock: &mut Stock, random: &mut Random) -> Option<Self> {
        let mut choice = loop {
            let choice = random.index(CHOICES);
            if !stock.offered[choice] && choice != NEVER_DRAWN {
                break choice;
            }
            if stock
                .offered
                .iter()
                .enumerate()
                .all(|(index, &taken)| taken || index == NEVER_DRAWN)
            {
                return None;
            }
        };
        if choice == ITEM_CHOICE {
            stock.since_item = i32::MIN / 2;
        } else {
            stock.since_item += 1;
        }
        if stock.since_item > 1 || (stock.since_item == 1 && random.happens(ITEM_AFTER_ODDS)) {
            choice = ITEM_CHOICE;
            stock.since_item = i32::MIN / 2;
        }
        stock.offered[choice] = true;
        let on_sale = !stock.sale_given && random.happens(SALE_ODDS);
        stock.sale_given |= on_sale;
        let kind = match choice {
            0 => WareKind::Heart,
            1 => WareKind::Bomb,
            5 => WareKind::Key,
            6 => WareKind::SoulHeart,
            2 => WareKind::Consumable(Consumable::from_color(1 + random.below_u8(CONSUMABLE_COLORS))?),
            // An item (3).
            _ => return None,
        };
        Some(Self { kind, on_sale })
    }

    /// `d.d` frame: 1 heart, 2 bomb, 3 pill, 12 key, 13 soul heart, each 4 frames on when on sale.
    pub fn clip_frame(self) -> u8 {
        const SALE_FRAMES_ON: u8 = 4;
        let frame = match self.kind {
            WareKind::Heart => 1,
            WareKind::Bomb => 2,
            WareKind::Key => 12,
            WareKind::SoulHeart => 13,
            WareKind::Consumable(_) => 3,
            WareKind::DevilDeal(frame) => return frame,
        };
        frame + if self.on_sale { SALE_FRAMES_ON } else { 0 }
    }

    /// The ware's picture: its frame of the clip, and for a pill or card which one.
    pub fn texture(self) -> TextureId {
        match self.kind {
            WareKind::Consumable(consumable) => TextureId::ShopConsumableWare {
                color: consumable.color_variant(),
                on_sale: self.on_sale,
            },
            _ => TextureId::ShopWare(self.clip_frame()),
        }
    }

    /// In the devil room: one in five a soul heart deal (frame 11), else frame 9 or 10.
    pub(super) fn devil_deal(random: &mut Random) -> Self {
        const SOUL_HEART_DEAL_ODDS: Odds = Odds::one_in(5);
        const SOUL_HEART_DEAL: u8 = 11;
        const FIRST_HEART_DEAL: u8 = 9;
        const HEART_DEALS: u8 = 2;
        let frame = if random.happens(SOUL_HEART_DEAL_ODDS) {
            SOUL_HEART_DEAL
        } else {
            FIRST_HEART_DEAL + random.below_u8(HEART_DEALS)
        };
        Self {
            kind: WareKind::DevilDeal(frame),
            on_sale: false,
        }
    }

    /// Sold for coins (the devil's deals are not, see [`WareKind::DevilDeal`]).
    pub fn is_for_coins(self) -> bool {
        !matches!(self.kind, WareKind::DevilDeal(_))
    }

    /// Coins, by the ware's picture frame.
    pub fn price(self) -> u8 {
        const PRICES: [u8; 18] = [0, 3, 5, 5, 15, 2, 3, 3, 7, 1, 2, 3, 5, 5, 0, 0, 3, 3];
        PRICES[usize::from(self.clip_frame())]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pill_ware_sells_a_pill_or_card_for_five_coins_or_three_on_sale() {
        let mut random = Random::with_seed(3);
        let mut seen = 0;
        for _ in 0..200 {
            let mut stock = Stock {
                offered: [false; CHOICES],
                since_item: 0,
                sale_given: false,
            };
            for ware in (0..CHOICES).filter_map(|_| Ware::roll(&mut stock, &mut random)) {
                if matches!(ware.kind, WareKind::Consumable(_)) {
                    seen += 1;
                    assert_eq!(ware.clip_frame(), if ware.on_sale { 7 } else { 3 });
                    assert_eq!(ware.price(), if ware.on_sale { 3 } else { 5 });
                }
            }
        }
        assert!(seen > 0, "the shop never offered a pill");
    }

    #[test]
    fn a_shop_never_offers_the_same_ware_twice() {
        for seed in 1..50 {
            let mut random = Random::with_seed(seed);
            let mut stock = Stock {
                offered: [false; CHOICES],
                since_item: 0,
                sale_given: false,
            };
            let wares: Vec<Ware> = (0..5).filter_map(|_| Ware::roll(&mut stock, &mut random)).collect();
            for (index, ware) in wares.iter().enumerate() {
                assert!(wares[index + 1..].iter().all(|other| other.kind != ware.kind));
                assert!(ware.price() > 0);
            }
            assert!(wares.iter().filter(|ware| ware.on_sale).count() <= 1);
        }
    }
}
