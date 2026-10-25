//! The shop's wares (pickup frame 15, layout marker 15): pickups with a price tag, bought by
//! touching them with enough coins; they do not block (`case 15` of the touch handler). Items are not sold yet,
//! so their places stay empty.

mod ware;

use glam::Vec2;

use crate::game::draw::{DrawItem, Rgba, actor_depth};
use crate::game::original_units::distance;
use crate::game::parameters::PickupParameters;
use crate::game::pickups::markers;
use crate::game::random::Random;
use crate::game::room::layout::RoomLayout;
pub use ware::{Ware, WareKind};

const WARE_MARKER: u8 = 15;
const FALLBACK_COLOR: Rgba = Rgba::opaque(0.9, 0.8, 0.2);

/// `it1`, `watz` and `salefail`: what the room's wares have already been.
struct Stock {
    offered: [bool; ware::CHOICES],
    since_item: i32,
    sale_given: bool,
}

#[derive(Clone, Debug)]
pub struct Wares {
    parameters: PickupParameters,
    /// What each place sells (`None` for wares not made yet), and where.
    on_offer: Vec<(Ware, Vec2)>,
}

impl Wares {
    pub fn from_layout(layout: &RoomLayout, parameters: PickupParameters, random: &mut Random) -> Self {
        let mut stock = Stock {
            offered: [false; ware::CHOICES],
            since_item: 0,
            sale_given: false,
        };
        let on_offer = markers(layout, parameters)
            .filter(|(frame, _)| *frame == WARE_MARKER)
            .filter_map(|(_, center)| Some((Ware::roll(&mut stock, random)?, center)))
            .collect();
        Self { parameters, on_offer }
    }

    #[cfg(test)]
    pub fn put_on_offer(&mut self, ware: Ware, center: Vec2) {
        self.on_offer.push((ware, center));
    }

    /// The devil room: every ware is a devil's deal instead.
    pub fn make_devil_deals(&mut self, random: &mut Random) {
        for (ware, _) in &mut self.on_offer {
            *ware = Ware::devil_deal(random);
        }
    }

    /// Buys the first ware the player touches that the coins pay for (`balljunk()` case 15,
    /// DoAction 7920–7998, prices in `Ware::price`). Wares never bump. The original sells a heart at
    /// full health too.
    pub fn buy(&mut self, player_position: Vec2, coins: u8) -> Option<(Ware, Vec2)> {
        let reach = self.parameters.collect_reach;
        let index = self.on_offer.iter().position(|&(ware, center)| {
            ware.is_for_coins() && distance(center, player_position) < reach && coins >= ware.price()
        })?;
        Some(self.on_offer.remove(index))
    }

    pub fn append_draw_items(&self, draw_items: &mut Vec<DrawItem>) {
        draw_items.extend(self.on_offer.iter().map(|(ware, center)| {
            DrawItem::textured(ware.texture(), *center, 1.0, FALLBACK_COLOR, actor_depth(center.y))
        }));
    }
}
