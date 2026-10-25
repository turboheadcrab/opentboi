use std::path::{Path, PathBuf};

use super::original_side::shape_of;
use super::report::Report;
use super::rust_side::RustShapes;
use super::{Shape, summary};
use crate::game_assets::script::{Expression, GameScript, Operator, Statement};
use crate::game_assets::{EXTRACTED_SWF_FILE_NAME, cache_directory};

const FRAME_DISTANCE: f64 = 7.0;

#[allow(clippy::unnecessary_box_returns, reason = "operands of expression trees are boxed")]
fn variable(name: &str) -> Box<Expression> {
    Box::new(Expression::Variable(name.to_owned()))
}

#[allow(clippy::unnecessary_box_returns, reason = "operands of expression trees are boxed")]
fn number(value: f64) -> Box<Expression> {
    Box::new(Expression::Number(value))
}

fn rust_shapes_of(source: &str) -> RustShapes {
    let directory = std::env::temp_dir().join("opentboi-primitive-tests");
    std::fs::create_dir_all(&directory).expect("a temporary directory");
    let path = directory.join(format!(
        "{:x}.rs",
        source.len() * 31 + source.bytes().map(usize::from).sum::<usize>()
    ));
    std::fs::write(&path, source).expect("a temporary file");
    RustShapes::scan(&[path], &directory)
}

fn original(statements: Vec<Statement>) -> GameScript {
    GameScript {
        frame_statements: statements,
        ..GameScript::default()
    }
}

#[test]
fn comparisons_and_operand_order_do_not_matter() {
    let ours = Shape::Operation(
        Operator::Greater,
        Box::new(Shape::Wildcard),
        Box::new(Shape::Number(4.0)),
    )
    .normalized();
    let theirs = Shape::Operation(Operator::Less, Box::new(Shape::Number(4.0)), Box::new(Shape::Wildcard)).normalized();
    assert_eq!(ours, theirs);
    let product = Shape::Operation(
        Operator::Multiply,
        Box::new(Shape::Wildcard),
        Box::new(Shape::Number(3.0)),
    );
    let swapped = Shape::Operation(
        Operator::Multiply,
        Box::new(Shape::Number(3.0)),
        Box::new(Shape::Wildcard),
    );
    assert_eq!(product.normalized(), swapped.normalized());
}

#[test]
fn names_and_text_of_the_original_never_reach_the_shape() {
    let expression = Expression::Binary(Operator::Subtract, variable("secret_name"), number(FRAME_DISTANCE));
    let rendered = shape_of(&expression).normalized().render(true);
    assert_eq!(rendered, "(_ Subtract 7)");
}

#[test]
fn a_common_number_alone_says_nothing() {
    let sum = |number| {
        Shape::Operation(
            Operator::Add,
            Box::new(Shape::Wildcard),
            Box::new(Shape::Number(number)),
        )
    };
    let scaled = |inner| Shape::Operation(Operator::Multiply, Box::new(inner), Box::new(Shape::Wildcard));
    let (weak, telling) = (scaled(sum(1.0)), scaled(sum(FRAME_DISTANCE)));
    assert!(!weak.is_telling());
    assert!(telling.is_telling());
}

#[test]
fn a_computation_in_our_code_is_found_whatever_it_is_called_and_however_it_is_written() {
    let rust =
        rust_shapes_of("const GAP: f32 = 7.0;\nfn ours(a: f32, b: f32) -> bool { 3.0 > a - b + GAP || a <= 1.0 }\n");
    let script = original(vec![Statement::BranchUnless(Expression::Binary(
        Operator::Less,
        Box::new(Expression::Binary(
            Operator::Add,
            Box::new(Expression::Binary(Operator::Subtract, variable("x"), variable("y"))),
            number(FRAME_DISTANCE),
        )),
        number(3.0),
    ))]);
    let report = Report::build(&script, &rust);
    let function = &report.functions[0];
    assert_eq!(
        function.found(),
        1,
        "the comparison with its sum is the same computation"
    );
    assert_eq!(
        function.best_site().map(|(site, _)| site.function.as_str()),
        Some("ours")
    );
}

#[test]
fn a_different_number_is_only_the_same_structure() {
    let rust = rust_shapes_of("fn ours(a: f32, b: f32, c: f32) -> f32 { (a - b) * c + 9.0 }\n");
    let sum = Expression::Binary(
        Operator::Add,
        Box::new(Expression::Binary(
            Operator::Multiply,
            Box::new(Expression::Binary(Operator::Subtract, variable("a"), variable("b"))),
            variable("c"),
        )),
        number(FRAME_DISTANCE),
    );
    let report = Report::build(&original(vec![Statement::Return(sum)]), &rust);
    let function = &report.functions[0];
    assert_eq!(function.found(), 0);
    assert_eq!(function.count(super::report::Finding::SameStructure), 1);
}

fn rust_sources(directory: &Path, files: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(directory).expect("source directory").flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_owned();
        if path.is_dir() {
            rust_sources(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs")
            && !name.starts_with("tests")
            && !name.ends_with("_tests.rs")
            && !path.ends_with("script/primitives.rs")
            && !path.to_string_lossy().contains("/primitives/")
        {
            files.push(path);
        }
    }
}

/// `cargo test original_arithmetic_in_our_code -- --ignored --nocapture`
#[test]
#[ignore = "requires the game SWF extracted into the user cache; prints a report for local use"]
fn original_arithmetic_in_our_code() {
    let swf_path = cache_directory()
        .expect("no cache directory")
        .join(EXTRACTED_SWF_FILE_NAME);
    let buffer = swf::decompress_swf(&*std::fs::read(swf_path).expect("extracted SWF should exist"))
        .expect("SWF should decompress");
    let movie = swf::parse_swf(&buffer).expect("SWF should parse");
    let script = GameScript::from_movie(&movie);
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    rust_sources(&root.join("src"), &mut files);
    let rust = RustShapes::scan(&files, root);
    let report = Report::build(&script, &rust);
    summary::print(&report, &super::scoped::judge(&script, &rust, &report));
}
