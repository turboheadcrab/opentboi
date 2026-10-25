//! Enemies created from a layout's markers on the room's first visit: 9 minus the marker's value is
//! the type, created just below the cell's center (DoAction 5438–5460). A fraction left in the
//! value (a half-transparent marker's -5.1) picks a variant (`create()`).

use glam::Vec2;

use crate::game::random::Random;
use crate::game::room::enemies::{EnemyParameters, Fly, Walker};
use crate::game::room::layout::LayoutCell;
use crate::game::room::{tile_center, world_to_stage};
use crate::game::texture_id::FlyKind;

/// The type is this minus the marker's value.
const ENEMY_TYPE_BASE: f32 = 9.0;
const GAPER_TYPE: f32 = 10.0;
const HEADLESS_TYPE: f32 = 11.0;
const HORF_TYPE: f32 = 12.0;
const CLOTTY_TYPE: f32 = 15.0;
const MULLIGAN_TYPE: f32 = 16.0;
const HOPPER_TYPE: f32 = 29.0;
const FLESH_MOUND_TYPE: f32 = 77.0;
const BLACK_FLY_TYPE: f32 = 13.0;
const POOTER_TYPE: f32 = 14.0;
const ATTACK_FLY_TYPE: f32 = 18.0;
const RED_FLY_TYPE: f32 = 61.0;
const MOTER_TYPE: f32 = 80.0;
/// Enemies start 10 px below their cell's center.
pub(super) const SPAWN_DROP: f32 = 10.0;

/// An enemy a marker cell created.
pub(super) enum Spawned {
    Fly(Fly),
    Walker(Box<Walker>),
}

/// The enemy a marker cell creates, if it is one the game has; `phase` is its place in the room.
pub(super) fn spawn(
    cell: &LayoutCell,
    phase: u32,
    parameters: &EnemyParameters,
    random: &mut Random,
) -> Option<Spawned> {
    let type_value = ENEMY_TYPE_BASE - cell.value;
    let enemy_type = type_value.round();
    let marked = type_value.fract() != 0.0;
    let stage_point = world_to_stage(tile_center(cell.column, cell.row)) + Vec2::new(0.0, SPAWN_DROP);
    let kind = match enemy_type {
        GAPER_TYPE => {
            let gaper = Walker::gaper(stage_point, !marked, phase, parameters, random);
            return Some(Spawned::Walker(Box::new(gaper)));
        }
        HEADLESS_TYPE => {
            let body = Walker::headless(stage_point, !marked, phase, parameters);
            return Some(Spawned::Walker(Box::new(body)));
        }
        HORF_TYPE => return Some(Spawned::Walker(Box::new(Walker::horf(stage_point, phase, parameters)))),
        CLOTTY_TYPE => {
            return Some(Spawned::Walker(Box::new(Walker::clotty(
                stage_point,
                phase,
                parameters,
            ))));
        }
        MULLIGAN_TYPE => {
            return Some(Spawned::Walker(Box::new(Walker::mulligan(
                stage_point,
                phase,
                parameters,
            ))));
        }
        HOPPER_TYPE => {
            return Some(Spawned::Walker(Box::new(Walker::hopper(
                stage_point,
                phase,
                parameters,
            ))));
        }
        FLESH_MOUND_TYPE => {
            return Some(Spawned::Walker(Box::new(Walker::flesh_mound(
                stage_point,
                phase,
                parameters,
            ))));
        }
        BLACK_FLY_TYPE => FlyKind::Black,
        ATTACK_FLY_TYPE => FlyKind::Attack,
        RED_FLY_TYPE => FlyKind::Red,
        MOTER_TYPE => FlyKind::Moter,
        POOTER_TYPE => pooter_variant(marked, parameters, random),
        _ => return None,
    };
    Some(Spawned::Fly(Fly::new(kind, stage_point, phase, parameters, random)))
}

/// The two-shot Pooter: a marked variant, or one of three rolls (1 in 21, 25, 100).
fn pooter_variant(marked: bool, parameters: &EnemyParameters, random: &mut Random) -> FlyKind {
    let odds = parameters.pooters.variant_odds;
    if marked || odds.iter().any(|&each| random.below(each) == 0) {
        FlyKind::SuperPooter
    } else {
        FlyKind::Pooter
    }
}
