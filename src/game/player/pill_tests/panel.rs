use super::{ONE_FRAME, game, use_pill_of};
use crate::game::player::pill_state::PillKind;
use crate::game::{Game, InputState, TextureId};

/// The characters of the stats panel, as drawn.
fn panel_text(game: &Game) -> String {
    let mut draw_items = Vec::new();
    game.fill_draw_items(&mut draw_items);
    draw_items
        .iter()
        .filter_map(|item| match item.texture {
            Some(TextureId::HudGlyph(glyph)) => Some(char::from(glyph)),
            _ => None,
        })
        .collect()
}

#[test]
fn the_stats_panel_follows_every_stat_pill() {
    for kind in [
        PillKind::Tears,
        PillKind::Range,
        PillKind::Speed,
        PillKind::ShotSpeed,
        PillKind::Luck,
    ] {
        let mut game = game();
        let before = panel_text(&game);
        use_pill_of(&mut game, kind, true);
        game.advance(&InputState::default(), ONE_FRAME);
        assert_ne!(panel_text(&game), before, "{kind:?}");
    }
}
