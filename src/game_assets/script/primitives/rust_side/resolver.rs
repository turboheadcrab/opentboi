//! The numbers and ranges our names stand for: numeric `const NAME: T = expression;` items (in a
//! module or a function body), range constants (`-0.5..=0.0`), and struct fields that are given a
//! number (the gameplay parameters). The bare name is the key; a field given different numbers in
//! different places is dropped. Also the sound each `SoundEvent` plays, read from the sound
//! catalog's `(SoundEvent::Name, &["file", ...])` entries.

use std::collections::HashMap;

use syn::visit::Visit;
use syn::{Expr, ExprRange, ExprStruct, ExprTuple, ImplItemConst, ItemConst, Lit, Member, Path, RangeLimits};

use super::shape_of;
use crate::game_assets::script::Operator;
use crate::game_assets::script::primitives::Shape;

/// Gameplay parameter structs are named `...Parameters`.
const PARAMETERS_SUFFIX: &str = "Parameters";
/// The type whose variants name the sounds the game plays.
const SOUND_EVENT_TYPE: &str = "SoundEvent";

/// A range's bounds, and whether the end is included.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub start: f64,
    pub end: f64,
    pub inclusive: bool,
}

/// The standard library's numbers our code uses (`std::f32::consts::TAU`).
const STANDARD_CONSTANTS: [(&str, f64); 2] = [("PI", std::f64::consts::PI), ("TAU", std::f64::consts::TAU)];

pub struct Resolver {
    pub constants: HashMap<String, f64>,
    pub ranges: HashMap<String, Bounds>,
    fields: HashMap<String, Option<f64>>,
    /// Each sound event's file name as the original builds it.
    sounds: HashMap<String, Shape>,
}

impl Default for Resolver {
    fn default() -> Self {
        Self {
            constants: STANDARD_CONSTANTS
                .iter()
                .map(|&(name, number)| (name.to_owned(), number))
                .collect(),
            ranges: HashMap::new(),
            fields: HashMap::new(),
            sounds: HashMap::new(),
        }
    }
}

impl Resolver {
    /// `SoundEvent::Name` as the sound's file name.
    pub fn sound(&self, path: &Path) -> Option<Shape> {
        let mut segments = path.segments.iter().rev();
        let variant = segments.next()?;
        (segments.next()?.ident == SOUND_EVENT_TYPE)
            .then(|| self.sounds.get(&variant.ident.to_string()).cloned())
            .flatten()
    }

    pub fn field(&self, name: &str) -> Option<f64> {
        self.fields.get(name).copied().flatten()
    }

    /// `start..end` or `start..=end` with numbers (or names of numbers) at both ends.
    pub fn bounds(&self, range: &ExprRange) -> Option<Bounds> {
        let number = |end: &Option<Box<Expr>>| match shape_of(end.as_deref()?, self).normalized() {
            Shape::Number(number) => Some(number),
            _ => None,
        };
        Some(Bounds {
            start: number(&range.start)?,
            end: number(&range.end)?,
            inclusive: matches!(range.limits, RangeLimits::Closed(_)),
        })
    }
}

#[derive(Default)]
pub struct ResolverCollector {
    pub resolver: Resolver,
}

impl ResolverCollector {
    fn constant(&mut self, name: String, expression: &Expr) {
        if let Expr::Range(range) = expression
            && let Some(bounds) = self.resolver.bounds(range)
        {
            self.resolver.ranges.insert(name, bounds);
        } else if let Shape::Number(number) = shape_of(expression, &self.resolver).normalized() {
            self.resolver.constants.insert(name, number);
        }
    }
}

impl<'ast> Visit<'ast> for ResolverCollector {
    fn visit_item_const(&mut self, item: &'ast ItemConst) {
        self.constant(item.ident.to_string(), &item.expr);
        syn::visit::visit_item_const(self, item);
    }

    /// A unit type's own constants (`Hearts::ONE`).
    fn visit_impl_item_const(&mut self, item: &'ast ImplItemConst) {
        self.constant(item.ident.to_string(), &item.expr);
        syn::visit::visit_impl_item_const(self, item);
    }

    fn visit_expr_tuple(&mut self, tuple: &'ast ExprTuple) {
        if let [Expr::Path(event), files] = &tuple.elems.iter().collect::<Vec<_>>()[..]
            && event.path.segments.len() == 2
            && event.path.segments[0].ident == SOUND_EVENT_TYPE
            && let Some(name) = sound_name(&string_literals(files))
        {
            self.resolver
                .sounds
                .insert(event.path.segments[1].ident.to_string(), name);
        }
        syn::visit::visit_expr_tuple(self, tuple);
    }

    /// Only the parameter structs' fields are constants; other structs start state off (a count at 0).
    fn visit_expr_struct(&mut self, item: &'ast ExprStruct) {
        let is_parameters = item
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident.to_string().ends_with(PARAMETERS_SUFFIX));
        for field in item.fields.iter().filter(|_| is_parameters) {
            let Member::Named(name) = &field.member else { continue };
            let Shape::Number(number) = shape_of(&field.expr, &self.resolver).normalized() else {
                continue;
            };
            let known = self.resolver.fields.entry(name.to_string()).or_insert(Some(number));
            if *known != Some(number) {
                *known = None;
            }
        }
        syn::visit::visit_expr_struct(self, item);
    }
}

/// The strings of `&["a", "b"]`.
fn string_literals(expression: &Expr) -> Vec<String> {
    let array = match expression {
        Expr::Reference(reference) => reference.expr.as_ref(),
        other => other,
    };
    let Expr::Array(array) = array else { return Vec::new() };
    array
        .elems
        .iter()
        .filter_map(|element| match element {
            Expr::Lit(literal) => match &literal.lit {
                Lit::Str(text) => Some(text.value()),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

/// One file, or variants numbered 0, 1, … between a shared start and end: the name the original
/// builds, the start, a random number below the count, then the end.
fn sound_name(files: &[String]) -> Option<Shape> {
    let first = files.first()?;
    if files.len() == 1 {
        return Some(Shape::text(first));
    }
    let extension = first.rfind('.')?;
    let number_at = first[..extension].rfind(|character: char| character.is_ascii_digit())?;
    let (start, end) = (&first[..number_at], &first[number_at + 1..]);
    let numbered = files
        .iter()
        .enumerate()
        .all(|(number, file)| *file == format!("{start}{number}{end}"));
    numbered.then(|| {
        let variant = Shape::Operation(
            Operator::Add,
            Box::new(Shape::text(start)),
            Box::new(Shape::Random(Box::new(Shape::Number(files.len() as f64)))),
        );
        Shape::Operation(Operator::Add, Box::new(variant), Box::new(Shape::text(end))).normalized()
    })
}
