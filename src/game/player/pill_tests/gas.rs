//! Bad Gas (`fart()`, DoAction 26697–26719).

use super::*;
use crate::game::TextureId;

#[test]
fn bad_gas_lets_out_a_cloud_with_a_sound() {
    let mut game = game();
    let sounds = use_pill_of(&mut game, PillKind::BadGas, false);
    assert!(sounds.contains(&SoundEvent::Fart), "{sounds:?}");
    let mut draw_items = Vec::new();
    game.fill_room_draw_items(&mut draw_items);
    assert!(
        draw_items
            .iter()
            .any(|item| matches!(item.texture, Some(TextureId::GasCloud(_))))
    );
}
