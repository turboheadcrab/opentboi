//! The active item's box in the HUD (`hud.it`, clip 794): its frames 1–6 fill with the charge,
//! 7 is empty, 8–15 flash once the item is charged (15 staying), 16 lights up as a cleared room
//! charges it. Inside, `d` (clip 782) shows the item's picture, its frame 20 none.

use glam::Vec2;

use crate::game::draw::{DrawItem, Rgba};
use crate::game::original_units::Frames;
use crate::game::pickups::item::Item;
use crate::game::room::stage_to_world;
use crate::game::texture_id::TextureId;

pub const ITEM_BOX_FRAME_COUNT: u8 = 16;
/// The box's clip around its origin, before the HUD scales it.
pub const TEXTURE_SIZE: Vec2 = Vec2::new(84.0, 88.0);
const EMPTY_FRAME: u8 = 7;
const TAKEN_FRAME: u8 = 8;
const LAST_FLASHING_FRAME: u8 = 14;
const CHARGED_FRAME: u8 = 15;
const CHARGE_GAINED_FRAME: u8 = 16;
/// Frames 1–6 by charge.
const CHARGE_STEPS: u8 = 6;
/// How long the box holds a taken item's flash, and lights up for a charge (`itb`).
const TAKEN_FLASH_FRAMES: Frames = Frames(15.0);
const CHARGE_GAINED_FLASH_FRAMES: Frames = Frames(5.0);
/// Where the HUD places the box, and how it scales it.
const BOX_POSITION: Vec2 = Vec2::new(409.5, 39.9);
const BOX_SCALE: Vec2 = Vec2::new(0.965_6, 0.864_7);
const BOX_DEPTH: f32 = 801.0;
const NO_FALLBACK: Rgba = Rgba {
    red: 0.0,
    green: 0.0,
    blue: 0.0,
    alpha: 0.0,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemBox {
    /// The box's frame, from 1.
    frame: u8,
    /// Counts down the flashes (`itb`).
    flash_frames: Frames,
    /// Whole frames stepped so far, and how far into the next one.
    partial_frame: Frames,
}

impl Default for ItemBox {
    fn default() -> Self {
        Self {
            frame: EMPTY_FRAME,
            flash_frames: Frames::ZERO,
            partial_frame: Frames::ZERO,
        }
    }
}

impl ItemBox {
    /// An active item taken: the box flashes from frame 8.
    pub fn item_taken(&mut self) {
        self.frame = TAKEN_FRAME;
        self.flash_frames = TAKEN_FLASH_FRAMES;
    }

    /// A cleared room gave the item a charge.
    pub fn charge_gained(&mut self) {
        self.flash_frames = CHARGE_GAINED_FLASH_FRAMES;
    }

    /// `charge` is the held item's (rooms cleared, rooms needed), `None` without an active item.
    pub fn advance(&mut self, charge: Option<(u8, u8)>, elapsed: Frames) {
        self.partial_frame += elapsed;
        while self.partial_frame >= Frames::ONE {
            self.partial_frame -= Frames::ONE;
            self.step(charge);
        }
    }

    fn step(&mut self, charge: Option<(u8, u8)>) {
        let Some((charge, needed)) = charge else {
            self.frame = EMPTY_FRAME;
            return;
        };
        let flash_frames = self.flash_frames;
        if charge >= needed {
            if self.frame <= LAST_FLASHING_FRAME {
                self.frame += 1;
            } else {
                self.flash_frames -= Frames::ONE;
                if flash_frames < Frames::ZERO {
                    self.frame = CHARGED_FRAME;
                }
            }
        } else {
            self.flash_frames -= Frames::ONE;
            self.frame = if flash_frames > Frames::ZERO {
                CHARGE_GAINED_FRAME
            } else {
                charge * CHARGE_STEPS / needed.max(1) + 1
            };
        }
    }

    pub fn draw_item(self, item: Option<Item>) -> DrawItem {
        let texture = TextureId::ActiveItemBox {
            item,
            frame: self.frame - 1,
        };
        let mut draw_item = DrawItem::textured(texture, stage_to_world(BOX_POSITION), 1.0, NO_FALLBACK, BOX_DEPTH);
        draw_item.size *= BOX_SCALE;
        draw_item
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frames_after(item_box: &mut ItemBox, charge: Option<(u8, u8)>, frames: u8) -> Vec<u8> {
        (0..frames)
            .map(|_| {
                item_box.advance(charge, Frames(1.0));
                item_box.frame
            })
            .collect()
    }

    #[test]
    fn a_taken_item_flashes_then_shows_charged() {
        let mut item_box = ItemBox::default();
        item_box.item_taken();
        let frames = frames_after(&mut item_box, Some((6, 6)), 30);
        assert_eq!(frames[..7], [9, 10, 11, 12, 13, 14, 15]);
        assert_eq!(frames.last(), Some(&CHARGED_FRAME));
    }

    #[test]
    fn a_used_item_fills_with_each_room_and_lights_up_for_each_charge() {
        let mut item_box = ItemBox::default();
        assert_eq!(frames_after(&mut item_box, Some((0, 6)), 1), [1]);
        item_box.charge_gained();
        assert_eq!(frames_after(&mut item_box, Some((1, 6)), 6), [16, 16, 16, 16, 16, 2]);
        assert_eq!(frames_after(&mut item_box, Some((5, 6)), 1), [6]);
        assert_eq!(frames_after(&mut item_box, None, 1), [EMPTY_FRAME]);
    }
}
