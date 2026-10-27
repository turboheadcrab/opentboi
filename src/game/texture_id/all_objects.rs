//! The breakable objects' and the pickups' textures for the list of every texture.

use super::{BreakableKind, PickupKind, PickupPhase, TextureId};
use crate::game::pickups::card::Card;
use crate::game::pickups::pill::{Pill, PillLook};

impl TextureId {
    /// Every stage, breaking and growing back of the breakable objects.
    pub(super) fn breakables() -> Vec<Self> {
        let mut all = Vec::new();
        for kind in BreakableKind::ALL {
            for stage in 0..kind.stage_count() {
                all.extend(
                    (0..kind.idle_frame_count(stage)).map(|idle_frame| Self::BreakableStage {
                        kind,
                        stage,
                        idle_frame,
                    }),
                );
            }
            all.extend((0..kind.destroyed_frame_count()).map(|frame| Self::BreakableDestroyed { kind, frame }));
            all.extend((0..kind.regrow_frame_count()).map(|frame| Self::BreakableRegrow { kind, frame }));
        }
        all
    }

    /// Every pickup's frames, with one card of each look and every item on its pedestal.
    pub(super) fn pickups() -> impl Iterator<Item = Self> {
        let card_arts = [Card::WheelOfFortune, Card::TwoOfSpades].map(PickupKind::Card);
        let item_pedestals = crate::game::pickups::item::Item::ALL.map(PickupKind::Item);
        PickupKind::ALL
            .into_iter()
            .chain(card_arts)
            .chain(crate::game::pickups::trinket::Trinket::ALL.map(PickupKind::Trinket))
            .chain(item_pedestals)
            .flat_map(|kind| {
                PickupPhase::ALL.into_iter().flat_map(move |phase| {
                    (0..phase.frame_count(kind)).map(move |frame| Self::Pickup { kind, phase, frame })
                })
            })
            .chain(Self::pill_pickups())
    }

    /// Every frame of a pill lying about, in each of the looks a run may give it.
    fn pill_pickups() -> impl Iterator<Item = Self> {
        let kind = PickupKind::Pill(Pill::ALL[0]);
        PillLook::ALL.into_iter().flat_map(move |look| {
            PickupPhase::ALL.into_iter().flat_map(move |phase| {
                (0..phase.frame_count(kind)).map(move |frame| Self::PillPickup(look, phase, frame))
            })
        })
    }
}
