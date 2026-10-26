//! What a walker is doing and what its turns and hits make.

use glam::Vec2;

use crate::game::original_units::{Frames, PixelsPerFrame};
use crate::game::room::blood::Splatter;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WalkerState {
    /// `d` frame 4: the appearing clip.
    Appearing,
    /// Acting on its own: walking, wandering or shooting.
    Active,
    /// `dones`: `d` frame 3 plays this far.
    Dying(Frames),
}

/// Which of its clip's frames shows while it walks (`walkframe()`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WalkerClip {
    /// `d` frame 1.
    Standing,
    /// `d` frame 2, this long since it started.
    Walking { since: Frames },
    /// `d` frame 5, this long since it started: the Clotty's shot, the Mulligan's burst.
    Shooting { since: Frames },
}

/// The Clotty's mode (`mode`): wandering, or shooting once its shot clip plays.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ClottyMode {
    Undecided,
    Shooting,
    Wandering,
}

/// What the Clotty and the Mulligan remember between frames.
#[derive(Clone, Copy, Debug)]
pub(super) struct Blob {
    /// The Clotty's mode (`mode`), whether this mode's shot has gone (`modedone`) and the frame it was
    /// last wandering (`mof`).
    pub(super) mode: ClottyMode,
    pub(super) shot_done: bool,
    pub(super) mode_frame: Frames,
    /// The Mulligan: looks left of being scared (`needmove`), failed searches for a refuge
    /// (`failfind`), the frame of its last whimper, pressed against something this frame (`gh`),
    /// and the flies it owes the room.
    pub(super) fear_looks: i32,
    pub(super) search_failures: f32,
    pub(super) last_whimper: Option<Frames>,
    pub(super) cornered: bool,
    pub(super) owes_flies: bool,
}

impl Default for Blob {
    fn default() -> Self {
        Self {
            mode: ClottyMode::Undecided,
            shot_done: false,
            mode_frame: Frames::ZERO,
            fear_looks: 0,
            search_failures: 0.0,
            last_whimper: None,
            cornered: false,
            owes_flies: false,
        }
    }
}

/// What a walker's turn made: a growl, or a shot leaving it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "independent sounds a walker can make in one frame"
)]
pub struct WalkerTurn {
    pub growled: bool,
    pub shot: Option<WalkerShot>,
    /// A Clotty's shot squelching out (`animal_squish`), or a scared Mulligan whimpering.
    pub squished: bool,
    pub whimpered: bool,
    /// A Hopper landing (`Meat_impacts_`).
    pub landed: bool,
    /// Blood it leaves on the floor (`splater()`).
    pub spilled: Option<Splatter>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WalkerShot {
    /// Horf's shot, aimed at this point (the player).
    Aimed(Vec2),
    /// The headless body's blood ball, flying at this velocity.
    Spurted(Vec2),
    /// Four shots along the axes (`quadf()`), each at this speed.
    Cross(PixelsPerFrame),
}

/// What a hit did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hurt {
    Survived,
    /// It lost its head instead of dying, and goes on as a headless body.
    Beheaded,
    Died,
}
