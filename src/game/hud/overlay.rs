//! Full-screen animations the original plays on `over`, above everything: for now the two
//! eternal halves becoming a heart (frame 16). Also the pickup message (`st2`).

use crate::game::draw::{DrawItem, Rgba};
use crate::game::hud::held_card::{SLOT_TAKEN_FRAME_COUNT, SLOT_USED_FRAME_COUNT, SlotAnimation};
use crate::game::hud::message::{MESSAGE_FRAME_COUNT, MESSAGE_VISIBLE_FRAMES, PickupMessage};
use crate::game::hud::name_banner::BannerName;
use crate::game::original_units::Frames;
use crate::game::parameters::ScreenFadeParameters;
use crate::game::room::{STAGE_CENTER, STAGE_SIZE, stage_to_world};
use crate::game::texture_id::{CARD_NAME_FRAME_COUNT, CARD_NAME_TEXTURE_CENTER_X, card_animation_frame_count};
use crate::game::texture_id::{ETERNAL_HEART_UNION_FRAME_COUNT, TextureId};

/// `hud.st1` sits at (27, 84.8) in the HUD.
const CARD_NAME_BANNER_POSITION: glam::Vec2 = glam::Vec2::new(27.0, 84.8);
/// Above the HUD and the stage surround, below the view masks.
const OVERLAY_DEPTH: f32 = 860.0;
/// The black cover is above all of those, below the death note and the view masks.
const BLACK_FADE_DEPTH: f32 = 870.0;
const NO_FALLBACK: Rgba = Rgba {
    red: 0.0,
    green: 0.0,
    blue: 0.0,
    alpha: 0.0,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct Overlay {
    /// Frames into the eternal hearts' animation while it plays.
    eternal_heart_union_frames: Option<Frames>,
    /// The pickup message playing, and for how long.
    message: Option<(PickupMessage, Frames)>,
    /// A card's or an item's name on the floor-name banner (`st11()`), and for how long.
    banner: Option<(BannerName, Frames)>,
    /// A card's full-screen animation (`over` frame), and for how long.
    card_animation: Option<(u8, Frames)>,
    /// The HUD's card slot animating: used (else taken), for how long.
    slot: Option<(bool, Frames)>,
    /// The HUD's trinket slot animating after a trinket was taken, for how long.
    trinket_slot: Option<Frames>,
    /// The black cover fading out after the screen changed (`goblack()`): its opacity and how much
    /// of it goes each frame.
    black_fade: Option<(f32, f32)>,
    /// The HUD's active item box.
    pub item_box: super::item_box::ItemBox,
}

impl Overlay {
    /// The screen goes black and fades back in (`goblack()`).
    pub fn flash_black(&mut self, fade: ScreenFadeParameters) {
        self.black_fade = Some((fade.start_opacity, fade.step_per_frame));
    }

    /// The overlay's frame 16.
    pub fn play_eternal_heart_union(&mut self) {
        self.eternal_heart_union_frames = Some(Frames::ZERO);
    }

    /// `st22()`: a new message restarts the clip.
    pub fn show_message(&mut self, message: PickupMessage) {
        self.message = Some((message, Frames::ZERO));
    }

    /// `st11()`: the banner (`hud.st1`) slides in with the name.
    pub fn show_name(&mut self, name: BannerName) {
        self.banner = Some((name, Frames::ZERO));
    }

    /// The card slot's frame 2 on taking a card, 3 on using it.
    pub fn animate_card_slot(&mut self, used: bool) {
        self.slot = Some((used, Frames::ZERO));
    }

    pub fn slot_animation(&self) -> SlotAnimation {
        self.slot.map(|(used, frames)| (used, frames.frame_number()))
    }

    /// The trinket slot's frame 2 on taking a trinket.
    pub fn animate_trinket_slot(&mut self) {
        self.trinket_slot = Some(Frames::ZERO);
    }

    pub fn trinket_slot_animation(&self) -> Option<u8> {
        self.trinket_slot.map(Frames::frame_number)
    }

    /// The overlay on `clip_frame`: The Devil, Death, The Sun.
    pub fn play_card_animation(&mut self, clip_frame: u8) {
        self.card_animation = Some((clip_frame, Frames::ZERO));
    }

    pub fn advance(&mut self, elapsed: Frames) {
        self.black_fade = self
            .black_fade
            .map(|(opacity, step)| (opacity - step * elapsed.get(), step))
            .filter(|&(opacity, _)| opacity > 0.0);
        self.slot = self
            .slot
            .map(|(used, frames)| (used, frames + elapsed))
            .filter(|&(used, frames)| {
                let length = if used {
                    SLOT_USED_FRAME_COUNT
                } else {
                    SLOT_TAKEN_FRAME_COUNT
                };
                frames < Frames::from(length)
            });
        self.trinket_slot = self
            .trinket_slot
            .map(|frames| frames + elapsed)
            .filter(|&frames| frames < Frames::from(SLOT_TAKEN_FRAME_COUNT));
        self.banner = self
            .banner
            .map(|(name, frames)| (name, frames + elapsed))
            .filter(|&(_, frames)| frames < Frames::from(CARD_NAME_FRAME_COUNT));
        self.card_animation = self
            .card_animation
            .map(|(clip_frame, frames)| (clip_frame, frames + elapsed))
            .filter(|&(clip_frame, frames)| frames < Frames::from(card_animation_frame_count(clip_frame)));
        // Frame 124 hides the clip.
        let last_frame = Frames::from(MESSAGE_FRAME_COUNT - 1);
        self.message = self
            .message
            .map(|(message, frames)| (message, frames + elapsed))
            .filter(|&(_, frames)| frames < last_frame);
        self.eternal_heart_union_frames = self
            .eternal_heart_union_frames
            .map(|frames| frames + elapsed)
            .filter(|&frames| frames < Frames::from(ETERNAL_HEART_UNION_FRAME_COUNT));
    }

    fn append_black_fade(&self, draw_items: &mut Vec<DrawItem>) {
        if let Some((opacity, _)) = self.black_fade {
            let black = Rgba {
                alpha: opacity,
                ..NO_FALLBACK
            };
            draw_items.push(DrawItem::solid(STAGE_CENTER, STAGE_SIZE, black, BLACK_FADE_DEPTH));
        }
    }

    pub fn append_draw_items(&self, draw_items: &mut Vec<DrawItem>) {
        self.append_black_fade(draw_items);
        if let Some((name, frames)) = self.banner {
            let texture = TextureId::NameBanner {
                name,
                frame: frames.frame_number(),
            };
            let texture_center = CARD_NAME_BANNER_POSITION + glam::Vec2::new(CARD_NAME_TEXTURE_CENTER_X, 0.0);
            let position = stage_to_world(texture_center);
            draw_items.push(DrawItem::textured(texture, position, 1.0, NO_FALLBACK, OVERLAY_DEPTH));
        }
        if let Some((clip_frame, frames)) = self.card_animation {
            let texture = TextureId::CardAnimation {
                clip_frame,
                frame: frames.frame_number(),
            };
            draw_items.push(DrawItem::textured(
                texture,
                STAGE_CENTER,
                1.0,
                NO_FALLBACK,
                OVERLAY_DEPTH,
            ));
        }
        if let Some((message, frames)) = self.message
            && MESSAGE_VISIBLE_FRAMES.contains(&frames.frame_number())
        {
            let texture = TextureId::PickupMessage {
                message,
                frame: frames.frame_number(),
            };
            draw_items.push(DrawItem::textured(
                texture,
                PickupMessage::position(),
                1.0,
                NO_FALLBACK,
                OVERLAY_DEPTH,
            ));
        }
        if let Some(frames) = self.eternal_heart_union_frames {
            draw_items.push(DrawItem::textured(
                TextureId::EternalHeartUnion(frames.frame_number()),
                STAGE_CENTER,
                1.0,
                NO_FALLBACK,
                OVERLAY_DEPTH,
            ));
        }
    }
}
