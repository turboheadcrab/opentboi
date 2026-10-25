//! Dying (`playerhurt()` at no health): Isaac slides to a stop deaf to the controls while the world
//! goes on, `isaacdies.wav` plays on the death clip's 32nd frame, and 51 frames later the death
//! note shows (the pause clip's frame 3) and the game stops. "Replay?" (Space) starts a new run
//! (button 1185, `newstart()`).

pub mod spotlight;

use crate::game::Game;
use crate::game::input_state::InputState;
use crate::game::original_units::Frames;
use crate::game::player::Surroundings;
use crate::game::player::animation::PlayerAnimation;
use crate::game::room;
use crate::game::sound_event::SoundEvent;
use crate::game::texture_id::FlyKind;

/// What killed Isaac: `playerhurt()` keeps who hurt him last, and the death note shows its
/// portrait (that frame of the note's `kill` clip).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Killer {
    /// A blast (portrait 4).
    Bomb,
    /// Blood creep, in `firecheck()` (portrait 5).
    Creep,
    /// A fireplace, in `firecheck()` (portrait 6).
    Fire,
    /// Spikes, or a curse doorway (portrait 7).
    Spikes,
    /// An attack fly (portrait 18), a red fly (61), a Moter (80), or a Pooter's shot (the
    /// shooter's type).
    Enemy(FlyKind),
    /// A blood machine, in `ader()` (portrait 201).
    BloodDonation,
    /// A Gaper's touch (portrait 10).
    Gaper,
    /// The headless body's touch, or its blood ball (portrait 11).
    Headless,
    /// Horf's touch, or its shot (portrait 12).
    Horf,
    /// A Clotty's touch, or its shots (portrait 15).
    Clotty,
    /// A Mulligan's touch, or its shots (portrait 16).
    Mulligan,
    /// A Hopper's touch (portrait 29).
    Hopper,
    /// A flesh mound's touch (portrait 77).
    FleshMound,
}

impl Killer {
    pub const ALL: [Self; 16] = [
        Self::Bomb,
        Self::Creep,
        Self::Fire,
        Self::Spikes,
        Self::Enemy(FlyKind::Attack),
        Self::Enemy(FlyKind::Pooter),
        Self::Enemy(FlyKind::Red),
        Self::Enemy(FlyKind::Moter),
        Self::BloodDonation,
        Self::Gaper,
        Self::Headless,
        Self::Horf,
        Self::Clotty,
        Self::Mulligan,
        Self::Hopper,
        Self::FleshMound,
    ];

    /// The original's number for it, which is the portrait's frame.
    pub fn portrait_frame(self) -> u16 {
        match self {
            Self::Bomb => 4,
            Self::Creep => 5,
            Self::Fire => 6,
            Self::Spikes => 7,
            Self::Enemy(FlyKind::Black) => 13,
            Self::Enemy(FlyKind::Pooter | FlyKind::SuperPooter) => 14,
            Self::Enemy(FlyKind::Attack) => 18,
            Self::Enemy(FlyKind::Red) => 61,
            Self::Enemy(FlyKind::Moter) => 80,
            Self::BloodDonation => 201,
            Self::Gaper => 10,
            Self::Headless => 11,
            Self::Horf => 12,
            Self::Clotty => 15,
            Self::Mulligan => 16,
            Self::Hopper => 29,
            Self::FleshMound => 77,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Death {
    pub killer: Killer,
    /// Time since the fatal hit.
    frames: Frames,
    /// "Replay?" answers a press, not a key already held.
    replay_held: bool,
}

impl Game {
    /// No health left: the death clip starts.
    pub(crate) fn die(&mut self, killer: Killer) {
        self.player.play_animation(PlayerAnimation::Dying);
        self.death = Some(Death {
            killer,
            frames: Frames::ZERO,
            replay_held: true,
        });
    }

    pub(crate) fn is_dying(&self) -> bool {
        self.death.is_some()
    }

    /// Time since the fatal hit, while dying.
    pub(crate) fn frames_dead(&self) -> Option<Frames> {
        self.death.map(|death| death.frames)
    }

    /// The killer once the death note shows.
    pub(crate) fn death_note(&self) -> Option<Killer> {
        let death = self.death?;
        (death.frames >= self.note_shows_after()).then_some(death.killer)
    }

    /// The input the world sees this frame: the controls while alive, nothing but the restart
    /// key while dying, and `None` once the death note stops the game (answering it starts a
    /// new run).
    pub(crate) fn advance_death(&mut self, input: &InputState, elapsed: Frames) -> Option<InputState> {
        let note_after = self.note_shows_after();
        let sound_frame = self.parameters.death.sound_frame;
        let Some(death) = &mut self.death else {
            return Some(*input);
        };
        if death.frames >= note_after {
            let pressed = input.replay && !death.replay_held;
            death.replay_held = input.replay;
            if pressed {
                self.restart();
            }
            return None;
        }
        let next = death.frames + elapsed;
        if death.frames < sound_frame && next >= sound_frame {
            self.pending_sound_events.push(SoundEvent::PlayerDied);
        }
        death.frames = next;
        death.replay_held = input.replay;
        Some(InputState {
            restart_held: input.restart_held,
            ..InputState::default()
        })
    }

    /// Isaac's own step: walking and shooting, or sliding to a stop while dying.
    pub(crate) fn move_player(&mut self, input: &InputState, elapsed: Frames) {
        let stats = self.player_stats();
        self.player.set_stats(stats);
        let (solid_tiles, spider_webs) = self.player_obstacles(stats.flying);
        let surroundings = Surroundings {
            // The wall ring is solid tiles, so open doorways can be walked into.
            room_interior: room::interior_bounds().shrunk_by(-room::TILE_SIZE),
            solid_tiles: &solid_tiles,
            spider_webs: &spider_webs,
        };
        if self.is_dying() {
            let retention = self.parameters.death.velocity_retention;
            self.player.update_dying(retention, elapsed, &surroundings);
        } else if let Some(new_tear) = self.player.update(input, elapsed, &surroundings, &mut self.random) {
            self.tears.push(new_tear);
            self.pending_sound_events.push(SoundEvent::TearFired);
        }
    }

    fn note_shows_after(&self) -> Frames {
        let death = self.parameters.death;
        death.sound_frame + death.note_after_sound
    }
}

#[cfg(test)]
mod tests;
