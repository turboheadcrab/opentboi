//! Pills (pickup frame 7 with `col` 1–6): six looks, each of which has an effect assigned at
//! random the first time it is used (see `player::pill_effects`). A pill is held in the same slot
//! as a card (`pilc`, below 7).

use crate::game::odds::Odds;
use crate::game::pickups::consumable::Consumable;
use crate::game::pickups::kind::PickupKind;
use crate::game::random::Random;
use crate::game::texture_id::TextureId;

/// One of the six pills, by `col`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Pill(u8);

impl Pill {
    pub const COUNT: u8 = 6;
    pub const ALL: [Self; 6] = [Self(1), Self(2), Self(3), Self(4), Self(5), Self(6)];

    /// A pill marker for frame 7 becomes `col = random(6) + 1`.
    pub fn roll(random: &mut Random) -> Self {
        Self(1 + random.below_u8(Self::COUNT))
    }

    /// `col`: also the frame of the pickup's and the HUD's `p` clip.
    pub fn color_variant(self) -> u8 {
        self.0
    }

    /// Its place among [`Self::ALL`].
    pub fn index(self) -> usize {
        usize::from(self.0 - 1)
    }
}

/// How a pill is drawn: the frame of the pill pictures (`pic()`, DoAction 29236). A run draws its six
/// pills with the six plain looks, but for each of three extra ones that may take the place of one
/// (`picc()`, DoAction 29229–29234).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PillLook(u8);

impl PillLook {
    /// The six plain looks, then the three extra ones (frames 101–103).
    pub const ALL: [Self; 9] = [
        Self(1),
        Self(2),
        Self(3),
        Self(4),
        Self(5),
        Self(6),
        Self(101),
        Self(102),
        Self(103),
    ];
    const EXTRA: [Self; 3] = [Self(101), Self(102), Self(103)];

    /// The frame of the pill pictures.
    pub fn frame(self) -> u8 {
        self.0
    }

    /// Its place among [`Self::ALL`].
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|&each| each == self).unwrap_or(0)
    }
}

/// The look each of the six pills has this run (the original's `picers` list).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PillLooks([PillLook; 6]);

impl Default for PillLooks {
    /// Every pill with its own plain look.
    fn default() -> Self {
        Self([
            PillLook(1),
            PillLook(2),
            PillLook(3),
            PillLook(4),
            PillLook(5),
            PillLook(6),
        ])
    }
}

impl PillLooks {
    /// A new run's looks (`newstats()`, DoAction 797–800): each extra look in turn replaces one pill's
    /// plain look, one time in two (`picc()`, DoAction 788–793); a later one can replace an earlier.
    pub fn shuffled(random: &mut Random) -> Self {
        let mut looks = Self::default();
        for extra in PillLook::EXTRA {
            if random.happens(Odds::EVEN) {
                looks.0[random.index(looks.0.len())] = extra;
            }
        }
        looks
    }

    pub fn look_of(self, pill: Pill) -> PillLook {
        self.0[pill.index()]
    }

    /// `texture` for something drawn from a pill's `col`, in the look this run gives that pill
    /// (`pic()`, DoAction 26266): a pill lying about, or the shop's pill ware.
    pub fn show(self, texture: TextureId) -> TextureId {
        match texture {
            TextureId::Pickup {
                kind: PickupKind::Pill(pill),
                phase,
                frame,
            } => TextureId::PillPickup(self.look_of(pill), phase, frame),
            TextureId::ShopConsumableWare { color, on_sale } => match Consumable::from_color(color) {
                Some(Consumable::Pill(pill)) => TextureId::ShopConsumableWare {
                    color: self.look_of(pill).frame(),
                    on_sale,
                },
                _ => texture,
            },
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pills_have_the_originals_colors_one_to_six() {
        let colors: Vec<u8> = Pill::ALL.iter().map(|pill| pill.color_variant()).collect();
        assert_eq!(colors, [1, 2, 3, 4, 5, 6]);
        assert_eq!(Pill::ALL[3].index(), 3);
    }

    #[test]
    /// DoAction 788–800: a run's looks are the plain six, each extra look replacing one now and then,
    /// no look twice.
    fn a_run_has_six_different_looks_and_the_extra_ones_turn_up() {
        let mut extras_seen = [false; 3];
        for seed in 0..200 {
            let looks = PillLooks::shuffled(&mut Random::with_seed(seed));
            let shown: Vec<PillLook> = Pill::ALL.iter().map(|&pill| looks.look_of(pill)).collect();
            for (index, each) in shown.iter().enumerate() {
                assert!(!shown[index + 1..].contains(each), "{shown:?}");
            }
            for (slot, extra) in PillLook::EXTRA.iter().enumerate() {
                extras_seen[slot] |= shown.contains(extra);
            }
        }
        assert!(extras_seen.iter().all(|&each| each));
        assert_eq!(PillLooks::default().look_of(Pill::ALL[4]).frame(), 5);
        assert_eq!(PillLook::ALL[7].index(), 7);
    }

    #[test]
    /// DoAction 26266: a pill lying about, or the shop's, is drawn in the look its pill has this run.
    fn pills_drawn_take_the_look_of_the_run() {
        use crate::game::pickups::phase::PickupPhase;
        let mut random = Random::with_seed(3);
        let looks = (0..100)
            .map(|_| PillLooks::shuffled(&mut random))
            .find(|looks| looks.look_of(Pill::ALL[1]).frame() > 100)
            .expect("an extra look turns up");
        let lying = TextureId::Pickup {
            kind: PickupKind::Pill(Pill::ALL[1]),
            phase: PickupPhase::Idle,
            frame: 4,
        };
        let look = looks.look_of(Pill::ALL[1]);
        assert_eq!(looks.show(lying), TextureId::PillPickup(look, PickupPhase::Idle, 4));
        let ware = TextureId::ShopConsumableWare {
            color: 2,
            on_sale: true,
        };
        assert_eq!(
            looks.show(ware),
            TextureId::ShopConsumableWare {
                color: look.frame(),
                on_sale: true
            }
        );
        let card = TextureId::ShopConsumableWare {
            color: 9,
            on_sale: false,
        };
        assert_eq!(looks.show(card), card, "a card ware keeps its picture");
    }

    #[test]
    fn rolled_pills_cover_all_six_looks() {
        let mut random = Random::default();
        let mut seen = [false; 6];
        for _ in 0..200 {
            seen[Pill::roll(&mut random).index()] = true;
        }
        assert!(seen.iter().all(|&each| each));
    }
}
