use super::*;
use crate::game::kind_for_value;
use crate::game_assets::{EXTRACTED_SWF_FILE_NAME, cache_directory};

/// Pickup markers report `-n / 100`.
const PICKUP_VALUES: std::ops::Range<f32> = -1.0..0.0;

#[test]
#[allow(
    clippy::float_cmp,
    reason = "unchanged values pass through and a rock becomes the block's exact value"
)]
fn alpha_rules_match_spawnb() {
    assert_eq!(value_after_alpha(1.0, 0.5), BLOCK_VALUE);
    assert!((value_after_alpha(1.5, 0.2) - 1.3).abs() < 1e-6);
    assert!((value_after_alpha(1.5, 0.5) - 1.4).abs() < 1e-6);
    assert_eq!(value_after_alpha(1.5, 1.0), 1.5);
    assert_eq!(value_after_alpha(-0.3, 0.1), -0.3);
}

/// The original decides in another order: by transparency first, and inside the half-transparent
/// case it turns a rock into a block before it looks at the unchanged range; it also works in
/// percent and in `f64`. Written that way here, it must agree with ours for every marker value and
/// every opacity a placement can have (the SWF stores it in 256ths).
#[test]
#[allow(clippy::float_cmp, reason = "the original compares the value with exactly 1")]
fn alpha_rules_agree_with_the_originals_order_for_every_opacity() {
    const OPACITY_STEPS: u16 = 256;
    const PERCENT: f64 = 100.0;
    let outside_unchanged = |value: f64| !(-0.5..=0.0).contains(&value);
    let in_original_order = |value: f64, percent: f64| {
        if percent < 30.0 {
            if outside_unchanged(value) { value - 0.2 } else { value }
        } else if percent < 75.0 {
            if value == 1.0 {
                1.9
            } else if outside_unchanged(value) {
                value - 0.1
            } else {
                value
            }
        } else {
            value
        }
    };
    let marker_values = TILE_MARKERS.iter().map(|(_, value)| *value);
    let pickup_values = (0..PICKUP_MARKER_NAMED_VARIANTS).map(|number| -f32::from(number) * PICKUP_MARKER_VALUE_STEP);
    let sweep = (-400_i16..=1000).map(|hundredths| f32::from(hundredths) / 200.0);
    for value in marker_values.chain(pickup_values).chain(sweep) {
        for steps in 0..=OPACITY_STEPS {
            let alpha = f32::from(steps) / f32::from(OPACITY_STEPS);
            let percent = f64::from(steps) * PERCENT / f64::from(OPACITY_STEPS);
            let expected = in_original_order(f64::from(value), percent);
            let ours = f64::from(value_after_alpha(value, alpha));
            assert!(
                (ours - expected).abs() < 1e-6,
                "value {value}, opacity {steps}/256: {ours} vs {expected}"
            );
        }
    }
}

/// Needs the SWF extracted from a real Steam install; run with `cargo test -- --ignored --nocapture`.
#[test]
#[ignore = "requires the game SWF extracted into the user cache"]
fn basement_layouts_hold_known_objects() {
    let swf_path = cache_directory()
        .expect("no cache directory")
        .join(EXTRACTED_SWF_FILE_NAME);
    let swf_bytes = std::fs::read(swf_path).expect("extracted SWF should exist");
    let swf_buffer = swf::decompress_swf(&swf_bytes[..]).expect("SWF should decompress");
    let movie = swf::parse_swf(&swf_buffer).expect("SWF should parse");
    let library = CharacterLibrary::new(&movie);
    let layouts = room_layouts(&library, BASEMENT_ROOM_SPRITE);
    assert_eq!(layouts.len(), 186);
    let treasure = basement_layouts(&library).treasure;
    assert_eq!(treasure.len(), 6);
    println!(
        "treasure rooms: {:?}",
        treasure.iter().map(|layout| layout.cells.len()).collect::<Vec<_>>()
    );
    let cells: Vec<&LayoutCell> = layouts.iter().flat_map(|layout| &layout.cells).collect();
    let unknown: Vec<f32> = cells
        .iter()
        .map(|cell| cell.value)
        .filter(|value| kind_for_value(*value).is_none() && !PICKUP_VALUES.contains(value) && *value > -1.0)
        .collect();
    // The black fly's marker (-4, type 13), fainter ones at -4.1 and -4.2.
    let flies = cells.iter().filter(|cell| (cell.value + 4.0).abs() < 0.25).count();
    println!("{flies} black flies");
    assert!(flies > 0, "black fly markers are read");
    // The Gaper's marker (-1, type 10), fainter ones at -1.1 and -1.2.
    let gapers = cells.iter().filter(|cell| (cell.value + 1.0).abs() < 0.25).count();
    println!("{gapers} Gapers");
    let (mut plain_only, mut calm_only, mut mixed) = (0, 0, 0);
    for layout in &layouts {
        let plain = layout.cells.iter().any(|cell| (cell.value + 1.0).abs() < 0.001);
        let calm = layout.cells.iter().any(|cell| cell.value < -1.05 && cell.value > -1.25);
        match (plain, calm) {
            (true, true) => mixed += 1,
            (true, false) => plain_only += 1,
            (false, true) => calm_only += 1,
            _ => {}
        }
    }
    println!("rooms with Gapers: {plain_only} plain only, {calm_only} faint only, {mixed} both");
    assert!(gapers > 0, "Gaper markers are read");
    assert_walkers_are_read(&layouts);
    let pickups = cells.iter().filter(|cell| PICKUP_VALUES.contains(&cell.value)).count();
    assert!(pickups > 0, "pickup markers are read");
    println!("{} cells, unknown values: {unknown:?}", cells.len());
    assert!(
        cells.iter().all(|cell| cell.column < 13 && cell.row < 7),
        "cells outside the interior"
    );
    assert!(unknown.is_empty(), "{unknown:?}");
    print_layouts(&layouts[1..4]);
}

/// The headless body (-2, type 11) and the Horf (-3, type 12) are placed by some rooms.
fn assert_walkers_are_read(layouts: &[RoomLayout]) {
    let rooms_with = |marker: f32| {
        layouts
            .iter()
            .filter(|layout| layout.cells.iter().any(|cell| (cell.value - marker).abs() < 0.25))
            .count()
    };
    println!(
        "rooms with headless bodies: {}, Horfs: {}",
        rooms_with(-2.0),
        rooms_with(-3.0)
    );
    assert!(rooms_with(-2.0) > 0, "headless markers are read");
    assert!(rooms_with(-3.0) > 0, "Horf markers are read");
}

/// Draws layouts as text: `O` pits and above, `p` poop and fire, `#` rocks and the rest.
fn print_layouts(layouts: &[RoomLayout]) {
    for (frame_index, layout) in (2..).zip(layouts) {
        println!("frame {frame_index}:");
        for row in 0..7 {
            let line: String = (0..13)
                .map(|column| {
                    layout
                        .cells
                        .iter()
                        .find(|cell| cell.column == column && cell.row == row)
                        .map_or('.', |cell| {
                            if cell.value >= 3.0 {
                                'O'
                            } else if cell.value > 1.45 {
                                'p'
                            } else {
                                '#'
                            }
                        })
                })
                .collect();
            println!("  {line}");
        }
    }
}
