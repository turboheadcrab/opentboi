//! Index of the SWF's drawable characters, sprite timelines and bitmaps.

use std::collections::HashMap;

use swf::{Button, CharacterId, DefineBitsLossless, EditText, Font, Shape, Tag, Text};

use super::timeline::SpriteTimeline;

pub enum Character<'movie> {
    Shape(&'movie Shape),
    Sprite(SpriteTimeline<'movie>),
    /// A button, drawn in its "up" state as Flash shows it when nothing hovers it.
    Button(&'movie Button<'movie>),
    Text(&'movie EditText<'movie>),
    StaticText(&'movie Text),
}

pub struct CharacterLibrary<'movie> {
    movie: &'movie swf::Swf<'movie>,
    characters: HashMap<CharacterId, Character<'movie>>,
    bitmaps: HashMap<CharacterId, &'movie DefineBitsLossless<'movie>>,
    fonts: HashMap<CharacterId, &'movie Font<'movie>>,
}

impl<'movie> CharacterLibrary<'movie> {
    pub fn new(movie: &'movie swf::Swf<'movie>) -> Self {
        let mut library = Self {
            movie,
            characters: HashMap::new(),
            bitmaps: HashMap::new(),
            fonts: HashMap::new(),
        };
        library.index_tags(&movie.tags);
        library
    }

    /// The whole movie, for reading its scripts.
    pub fn movie(&self) -> &'movie swf::Swf<'movie> {
        self.movie
    }

    pub fn character(&self, character_id: CharacterId) -> Option<&Character<'movie>> {
        self.characters.get(&character_id)
    }

    pub fn font(&self, font_id: CharacterId) -> Option<&'movie Font<'movie>> {
        self.fonts.get(&font_id).copied()
    }

    pub fn bitmap(&self, character_id: CharacterId) -> Option<&'movie DefineBitsLossless<'movie>> {
        self.bitmaps.get(&character_id).copied()
    }

    /// Sprites can define further characters inside their own tag lists, so this recurses.
    fn index_tags(&mut self, tags: &'movie [Tag<'movie>]) {
        for tag in tags {
            match tag {
                Tag::DefineShape(shape) => {
                    self.characters.insert(shape.id, Character::Shape(shape));
                }
                Tag::DefineBitsLossless(bitmap) => {
                    self.bitmaps.insert(bitmap.id, bitmap);
                }
                Tag::DefineText(text) | Tag::DefineText2(text) => {
                    self.characters.insert(text.id, Character::StaticText(text));
                }
                Tag::DefineEditText(edit_text) => {
                    self.characters.insert(edit_text.id(), Character::Text(edit_text));
                }
                Tag::DefineFont2(font) => {
                    self.fonts.insert(font.id, font);
                }
                Tag::DefineButton2(button) => {
                    self.characters.insert(button.id, Character::Button(button));
                }
                Tag::DefineSprite(sprite) => {
                    let timeline = SpriteTimeline::new(&sprite.tags, sprite.num_frames);
                    self.characters.insert(sprite.id, Character::Sprite(timeline));
                    self.index_tags(&sprite.tags);
                }
                _ => {}
            }
        }
    }
}
