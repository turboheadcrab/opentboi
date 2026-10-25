//! The fixed texts the script hands to its message and name functions.

use super::GameScript;
use super::expression::{Expression, Statement};
use crate::game::{HUD_NAME_COUNT, MESSAGES_BEFORE_PILLS, PillText};

impl GameScript {
    /// What each of [`PillText::ALL`] says, in that order; empty where the script lacks it.
    pub fn pill_texts(&self) -> Vec<String> {
        let messages = self.message_texts();
        let names = messages
            .get(MESSAGES_BEFORE_PILLS)
            .map(|first| self.pill_names(first, HUD_NAME_COUNT))
            .unwrap_or_default();
        let banners = self.banner_texts();
        PillText::ALL
            .into_iter()
            .map(|text| {
                text_at(&names, text.name_position())
                    .or_else(|| text_at(&messages, text.message_position()))
                    .or_else(|| text_at(&banners, text.banner_position()))
                    .cloned()
                    .unwrap_or_default()
            })
            .collect()
    }

    /// The texts the message function shows (`st22()`), in the script's order. The
    /// function is `st22`, or where names are obfuscated, the one most often called with a
    /// single text.
    pub fn message_texts(&self) -> Vec<String> {
        const MESSAGE_FUNCTION: &str = "st22";
        let single_text_calls = self.single_text_calls();
        let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        for (name, _) in &single_text_calls {
            *counts.entry(name).or_default() += 1;
        }
        let function = if counts.contains_key(MESSAGE_FUNCTION) {
            MESSAGE_FUNCTION
        } else {
            counts
                .into_iter()
                .max_by_key(|&(_, count)| count)
                .map_or("", |(name, _)| name)
        };
        texts_of_calls_to(single_text_calls, function)
    }

    /// The fixed texts the name banner function shows (`st11()`), in the script's order; empty
    /// where names are obfuscated.
    pub fn banner_texts(&self) -> Vec<String> {
        const BANNER_FUNCTION: &str = "st11";
        texts_of_calls_to(self.single_text_calls(), BANNER_FUNCTION)
    }

    /// The texts the held pill's name can show, in the script's order: first the one for a pill
    /// not yet identified, then each effect's. Found by shape: they are the texts assigned one after
    /// another from just before the first pill message (`first_message`) on.
    pub fn pill_names(&self, first_message: &str, count: usize) -> Vec<String> {
        let assigned: Vec<&str> = self
            .statements()
            .filter_map(|statement| match statement {
                Statement::Assign {
                    value: Expression::Text(text),
                    ..
                } => Some(text.as_str()),
                _ => None,
            })
            .collect();
        let Some(first) = assigned.iter().position(|&text| text == first_message) else {
            return Vec::new();
        };
        assigned
            .into_iter()
            .skip(first.saturating_sub(1))
            .take(count)
            .map(str::to_owned)
            .collect()
    }

    /// Every call that hands one fixed text to a function: that function's name and the text.
    fn single_text_calls(&self) -> Vec<(&str, &str)> {
        self.statements()
            .filter_map(|statement| match statement {
                Statement::Evaluate(Expression::Call(name, arguments)) => match arguments.as_slice() {
                    [Expression::Text(text)] => Some((name.as_str(), text.as_str())),
                    _ => None,
                },
                _ => None,
            })
            .collect()
    }
}

fn text_at(texts: &[String], position: Option<usize>) -> Option<&String> {
    position.and_then(|place| texts.get(place))
}

fn texts_of_calls_to(calls: Vec<(&str, &str)>, function: &str) -> Vec<String> {
    calls
        .into_iter()
        .filter(|(name, _)| *name == function)
        .map(|(_, text)| text.to_owned())
        .collect()
}
