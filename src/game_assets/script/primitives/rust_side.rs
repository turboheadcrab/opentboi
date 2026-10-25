//! Our Rust sources as [`Shape`]s: every expression with an operator in it, and every telling
//! number, with the function that holds it; and the original functions each of our functions names
//! in its doc comment (`` `slotf()` ``), its counterparts. Named constants with a number as their
//! value count as that number.

mod methods;
mod resolver;

use std::path::{Path, PathBuf};

use syn::visit::Visit;
use syn::{Attribute, BinOp, Expr, ExprBinary, ImplItemFn, ItemFn, Lit, Member, UnOp};

use super::Shape;
use crate::game_assets::script::Operator;
use methods::method_shape;
use resolver::{Resolver, ResolverCollector};

/// Constants may be defined from other constants, defined later or in another file; a few passes
/// over all files resolve the chains we have.
const CONSTANT_PASSES: usize = 4;
/// `Frames::new(30.0)` and `Pixels::new(4.0).get()` are the number inside.
const CONSTRUCTOR: &str = "new";
const CONVERSION: &str = "from";
const GETTER: &str = "get";
const DOC_ATTRIBUTE: &str = "doc";

/// Where in our code a shape was found.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Site {
    pub file: PathBuf,
    pub function: String,
}

/// Every operator-bearing shape in the files, with where it occurs.
#[derive(Default)]
pub struct RustShapes {
    pub found: Vec<(Shape, Site)>,
    /// Original function names our functions' doc comments cite, with the citing function.
    pub citations: Vec<(String, Site)>,
}

impl RustShapes {
    pub fn scan(files: &[PathBuf], root: &Path) -> Self {
        let parsed: Vec<(PathBuf, syn::File)> = files
            .iter()
            .filter_map(|path| {
                let text = std::fs::read_to_string(path).ok()?;
                Some((
                    path.strip_prefix(root).unwrap_or(path).to_path_buf(),
                    syn::parse_file(&text).ok()?,
                ))
            })
            .collect();
        let mut collector = ResolverCollector::default();
        for _ in 0..CONSTANT_PASSES {
            for (_, file) in &parsed {
                collector.visit_file(file);
            }
        }
        let mut shapes = Self::default();
        for (path, file) in &parsed {
            let mut visitor = ShapeCollector {
                resolver: &collector.resolver,
                file: path,
                functions: Vec::new(),
                found: &mut shapes.found,
                citations: &mut shapes.citations,
            };
            visitor.visit_file(file);
        }
        shapes
    }
}

struct ShapeCollector<'scan> {
    resolver: &'scan Resolver,
    file: &'scan Path,
    functions: Vec<String>,
    found: &'scan mut Vec<(Shape, Site)>,
    citations: &'scan mut Vec<(String, Site)>,
}

impl ShapeCollector<'_> {
    fn site(&self) -> Site {
        Site {
            file: self.file.to_path_buf(),
            function: self.functions.last().cloned().unwrap_or_default(),
        }
    }

    fn enter(&mut self, name: String, attributes: &[Attribute]) {
        self.functions.push(name);
        let site = self.site();
        for cited in cited_functions(attributes) {
            self.citations.push((cited, site.clone()));
        }
    }
}

/// Names written as `` `name(`` in doc comments: how our comments name the original's functions.
fn cited_functions(attributes: &[Attribute]) -> Vec<String> {
    let mut names = Vec::new();
    for attribute in attributes
        .iter()
        .filter(|attribute| attribute.path().is_ident(DOC_ATTRIBUTE))
    {
        let syn::Meta::NameValue(pair) = &attribute.meta else {
            continue;
        };
        let Expr::Lit(literal) = &pair.value else { continue };
        let Lit::Str(text) = &literal.lit else { continue };
        for quoted in text.value().split('`').skip(1).step_by(2) {
            if let Some((name, _)) = quoted.split_once('(')
                && !name.is_empty()
                && name
                    .chars()
                    .all(|character| character.is_alphanumeric() || character == '_')
            {
                names.push(name.to_owned());
            }
        }
    }
    names
}

impl<'ast> Visit<'ast> for ShapeCollector<'_> {
    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        self.enter(function.sig.ident.to_string(), &function.attrs);
        syn::visit::visit_item_fn(self, function);
        self.functions.pop();
    }

    fn visit_impl_item_fn(&mut self, function: &'ast ImplItemFn) {
        self.enter(function.sig.ident.to_string(), &function.attrs);
        syn::visit::visit_impl_item_fn(self, function);
        self.functions.pop();
    }

    fn visit_expr(&mut self, expression: &'ast Expr) {
        let shape = shape_of(expression, self.resolver).normalized();
        if shape.operator_count() > 0 || shape.telling_number_count() > 0 || shape.is_sound_name() {
            let site = self.site();
            self.found.push((shape, site));
        }
        syn::visit::visit_expr(self, expression);
    }
}

pub(super) fn shape_of(expression: &Expr, resolver: &Resolver) -> Shape {
    let shape = |inner: &Expr| shape_of(inner, resolver);
    match expression {
        Expr::Lit(literal) => match &literal.lit {
            Lit::Int(number) => number.base10_parse::<f64>().map_or(Shape::Wildcard, Shape::Number),
            Lit::Float(number) => number.base10_parse::<f64>().map_or(Shape::Wildcard, Shape::Number),
            Lit::Str(text) => Shape::text(&text.value()),
            _ => Shape::Wildcard,
        },
        Expr::Paren(inner) => shape(&inner.expr),
        Expr::Group(inner) => shape(&inner.expr),
        Expr::Cast(inner) => shape(&inner.expr),
        Expr::Reference(inner) => shape(&inner.expr),
        // `SoundEvent::Name`: the sound it plays; `NAME` or `module::NAME`: the last segment is the
        // constant's own name.
        Expr::Path(path) => resolver.sound(&path.path).unwrap_or_else(|| {
            path.path
                .segments
                .last()
                .and_then(|segment| resolver.constants.get(&segment.ident.to_string()))
                .map_or(Shape::Wildcard, |number| Shape::Number(*number))
        }),
        Expr::Field(field) => match &field.member {
            Member::Named(name) => resolver.field(&name.to_string()).map_or(Shape::Wildcard, Shape::Number),
            Member::Unnamed(_) => Shape::Wildcard,
        },
        Expr::Call(call) if is_constructor(&call.func) && call.args.len() == 1 => shape(&call.args[0]),
        Expr::MethodCall(call) if call.method == GETTER && call.args.is_empty() => shape(&call.receiver),
        Expr::MethodCall(call) => method_shape(call, resolver),
        Expr::Unary(unary) => match unary.op {
            UnOp::Not(_) => Shape::Not(Box::new(shape(&unary.expr))),
            UnOp::Neg(_) => Shape::Operation(
                Operator::Multiply,
                Box::new(Shape::Number(-1.0)),
                Box::new(shape(&unary.expr)),
            ),
            _ => Shape::Wildcard,
        },
        Expr::Binary(binary) => binary_shape(binary, resolver),
        _ => Shape::Wildcard,
    }
}

/// `<=`, `>=` and `!=` become the negation of the opposite comparison, as the original's bytecode
/// has only `<`, `>` and `==`; `x += y` is the operation without its assignment.
fn binary_shape(binary: &ExprBinary, resolver: &Resolver) -> Shape {
    let (left, right) = (shape_of(&binary.left, resolver), shape_of(&binary.right, resolver));
    let operation = |operator| Shape::Operation(operator, Box::new(left.clone()), Box::new(right.clone()));
    let negated = |operator| Shape::Not(Box::new(operation(operator)));
    match binary.op {
        BinOp::Add(_) | BinOp::AddAssign(_) => operation(Operator::Add),
        BinOp::Sub(_) | BinOp::SubAssign(_) => operation(Operator::Subtract),
        BinOp::Mul(_) | BinOp::MulAssign(_) => operation(Operator::Multiply),
        BinOp::Div(_) | BinOp::DivAssign(_) => operation(Operator::Divide),
        BinOp::Rem(_) | BinOp::RemAssign(_) => operation(Operator::Modulo),
        BinOp::Lt(_) => operation(Operator::Less),
        BinOp::Gt(_) => operation(Operator::Greater),
        BinOp::Le(_) => negated(Operator::Greater),
        BinOp::Ge(_) => negated(Operator::Less),
        BinOp::Eq(_) => operation(Operator::Equals),
        BinOp::Ne(_) => negated(Operator::Equals),
        BinOp::And(_) => operation(Operator::LogicalAnd),
        BinOp::Or(_) => operation(Operator::LogicalOr),
        BinOp::BitAnd(_) => operation(Operator::BitAnd),
        BinOp::BitOr(_) => operation(Operator::BitOr),
        _ => Shape::Wildcard,
    }
}

/// `Type::new`, which wraps one number in a unit type, `Type::from`, which converts one, and a
/// tuple struct's own name (`PixelsPerFrame(5.0)`).
fn is_constructor(function: &Expr) -> bool {
    matches!(function, Expr::Path(path) if path.path.segments.last().is_some_and(|segment| {
        segment.ident == CONSTRUCTOR
            || segment.ident == CONVERSION
            || segment.ident.to_string().starts_with(char::is_uppercase)
    }))
}
