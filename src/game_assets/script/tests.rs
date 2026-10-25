use super::*;
use crate::game_assets::{EXTRACTED_SWF_FILE_NAME, cache_directory};

const DEMO_SWF: &str = "downloads/581168_Isaac_demo_Secure.swf";
/// The sizes of object types 4 (bombs keep this far from walls) and 5 (pickups), as transcribed.
const BOMB_SIZE: f64 = 16.0;
const PICKUP_SIZE: f64 = 12.0;

fn read(swf_bytes: &[u8]) -> GameScript {
    let swf_buffer = swf::decompress_swf(swf_bytes).expect("SWF should decompress");
    let movie = swf::parse_swf(&swf_buffer).expect("SWF should parse");
    GameScript::from_movie(&movie)
}

fn summary(script: &GameScript) -> String {
    let sizes = script.array_assigned_to("sizes").map(|sizes| {
        sizes
            .iter()
            .take(6)
            .map(|size| size.number().map_or("?".to_owned(), |number| number.to_string()))
            .collect::<Vec<_>>()
    });
    format!(
        "{} functions, {} frame statements, sizes starts {sizes:?}",
        script.functions.len(),
        script.frame_statements.len()
    )
}

#[test]
#[ignore = "requires the game SWF extracted into the user cache"]
fn the_game_script_gives_the_transcribed_sizes() {
    let path = cache_directory()
        .expect("no cache directory")
        .join(EXTRACTED_SWF_FILE_NAME);
    let script = read(&std::fs::read(path).expect("extracted SWF should exist"));
    println!("game: {}", summary(&script));
    let sizes = script.array_assigned_to("sizes").expect("sizes should be assigned");
    assert_eq!(sizes[4].number(), Some(BOMB_SIZE));
    assert_eq!(sizes[5].number(), Some(PICKUP_SIZE));
    let cards = script.card_names().expect("tart()'s card names");
    assert_eq!(cards.len(), 27);
    assert_eq!(cards[0], "X Wheel of fortune");
    assert_eq!(cards[26], "The Joker");
    assert_eq!(script.fortune_messages().len(), usize::from(crate::game::FORTUNE_COUNT));
    let messages = script.message_texts();
    assert_eq!(messages[..4], ["a Nickel", "a Dime", "Trolololol", "1 + 1 free"]);
    let pill_texts = script.pill_texts();
    assert_eq!(pill_texts.len(), crate::game::PillText::ALL.len());
    for (text, words) in crate::game::PillText::ALL.into_iter().zip(&pill_texts) {
        assert!(!words.is_empty(), "{text:?} has no text in the script");
    }
    let slot_result = script.function("slotf").expect("slotf() should exist");
    assert!(!slot_result.statements.is_empty(), "slotf() has statements");
}

#[test]
#[ignore = "requires the browser demo SWF in downloads/"]
fn the_demo_script_can_be_read() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(DEMO_SWF);
    let script = read(&std::fs::read(path).expect("demo SWF should exist"));
    println!("demo: {}", summary(&script));
    let names: Vec<&str> = script
        .functions
        .iter()
        .take(30)
        .map(|function| function.name.as_str())
        .collect();
    println!("demo functions: {names:?}");
    println!("demo card names: {:?}", script.card_names());
}
