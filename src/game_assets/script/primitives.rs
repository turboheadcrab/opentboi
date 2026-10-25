//! Compares the original's arithmetic with ours. Both sides are reduced to the same neutral
//! [`Shape`]: operators and numbers stay, everything that has a name (variables, members, calls,
//! fields, methods) becomes a wildcard, because names differ between the two programs but the
//! computation does not. A statement of the original whose shape also occurs in our Rust is evidence
//! that we implement that very computation (see `docs/original-coverage.md`, "Primitive matching").
//! Shapes carry no names and no text of the original, only operators and numbers.

mod keys;
mod original_side;
mod report;
mod rust_side;
mod scoped;
mod summary;
#[cfg(test)]
mod tests;

use super::expression::Operator;

/// Numbers that appear everywhere and so say nothing about where an expression comes from.
const COMMON_NUMBERS: [f64; 4] = [-1.0, 0.0, 1.0, 2.0];
/// An expression needs this many operators to count as telling at all (`x < 4` is everywhere)...
const MINIMUM_TELLING_OPERATORS: usize = 2;
/// ...and without a telling number, this many (names are wildcards, so few operators alone match anything).
const OPERATORS_WITHOUT_TELLING_NUMBER: usize = 5;

/// The original's sound function plays a name ending in `.mp` as `.mp3`; sound file names are telling.
const SHORT_MP3_EXTENSION: &str = ".mp";
const MP3_EXTENSION: &str = ".mp3";
const SOUND_EXTENSIONS: [&str; 2] = [".wav", MP3_EXTENSION];

/// How numbers are written when a shape is turned into a key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Numbers {
    /// As `f32` (the game's precision), so `f64` constants of the original equal ours.
    Exact,
    /// Only their significant digits: 30 and 0.3 are the same number written in other units.
    Scaled,
    /// Every number is a wildcard: the same structure.
    Hidden,
}

/// A computation with its names removed.
#[derive(Clone, Debug, PartialEq)]
pub enum Shape {
    Number(f64),
    Text(String),
    /// Anything with a name or that is not arithmetic.
    Wildcard,
    Not(Box<Shape>),
    Operation(Operator, Box<Shape>, Box<Shape>),
    /// A whole number below the operand: AVM1's own random-number instruction, our `Random::below`.
    Random(Box<Shape>),
    /// A fraction in 0..1: Flash's random fraction, our `Random::next_unit`.
    RandomUnit,
    /// A distance in a random direction: the original's own `crand()` (its second call hands out the
    /// other axis), our `Random::direction`.
    RandomDirection(Box<Shape>),
}

impl Shape {
    /// A text, with a sound name's short `.mp` written out as the `.mp3` it plays.
    pub fn text(text: &str) -> Self {
        if text.ends_with(SHORT_MP3_EXTENSION) {
            Self::Text(format!("{text}3"))
        } else {
            Self::Text(text.to_owned())
        }
    }

    /// A sound's file name, or the end of one (`".wav"` after a random variant number).
    pub fn is_sound_name(&self) -> bool {
        matches!(self, Self::Text(text) if SOUND_EXTENSIONS.iter().any(|extension| text.ends_with(extension)))
    }

    /// Applies the identities both languages share: `a > b` is `b < a`, constants are folded, and
    /// the operands of commutative operators are put in a fixed order.
    pub fn normalized(self) -> Self {
        match self {
            Self::Not(inner) => Self::Not(Box::new(inner.normalized())),
            Self::Random(inner) => Self::Random(Box::new(inner.normalized())),
            Self::RandomDirection(inner) => Self::RandomDirection(Box::new(inner.normalized())),
            Self::Operation(operator @ (Operator::Add | Operator::Multiply), left, right) => {
                Self::regrouped(operator, *left, *right)
            }
            Self::Operation(operator, left, right) => {
                let (left, right) = (left.normalized(), right.normalized());
                Self::operation(operator, left, right)
            }
            other => other,
        }
    }

    /// A chain of one commutative operator with its numbers folded into one, whatever the grouping:
    /// `(pi * x) * 2` and `x * (2 * pi)` are the same shape.
    fn regrouped(operator: Operator, left: Self, right: Self) -> Self {
        let mut operands = Vec::new();
        for operand in [left, right] {
            operand.flatten_into(operator, &mut operands);
        }
        let numbers: Vec<f64> = operands
            .iter()
            .filter_map(|operand| {
                if let Self::Number(number) = operand {
                    Some(*number)
                } else {
                    None
                }
            })
            .collect();
        operands.retain(|operand| !matches!(operand, Self::Number(_)));
        if let Some(folded) = numbers
            .into_iter()
            .reduce(|left, right| fold(operator, left, right).unwrap_or(left))
        {
            operands.push(Self::Number(folded));
        }
        operands.sort_by_key(|operand| operand.render(true));
        let mut operands = operands.into_iter();
        let first = operands.next().unwrap_or(Self::Wildcard);
        operands.fold(first, |chain, operand| {
            Self::Operation(operator, Box::new(chain), Box::new(operand))
        })
    }

    fn flatten_into(self, operator: Operator, operands: &mut Vec<Self>) {
        match self.normalized() {
            Self::Operation(inner, left, right) if inner == operator => {
                left.flatten_into(operator, operands);
                right.flatten_into(operator, operands);
            }
            other => operands.push(other),
        }
    }

    /// The shape as text, with its numbers or with every number a wildcard.
    pub fn render(&self, with_numbers: bool) -> String {
        self.render_with(if with_numbers { Numbers::Exact } else { Numbers::Hidden })
    }

    pub fn render_with(&self, numbers: Numbers) -> String {
        match self {
            Self::Number(number) => match numbers {
                Numbers::Exact => format!("{}", game_precision(*number)),
                Numbers::Scaled => significant_digits(*number),
                Numbers::Hidden => "_".to_owned(),
            },
            Self::Wildcard => "_".to_owned(),
            Self::Text(text) => format!("{text:?}"),
            Self::Not(inner) => format!("!{}", inner.render_with(numbers)),
            Self::Random(limit) => format!("random({})", limit.render_with(numbers)),
            Self::RandomUnit => "random()".to_owned(),
            Self::RandomDirection(length) => format!("direction({})", length.render_with(numbers)),
            Self::Operation(operator, left, right) => {
                format!(
                    "({} {operator:?} {})",
                    left.render_with(numbers),
                    right.render_with(numbers)
                )
            }
        }
    }

    /// Whether a number or a random primitive is in it.
    pub fn has_number(&self) -> bool {
        match self {
            Self::Number(_) | Self::Random(_) | Self::RandomUnit | Self::RandomDirection(_) => true,
            Self::Not(inner) => inner.has_number(),
            Self::Operation(_, left, right) => left.has_number() || right.has_number(),
            _ => false,
        }
    }

    /// Arithmetic, comparison and logic operators; a negation alone does not count (the bytecode
    /// writes every "not equal" and "at most" as one, and `switch` cases become negated equals).
    pub fn operator_count(&self) -> usize {
        match self {
            Self::Not(inner) => inner.operator_count(),
            Self::Operation(_, left, right) => 1 + left.operator_count() + right.operator_count(),
            Self::Random(limit) | Self::RandomDirection(limit) => 1 + limit.operator_count(),
            _ => 0,
        }
    }

    /// Operators other than the random primitive: a roll like "one in three" is in every big
    /// function and says nothing about where a computation went.
    fn arithmetic_operator_count(&self) -> usize {
        match self {
            Self::Not(inner) | Self::Random(inner) | Self::RandomDirection(inner) => inner.arithmetic_operator_count(),
            Self::Operation(_, left, right) => 1 + left.arithmetic_operator_count() + right.arithmetic_operator_count(),
            _ => 0,
        }
    }

    pub fn telling_number_count(&self) -> usize {
        match self {
            Self::Number(number) => usize::from(!COMMON_NUMBERS.contains(number)),
            Self::Not(inner) | Self::Random(inner) | Self::RandomDirection(inner) => inner.telling_number_count(),
            Self::Operation(_, left, right) => left.telling_number_count() + right.telling_number_count(),
            _ => 0,
        }
    }

    /// Whether finding this shape elsewhere means something: it has two operators and a number other
    /// than the common ones, or at least five operators.
    pub fn is_telling(&self) -> bool {
        let operators = self.arithmetic_operator_count();
        operators >= MINIMUM_TELLING_OPERATORS
            && (self.telling_number_count() >= 1 || operators >= OPERATORS_WITHOUT_TELLING_NUMBER)
    }

    /// This shape and every shape inside it, largest first.
    pub fn parts(&self) -> Vec<&Self> {
        let mut parts = vec![self];
        match self {
            Self::Not(inner) | Self::Random(inner) | Self::RandomDirection(inner) => parts.extend(inner.parts()),
            Self::Operation(_, left, right) => {
                parts.extend(left.parts());
                parts.extend(right.parts());
            }
            _ => {}
        }
        parts
    }
}

/// The number as the game holds it: the original's `f64` constants written as our `f32`.
#[allow(
    clippy::cast_possible_truncation,
    reason = "rounding to the game's precision is the point"
)]
fn game_precision(number: f64) -> f32 {
    number as f32
}

/// The digits of a number without its decimal point or sign of scale: 30, 0.3 and 3 are all "3".
fn significant_digits(number: f64) -> String {
    let scientific = format!("{:e}", game_precision(number));
    let mantissa = scientific.split('e').next().unwrap_or_default();
    mantissa.replace('.', "")
}

pub(super) fn fold(operator: Operator, left: f64, right: f64) -> Option<f64> {
    match operator {
        Operator::Add => Some(left + right),
        Operator::Subtract => Some(left - right),
        Operator::Multiply => Some(left * right),
        Operator::Divide if right != 0.0 => Some(left / right),
        _ => None,
    }
}
