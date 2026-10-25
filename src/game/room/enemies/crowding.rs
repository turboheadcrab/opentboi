//! Enemies pushing each other apart (`ballhit()` over every pair sharing a cell of the original's
//! bucket grid, see `crowding/buckets.rs`): two enemies closer than their sizes added part by the
//! overlap at once, while the speed they trade along the line joining them is gathered over the
//! whole pass and added at its end, from the speeds they had when it began. A dying enemy takes
//! no part; a headless body does not push other enemies (`outway`), and a fly only on every other
//! frame (its `outway` follows `fra2`).

mod buckets;

use glam::Vec2;

use crate::game::ball_contact::{Body, ball_hit};
use crate::game::original_units::{Frames, Mass, Pixels};
use crate::game::room::enemies::walker::WalkerKind;
use crate::game::room::enemies::{Enemies, fly};
use buckets::Buckets;
use std::collections::HashSet;

/// Which enemy of the room a body in the pass is.
#[derive(Clone, Copy, Debug)]
enum Occupant {
    Fly(usize),
    Walker(usize),
}

/// An enemy in the pass: where it is (stage coordinates, moved by the pushes), its speed when the
/// pass began, and the speed it has gathered.
struct Entry {
    occupant: Occupant,
    number: u32,
    start: Vec2,
    position: Vec2,
    velocity: Vec2,
    gathered: Vec2,
    mass: Mass,
    size: Pixels,
}

/// A change in world space (y up) as one in stage space (y down).
fn stage_change(world: Vec2) -> Vec2 {
    Vec2::new(world.x, -world.y)
}

impl Enemies {
    /// Every enemy that takes part in this frame's pass, in the order they were created.
    fn crowd(&self) -> Vec<Entry> {
        let parameters = self.parameters;
        let flies_take_part = self.frames.whole() % 2 == 1;
        let mut entries: Vec<Entry> = Vec::new();
        for (index, fly) in self.flies.iter().enumerate() {
            let appearing = matches!(fly.state, fly::FlyState::Appearing);
            if fly.is_alive() && (appearing || flies_take_part) {
                entries.push(Entry {
                    occupant: Occupant::Fly(index),
                    number: fly.shooter.phase,
                    start: fly.position,
                    position: fly.position,
                    velocity: fly.velocity,
                    gathered: Vec2::ZERO,
                    mass: fly::kind_parameters(fly.kind, &parameters).mass,
                    size: fly::kind_parameters(fly.kind, &parameters).size,
                });
            }
        }
        for (index, walker) in self.walkers.iter().enumerate() {
            if walker.is_alive() && walker.kind != WalkerKind::Headless {
                let body = walker.kind.body(&parameters);
                entries.push(Entry {
                    occupant: Occupant::Walker(index),
                    number: walker.phase(),
                    start: walker.position,
                    position: walker.position,
                    velocity: walker.velocity,
                    gathered: Vec2::ZERO,
                    mass: body.mass,
                    size: body.size,
                });
            }
        }
        entries.sort_by_key(|entry| entry.number);
        entries
    }

    /// Pushes every pair of enemies in a shared cell that overlap apart.
    pub(super) fn separate_enemies(&mut self, elapsed: Frames) {
        let mut entries = self.crowd();
        let mut buckets = Buckets::new();
        for (index, entry) in entries.iter().enumerate() {
            buckets.enter(index, entry.position);
        }
        let mut tested = HashSet::new();
        for (first, second) in buckets.pairs() {
            if !tested.insert((first.max(second), first.min(second))) {
                continue;
            }
            let body = |entry: &Entry| Body {
                position: crate::game::room::stage_to_world(entry.position),
                velocity: Vec2::new(entry.velocity.x, -entry.velocity.y),
                mass: entry.mass,
            };
            let reach = Pixels(entries[first].size.get() + entries[second].size.get());
            let Some(contact) = ball_hit(body(&entries[first]), body(&entries[second]), reach, 1.0, elapsed) else {
                continue;
            };
            entries[first].position += stage_change(contact.first_offset);
            entries[first].gathered += stage_change(contact.first_velocity_change);
            entries[second].position += stage_change(contact.second_offset);
            entries[second].gathered += stage_change(contact.second_velocity_change);
        }
        for entry in entries {
            let moved = entry.position - entry.start;
            match entry.occupant {
                Occupant::Fly(index) => {
                    self.flies[index].position += moved;
                    self.flies[index].velocity += entry.gathered;
                }
                Occupant::Walker(index) => {
                    self.walkers[index].position += moved;
                    self.walkers[index].velocity += entry.gathered;
                }
            }
        }
    }
}
