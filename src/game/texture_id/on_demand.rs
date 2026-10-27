//! Textures rendered only when first drawn, too many to make up front; listed for the tests that
//! render every texture.

use super::{IsaacPart, TextureId};
use crate::game::pickups::card::Card;
use crate::game::player::animation::{Held, PlayerAnimation};

impl TextureId {
    /// Name banners, machine frames, Isaac holding things up and in the ride's colors.
    pub fn on_demand_textures() -> Vec<Self> {
        let mut textures: Vec<Self> = crate::game::hud::name_banner::BannerName::all()
            .flat_map(|name| (0..super::CARD_NAME_FRAME_COUNT).map(move |frame| Self::NameBanner { name, frame }))
            .collect();
        let held = Card::ALL
            .map(Held::Card)
            .into_iter()
            .chain(crate::game::pickups::pill::Pill::ALL.map(Held::Pill))
            .chain(crate::game::pickups::trinket::Trinket::ALL.map(Held::Trinket))
            .chain(Held::WARES)
            .chain(crate::game::pickups::item::Item::ALL.map(Held::Item));
        let animations = held
            .clone()
            .map(PlayerAnimation::HoldingUp)
            .chain(held.map(PlayerAnimation::PickingUp))
            .chain([
                PlayerAnimation::Teleport,
                PlayerAnimation::TeleportArrival,
                PlayerAnimation::Dying,
                PlayerAnimation::ThumbsUp,
                PlayerAnimation::ThumbsDown,
            ]);
        for animation in animations {
            textures.extend((0..animation.frame_count()).map(|frame| Self::IsaacAnimation { animation, frame }));
        }
        textures.extend(
            crate::game::hud::message::PickupMessage::pill_messages().flat_map(|message| {
                crate::game::hud::message::MESSAGE_VISIBLE_FRAMES
                    .map(move |frame| Self::PickupMessage { message, frame })
            }),
        );
        textures.extend(Self::machine_textures());
        // The ride keeps his plain look, so only plain parts flash in colors.
        let tinted = IsaacPart::all()
            .filter(|part| !part.is_demonic())
            .flat_map(|part| super::RideColor::ALL.map(|color| (part, color)));
        textures.extend(tinted.map(|(part, color)| Self::Isaac {
            part,
            tint: Some(color),
        }));
        textures
    }

    fn machine_textures() -> Vec<Self> {
        use crate::game::machines::clip::{self, animation_length};
        let slot_frames = (clip::SLOT_IDLE..=clip::LAST_RESULT).chain([clip::SLOT_BROKEN]);
        let other_frames = [clip::BLOOD_IDLE, clip::DONATING, clip::BLOOD_BROKEN]
            .into_iter()
            .chain(clip::SHELL_IDLE..=clip::RIGHT_SHELL);
        let with_alternate = slot_frames
            .chain(clip::BEGGAR_IDLE..=clip::BEGGAR_LEAVES)
            .flat_map(|frame| [(false, frame), (true, frame)]);
        with_alternate
            .chain(other_frames.map(|frame| (false, frame)))
            .flat_map(|(alternate, frame)| {
                let prizes: &[u8] = if frame == clip::SHUFFLING {
                    &[0, 1, 2, 3, 5]
                } else {
                    &[0]
                };
                prizes.iter().flat_map(move |&prize| {
                    (0..animation_length(frame)).map(move |animation| Self::Machine {
                        alternate,
                        frame,
                        prize,
                        animation,
                    })
                })
            })
            .collect()
    }
}
