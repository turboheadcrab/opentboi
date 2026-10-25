//! `ballhit()`: two round things that overlap move apart by the overlap and trade momentum, each
//! by the other's share of the mass.

use crate::game::original_units::{Frames, Mass, Pixels};
use glam::Vec2;

#[derive(Clone, Copy, Debug)]
pub struct Body {
    pub position: Vec2,
    /// Pixels per original frame.
    pub velocity: Vec2,
    /// `masses[s]`.
    pub mass: Mass,
}

/// How each body changes; `None` from [`ball_hit`] when they do not touch.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Contact {
    pub first_offset: Vec2,
    pub first_velocity_change: Vec2,
    pub second_offset: Vec2,
    pub second_velocity_change: Vec2,
}

/// Bodies closer than `reach` (their two sizes) part by the overlap times `overlap_scale` (half for
/// bombs), and each gets the relative speed along the line between them, by the other's share of
/// the mass. Exactly on top of each other nothing happens.
pub fn ball_hit(first: Body, second: Body, reach: Pixels, overlap_scale: f32, elapsed: Frames) -> Option<Contact> {
    let apart = second.position - first.position;
    let distance = apart.length();
    let reach = reach.get();
    if distance <= 0.0 || distance >= reach {
        return None;
    }
    let first_share = first.mass / (first.mass + second.mass);
    let second_share = 1.0 - first_share;
    let overlap = apart * ((reach - distance) / distance * overlap_scale);
    let relative_speed = (second.velocity - first.velocity).length();
    let impulse = apart * (relative_speed / distance) * elapsed.get();
    Some(Contact {
        first_offset: -overlap * second_share,
        first_velocity_change: -impulse * second_share,
        second_offset: overlap * first_share,
        second_velocity_change: impulse * first_share,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walking_into_something_light_kicks_it_away() {
        let player = Body {
            position: Vec2::ZERO,
            velocity: Vec2::new(3.0, 0.0),
            mass: Mass(5.0),
        };
        let heart = Body {
            position: Vec2::new(20.0, 0.0),
            velocity: Vec2::ZERO,
            mass: Mass(3.0),
        };
        let contact = ball_hit(player, heart, Pixels(22.0), 1.0, Frames(1.0)).expect("they touch");
        assert!(contact.second_velocity_change.x > contact.first_velocity_change.x.abs());
        assert!(contact.first_offset.x < 0.0 && contact.second_offset.x > 0.0);
        assert!(
            ball_hit(
                player,
                Body {
                    position: Vec2::ZERO,
                    ..heart
                },
                Pixels(22.0),
                1.0,
                Frames(1.0)
            )
            .is_none()
        );
    }
}
