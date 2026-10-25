//! Player tears, following the original's `plff` (firing) and `create` (tear type 2) code.

use glam::Vec2;

use crate::game::bounds::Bounds;
use crate::game::draw::{DrawItem, Rgba, TEAR_DEPTH, TEAR_SHADOW_DEPTH};
use crate::game::original_units::{Damage, Frames, Pixels, PixelsPerFrame, length};
use crate::game::parameters::TearParameters;
use crate::game::random::Random;
use crate::game::room::stage_area::leaves_flight_area;
use crate::game::room::world_to_stage;
use crate::game::texture_id::TextureId;

const TEAR_FALLBACK_COLOR: Rgba = Rgba::opaque(0.55, 0.75, 1.0);
const SHADOW_FALLBACK_COLOR: Rgba = Rgba {
    red: 0.0,
    green: 0.0,
    blue: 0.0,
    alpha: 0.3,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TearStatus {
    Flying,
    HitWall,
    /// Index into the solid tiles passed to [`Tear::status`].
    HitSolidTile(usize),
    Landed,
}

#[derive(Clone, Debug)]
pub struct Tear {
    parameters: TearParameters,
    /// Point on the floor under the tear.
    position: Vec2,
    /// Its floor point and drawn center when the current tick began.
    previous_position: Vec2,
    previous_drawn_center: Vec2,
    velocity: Vec2,
    initial_fall_speed: PixelsPerFrame,
    age_frames: Frames,
    scale: f32,
    slowed_by_web: bool,
    /// The dark tear picture instead of the plain one.
    dark: bool,
}

impl Tear {
    /// `side` is +1 or -1 and alternates between shots. The aim speed is also the minimum speed
    /// after adding the shooter's momentum.
    pub fn fire(
        origin: Vec2,
        direction: Vec2,
        shooter_velocity: Vec2,
        side: f32,
        parameters: TearParameters,
        random: &mut Random,
    ) -> Self {
        let speed_per_frame = parameters.speed_per_frame;
        let mut velocity_per_frame =
            direction * speed_per_frame + shooter_velocity * parameters.inherited_velocity_ratio;
        if length(velocity_per_frame) < Pixels(speed_per_frame.get()) {
            velocity_per_frame = velocity_per_frame.normalize_or(direction) * speed_per_frame;
        }
        velocity_per_frame *= parameters.speed_multiplier;
        let side_offset_ratio =
            parameters.side_offset_minimum + random.next_unit() * parameters.side_offset_random_range;
        let side_offset = velocity_per_frame.perp() * side * side_offset_ratio;
        let position = origin + side_offset;
        let initial_fall_speed = parameters.initial_fall_speed_spread * random.centered() - parameters.initial_rise;
        Self {
            parameters,
            position,
            previous_position: position,
            previous_drawn_center: position + Vec2::new(0.0, parameters.start_height.get()),
            velocity: velocity_per_frame,
            initial_fall_speed,
            age_frames: Frames::ZERO,
            scale: parameters.scale_for_damage(parameters.base_damage),
            slowed_by_web: false,
            dark: false,
        }
    }

    /// Draws it as the dark tear the devil's boost gives (`b500`).
    #[must_use]
    pub fn with_dark_look(mut self, dark: bool) -> Self {
        self.dark = dark;
        self
    }

    /// Slows the tear the first time it is over a spider web; later webs do not slow it again.
    pub fn slow_down_if_over(&mut self, webs: &[Bounds]) {
        if !self.slowed_by_web && webs.iter().any(|web| web.contains(self.position)) {
            self.slowed_by_web = true;
            self.velocity *= self.parameters.web_slowdown;
        }
    }

    pub fn velocity(&self) -> Vec2 {
        self.velocity
    }

    pub fn advance(&mut self, elapsed: Frames) {
        self.previous_position = self.position;
        self.previous_drawn_center = self.drawn_center();
        self.position += self.velocity * elapsed;
        self.age_frames += elapsed;
    }

    pub fn status(&self, solid_tiles: &[Bounds]) -> TearStatus {
        if leaves_flight_area(world_to_stage(self.position)) {
            TearStatus::HitWall
        } else if let Some(tile_index) = solid_tiles.iter().position(|tile| tile.contains(self.position)) {
            TearStatus::HitSolidTile(tile_index)
        } else if self.height() <= self.parameters.landing_height {
            TearStatus::Landed
        } else {
            TearStatus::Flying
        }
    }

    /// `dmg`: what it does to whatever it hits.
    pub fn damage(&self) -> Damage {
        self.parameters.base_damage
    }

    /// The point on the floor under it, where it touches things (`xp`, `yp`).
    pub fn floor_point(&self) -> Vec2 {
        self.position
    }

    /// Whether it was fired this very frame (it touches nothing yet).
    pub fn is_new(&self) -> bool {
        self.age_frames < Frames(1.0)
    }

    /// Where the tear body is drawn: its floor point raised by its height.
    pub fn drawn_center(&self) -> Vec2 {
        self.position + Vec2::new(0.0, self.height().get())
    }

    pub fn scale(&self) -> f32 {
        self.scale
    }

    /// Closed form of the original per-frame fall, valid between frames too.
    fn height(&self) -> Pixels {
        let frames = self.age_frames.get();
        let retention = self.parameters.fall_speed_retention.0;
        let terminal_fall_speed = self.parameters.fall_acceleration.get() / (1.0 - retention);
        let decay = retention.powf(frames);
        let fallen = frames * terminal_fall_speed
            + (self.initial_fall_speed.get() - terminal_fall_speed) * retention * (1.0 - decay) / (1.0 - retention);
        self.parameters.start_height - Pixels(fallen)
    }

    pub fn append_draw_items(&self, draw_items: &mut Vec<DrawItem>) {
        draw_items.push(DrawItem {
            motion: self.position - self.previous_position,
            ..DrawItem::textured(
                TextureId::TearShadow,
                self.position,
                self.scale,
                SHADOW_FALLBACK_COLOR,
                TEAR_SHADOW_DEPTH,
            )
        });
        let (texture, body_scale) = if self.dark {
            (TextureId::DarkTear, self.scale * self.parameters.dark_body_scale)
        } else {
            (TextureId::Tear, self.scale)
        };
        draw_items.push(DrawItem {
            motion: self.drawn_center() - self.previous_drawn_center,
            ..DrawItem::textured(
                texture,
                self.drawn_center(),
                body_scale,
                TEAR_FALLBACK_COLOR,
                TEAR_DEPTH,
            )
        });
    }
}

#[cfg(test)]
mod tests;
