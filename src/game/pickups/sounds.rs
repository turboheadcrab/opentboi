//! Sounds pickups make as they arrive: `aicol()` plays one when the landing clip reaches its 25th
//! frame once the room has run 35 frames; the bomb's landing clip (2038) plays its own on frame 26;
//! a chest dropped after the room's first 10 frames plays `Chest_Drop.mp` when it appears.

use crate::game::original_units::FIRST_CLIP_FRAME;
use crate::game::original_units::Frames;
use crate::game::pickups::kind::{CoinKind, PickupKind};
use crate::game::sound_event::SoundEvent;

/// Landing clip frames (1-based) that make the sounds.
const LANDING_SOUND_FRAME: Frames = Frames(25.0);
const BOMB_LANDING_SOUND_FRAME: Frames = Frames(26.0);
const QUIET_ROOM_START_FRAMES: Frames = Frames(35.0);
const QUIET_CHEST_ROOM_START_FRAMES: Frames = Frames(10.0);

/// The sound for a pickup whose landing went from `age_before` to `age_after` frames this tick,
/// `frames_in_room` into the room.
pub fn landing_sound(
    kind: PickupKind,
    age_before: Frames,
    age_after: Frames,
    frames_in_room: Frames,
) -> Option<SoundEvent> {
    if kind.is_chest() {
        let just_appeared = age_before <= Frames::ZERO;
        return (just_appeared && frames_in_room > QUIET_CHEST_ROOM_START_FRAMES).then_some(SoundEvent::ChestDropped);
    }
    let reaches = |clip_frame: Frames| {
        let frame_index = clip_frame - FIRST_CLIP_FRAME;
        age_before < frame_index && age_after >= frame_index
    };
    if matches!(kind, PickupKind::Bomb(_)) {
        return reaches(BOMB_LANDING_SOUND_FRAME).then_some(SoundEvent::BombLanded);
    }
    if !reaches(LANDING_SOUND_FRAME) || frames_in_room <= QUIET_ROOM_START_FRAMES {
        return None;
    }
    Some(match kind {
        PickupKind::Heart(_) => SoundEvent::HeartLanded,
        PickupKind::Coin(CoinKind::Penny) => SoundEvent::PennyLanded,
        PickupKind::Coin(CoinKind::Nickel) => SoundEvent::NickelLanded,
        PickupKind::Coin(CoinKind::Dime) => SoundEvent::DimeLanded,
        PickupKind::Key(_) => SoundEvent::KeyLanded,
        // Cards scamper; pills jump (`Fetus_Jump_` 0–1).
        PickupKind::Card(_) => SoundEvent::CardLanded,
        PickupKind::Pill(_) => SoundEvent::PillLanded,
        PickupKind::Bomb(_)
        | PickupKind::Chest(_)
        | PickupKind::ItemPedestal
        | PickupKind::Item(_)
        | PickupKind::Trinket(_) => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coins_ring_on_landing_frame_25_but_not_as_the_room_starts() {
        let penny = PickupKind::Coin(CoinKind::Penny);
        assert_eq!(
            landing_sound(penny, Frames(23.5), Frames(24.0), Frames(40.0)),
            Some(SoundEvent::PennyLanded)
        );
        assert_eq!(
            landing_sound(penny, Frames(24.0), Frames(24.5), Frames(40.0)),
            None,
            "once"
        );
        assert_eq!(
            landing_sound(penny, Frames(23.5), Frames(24.0), Frames(30.0)),
            None,
            "quiet at room start"
        );
    }
}
