//! Method calls that stand for a primitive: our random generator's, and a range's `contains`.

use syn::{Expr, ExprMethodCall};

use super::resolver::{Bounds, Resolver};
use super::shape_of;
use crate::game_assets::script::Operator;
use crate::game_assets::script::primitives::Shape;

/// Our random generator's whole number below a limit and fraction in 0..1, the counterparts of
/// AVM1's random-number instruction and Flash's random fraction.
const RANDOM_BELOW: &str = "below";
const RANDOM_UNIT: &str = "next_unit";
/// Its index into a list (a random number too) and its distance in a random direction (`crand()`).
const RANDOM_INDEX: &str = "index";
const RANDOM_SMALL: &str = "below_u8";
const RANDOM_DIRECTION: &str = "direction";
/// `range.contains(&x)`.
const CONTAINS: &str = "contains";

/// Our random generator's calls, and a range's `contains` written as the comparisons it makes.
pub(super) fn method_shape(call: &ExprMethodCall, resolver: &Resolver) -> Shape {
    match (call.method.to_string().as_str(), call.args.len()) {
        (RANDOM_BELOW | RANDOM_INDEX | RANDOM_SMALL, 1) => Shape::Random(Box::new(shape_of(&call.args[0], resolver))),
        (RANDOM_DIRECTION, 1) => Shape::RandomDirection(Box::new(shape_of(&call.args[0], resolver))),
        (RANDOM_UNIT, 0) => Shape::RandomUnit,
        (CONTAINS, 1) => range_of(&call.receiver, resolver).map_or(Shape::Wildcard, |bounds| {
            let value = shape_of(&call.args[0], resolver);
            contains_shape(bounds, value)
        }),
        _ => Shape::Wildcard,
    }
}

/// A range written out, or the name of a range constant.
fn range_of(expression: &Expr, resolver: &Resolver) -> Option<Bounds> {
    match expression {
        Expr::Paren(inner) => range_of(&inner.expr, resolver),
        Expr::Reference(inner) => range_of(&inner.expr, resolver),
        Expr::Range(range) => resolver.bounds(range),
        Expr::Path(path) => resolver
            .ranges
            .get(&path.path.segments.last()?.ident.to_string())
            .copied(),
        _ => None,
    }
}

/// `start <= value && value <= end` (or `< end`), in the bytecode's terms: `!(value < start)` and
/// `!(end < value)`.
fn contains_shape(bounds: Bounds, value: Shape) -> Shape {
    let from_start = Shape::Not(Box::new(Shape::Operation(
        Operator::Less,
        Box::new(value.clone()),
        Box::new(Shape::Number(bounds.start)),
    )));
    let to_end = if bounds.inclusive {
        Shape::Not(Box::new(Shape::Operation(
            Operator::Less,
            Box::new(Shape::Number(bounds.end)),
            Box::new(value),
        )))
    } else {
        Shape::Operation(Operator::Less, Box::new(value), Box::new(Shape::Number(bounds.end)))
    };
    Shape::Operation(Operator::LogicalAnd, Box::new(from_start), Box::new(to_end))
}
