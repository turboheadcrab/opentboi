//! Keys for looking a shape up: a test the same whichever way round it is written.

use super::{Numbers, Shape, fold};
use crate::game_assets::script::Operator;

impl Shape {
    /// The same test whichever way round it is written: a condition and its negation decide the same
    /// thing (only the branch taken differs), and negations are pushed inside `&&` and `||`.
    pub fn test_key(&self, numbers: Numbers) -> String {
        let plain = self.clone().without_outer_negations().render_with(numbers);
        let negated = Self::Not(Box::new(self.clone()))
            .without_outer_negations()
            .render_with(numbers);
        plain.min(negated)
    }

    /// Negations moved inward (De Morgan) and double ones dropped, then put in order again.
    fn without_outer_negations(self) -> Self {
        match self {
            Self::Not(inner) => match *inner {
                Self::Not(twice) => twice.without_outer_negations(),
                Self::Operation(operator @ (Operator::LogicalAnd | Operator::LogicalOr), left, right) => {
                    let flipped = if operator == Operator::LogicalAnd {
                        Operator::LogicalOr
                    } else {
                        Operator::LogicalAnd
                    };
                    let left = Self::Not(left).without_outer_negations();
                    let right = Self::Not(right).without_outer_negations();
                    Self::operation(flipped, left, right)
                }
                other => Self::Not(Box::new(other)),
            },
            Self::Operation(operator @ (Operator::LogicalAnd | Operator::LogicalOr), left, right) => Self::operation(
                operator,
                left.without_outer_negations(),
                right.without_outer_negations(),
            ),
            other => other,
        }
    }

    pub(super) fn operation(operator: Operator, left: Self, right: Self) -> Self {
        if let (Self::Number(left), Self::Number(right)) = (&left, &right)
            && let Some(folded) = fold(operator, *left, *right)
        {
            return Self::Number(folded);
        }
        match operator {
            Operator::Greater => Self::Operation(Operator::Less, Box::new(right), Box::new(left)),
            Operator::Add
            | Operator::Multiply
            | Operator::Equals
            | Operator::StrictEquals
            | Operator::LogicalAnd
            | Operator::LogicalOr
            | Operator::BitAnd
            | Operator::BitOr => {
                let (first, second) = if left.render(true) <= right.render(true) {
                    (left, right)
                } else {
                    (right, left)
                };
                Self::Operation(operator, Box::new(first), Box::new(second))
            }
            _ => Self::Operation(operator, Box::new(left), Box::new(right)),
        }
    }
}
