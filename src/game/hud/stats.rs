//! Rebirth's stats panel, for debugging (on for now, `--no-stats` hides it): speed, tears, damage,
//! range, shot speed and luck on the left sidebar (or over the room's left wall without one), in
//! the original's units, with the room's card boosts.

use glam::Vec2;

use crate::game::draw::{DrawItem, Rgba};
use crate::game::floor::Floor;
use crate::game::floor::plan::RoomRole;
use crate::game::parameters::{FiringParameters, GameParameters, MovementParameters};
use crate::game::player::boosts::PlayerStats;
use crate::game::room::STAGE_SIZE;
use crate::game::room::stage_to_world;
use crate::game::texture_id::TextureId;
use crate::game::{Game, pixels_per_world_unit};

/// The characters the panel can show (see `texture_catalog::glyphs`).
pub const HUD_GLYPHS: &str = "-.0123456789:ABCDEFGHIJKLMNOPQRSTUVWXYZ";
/// Stage position of the first line's first character, below the HUD over the left wall.
const FIRST_LINE: Vec2 = Vec2::new(14.0, 120.0);
/// The glyphs are centered in their textures, about 9 units tall and up to 12 wide (W).
const LINE_SPACING: f32 = 14.0;
const CHARACTER_ADVANCE: f32 = 13.0;
/// Above the room, below the view masks.
const STATS_DEPTH: f32 = 850.0;
const FALLBACK_COLOR: Rgba = Rgba::opaque(1.0, 1.0, 1.0);
const DECIMALS: usize = 2;
/// The sidebar is used once it is at least this wide (world units); margins on both sides.
const MINIMUM_SIDEBAR_WIDTH: f32 = 60.0;
const SIDEBAR_MARGIN: f32 = 10.0;

/// The rooms a floor may lack (rolled in the floor setup, DoAction 32213–32232), with their
/// labels.
const OPTIONAL_ROOMS: [(RoomRole, &str); 4] = [
    (RoomRole::Library, "LIB"),
    (RoomRole::Sacrifice, "SAC"),
    (RoomRole::Curse, "CUR"),
    (RoomRole::MiniBoss, "MIN"),
];

/// What the panel tells about the floor, for re-rolling one with something wanted.
#[derive(Clone, Copy, Debug)]
pub struct FloorFacts {
    pub seed: u32,
    /// Whether each of [`OPTIONAL_ROOMS`] is on the floor.
    pub optional_rooms: [bool; OPTIONAL_ROOMS.len()],
}

impl FloorFacts {
    pub fn for_floor(floor: &Floor, seed: u32) -> Self {
        Self {
            seed,
            optional_rooms: OPTIONAL_ROOMS.map(|(role, _)| floor.cell_with(role).is_some()),
        }
    }
}

/// Each stat's label and value.
fn lines(parameters: &GameParameters, stats: PlayerStats) -> [(&'static str, f32); 6] {
    let movement = MovementParameters {
        speed_stat: stats.speed,
        ..parameters.movement
    };
    let firing = FiringParameters {
        delay_bonus: parameters.firing.delay_bonus + stats.delay_bonus,
        delay_offset: parameters.firing.delay_offset + stats.delay_offset,
        ..parameters.firing
    };
    [
        ("SPD", movement.top_speed_per_frame().get()),
        ("TRS", firing.frames_between_shots().get()),
        ("DMG", stats.damage.get()),
        ("RNG", (parameters.tear.start_height + stats.tear_height_bonus).get()),
        (
            "SHT",
            (parameters.tear.speed_per_frame * stats.shot_speed_multiplier).get(),
        ),
        ("LCK", f32::from(i16::try_from(stats.luck).unwrap_or_default())),
    ]
}

impl Game {
    /// The frontend's output size in pixels, so the stats panel can use the sidebar art in
    /// widescreen.
    pub fn set_output_size(&mut self, output_pixels: Vec2) {
        self.visible_size = output_pixels / pixels_per_world_unit(output_pixels);
    }
}

/// Where the panel starts (stage coordinates) and how large its glyphs are: on the left sidebar
/// when the output is wide enough to show one, else over the room's left wall.
fn placement(visible_size: Vec2, widest_line: usize) -> (Vec2, f32) {
    let sidebar_width = (visible_size.x - STAGE_SIZE.x) / 2.0;
    if sidebar_width < MINIMUM_SIDEBAR_WIDTH {
        return (FIRST_LINE, 1.0);
    }
    let fitting_scale = (sidebar_width - 2.0 * SIDEBAR_MARGIN) / (widest_line as f32 * CHARACTER_ADVANCE);
    let first_glyph_center = Vec2::new(SIDEBAR_MARGIN - sidebar_width, FIRST_LINE.y);
    (first_glyph_center, fitting_scale.min(1.0))
}

pub fn append_draw_items(
    parameters: &GameParameters,
    stats: PlayerStats,
    floor: FloorFacts,
    visible_size: Vec2,
    draw_items: &mut Vec<DrawItem>,
) {
    let mut texts: Vec<String> = lines(parameters, stats)
        .map(|(label, value)| format!("{label} {value:.DECIMALS$}"))
        .into();
    for ((_, label), present) in OPTIONAL_ROOMS.into_iter().zip(floor.optional_rooms) {
        texts.push(format!("{label} {}", if present { "YES" } else { "NO" }));
    }
    texts.push(format!("SEED {}", floor.seed));
    let widest_line = texts.iter().map(String::len).max().unwrap_or(1);
    let (first_line, scale) = placement(visible_size, widest_line);
    for (line_index, text) in texts.iter().enumerate() {
        let line_start = first_line + Vec2::new(0.0, line_index as f32 * LINE_SPACING * scale);
        for (column, glyph) in text.bytes().enumerate() {
            if !HUD_GLYPHS.as_bytes().contains(&glyph) {
                continue;
            }
            let stage_point = line_start + Vec2::new(column as f32 * CHARACTER_ADVANCE * scale, 0.0);
            draw_items.push(DrawItem::textured(
                TextureId::HudGlyph(glyph),
                stage_to_world(stage_point),
                scale,
                FALLBACK_COLOR,
                STATS_DEPTH,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widescreen_puts_the_panel_on_the_sidebar() {
        let (start, scale) = placement(STAGE_SIZE, 9);
        assert_eq!((start, scale), (FIRST_LINE, 1.0), "4:3 has no sidebar");
        let widescreen = Vec2::new(STAGE_SIZE.y * 16.0 / 9.0, STAGE_SIZE.y);
        let (start, scale) = placement(widescreen, 9);
        assert!(start.x < 0.0, "left of the stage");
        assert!(start.x + 9.0 * CHARACTER_ADVANCE * scale < 0.0, "fits the sidebar");
    }

    #[test]
    fn the_panel_shows_what_pills_do_to_the_stats() {
        use crate::game::player::boosts::RoomBoosts;
        use crate::game::player::pill_state::PillState;
        let parameters = GameParameters::ORIGINAL;
        let value = |lines: [(&str, f32); 6], label: &str| lines.iter().find(|line| line.0 == label).map(|line| line.1);
        let plain = lines(
            &parameters,
            RoomBoosts::default().stats(&parameters, &PillState::default()),
        );
        let mut pills = PillState::default();
        pills.tears = 1.0;
        pills.range = 1.0;
        pills.speed = 1.0;
        pills.shot_speed = 1.0;
        pills.luck = 2;
        let boosted = lines(&parameters, RoomBoosts::default().stats(&parameters, &pills));
        assert!(
            value(boosted, "TRS") < value(plain, "TRS"),
            "tears up shortens the delay"
        );
        assert!(value(boosted, "SHT") > value(plain, "SHT"));
        assert_eq!(value(boosted, "LCK"), Some(2.0));
    }
}
