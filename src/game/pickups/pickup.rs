//! One pickup lying in a room: it lands when it appears, slides to a stop if it was thrown out of
//! a chest, and plays its pickup (or opening) animation once taken.

use glam::Vec2;

use crate::game::bounds::Bounds;
use crate::game::draw::{DrawItem, Rgba, actor_depth};
use crate::game::original_units::{Frames, Pixels, Retention, distance};
use crate::game::parameters::WallParameters;
use crate::game::pickups::kind::{BROWN_CHEST_FRAME, PickupKind};
use crate::game::pickups::phase::PickupPhase;
use crate::game::texture_id::TextureId;
use crate::game::wall_contact::rebound;

const FALLBACK_COLOR: Rgba = Rgba::opaque(0.9, 0.8, 0.2);
/// `aicol()`: a pickup keeps 96.5 % of its speed each frame, and the heavy ones 80 % of that again:
/// those drawn from the pickup clip's chest frame on, chests, cards and pedestals (the original
/// means to spare cards, but its check misspells the frame and so spares nothing).
const VELOCITY_RETENTION: Retention = Retention(0.965);
const HEAVY_VELOCITY_RETENTION: Retention = Retention(0.8);
const FIRST_HEAVY_FRAME: u8 = BROWN_CHEST_FRAME;
/// A pickup's size: how far it keeps from walls and obstacles.
const RADIUS: Pixels = Pixels(12.0);
/// A placed item pedestal clings to where it stands: each frame it keeps only 30 % of how far it was
/// moved, the spot follows only onto free floor, and a jump of 30 or more snaps it back.
const PEDESTAL_HOLD: Retention = Retention(0.7);
const PEDESTAL_SNAP_REACH: Pixels = Pixels(30.0);

#[derive(Clone, Debug)]
pub struct Pickup {
    pub kind: PickupKind,
    pub center: Vec2,
    /// Where it was when the current tick began.
    pub previous_center: Vec2,
    velocity: Vec2,
    pub age_frames: Frames,
    /// Time since it was taken (or a chest opened), while that animation plays.
    pub collected_frames: Option<Frames>,
    /// Where a placed pedestal stands (`xpp`); pedestals thrown out of chests (`alt`) move freely.
    anchor: Option<Vec2>,
    /// A troll bomb has shown its message.
    pub troll_announced: bool,
    /// A card or active item handed back by a swap cannot be taken while this runs, and touching it
    /// starts it again (`balljunk()` case 7; for items, a stand-in for the pedestal's swap).
    pub swap_blocked_frames: Frames,
    /// Age when its landing animation last started (it can replay).
    landing_started_at: Frames,
}

impl Pickup {
    /// Kept `RADIUS` inside the room, wherever it was asked to appear.
    pub fn new(kind: PickupKind, center: Vec2, velocity: Vec2) -> Self {
        let center = crate::game::room::interior_bounds().shrunk_by(RADIUS).clamp(center);
        Self {
            kind,
            center,
            previous_center: center,
            velocity,
            age_frames: Frames::ZERO,
            collected_frames: None,
            anchor: None,
            troll_announced: false,
            swap_blocked_frames: Frames::ZERO,
            landing_started_at: Frames::ZERO,
        }
    }

    /// A pickup placed by the room's layout: an item pedestal clings to its spot.
    pub fn placed(kind: PickupKind, center: Vec2) -> Self {
        let pickup = Self::new(kind, center, Vec2::ZERO);
        Self {
            anchor: kind.is_pedestal().then_some(pickup.center),
            ..pickup
        }
    }

    /// Ages and moves it; `false` once it is gone (taken and its animation over). An opened chest
    /// stays.
    pub fn advance(
        &mut self,
        elapsed: Frames,
        room_interior: Bounds,
        solid_tiles: &[Bounds],
        walls: WallParameters,
    ) -> bool {
        self.age_frames += elapsed;
        self.swap_blocked_frames = (self.swap_blocked_frames - elapsed).max(Frames::ZERO);
        self.velocity *= VELOCITY_RETENTION.over(elapsed);
        if self.kind.clip_frame() >= FIRST_HEAVY_FRAME {
            self.velocity *= HEAVY_VELOCITY_RETENTION.over(elapsed);
        }
        let target = self.center + self.velocity * elapsed;
        self.move_to(target, room_interior, solid_tiles);
        let blocked = target - self.center;
        self.velocity = rebound(self.velocity, blocked, walls);
        self.cling_to_anchor(elapsed, solid_tiles);
        let Some(collected_frames) = &mut self.collected_frames else {
            return true;
        };
        *collected_frames += elapsed;
        self.kind.is_chest() || *collected_frames < Frames::from(self.kind.collected_frame_count())
    }

    fn cling_to_anchor(&mut self, elapsed: Frames, solid_tiles: &[Bounds]) {
        let Some(anchor) = &mut self.anchor else { return };
        if distance(self.center, *anchor) < PEDESTAL_SNAP_REACH {
            let kept = anchor.lerp(self.center, 1.0 - PEDESTAL_HOLD.over(elapsed));
            if !solid_tiles.iter().any(|tile| tile.contains(kept)) {
                *anchor = kept;
            }
        }
        self.center = *anchor;
    }

    /// `pillc()` handing the held card or pill back (DoAction 7139–7152): this pickup becomes it
    /// (DoAction 7143), cannot be taken for a while, lands again (DoAction 7148) and is tossed up by
    /// half of how far the player stands above it (DoAction 7151).
    pub fn hand_back(&mut self, held: PickupKind, player_position: Vec2, block: Frames) {
        const TOSS_PER_PIXEL_ABOVE: f32 = 0.5;
        self.kind = held;
        self.swap_blocked_frames = block;
        self.landing_started_at = self.age_frames;
        self.velocity.y += (player_position.y - self.center.y).max(0.0) * TOSS_PER_PIXEL_ABOVE;
    }

    /// A blast's shove: keeps `retention` of the velocity and adds `push`.
    pub fn knock_back(&mut self, retention: Retention, push: Vec2) {
        self.velocity = self.velocity * retention.0 + push;
    }

    pub fn velocity(&self) -> Vec2 {
        self.velocity
    }

    pub fn slow_down(&mut self, retention: f32) {
        self.velocity *= retention;
    }

    pub fn accelerate(&mut self, change: Vec2) {
        self.velocity += change;
    }

    /// Moves toward `target`, kept `RADIUS` inside the room and out of solid tiles; returns how
    /// far it really moved.
    pub fn move_to(&mut self, target: Vec2, room_interior: Bounds, solid_tiles: &[Bounds]) -> Vec2 {
        let mut next = room_interior.shrunk_by(RADIUS).clamp(target);
        for tile in solid_tiles {
            next = tile.push_circle_out(next, RADIUS);
        }
        let moved = next - self.center;
        self.center = next;
        moved
    }

    pub fn is_lying(&self) -> bool {
        self.collected_frames.is_none()
    }

    pub fn draw_item(&self) -> DrawItem {
        let phase = self.phase();
        DrawItem {
            motion: self.center - self.previous_center,
            ..DrawItem::textured(
                TextureId::Pickup {
                    kind: self.kind.art(),
                    phase,
                    frame: self.frame(),
                },
                self.center + Vec2::new(0.0, phase.art_lift().get()),
                1.0,
                FALLBACK_COLOR,
                actor_depth(self.center.y),
            )
        }
    }

    fn phase(&self) -> PickupPhase {
        if self.collected_frames.is_some() {
            PickupPhase::Collected
        } else if self.age_frames - self.landing_started_at < Frames::from(self.kind.landing_frame_count()) {
            PickupPhase::Landing
        } else {
            PickupPhase::Idle
        }
    }

    fn frame(&self) -> u8 {
        let phase = self.phase();
        let frame_count = phase.frame_count(self.kind).max(1);
        let elapsed = match phase {
            PickupPhase::Collected => self.collected_frames.unwrap_or_default(),
            PickupPhase::Landing => self.age_frames - self.landing_started_at,
            PickupPhase::Idle => {
                self.age_frames - self.landing_started_at - Frames::from(self.kind.landing_frame_count())
            }
        };
        // An opened chest stays open on the animation's last frame; the rest loop.
        if phase == PickupPhase::Collected {
            elapsed.held_frame(frame_count)
        } else {
            elapsed.looped_frame(frame_count)
        }
    }
}
