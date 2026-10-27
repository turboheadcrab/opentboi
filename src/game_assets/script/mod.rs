//! Reads the original's ActionScript 2 bytecode (AVM1) back into statements, so gameplay values
//! and formulas can be taken from whichever SWF the player has (see
//! `docs/parameter-extraction.md`). Frame scripts of the main timeline and of every sprite are
//! read; function bodies become [`ScriptFunction`]s.

mod decoder;
mod expression;
#[cfg(test)]
mod primitives;
mod texts;

use crate::game::{FORTUNE_COUNT, TAROT_COUNT, Trinket};
pub use decoder::ScriptFunction;
#[cfg(test)]
pub use expression::Operator;
pub use expression::{Expression, Statement};

/// Every script of a movie, decoded.
#[derive(Clone, Debug, Default)]
pub struct GameScript {
    /// Frame scripts' own statements (outside functions), in tag order.
    pub frame_statements: Vec<Statement>,
    pub functions: Vec<ScriptFunction>,
    /// Some bytecode could not be read, so values looked up may be missing.
    pub damaged: bool,
}

impl GameScript {
    pub fn from_movie(movie: &swf::Swf) -> Self {
        let mut script = Self::default();
        let version = movie.header.version();
        script.read_tags(&movie.tags, version);
        if script.damaged {
            println!("Parts of the game's script could not be read; values read from it may be missing");
        }
        script
    }

    fn read_tags(&mut self, tags: &[swf::Tag], version: u8) {
        for tag in tags {
            match tag {
                swf::Tag::DoAction(bytecode) => self.add(decoder::decode(bytecode, version, &[])),
                swf::Tag::DoInitAction { action_data, .. } => {
                    self.add(decoder::decode(action_data, version, &[]));
                }
                swf::Tag::DefineSprite(sprite) => self.read_tags(&sprite.tags, version),
                _ => {}
            }
        }
    }

    fn add(&mut self, block: decoder::Block) {
        self.damaged |= block.damaged;
        self.frame_statements.extend(block.statements);
        self.functions.extend(block.functions);
    }

    /// The first function called `name`.
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "for reading gameplay values; tests use it already")
    )]
    pub fn function(&self, name: &str) -> Option<&ScriptFunction> {
        self.functions.iter().find(|function| function.name == name)
    }

    /// Every statement: frame scripts first, then each function's.
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "for reading gameplay values; tests use it already")
    )]
    pub fn statements(&self) -> impl Iterator<Item = &Statement> {
        self.frame_statements
            .iter()
            .chain(self.functions.iter().flat_map(|function| &function.statements))
    }

    /// The cards' names as `tart()` lists them, found by shape (function names may be obfuscated):
    /// a list of exactly 22 names (the tarot cards, in `col` order), then, from Wrath of the Lamb
    /// on, elements 63 to 67 of it set one by one.
    pub fn card_names(&self) -> Option<Vec<String>> {
        const FIRST_PLAYING_CARD_INDEX: u8 = 63;
        const PLAYING_CARD_COUNT: u8 = 5;
        self.functions.iter().find_map(|function| {
            let tarot = function
                .statements
                .iter()
                .find_map(|statement| statement_text_list(statement, usize::from(TAROT_COUNT)))?;
            let playing_card = |index: u8| {
                function.statements.iter().find_map(|statement| match statement {
                    Statement::Assign {
                        target: Expression::Member(_, position),
                        value: Expression::Text(name),
                    } if position.number() == Some(f64::from(index)) => Some(name.clone()),
                    _ => None,
                })
            };
            let playing = (0..PLAYING_CARD_COUNT).map_while(|offset| playing_card(FIRST_PLAYING_CARD_INDEX + offset));
            Some(tarot.into_iter().chain(playing).collect())
        })
    }

    /// The trinkets' names, by `col` less the first trinket's (the order of `Trinket::ALL`): the only
    /// list of exactly that many texts in the movie, the one `pillc()` names a trinket taken from.
    /// Empty if there is none.
    pub fn trinket_names(&self) -> Vec<String> {
        self.functions
            .iter()
            .flat_map(|function| &function.statements)
            .find_map(|statement| statement_text_list(statement, Trinket::ALL.len()))
            .unwrap_or_default()
    }

    /// The fortunes the fortune machine tells, in the script's order: the only list of exactly
    /// [`FORTUNE_COUNT`] texts in the movie. Empty if there is none.
    pub fn fortune_messages(&self) -> Vec<String> {
        self.functions
            .iter()
            .flat_map(|function| &function.statements)
            .chain(&self.frame_statements)
            .find_map(|statement| statement_text_list(statement, usize::from(FORTUNE_COUNT)))
            .unwrap_or_default()
    }

    /// The items' names, by item number (the list starts with a placeholder for number 0); empty
    /// if the movie has no such list.
    pub fn item_names(&self) -> Vec<String> {
        const ITEM_NAMES_VARIABLE: &str = "st1";
        self.array_assigned_to(ITEM_NAMES_VARIABLE)
            .map(|names| {
                names
                    .iter()
                    .map(|name| match name {
                        Expression::Text(text) => text.clone(),
                        _ => String::new(),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The elements of the first array literal assigned to the variable `name` (`sizes = [...]`).
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "for reading gameplay values; tests use it already")
    )]
    pub fn array_assigned_to(&self, name: &str) -> Option<&[Expression]> {
        self.statements().find_map(|statement| match statement {
            Statement::Assign {
                target,
                value: Expression::Array(elements),
            } if target.name() == Some(name) => Some(elements.as_slice()),
            _ => None,
        })
    }
}

#[cfg(test)]
mod tests;

/// The first list of exactly `length` texts anywhere in `statement`.
fn statement_text_list(statement: &Statement, length: usize) -> Option<Vec<String>> {
    match statement {
        Statement::Assign { target, value } => text_list(target, length).or_else(|| text_list(value, length)),
        Statement::Evaluate(expression) | Statement::BranchUnless(expression) | Statement::Return(expression) => {
            text_list(expression, length)
        }
    }
}

fn text_list(expression: &Expression, length: usize) -> Option<Vec<String>> {
    match expression {
        Expression::Array(elements) if elements.len() == length => elements
            .iter()
            .map(|element| match element {
                Expression::Text(text) => Some(text.clone()),
                _ => None,
            })
            .collect(),
        Expression::Array(elements) | Expression::Call(_, elements) => {
            elements.iter().find_map(|element| text_list(element, length))
        }
        Expression::Member(object, member) | Expression::Binary(_, object, member) => {
            text_list(object, length).or_else(|| text_list(member, length))
        }
        Expression::Not(operand) => text_list(operand, length),
        Expression::MethodCall(object, method, arguments) => text_list(object, length)
            .or_else(|| text_list(method, length))
            .or_else(|| arguments.iter().find_map(|argument| text_list(argument, length))),
        _ => None,
    }
}
