use std::io::Cursor;

use rodio::Source;

use super::*;
use crate::game_assets::{EXTRACTED_SWF_FILE_NAME, cache_directory, load_assets};

/// Flash's MP3 rates as used by the game; the key sounds are at half rate.
const EXPECTED_SAMPLE_RATES: [u32; 2] = [22_050, 44_100];
const EXPECTED_CHANNEL_COUNT: u16 = 1;
const SHORTEST_PLAUSIBLE_CLIP_SECONDS: f32 = 0.1;
/// The longest catalogued clips are the jingles: secret room 5.04 s, treasure room 2.64 s.
const LONGEST_PLAUSIBLE_CLIP_SECONDS: f32 = 8.0;

/// Needs the SWF extracted from a real Steam install; run with `cargo test -- --ignored`.
#[test]
#[ignore = "requires the game SWF extracted into the user cache"]
fn every_catalogued_clip_decodes_as_short_mono_audio() {
    let swf_path = cache_directory()
        .expect("no cache directory")
        .join(EXTRACTED_SWF_FILE_NAME);
    let sound_bank = load_assets(&swf_path)
        .expect("extracted SWF should parse")
        .sound_bank
        .expect("every catalogued sound should be present");

    for (event, export_names) in SOUND_CATALOG {
        let clips = sound_bank.variants(event);
        assert_eq!(clips.len(), export_names.len(), "{event:?} is missing variants");
        for (clip, export_name) in clips.iter().zip(export_names) {
            let decoder = rodio::Decoder::new(Cursor::new(clip.mp3.clone()))
                .unwrap_or_else(|decode_error| panic!("{export_name} does not decode: {decode_error}"));
            let sample_rate = decoder.sample_rate().get();
            assert!(
                EXPECTED_SAMPLE_RATES.contains(&sample_rate),
                "{export_name}: {sample_rate} Hz"
            );
            assert_eq!(decoder.channels().get(), EXPECTED_CHANNEL_COUNT, "{export_name}");
            let decoded_samples = decoder.count();
            // The sound itself, after the encoder's delay, lies within what decodes.
            assert!(
                clip.leading_samples + clip.sample_count <= decoded_samples,
                "{export_name}: {} + {} beyond {decoded_samples}",
                clip.leading_samples,
                clip.sample_count
            );
            let decoded_seconds = decoded_samples as f32 / sample_rate as f32;
            assert!(
                (SHORTEST_PLAUSIBLE_CLIP_SECONDS..LONGEST_PLAUSIBLE_CLIP_SECONDS).contains(&decoded_seconds),
                "{export_name} decoded to {decoded_seconds} s"
            );
        }
    }
}

/// Every track the director can ask for is in the game's files, with its intro and its loop.
#[test]
#[ignore = "requires the game SWF extracted into the user cache"]
fn every_music_track_is_in_the_swf() {
    let swf_path = cache_directory()
        .expect("no cache directory")
        .join(EXTRACTED_SWF_FILE_NAME);
    let sound_bank = load_assets(&swf_path)
        .expect("extracted SWF should parse")
        .sound_bank
        .expect("every catalogued sound should be present");
    assert_eq!(sound_bank.all_music().len(), music::MUSIC_CATALOG.len());
    let basement = &sound_bank.all_music()[&MusicTrack::Basement];
    assert!(basement.opening.is_some());
    assert!(basement.looped.sample_count > basement.intro.sample_count);
}

/// Every jingle that takes the music down is longer than the second it lets the music back up
/// before its end.
#[test]
#[ignore = "requires the game SWF extracted into the user cache"]
fn every_music_ducking_jingle_is_longer_than_its_tail() {
    let swf_path = cache_directory()
        .expect("no cache directory")
        .join(EXTRACTED_SWF_FILE_NAME);
    let sound_bank = load_assets(&swf_path)
        .expect("extracted SWF should parse")
        .sound_bank
        .expect("every catalogued sound should be present");
    let lengths: Vec<(SoundEvent, f32)> = sound_bank.jingle_lengths().collect();
    assert_eq!(lengths.len(), 7, "{lengths:?}");
    for (event, seconds) in lengths {
        assert!(seconds > 1.0, "{event:?} lasts {seconds} s");
    }
}
