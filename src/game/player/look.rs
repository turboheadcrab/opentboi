//! Chooses Isaac's body and head frames the way the original's player update does.

use glam::Vec2;

use crate::game::draw::{ACTOR_PART_DEPTH_STEP, DrawItem, Rgba, actor_depth};
use crate::game::original_units::Frames;
use crate::game::parameters::{FiringParameters, MovementParameters};
use crate::game::texture_id::{
    ISAAC_HORIZONTAL_WALK_FRAME_COUNT, ISAAC_VERTICAL_WALK_FRAME_COUNT, IsaacPart, RideColor, TextureId,
};

/// Texture centers relative to the floor point, y up. The body texture is centered on the body
/// clip's mirror axis (x = 0) so flipping it matches the original's mirrored clip.
const BODY_CENTER_OFFSET: Vec2 = Vec2::new(0.0, 8.0);
const HEAD_CENTER_OFFSET: Vec2 = Vec2::new(0.0, 24.0);
/// Head clip frames 1–4 face down, right, up, left; frames 5–8 are the same with the shooting face.
const HEAD_FRAME_DOWN: u8 = 0;
const HEAD_FRAME_RIGHT: u8 = 1;
const HEAD_FRAME_UP: u8 = 2;
const HEAD_FRAME_LEFT: u8 = 3;
const HEAD_SHOOTING_FRAME_OFFSET: u8 = 4;
const SKIN_FALLBACK_COLOR: Rgba = Rgba::opaque(0.96, 0.82, 0.76);

/// What Isaac looks like apart from his motion.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Outfit {
    /// Flying hides his legs.
    pub flying: bool,
    /// On the unicorn, with its head (the hats would be hidden too).
    pub riding_unicorn: bool,
    /// The color he flashes on the unicorn.
    pub tint: Option<RideColor>,
    /// The demonic look (the Empress); the unicorn ride keeps his plain body.
    pub demonic: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BodyMotion {
    Standing,
    WalkingVertically,
    WalkingHorizontally,
}

#[derive(Clone, Debug)]
pub struct PlayerLook {
    movement: MovementParameters,
    firing: FiringParameters,
    body_motion: BodyMotion,
    /// Time in the current walk cycle; restarts when the walk direction changes.
    walk_cycle_frames: Frames,
    /// The body keeps its last horizontal facing, also while walking vertically or standing.
    body_faces_left: bool,
    last_shot_direction: Vec2,
    frames_since_last_shot: Option<Frames>,
}

impl PlayerLook {
    /// The firing rules change with the fire rate.
    pub fn set_firing(&mut self, firing: FiringParameters) {
        self.firing = firing;
    }

    pub fn new(movement: MovementParameters, firing: FiringParameters) -> Self {
        Self {
            movement,
            firing,
            body_motion: BodyMotion::Standing,
            walk_cycle_frames: Frames::ZERO,
            body_faces_left: false,
            last_shot_direction: Vec2::NEG_Y,
            frames_since_last_shot: None,
        }
    }

    pub fn update(&mut self, velocity: Vec2, fired_direction: Option<Vec2>, elapsed: Frames) {
        let walking_minimum_speed = self.movement.walking_minimum_speed_per_frame;
        let motion = if velocity.length() < walking_minimum_speed.get() {
            BodyMotion::Standing
        } else if velocity.x.abs() > velocity.y.abs() {
            self.body_faces_left = velocity.x < 0.0;
            BodyMotion::WalkingHorizontally
        } else {
            BodyMotion::WalkingVertically
        };
        if motion == self.body_motion {
            self.walk_cycle_frames += elapsed;
        } else {
            self.body_motion = motion;
            self.walk_cycle_frames = Frames::ZERO;
        }

        if let Some(frames) = &mut self.frames_since_last_shot {
            *frames += elapsed;
        }
        if let Some(direction) = fired_direction {
            self.last_shot_direction = direction;
            self.frames_since_last_shot = Some(Frames::ZERO);
        }
    }

    fn body_part(&self) -> IsaacPart {
        let cycle = self.walk_cycle_frames;
        match self.body_motion {
            BodyMotion::Standing => IsaacPart::BodyStanding,
            BodyMotion::WalkingVertically => {
                IsaacPart::BodyWalkVertical(cycle.looped_frame(ISAAC_VERTICAL_WALK_FRAME_COUNT))
            }
            BodyMotion::WalkingHorizontally => {
                IsaacPart::BodyWalkHorizontal(cycle.looped_frame(ISAAC_HORIZONTAL_WALK_FRAME_COUNT))
            }
        }
    }

    /// Faces the last shot while the face timer is non-negative, squinting while it is above the
    /// shooting threshold, and looks down otherwise. Walking does not turn the head. The unicorn's
    /// head turns the same way but never shoots.
    fn head_part(&self, riding_unicorn: bool) -> IsaacPart {
        let head: fn(u8) -> IsaacPart = if riding_unicorn {
            IsaacPart::UnicornHead
        } else {
            IsaacPart::Head
        };
        let Some(frames) = self.frames_since_last_shot else {
            return head(HEAD_FRAME_DOWN);
        };
        let face_timer = self.firing.face_timer_after_shot() - frames.floor();
        if face_timer < Frames::ZERO {
            return head(HEAD_FRAME_DOWN);
        }
        let direction_frame = head_frame_for(self.last_shot_direction);
        if face_timer > self.firing.shooting_face_above && !riding_unicorn {
            head(direction_frame + HEAD_SHOOTING_FRAME_OFFSET)
        } else {
            head(direction_frame)
        }
    }

    /// `scale` is Isaac's size (`player._xscale`), around his floor point.
    pub fn append_draw_items(&self, floor_point: Vec2, scale: f32, outfit: Outfit, draw_items: &mut Vec<DrawItem>) {
        let demonic = outfit.demonic && !outfit.riding_unicorn;
        let texture = |part: IsaacPart| TextureId::Isaac {
            part: if demonic { part.demonic() } else { part },
            tint: outfit.tint,
        };
        let body_depth = actor_depth(floor_point.y);
        draw_items.push(DrawItem {
            flip_horizontally: self.body_faces_left,
            ..DrawItem::textured(
                texture(if outfit.flying {
                    IsaacPart::FlyingBody
                } else {
                    self.body_part()
                }),
                floor_point + BODY_CENTER_OFFSET * scale,
                scale,
                SKIN_FALLBACK_COLOR,
                body_depth,
            )
        });
        draw_items.push(DrawItem::textured(
            texture(self.head_part(outfit.riding_unicorn)),
            floor_point + HEAD_CENTER_OFFSET * scale,
            scale,
            SKIN_FALLBACK_COLOR,
            body_depth + ACTOR_PART_DEPTH_STEP,
        ));
    }
}

/// Cardinal direction (y up) to head frame.
fn head_frame_for(direction: Vec2) -> u8 {
    if direction.x.abs() > direction.y.abs() {
        if direction.x > 0.0 {
            HEAD_FRAME_RIGHT
        } else {
            HEAD_FRAME_LEFT
        }
    } else if direction.y > 0.0 {
        HEAD_FRAME_UP
    } else {
        HEAD_FRAME_DOWN
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::parameters::GameParameters;

    fn look() -> PlayerLook {
        PlayerLook::new(GameParameters::ORIGINAL.movement, GameParameters::ORIGINAL.firing)
    }

    #[test]
    fn head_squints_for_four_frames_then_faces_the_shot_then_looks_down() {
        let mut look = look();
        look.update(Vec2::ZERO, Some(Vec2::X), Frames(0.0));
        assert_eq!(
            look.head_part(false),
            IsaacPart::Head(HEAD_FRAME_RIGHT + HEAD_SHOOTING_FRAME_OFFSET)
        );
        assert_eq!(look.head_part(true), IsaacPart::UnicornHead(HEAD_FRAME_RIGHT));
        look.update(Vec2::ZERO, None, Frames(4.0));
        assert_eq!(look.head_part(false), IsaacPart::Head(HEAD_FRAME_RIGHT));
        look.update(Vec2::ZERO, None, Frames(8.0));
        assert_eq!(look.head_part(false), IsaacPart::Head(HEAD_FRAME_DOWN));
    }

    #[test]
    fn body_walks_sideways_mirrored_when_moving_left() {
        let mut look = look();
        look.update(Vec2::new(-6.0, 0.6), None, Frames(3.0));
        assert!(matches!(look.body_part(), IsaacPart::BodyWalkHorizontal(_)));
        assert!(look.body_faces_left);
        look.update(Vec2::new(0.0, 0.3), None, Frames(3.0));
        assert_eq!(look.body_part(), IsaacPart::BodyStanding);
    }
}
