//! The trinket held, in the HUD's top left corner (`hud.extra2` the picture, `hud.black` the strip
//! behind its name, `hud.tex2` the name, `hud.plix` the ring round the picture). The picture, strip
//! and name are one texture per trinket, as the strip's width follows the name; the ring is swapped
//! for the card slot's animation while a trinket is being taken (`plix` frame 2).

use glam::Vec2;

use crate::game::draw::{DrawItem, Rgba};
use crate::game::pickups::trinket::Trinket;
use crate::game::room::stage_to_world;
use crate::game::texture_id::TextureId;

/// The corner of the HUD the slot's texture covers: around its center, this big.
pub const SLOT_CENTER: Vec2 = Vec2::new(108.0, 94.0);
pub const SLOT_SIZE: Vec2 = Vec2::new(216.0, 68.0);
/// `hud.plix` at (4.1, 69.95), where the ring's animation has its origin.
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

/// Whether the ring round the trinket's picture is drawn (`plix` frame 1) or not (frame 2 plays in
/// its place).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SlotRing {
    Shown,
    Hidden,
}

impl SlotRing {
    pub const ALL: [Self; 2] = [Self::Shown, Self::Hidden];

    pub const fn index(self) -> usize {
        match self {
            Self::Shown => 0,
            Self::Hidden => 1,
        }
    }
}

/// The slot for `trinket` (none held draws nothing), with the frame of the taking animation while
/// it plays.
pub fn append_draw_items(trinket: Option<Trinket>, taking_frame: Option<u8>, draw_items: &mut Vec<DrawItem>) {
    let Some(trinket) = trinket else { return };
    let ring = if taking_frame.is_some() {
        SlotRing::Hidden
    } else {
        SlotRing::Shown
    };
    draw_items.push(DrawItem::textured(
        TextureId::TrinketSlot(trinket, ring),
        stage_to_world(SLOT_CENTER),
        1.0,
        NO_FALLBACK,
        SLOT_DEPTH,
    ));
    if let Some(frame) = taking_frame {
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

    fn textures(trinket: Option<Trinket>, taking_frame: Option<u8>) -> Vec<TextureId> {
        let mut draw_items = Vec::new();
        append_draw_items(trinket, taking_frame, &mut draw_items);
        draw_items.iter().filter_map(|item| item.texture).collect()
    }

    #[test]
    fn no_trinket_draws_no_slot() {
        assert_eq!(textures(None, None), []);
        assert_eq!(textures(None, Some(0)), []);
    }

    #[test]
    fn a_trinket_held_shows_its_slot_with_the_ring() {
        assert_eq!(
            textures(Some(Trinket::Tick), None),
            [TextureId::TrinketSlot(Trinket::Tick, SlotRing::Shown)]
        );
    }

    #[test]
    fn a_trinket_being_taken_swaps_the_ring_for_the_animation() {
        assert_eq!(
            textures(Some(Trinket::Tick), Some(3)),
            [
                TextureId::TrinketSlot(Trinket::Tick, SlotRing::Hidden),
                TextureId::CardSlotAnimation { used: false, frame: 3 },
            ]
        );
    }
}
