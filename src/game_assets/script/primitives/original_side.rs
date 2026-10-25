//! The original's statements as [`Shape`]s.

use super::Shape;
use crate::game_assets::script::{Expression, Statement};

/// The name of the random-number instruction and of the random-fraction method.
const RANDOM: &str = "random";
/// The original's own function for a distance in a random direction.
const RANDOM_DIRECTION: &str = "crand";

pub fn shape_of(expression: &Expression) -> Shape {
    match expression {
        Expression::Number(number) => Shape::Number(*number),
        Expression::Text(text) => Shape::text(text),
        Expression::Not(inner) => Shape::Not(Box::new(shape_of(inner))),
        Expression::Binary(operator, left, right) => {
            Shape::Operation(*operator, Box::new(shape_of(left)), Box::new(shape_of(right)))
        }
        // The decoder names AVM1's random-number instruction like a call.
        Expression::Call(name, arguments) if name == RANDOM && arguments.len() == 1 => {
            Shape::Random(Box::new(shape_of(&arguments[0])))
        }
        Expression::Call(name, arguments) if name == RANDOM_DIRECTION && arguments.len() == 1 => {
            Shape::RandomDirection(Box::new(shape_of(&arguments[0])))
        }
        Expression::MethodCall(_, method, arguments)
            if arguments.is_empty() && matches!(method.as_ref(), Expression::Text(name) if name == RANDOM) =>
        {
            Shape::RandomUnit
        }
        _ => Shape::Wildcard,
    }
}

/// The computations a statement performs: what it assigns, tests, returns or passes to a call.
pub fn roots_of(statement: &Statement) -> Vec<Shape> {
    let expressions: Vec<&Expression> = match statement {
        Statement::Assign { value, .. } => vec![value],
        Statement::BranchUnless(condition) | Statement::Return(condition) => vec![condition],
        Statement::Evaluate(Expression::Call(_, arguments) | Expression::MethodCall(_, _, arguments)) => {
            arguments.iter().collect()
        }
        Statement::Evaluate(_) => Vec::new(),
    };
    expressions
        .into_iter()
        .map(|expression| shape_of(expression).normalized())
        .collect()
}
