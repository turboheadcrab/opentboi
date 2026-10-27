//! The trinkets held, in the HUD's top left corner. The first slot is `hud.extra2` the picture,
//! `hud.black` the strip behind its name, `hud.tex2` the name and `hud.plix` the ring round the
//! picture; the second (with Mom's Purse) is `extra3`, `black2`, `tex3` and `plix2`. A slot's
//! picture, strip and name are one texture per trinket, as the strip's width follows the name; the
//! first ring is swapped for the card slot's animation while a trinket is being taken (`plix`
//! frame 2).

use glam::Vec2;

use crate::game::draw::{DrawItem, Rgba};
use crate::game::pickups::trinket::Trinket;
use crate::game::room::stage_to_world;
use crate::game::texture_id::TextureId;

/// Every slot's texture is this big, around its [`SlotView::center`].
pub const SLOT_SIZE: Vec2 = Vec2::new(216.0, 68.0);
/// `hud.plix` at (4.1, 69.95), where the first ring's animation has its origin.
const RING_POSITION: Vec2 = Vec2::new(4.1, 69.95);
/// As deep as the card's picture; the ring's animation lies just under it.
const SLOT_DEPTH: f32 = 802.0;
const RING_ANIMATION_DEPTH: f32 = 801.5;
const NO_FALLBACK: Rgba = Rgba {
    red: 0.0,
    green: 0.0,
    blue: 0.0,
    alpha: 0.0,
};

/// Which slot a texture shows, and whether the ring round the first one is there (`plix` frame 1)
/// or not (frame 2 plays in its place).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SlotView {
    First,
    FirstBeingTaken,
    Second,
}

impl SlotView {
    pub const ALL: [Self; 3] = [Self::First, Self::FirstBeingTaken, Self::Second];

    pub const fn index(self) -> usize {
        match self {
            Self::First => 0,
            Self::FirstBeingTaken => 1,
            Self::Second => 2,
        }
    }

    /// The corner of the HUD (stage coordinates) the texture covers.
    pub const fn center(self) -> Vec2 {
        match self {
            Self::First | Self::FirstBeingTaken => Vec2::new(108.0, 94.0),
            Self::Second => Vec2::new(144.0, 118.0),
        }
    }
}

/// The slots for the trinkets held (`trix` first, then `atrix`), with the frame of the first
/// one's taking animation while it plays.
pub fn append_draw_items(trinkets: &[Trinket], taking_frame: Option<u8>, draw_items: &mut Vec<DrawItem>) {
    let views = [
        if taking_frame.is_some() {
            SlotView::FirstBeingTaken
        } else {
            SlotView::First
        },
        SlotView::Second,
    ];
    for (&trinket, view) in trinkets.iter().zip(views) {
        draw_items.push(DrawItem::textured(
            TextureId::TrinketSlot(trinket, view),
            stage_to_world(view.center()),
            1.0,
            NO_FALLBACK,
            SLOT_DEPTH,
        ));
    }
    if let (Some(frame), false) = (taking_frame, trinkets.is_empty()) {
        draw_items.push(DrawItem::textured(
            TextureId::CardSlotAnimation { used: false, frame },
            stage_to_world(RING_POSITION),
            1.0,
            NO_FALLBACK,
            RING_ANIMATION_DEPTH,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn textures(trinkets: &[Trinket], taking_frame: Option<u8>) -> Vec<TextureId> {
        let mut draw_items = Vec::new();
        append_draw_items(trinkets, taking_frame, &mut draw_items);
        draw_items.iter().filter_map(|item| item.texture).collect()
    }

    #[test]
    fn no_trinket_draws_no_slot() {
        assert_eq!(textures(&[], None), []);
        assert_eq!(textures(&[], Some(0)), []);
    }

    #[test]
    fn a_trinket_held_shows_its_slot_with_the_ring() {
        assert_eq!(
            textures(&[Trinket::Tick], None),
            [TextureId::TrinketSlot(Trinket::Tick, SlotView::First)]
        );
    }

    #[test]
    fn a_trinket_being_taken_swaps_the_ring_for_the_animation() {
        assert_eq!(
            textures(&[Trinket::Tick], Some(3)),
            [
                TextureId::TrinketSlot(Trinket::Tick, SlotView::FirstBeingTaken),
                TextureId::CardSlotAnimation { used: false, frame: 3 },
            ]
        );
    }

    #[test]
    fn a_second_trinket_has_its_own_slot() {
        assert_eq!(
            textures(&[Trinket::Tick, Trinket::Cancer], None),
            [
                TextureId::TrinketSlot(Trinket::Tick, SlotView::First),
                TextureId::TrinketSlot(Trinket::Cancer, SlotView::Second),
            ]
        );
    }
}
