use super::*;
use crate::game::parameters::GameParameters;

const ONE_FRAME: Frames = Frames(1.0);

fn director() -> MusicDirector {
    let mut director = MusicDirector::new(GameParameters::ORIGINAL.music);
    director.drain().for_each(drop);
    director
}

fn run(director: &mut MusicDirector, frames: u32, wanted: MusicTrack) -> Vec<MusicEvent> {
    for _ in 0..frames {
        director.advance(ONE_FRAME, wanted);
    }
    director.drain().collect()
}

#[test]
fn a_new_run_opens_with_its_opening_music() {
    let mut director = MusicDirector::new(GameParameters::ORIGINAL.music);
    assert_eq!(
        director.drain().collect::<Vec<_>>(),
        [MusicEvent::Play {
            track: MusicTrack::Basement,
            start: MusicStart::NewRun,
            volume: 0.5
        }]
    );
}

#[test]
fn the_track_goes_on_while_the_room_wants_it() {
    let mut director = director();
    assert_eq!(run(&mut director, 300, MusicTrack::Basement), vec![]);
}

#[test]
fn a_special_room_fades_the_floor_music_out_and_then_plays_its_own_from_its_intro() {
    let mut director = director();
    let events = run(&mut director, 1, MusicTrack::Shop);
    assert_eq!(events, [MusicEvent::FadeOut { seconds: 7.0 / 30.0 }]);
    let events = run(&mut director, 10, MusicTrack::Shop);
    assert_eq!(
        events,
        [MusicEvent::Play {
            track: MusicTrack::Shop,
            start: MusicStart::Intro,
            volume: 0.5
        }]
    );
}

#[test]
fn leaving_a_special_room_goes_back_to_the_loop_without_the_intro() {
    let mut director = director();
    run(&mut director, 12, MusicTrack::Shop);
    let events = run(&mut director, 12, MusicTrack::Basement);
    assert_eq!(
        events.last(),
        Some(&MusicEvent::Play {
            track: MusicTrack::Basement,
            start: MusicStart::Loop,
            volume: 0.5
        })
    );
}

#[test]
fn death_plays_its_own_track_from_the_intro() {
    let mut director = director();
    let events = run(&mut director, 12, MusicTrack::Death);
    assert_eq!(
        events.last(),
        Some(&MusicEvent::Play {
            track: MusicTrack::Death,
            start: MusicStart::Intro,
            volume: 0.5
        })
    );
}

const JINGLE_LENGTH: Frames = Frames(90.0);

fn ducks(events: &[MusicEvent]) -> Vec<f32> {
    events
        .iter()
        .filter_map(|event| match event {
            MusicEvent::Duck { factor } => Some(*factor),
            _ => None,
        })
        .collect()
}

#[test]
fn a_room_jingle_takes_the_music_down_to_its_floor_and_it_comes_back_after() {
    let mut director = director();
    director.set_jingle_lengths([(SoundEvent::TreasureRoomEntered, JINGLE_LENGTH)]);
    director.hear(&[SoundEvent::TreasureRoomEntered]);
    let during = ducks(&run(&mut director, 60, MusicTrack::Basement));
    // 35 % loses 7 a frame: 28, 21, 14, 7, then the floor of 5.
    assert!((during[0] - 28.0 / 35.0).abs() < 1e-5);
    assert!((during[3] - 7.0 / 35.0).abs() < 1e-5);
    assert!((during[4] - 5.0 / 35.0).abs() < 1e-5);
    assert_eq!(during.len(), 5, "no change once at the floor");
    // The jingle lets go a second before its end (frame 60): 30 frames to climb 30 points.
    let after = ducks(&run(&mut director, 40, MusicTrack::Basement));
    assert!(after.windows(2).all(|pair| pair[0] < pair[1]), "climbing steadily");
    assert!((after.last().copied().unwrap_or_default() - 1.0).abs() < 1e-5);
}

#[test]
fn sounds_that_are_not_jingles_leave_the_music_alone() {
    let mut director = director();
    director.set_jingle_lengths([(SoundEvent::TreasureRoomEntered, JINGLE_LENGTH)]);
    director.hear(&[SoundEvent::TearFired, SoundEvent::DoorsOpened]);
    assert_eq!(ducks(&run(&mut director, 60, MusicTrack::Basement)), Vec::<f32>::new());
}

#[test]
fn only_the_room_and_card_jingles_duck_the_music() {
    assert!(SoundEvent::TreasureRoomEntered.ducks_music());
    assert!(SoundEvent::SecretRoomFound.ducks_music());
    assert!(
        !SoundEvent::BossRoomEntered.ducks_music(),
        "the portcullis is no jingle"
    );
    assert!(!SoundEvent::TearFired.ducks_music());
}

/// The whole way: a jingle among the game's sounds, once the frontend asks for them, takes the music
/// down on the following frames.
#[test]
fn a_jingle_heard_through_the_games_sounds_ducks_the_music() {
    use crate::game::RoomAppearance;
    use crate::game::input_state::InputState;
    use crate::game::room::layout::RoomLayouts;
    let mut game = Game::new(
        RoomAppearance::Placeholder,
        &GameParameters::ORIGINAL,
        &RoomLayouts::default(),
        1,
    );
    game.set_jingle_lengths([(SoundEvent::TreasureRoomEntered, 2.64)]);
    game.drain_music_events().for_each(drop);
    game.pending_sound_events.push(SoundEvent::TreasureRoomEntered);
    assert_eq!(game.drain_sound_events().count(), 1);
    for _ in 0..4 {
        game.advance(&InputState::default(), ONE_FRAME);
    }
    let factors = ducks(&game.drain_music_events().collect::<Vec<_>>());
    assert!(
        factors.len() >= 3 && factors.windows(2).all(|pair| pair[1] < pair[0]),
        "{factors:?}"
    );
}
