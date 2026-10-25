//! Enemies' clips: the black fly `b13` (1517) has `d` frame 1 flying (its bob 1515 holding the
//! wings 1514), 3 dying (555) and 4 appearing (1516); the attack fly `b18` (5053) the same frames
//! with its flap (578) and appearing (5052); the Pooter `b14` (4990) 1 flying and 5 shooting, each
//! showing `hx` on the variant's frame, 3 dying, 4 appearing.
//! Enemy shots are `bullet1` (1459): a shadow (341, drawn as the tear's) and the ball `d`, pulsing
//! in brightness (`bull`); they end in a `bloodsplash` (1488).

use swf::CharacterId;

use super::{TextureSource, centered_on, framed};
use crate::game::{FlyAnimation, FlyKind, TextureId};
use crate::game_assets::vector_art::{ChildState, FrameSelection, RenderOverrides, Tint};

const BLACK_FLY_CLIP: CharacterId = 1517;
const ATTACK_FLY_CLIP: CharacterId = 5053;
const POOTER_CLIP: CharacterId = 4990;
const RED_FLY_CLIP: CharacterId = 4872;
const MOTER_CLIP: CharacterId = 4904;
const FLYING_FRAME: u16 = 1;
const DYING_FRAME: u16 = 3;
const APPEARING_FRAME: u16 = 4;
const SHOOTING_FRAME: u16 = 5;
const SHOT_CLIP: CharacterId = 1459;
const SHOT_SHADOW: CharacterId = 341;
/// The ball `d` sits 23.75 above the clip's origin as authored (the game moves it to `dy`); the
/// texture is centered on it.
const SHOT_BALL_CENTER: (f32, f32) = (0.0, -23.75);
const BLOOD_SPLASH_CLIP: CharacterId = 1488;

/// While a hit's flash lasts (`colorit()`): red, 0.4 of the red and half of the green and blue
/// kept, plus 200 red.
const HURT_TINT: Tint = Tint {
    multiply_per_mille: [400, 500, 500],
    add: [200, 0, 0],
};

macro_rules! overrides {
    ($tint:expr; $($path:literal => $state:expr),*) => {
        RenderOverrides {
            children: &[$(($path, $state)),*],
            texts: &[],
            hidden_characters: &[],
            character_frames: &[],
            tint: $tint,
        }
    };
}

static HURT_FLASH: RenderOverrides = overrides!(Some(HURT_TINT););
/// The wings (`d.d`, 1514) on frame 1 or 2: each fly starts them on a random one; without and
/// with the hit flash.
static WINGS: [[RenderOverrides; 2]; 2] = [
    [
        overrides!(None; "d.d" => ChildState::Frame(1)),
        overrides!(None; "d.d" => ChildState::Frame(2)),
    ],
    [
        overrides!(Some(HURT_TINT); "d.d" => ChildState::Frame(1)),
        overrides!(Some(HURT_TINT); "d.d" => ChildState::Frame(2)),
    ],
];
/// While appearing, the clip's `d` (4949) holds `hx`, which would otherwise cycle through its
/// frames: the script puts it on the variant's frame every frame (DoAction 31501).
static POOTER_APPEARING: [[RenderOverrides; 2]; 2] = [
    [
        overrides!(None; "d.hx" => ChildState::Frame(1)),
        overrides!(None; "d.hx" => ChildState::Frame(2)),
    ],
    [
        overrides!(Some(HURT_TINT); "d.hx" => ChildState::Frame(1)),
        overrides!(Some(HURT_TINT); "d.hx" => ChildState::Frame(2)),
    ],
];
/// The Pooter's `hx` on its variant's frame: 1 the Pooter, 2 the two-shot one.
static POOTER_VARIANTS: [[RenderOverrides; 2]; 2] = [
    [
        overrides!(None; "hx" => ChildState::Frame(1)),
        overrides!(None; "hx" => ChildState::Frame(2)),
    ],
    [
        overrides!(Some(HURT_TINT); "hx" => ChildState::Frame(1)),
        overrides!(Some(HURT_TINT); "hx" => ChildState::Frame(2)),
    ],
];

macro_rules! shot_brightness {
    ($($per_mille:literal)*) => {
        [$(RenderOverrides {
            children: &[("d", ChildState::Brightness { per_mille: $per_mille })],
            texts: &[],
            hidden_characters: &[SHOT_SHADOW],
            character_frames: &[],
            tint: None,
        }),*]
    };
}

/// `bull`'s multipliers 0.45, 1 and 1.85 (`actions2()`), the shadow drawn apart.
static SHOT_BRIGHTNESS: [RenderOverrides; 3] = shot_brightness!(450 1000 1850);

pub(super) fn source_for(texture_id: TextureId) -> TextureSource {
    match texture_id {
        TextureId::EnemyShot(step) => TextureSource {
            overrides: &SHOT_BRIGHTNESS[usize::from(step).min(SHOT_BRIGHTNESS.len() - 1)],
            ..centered_on(SHOT_CLIP, 1, SHOT_BALL_CENTER)
        },
        TextureId::BloodSplash(frame) => framed(BLOOD_SPLASH_CLIP, u16::from(frame) + 1),
        TextureId::Fly {
            kind,
            animation,
            frame,
            wing,
            flashing,
        } => fly_source(kind, animation, frame, wing, flashing),
        _ => framed(BLACK_FLY_CLIP, FLYING_FRAME),
    }
}

fn fly_source(kind: FlyKind, animation: FlyAnimation, frame: u8, wing: u8, flashing: bool) -> TextureSource {
    let clip = match kind {
        FlyKind::Black => BLACK_FLY_CLIP,
        FlyKind::Attack => ATTACK_FLY_CLIP,
        FlyKind::Pooter | FlyKind::SuperPooter => POOTER_CLIP,
        FlyKind::Red => RED_FLY_CLIP,
        FlyKind::Moter => MOTER_CLIP,
    };
    let clip_frame = match animation {
        FlyAnimation::Appearing => APPEARING_FRAME,
        FlyAnimation::Flying => FLYING_FRAME,
        FlyAnimation::Dying => DYING_FRAME,
        FlyAnimation::Shooting => SHOOTING_FRAME,
    };
    let source = TextureSource {
        selection: FrameSelection {
            frame_number: clip_frame,
            frames_since_frame_started: u16::from(frame),
        },
        ..framed(clip, clip_frame)
    };
    let flash = usize::from(flashing);
    let overrides = match (kind, animation) {
        (FlyKind::Black, FlyAnimation::Flying) => &WINGS[flash][usize::from(wing.min(1))],
        (FlyKind::Pooter, FlyAnimation::Flying | FlyAnimation::Shooting) | (FlyKind::Red, FlyAnimation::Flying) => {
            &POOTER_VARIANTS[flash][0]
        }
        (FlyKind::SuperPooter, FlyAnimation::Flying | FlyAnimation::Shooting) => &POOTER_VARIANTS[flash][1],
        (FlyKind::Pooter | FlyKind::Red, FlyAnimation::Appearing) => &POOTER_APPEARING[flash][0],
        (FlyKind::SuperPooter, FlyAnimation::Appearing) => &POOTER_APPEARING[flash][1],
        _ if flashing => &HURT_FLASH,
        _ => return source,
    };
    TextureSource { overrides, ..source }
}
