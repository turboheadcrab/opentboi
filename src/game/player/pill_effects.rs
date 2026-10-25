//! What a pill does when used (`pillc()`'s use branch, DoAction 27990–28240): its look takes an
//! effect at random the first time (`piller()`), a message says what it did, and Isaac cheers or
//! groans (`emo()`). What needs what does not exist yet (blue flies, the poison cloud and its
//! farting sound, which the sound bank cannot play, the bombs
//! the diarrhea pill drops, the hat and face of puberty, Isaac's color flash) only shows its
//! message.

use crate::game::Game;
use crate::game::hud::message::PickupMessage;
use crate::game::hud::name_banner::BannerName;
use crate::game::hud::pill_text::PillText;
use crate::game::odds::Odds;
use crate::game::original_units::Hearts;
use crate::game::pickups::pill::Pill;
use crate::game::player::animation::PlayerAnimation;
use crate::game::player::death::Killer;
use crate::game::player::pill_state::{PillEffect, PillKind, PillRollContext};
use crate::game::sound_event::SoundEvent;

/// Puberty is the hat one time in two, else the pill-eater's face.
const HAT_ODDS: Odds = Odds::one_in(2);
/// Health Up even with this much health or less, whichever pill version it is.
const HEALTH_UP_BELOW: Hearts = Hearts(1.0);

impl Game {
    /// Uses `pill`: gives its look an effect if it had none, then applies it.
    pub(crate) fn use_pill(&mut self, pill: Pill) {
        let context = PillRollContext {
            hurt: self.inventory.red_hearts < self.inventory.maximum_red_hearts,
            hearts_in_all: self.inventory.red_hearts + self.inventory.soul_hearts,
            has_bombs_or_keys: self.inventory.bombs > 0 || self.inventory.keys > 0,
            lucky: self.inventory.pills_only_help(),
        };
        let effect = self.inventory.pills.assign(pill, context, &mut self.random);
        self.apply_pill(effect, effect.good || context.lucky);
    }

    fn apply_pill(&mut self, effect: PillEffect, helps: bool) {
        let amounts = self.parameters.pills;
        let pills = &mut self.inventory.pills;
        match effect.kind {
            PillKind::FullHealth => {
                self.inventory.heal_fully();
                self.cheer(PillText::FullHealth);
            }
            PillKind::Puberty => self.use_puberty(),
            PillKind::Diarrhea => self.say(PillText::OhNo),
            PillKind::BallsOfSteel => {
                self.inventory.soul_hearts += amounts.soul_hearts;
                self.cheer(PillText::BallsOfSteel);
            }
            PillKind::PrettyFly => {
                pills.flies += amounts.fly_gain;
                self.cheer(PillText::PrettyFly);
            }
            PillKind::BadTrip => self.use_bad_trip(),
            PillKind::Health => self.use_health(helps),
            PillKind::BombsAreKeys => {
                std::mem::swap(&mut self.inventory.bombs, &mut self.inventory.keys);
                self.cheer(PillText::BombsAreKeys);
            }
            PillKind::Tears | PillKind::Range | PillKind::Speed | PillKind::ShotSpeed => {
                self.use_stat_pill(effect.kind, helps);
            }
            PillKind::BadGas => self.say(PillText::BadGas),
            PillKind::Friends => {
                self.blue_flies.owe(u32::from(amounts.blue_flies));
                self.say(PillText::Friends);
            }
            PillKind::Luck => self.use_luck(helps),
            PillKind::TelePills => {
                self.say(PillText::TelePills);
                self.teleport_anywhere();
            }
        }
    }

    /// 9, 10, 11 and 17: tears, range, speed or shot speed up or down.
    fn use_stat_pill(&mut self, kind: PillKind, helps: bool) {
        let amounts = self.parameters.pills;
        let pills = &mut self.inventory.pills;
        let (stat, gain, loss, good, bad) = match kind {
            PillKind::Tears => (
                &mut pills.tears,
                amounts.tears_up,
                amounts.tears_down,
                PillText::TearsUp,
                PillText::TearsDown,
            ),
            PillKind::Range => (
                &mut pills.range,
                amounts.range_up,
                amounts.range_down,
                PillText::RangeUp,
                PillText::RangeDown,
            ),
            PillKind::Speed => (
                &mut pills.speed,
                amounts.speed_up,
                amounts.speed_down,
                PillText::SpeedUp,
                PillText::SpeedDown,
            ),
            _ => (
                &mut pills.shot_speed,
                amounts.shot_speed_up,
                amounts.shot_speed_down,
                PillText::ShotSpeedUp,
                PillText::ShotSpeedDown,
            ),
        };
        *stat += if helps { gain } else { -loss };
        self.result(helps, good, bad);
    }

    /// 14: one luck more or less.
    fn use_luck(&mut self, helps: bool) {
        self.inventory.pills.luck += if helps { 1 } else { -1 };
        self.result(helps, PillText::LuckUp, PillText::LuckDown);
    }

    /// 2: a hat ("Puberty") or the face ("I found Pills", "And ate them!", `derp.mp`).
    fn use_puberty(&mut self) {
        if self.random.happens(HAT_ODDS) {
            self.overlay.show_name(BannerName::Pill(PillText::Puberty));
        } else {
            self.overlay.show_name(BannerName::Pill(PillText::IFoundPills));
            self.say(PillText::AndAteThem);
            self.pending_sound_events.push(SoundEvent::Derp);
        }
    }

    /// 6: a heart of damage even in the blink after a hit, unless that would be the last of it.
    fn use_bad_trip(&mut self) {
        let hearts_in_all = self.inventory.red_hearts + self.inventory.soul_hearts;
        if hearts_in_all > Hearts::ONE {
            self.inventory.end_invulnerability();
            self.hurt_player(self.parameters.pills.bad_trip_damage, Killer::BloodDonation);
            self.say(PillText::BadTrip);
        } else {
            self.inventory.heal_fully();
            self.cheer(PillText::FullHealth);
        }
    }

    /// 7: a heart container more, or one less; with one or none, always more.
    fn use_health(&mut self, helps: bool) {
        let inventory = &mut self.inventory;
        if helps || inventory.maximum_red_hearts <= HEALTH_UP_BELOW {
            inventory.maximum_red_hearts += Hearts::ONE;
            self.cheer(PillText::HealthUp);
        } else {
            inventory.maximum_red_hearts = (inventory.maximum_red_hearts - Hearts::ONE).max(Hearts::ZERO);
            inventory.red_hearts = inventory.red_hearts.min(inventory.maximum_red_hearts);
            self.groan(PillText::HealthDown);
        }
    }

    /// A good or bad result with its message and Isaac's reaction.
    fn result(&mut self, helps: bool, good: PillText, bad: PillText) {
        if helps {
            self.cheer(good);
        } else {
            self.groan(bad);
        }
    }

    fn say(&mut self, text: PillText) {
        self.overlay.show_message(PickupMessage::Pill(text));
    }

    /// `goodpill()` and `emo()`: a good pill's message, sound and thumbs up.
    fn cheer(&mut self, text: PillText) {
        self.say(text);
        self.pending_sound_events.push(SoundEvent::GoodPill);
        self.pending_sound_events.push(SoundEvent::ThumbsUp);
        self.player.play_animation(PlayerAnimation::ThumbsUp);
    }

    /// A bad pill's message, `emo(true)`'s sound and thumbs down.
    fn groan(&mut self, text: PillText) {
        self.say(text);
        self.pending_sound_events.push(SoundEvent::ThumbsDown);
        self.player.play_animation(PlayerAnimation::ThumbsDown);
    }
}
