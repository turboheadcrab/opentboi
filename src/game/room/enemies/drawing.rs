//! How enemies look: the flies' clips (`b13`, `b18`, `b14`), `d` frame 4 appearing, 1 flying (the
//! black fly's 16-frame bob with the 2-frame wings started on a random frame, the attack fly's
//! 4-frame flap, the Pooter's 2-frame flap), 3 dying, 5 a Pooter shooting (mirrored to face a
//! player on its right, `sideflip()`). Shots are drawn like tears: a shadow on the floor and the
//! ball raised by its height, pulsing in brightness every three frames.

use crate::game::draw::{ACTOR_PART_DEPTH_STEP, DrawItem, Rgba, TEAR_DEPTH, TEAR_SHADOW_DEPTH, actor_depth};
use crate::game::original_units::Frames;
use crate::game::room::enemies::blob_drawing::{clotty_texture, mulligan_texture};
use crate::game::room::enemies::ground_drawing::{flesh_mound_texture, hopper_texture};
use crate::game::room::enemies::{
    Enemies, EnemyParameters, Fly, FlyState, Walker, WalkerClip, WalkerKind, WalkerState, fly::kind_parameters,
};
use crate::game::room::stage_to_world;
use crate::game::texture_id::{
    ENEMY_SHOT_BRIGHTNESS_STEPS, FlyAnimation, GaperAnimation, HeadlessAnimation, HorfAnimation, TextureId,
};

const FALLBACK_COLOR: Rgba = Rgba::opaque(0.1, 0.1, 0.1);
const SHOT_FALLBACK_COLOR: Rgba = Rgba::opaque(0.6, 0.05, 0.05);
const SHADOW_FALLBACK_COLOR: Rgba = Rgba {
    red: 0.0,
    green: 0.0,
    blue: 0.0,
    alpha: 0.3,
};

impl Enemies {
    pub fn append_draw_items(&self, draw_items: &mut Vec<DrawItem>) {
        draw_items.extend(self.flies.iter().map(|fly| {
            let center = stage_to_world(fly.position);
            DrawItem {
                motion: center - stage_to_world(fly.previous_position),
                flip_horizontally: fly.kind.is_pooter() && fly.shooter.faces_right,
                tint: self.poison_tint(fly.shooter.phase).unwrap_or(Rgba::WHITE),
                ..DrawItem::textured(texture(fly, self), center, 1.0, FALLBACK_COLOR, actor_depth(center.y))
            }
        }));
        for walker in &self.walkers {
            let first_item = draw_items.len();
            walker_draw_items(walker, &self.parameters, draw_items);
            if let Some(tint) = self.poison_tint(walker.phase()) {
                for item in &mut draw_items[first_item..] {
                    item.tint = tint;
                }
            }
        }
        let brightness = self.frames.looped_frame(ENEMY_SHOT_BRIGHTNESS_STEPS);
        for shot in &self.shots {
            let floor_point = stage_to_world(shot.position);
            let drawn = stage_to_world(shot.drawn_point());
            draw_items.push(DrawItem {
                motion: floor_point - stage_to_world(shot.previous_position),
                ..DrawItem::textured(
                    TextureId::TearShadow,
                    floor_point,
                    1.0,
                    SHADOW_FALLBACK_COLOR,
                    TEAR_SHADOW_DEPTH,
                )
            });
            draw_items.push(DrawItem {
                motion: drawn - stage_to_world(shot.previous_drawn_point()),
                ..DrawItem::textured(
                    TextureId::EnemyShot(brightness),
                    drawn,
                    1.0,
                    SHOT_FALLBACK_COLOR,
                    TEAR_DEPTH,
                )
            });
        }
        draw_items.extend(
            self.splashes
                .iter()
                .map(crate::game::attacks::splash::Splash::draw_item),
        );
    }
}

fn walker_draw_items(walker: &Walker, parameters: &EnemyParameters, draw_items: &mut Vec<DrawItem>) {
    let center = stage_to_world(walker.position);
    let item = |texture, depth| DrawItem {
        motion: center - stage_to_world(walker.previous_position),
        flip_horizontally: walker.mirrored,
        ..DrawItem::textured(texture, center, walker.scale, FALLBACK_COLOR, depth)
    };
    draw_items.push(item(walker_texture(walker, parameters), actor_depth(center.y)));
    // A walking Gaper's head is its own clip, left where it was when the body began to walk.
    if walker.kind == WalkerKind::Gaper
        && walker.state == WalkerState::Active
        && matches!(walker.clip, WalkerClip::Walking { .. })
    {
        let head = TextureId::Gaper {
            animation: GaperAnimation::Head,
            frame: walker.head,
            flashing: walker.is_flashing(),
        };
        draw_items.push(item(head, actor_depth(center.y) + ACTOR_PART_DEPTH_STEP));
    }
}

fn texture(fly: &Fly, enemies: &Enemies) -> TextureId {
    let (animation, frames) = match (fly.state, fly.shooter.animation_age) {
        (FlyState::Appearing, _) => (FlyAnimation::Appearing, fly.age),
        (FlyState::Flying, Some(shooting)) => (FlyAnimation::Shooting, shooting),
        (FlyState::Flying, None) => (
            FlyAnimation::Flying,
            fly.age - kind_parameters(fly.kind, &enemies.parameters).appearing_frames,
        ),
        (FlyState::Dying(frames), _) => (FlyAnimation::Dying, frames),
    };
    let count = fly.kind.frame_count(animation);
    // The flight loops; the others hold their last frame.
    let frame = if animation == FlyAnimation::Flying {
        frames.looped_frame(count)
    } else {
        frames.held_frame(count)
    };
    let wing = (fly.age + Frames::from(fly.wing_phase)).looped_frame(fly.kind.wing_frame_count());
    TextureId::Fly {
        kind: fly.kind,
        animation,
        frame,
        wing,
        flashing: fly.is_flashing(),
    }
}

fn walker_texture(walker: &Walker, parameters: &EnemyParameters) -> TextureId {
    match walker.kind {
        WalkerKind::Gaper => gaper_texture(walker),
        WalkerKind::Headless => headless_texture(walker),
        WalkerKind::Horf => horf_texture(walker),
        WalkerKind::Clotty => clotty_texture(walker),
        WalkerKind::Mulligan => mulligan_texture(walker),
        WalkerKind::Hopper => hopper_texture(walker),
        WalkerKind::FleshMound => flesh_mound_texture(walker, parameters.flesh_mounds.appearing_frames),
    }
}

fn gaper_texture(walker: &Walker) -> TextureId {
    let (animation, frame) = match (walker.state, walker.clip) {
        (WalkerState::Appearing, _) => (
            GaperAnimation::Appearing {
                frantic: walker.frantic,
            },
            walker
                .age
                .held_frame(GaperAnimation::Appearing { frantic: false }.frame_count()),
        ),
        (WalkerState::Active, WalkerClip::Standing) => (GaperAnimation::Standing, walker.head),
        (WalkerState::Active, WalkerClip::Shooting { .. }) => (GaperAnimation::Standing, 0),
        (WalkerState::Active, WalkerClip::Walking { since }) => (
            GaperAnimation::Walking,
            since.looped_frame(GaperAnimation::Walking.frame_count()),
        ),
        (WalkerState::Dying(frames), _) => (
            GaperAnimation::Dying,
            frames.held_frame(GaperAnimation::Dying.frame_count()),
        ),
    };
    TextureId::Gaper {
        animation,
        frame,
        flashing: walker.is_flashing(),
    }
}

fn headless_texture(walker: &Walker) -> TextureId {
    let spurting = walker.spurts;
    let (animation, frames) = match (walker.state, walker.clip) {
        (WalkerState::Appearing, _) => (HeadlessAnimation::Appearing { spurting }, walker.age),
        (WalkerState::Active, WalkerClip::Standing | WalkerClip::Shooting { .. }) => {
            (HeadlessAnimation::Standing { spurting }, walker.clip_age)
        }
        (WalkerState::Active, WalkerClip::Walking { .. }) => (HeadlessAnimation::Walking, walker.clip_age),
        (WalkerState::Dying(frames), _) => (HeadlessAnimation::Dying, frames),
    };
    let count = animation.frame_count();
    let frame = if matches!(
        animation,
        HeadlessAnimation::Appearing { .. } | HeadlessAnimation::Dying
    ) {
        frames.held_frame(count)
    } else {
        frames.looped_frame(count)
    };
    TextureId::Headless {
        animation,
        frame,
        flashing: walker.is_flashing(),
    }
}

fn horf_texture(walker: &Walker) -> TextureId {
    let (animation, frames) = match (walker.state, walker.shooter.animation_age) {
        (WalkerState::Appearing, _) => (HorfAnimation::Appearing, walker.age),
        (WalkerState::Active, Some(shooting)) => (HorfAnimation::Shooting, shooting),
        (WalkerState::Active, None) => (HorfAnimation::Idle, walker.age),
        (WalkerState::Dying(frames), _) => (HorfAnimation::Dying, frames),
    };
    let count = animation.frame_count();
    let frame = if animation == HorfAnimation::Idle {
        frames.looped_frame(count)
    } else {
        frames.held_frame(count)
    };
    TextureId::Horf {
        animation,
        frame,
        flashing: walker.is_flashing(),
    }
}
