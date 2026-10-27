use super::*;
use crate::game::parameters::GameParameters;

const SPLATS: SplatParameters = GameParameters::ORIGINAL.splats;

fn splatter(look: Option<u8>) -> Splatter {
    Splatter {
        stage_point: Vec2::new(320.0, 280.0),
        look,
        scale: 1.0,
    }
}

#[test]
fn a_splat_lands_a_few_pixels_off_and_five_up() {
    let mut blood = BloodSplats::default();
    let mut random = Random::with_seed(3);
    for _ in 0..50 {
        blood.splatter(splatter(Some(2)), &SPLATS, &mut random);
    }
    // The stage point (320, 280) is the world's origin; 5 px up on the stage is 5 up in the world.
    let lifted = Vec2::new(0.0, SPLATS.lift.get());
    assert!(
        blood
            .splats
            .iter()
            .all(|splat| splat.center.distance(lifted) <= SPLATS.jitter.get() + 1e-4)
    );
    assert!(blood.splats.iter().any(|splat| splat.upside_down));
    assert!(blood.splats.iter().any(|splat| !splat.upside_down));
}

#[test]
fn no_look_keeps_the_one_drawn_last() {
    let mut blood = BloodSplats::default();
    let mut random = Random::with_seed(1);
    blood.splatter(splatter(None), &SPLATS, &mut random);
    blood.splatter(splatter(Some(6)), &SPLATS, &mut random);
    blood.splatter(splatter(None), &SPLATS, &mut random);
    let looks: Vec<u8> = blood.splats.iter().map(|splat| splat.look).collect();
    assert_eq!(looks, [0, 6, 6], "the clip starts on its first frame");
}

#[test]
fn the_blood_tint_changes_every_seventh_frame_within_its_range() {
    let mut blood = BloodSplats::default();
    let mut random = Random::with_seed(5);
    blood.retint(Frames::from_number(6), &SPLATS, &mut random);
    assert_eq!(blood.tint, Rgba::WHITE);
    blood.retint(Frames::from_number(14), &SPLATS, &mut random);
    let Rgba { red, green, blue, .. } = blood.tint;
    for channel in [red, green, blue] {
        assert!((SPLATS.tint_least..=SPLATS.tint_least + SPLATS.tint_spread).contains(&channel));
    }
    assert_ne!(blood.tint, Rgba::WHITE);
}

#[test]
fn splats_are_drawn_at_the_layers_opacity_and_go_with_the_room() {
    let mut blood = BloodSplats::default();
    let mut random = Random::with_seed(2);
    for _ in 0..10 {
        blood.splatter(splatter(Some(1)), &SPLATS, &mut random);
    }
    let mut items = Vec::new();
    blood.append_draw_items(&SPLATS, &mut items);
    assert_eq!(items.len(), 10);
    // Drawn solid into the splat layer, which shows at 70 %: overlaps do not add up.
    assert!(items.iter().all(|item| item.layer == Some(splat_layer(&SPLATS))));
    assert!(items.iter().all(|item| (item.tint.alpha - 1.0).abs() < f32::EPSILON));
    assert!(
        items
            .iter()
            .all(|item| item.flip_horizontally == (item.rotation_radians != 0.0)),
        "upside down is mirrored and turned"
    );
    blood.clear();
    assert_eq!(blood.count(), 0);
}

#[test]
fn looks_cover_the_red_frames_and_a_zero_keeps_the_last() {
    let mut random = Random::with_seed(9);
    let size = SPLATS.dying;
    let looks: Vec<u8> = (0..400)
        .filter_map(|_| Splatter::any_look(Vec2::ZERO, size, &SPLATS, &mut random).look)
        .collect();
    assert_eq!(looks.len(), 400);
    assert!(looks.iter().all(|&look| look < SPLATS.look_count));
    assert!(looks.contains(&(SPLATS.look_count - 1)));
    let or_last: Vec<Option<u8>> = (0..400)
        .map(|_| Splatter::any_look_or_last(Vec2::ZERO, size, &SPLATS, &mut random).look)
        .collect();
    assert!(or_last.contains(&None));
    assert!(
        !or_last.contains(&Some(SPLATS.look_count - 1)),
        "random(10) never reaches frame 10"
    );
    assert_eq!(SPLATS.look_count, crate::game::texture_id::BLOOD_SPLAT_LOOK_COUNT);
}
