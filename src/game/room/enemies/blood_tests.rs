//! The blood the enemies leave (`splater()`) and the marks walkers make on the grid.

use super::tests::{FAR_PLAYER, ONE_FRAME, parameters, room_with_flies};
use super::*;
use crate::game::original_units::Pixels;
use crate::game::room::interior_bounds;

/// Marker values: the black fly (6601), the Pooter (6603), the Clotty (6618) and the Gaper (6597).
const FLY_MARKER: f32 = -4.0;
const POOTER_MARKER: f32 = -5.0;
const CLOTTY_MARKER: f32 = -6.0;
const GAPER_MARKER: f32 = -1.0;
/// Past every enemy's appearing clip.
const SETTLING_FRAMES: u32 = 60;

fn advance(enemies: &mut Enemies, random: &mut Random) {
    enemies.advance(ONE_FRAME, &[], FAR_PLAYER, interior_bounds(), random, &mut Vec::new());
    enemies.remove_finished();
}

fn settled(values: &[f32], random: &mut Random) -> Enemies {
    let mut enemies = room_with_flies(values);
    for _ in 0..SETTLING_FRAMES {
        advance(&mut enemies, random);
    }
    enemies.take_splatters();
    enemies
}

fn kill_all(enemies: &mut Enemies, random: &mut Random) {
    enemies.blast(Vec2::ZERO, Pixels(1000.0), |_| true, random, &mut Vec::new());
    assert!(!enemies.any_alive());
}

/// The splats left over the next `frames` frames.
fn splats_over(enemies: &mut Enemies, frames: u32, random: &mut Random) -> Vec<Splatter> {
    (0..frames)
        .flat_map(|_| {
            advance(enemies, random);
            enemies.take_splatters()
        })
        .collect()
}

#[test]
fn a_dying_fly_bleeds_on_its_death_clips_first_three_frames() {
    let mut random = Random::with_seed(4);
    let mut enemies = settled(&[FLY_MARKER], &mut random);
    let position = enemies.flies[0].position;
    kill_all(&mut enemies, &mut random);
    let splats = splats_over(&mut enemies, 10, &mut random);
    assert_eq!(splats.len(), 3);
    assert!(splats.iter().all(|splat| splat.stage_point == position));
    let dying = parameters().splats.dying;
    assert!(
        splats
            .iter()
            .all(|splat| (dying.least..=dying.least + dying.spread).contains(&splat.scale))
    );
}

#[test]
fn a_dying_pooter_bleeds_smaller() {
    let mut random = Random::with_seed(4);
    let mut enemies = settled(&[POOTER_MARKER], &mut random);
    let position = enemies.flies[0].position;
    kill_all(&mut enemies, &mut random);
    // Its shots still in flight drip blood of their own.
    let splats: Vec<Splatter> = splats_over(&mut enemies, 10, &mut random)
        .into_iter()
        .filter(|splat| splat.stage_point == position)
        .collect();
    let splat_parameters = parameters().splats;
    let largest = (splat_parameters.dying.least + splat_parameters.dying.spread) * splat_parameters.pooter_dying_factor;
    assert_eq!(splats.len(), 3);
    assert!(splats.iter().all(|splat| splat.scale <= largest));
}

#[test]
fn a_clotty_bleeds_every_fifth_frame_where_it_stands() {
    let mut random = Random::with_seed(6);
    let mut enemies = settled(&[CLOTTY_MARKER], &mut random);
    let mut own = 0;
    for _ in 0..50 {
        let position = enemies.walkers[0].position;
        advance(&mut enemies, &mut random);
        own += enemies
            .take_splatters()
            .iter()
            .filter(|splat| splat.stage_point == position)
            .count();
    }
    assert_eq!(own, 10);
}

#[test]
fn a_walking_gaper_marks_its_cell_for_the_others_line_checks() {
    let mut random = Random::with_seed(1);
    let enemies = settled(&[GAPER_MARKER], &mut random);
    assert_ne!(enemies.tile_marks, TileMarks::default());
    let flies = settled(&[FLY_MARKER], &mut random);
    assert_eq!(flies.tile_marks, TileMarks::default(), "flies leave no marks");
}
