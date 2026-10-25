use super::*;

fn bomb_hint_texture(game: &Game) -> Option<TextureId> {
    let mut draw_items = Vec::new();
    game.fill_room_draw_items(&mut draw_items);
    draw_items
        .iter()
        .filter_map(|item| item.texture)
        .find(|texture| matches!(texture, TextureId::StartRoomHint(StartRoomControl::Bomb, _)))
}

#[test]
fn the_start_room_hint_follows_the_last_device_used() {
    let mut game = Game::new(
        RoomAppearance::Placeholder,
        &GameParameters::ORIGINAL,
        &RoomLayouts::default(),
        1,
    );
    let keyboard = TextureId::StartRoomHint(StartRoomControl::Bomb, InputDevice::KeyboardAndMouse);
    let gamepad = TextureId::StartRoomHint(StartRoomControl::Bomb, InputDevice::Gamepad(GamepadKind::Xbox));
    let playstation = TextureId::StartRoomHint(StartRoomControl::Bomb, InputDevice::Gamepad(GamepadKind::PlayStation));
    assert_eq!(
        bomb_hint_texture(&game),
        Some(keyboard),
        "the keyboard until told otherwise"
    );
    let pad_input = InputState {
        device_in_use: Some(InputDevice::Gamepad(GamepadKind::Xbox)),
        ..InputState::default()
    };
    game.update(&pad_input, 0.0);
    assert_eq!(bomb_hint_texture(&game), Some(gamepad));
    game.update(&InputState::default(), 0.0);
    assert_eq!(bomb_hint_texture(&game), Some(gamepad), "idle frames change nothing");
    let other_pad_input = InputState {
        device_in_use: Some(InputDevice::Gamepad(GamepadKind::PlayStation)),
        ..InputState::default()
    };
    game.update(&other_pad_input, 0.0);
    assert_eq!(
        bomb_hint_texture(&game),
        Some(playstation),
        "the hint names the pad's own buttons"
    );
    let key_input = InputState {
        device_in_use: Some(InputDevice::KeyboardAndMouse),
        ..InputState::default()
    };
    game.update(&key_input, 0.0);
    assert_eq!(bomb_hint_texture(&game), Some(keyboard));
}
