use crate::game::original_units::{Frames, Hearts};
use crate::game::{Game, GameParameters, InputState, RoomAppearance, RoomLayouts};

const ONE_FRAME: Frames = Frames(1.0);

fn game_standing_in_creep() -> Game {
    let mut game = Game::new(
        RoomAppearance::Placeholder,
        &GameParameters::ORIGINAL,
        &RoomLayouts::default(),
        1,
    );
    for _ in 0..40 {
        game.advance(&InputState::default(), ONE_FRAME);
    }
    let position = game.player.position();
    game.floor.current_room_mut().creep.spawn(position, 1.0);
    game
}

#[test]
fn standing_in_blood_creep_hurts_by_a_heart() {
    let mut game = game_standing_in_creep();
    let before = game.inventory.red_hearts;
    game.advance(&InputState::default(), ONE_FRAME);
    assert_eq!(game.inventory.red_hearts, before - Hearts::ONE);
}

#[test]
fn creep_in_a_dark_room_slows_instead_of_hurting() {
    let mut game = game_standing_in_creep();
    game.floor.current_room_mut().creep.make_slowing();
    let before = game.inventory.red_hearts;
    game.advance(&InputState::default(), ONE_FRAME);
    assert_eq!(game.inventory.red_hearts, before);
    assert!(game.player.is_slowed());
}
