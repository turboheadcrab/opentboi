use super::*;
use crate::game_assets::script::{Expression, Operator, ScriptFunction};

/// A Rust source written to a temporary file and scanned like ours.
fn rust_shapes_of(source: &str) -> RustShapes {
    let directory = std::env::temp_dir().join("opentboi-scoped-tests");
    std::fs::create_dir_all(&directory).expect("a temporary directory");
    let path = directory.join(format!(
        "{:x}.rs",
        source.bytes().map(usize::from).sum::<usize>() * 7 + source.len()
    ));
    std::fs::write(&path, source).expect("a temporary file");
    RustShapes::scan(&[path], &directory)
}

#[allow(clippy::unnecessary_box_returns, reason = "operands of expression trees are boxed")]
fn name(text: &str) -> Box<Expression> {
    Box::new(Expression::Variable(text.to_owned()))
}

#[allow(clippy::unnecessary_box_returns, reason = "operands of expression trees are boxed")]
fn number(value: f64) -> Box<Expression> {
    Box::new(Expression::Number(value))
}

#[allow(clippy::unnecessary_box_returns, reason = "operands of expression trees are boxed")]
fn binary(operator: Operator, left: Box<Expression>, right: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Binary(operator, left, right))
}

#[allow(clippy::unnecessary_box_returns, reason = "operands of expression trees are boxed")]
fn random(limit: f64) -> Box<Expression> {
    Box::new(Expression::Call("random".to_owned(), vec![Expression::Number(limit)]))
}

fn assign(value: Expression) -> Statement {
    Statement::Assign {
        target: Expression::Register(1),
        value,
    }
}

fn test(condition: Expression) -> Statement {
    Statement::BranchUnless(condition)
}

/// Judges an original function `roll` against the Rust in `source`.
fn judged(source: &str, statements: Vec<Statement>) -> Vec<ScopedFunction> {
    let script = GameScript {
        functions: vec![ScriptFunction {
            name: "roll".to_owned(),
            statements,
        }],
        ..GameScript::default()
    };
    let rust = rust_shapes_of(source);
    let report = Report::build(&script, &rust);
    judge(&script, &rust, &report)
}

fn findings(function: &ScopedFunction) -> Vec<Finding> {
    function.details.iter().map(|(finding, _)| *finding).collect()
}

#[test]
fn a_function_named_in_a_doc_comment_is_the_counterpart() {
    let source = "const FIRST: u8 = 3;\nconst RESULTS: i32 = 21;\n/// `roll()` without one item.\nfn ours(random: &mut Random) -> u8 { FIRST + random.below(RESULTS) as u8 }\nfn other(random: &mut Random) -> u8 { 3 + random.below(21) as u8 }\n";
    let functions = judged(
        source,
        vec![
            assign(*binary(Operator::Add, number(3.0), random(18.0))),
            assign(*binary(Operator::Add, number(3.0), random(21.0))),
        ],
    );
    assert_eq!(functions.len(), 1);
    assert_eq!(functions[0].counterparts.len(), 1, "only the function that names it");
    assert_eq!(functions[0].counterparts[0].function, "ours");
    assert_eq!(
        findings(&functions[0]),
        [Finding::NotFound, Finding::Exact],
        "the other branch's roll is not ours with other numbers: ours is the second statement's"
    );
}

#[test]
fn a_test_counts_whichever_way_round_and_a_range_is_its_comparisons() {
    let source = "const KEPT: std::ops::RangeInclusive<f32> = -0.5..=0.0;\n/// `roll()`.\nfn ours(value: f32) -> bool { KEPT.contains(&value) }\n";
    let below = test(*binary(Operator::Less, name("value"), number(-0.5)));
    let above = test(Expression::Not(binary(Operator::Greater, name("value"), number(0.0))));
    let functions = judged(source, vec![below, above]);
    assert_eq!(findings(&functions[0]), [Finding::Exact, Finding::Exact]);
}

#[test]
fn a_number_in_other_units_is_the_same_structure_but_another_number_is_not() {
    let source = "const FAINT: f32 = 0.30;\n/// `roll()`.\nfn ours(alpha: f32) -> bool { alpha < FAINT }\n";
    let percent = test(*binary(Operator::Less, name("alpha"), number(30.0)));
    let other = test(*binary(Operator::Less, name("alpha"), number(9.0)));
    let functions = judged(source, vec![percent, other]);
    assert_eq!(findings(&functions[0]), [Finding::SameStructure, Finding::NotFound]);
}

#[test]
fn constants_are_regrouped_and_compared_at_the_games_precision() {
    let source = "const FULL_TURN: f32 = std::f32::consts::TAU;\n/// `roll()`.\nfn ours(random: &mut Random) -> f32 { random.next_unit() * FULL_TURN }\n";
    let unit = Box::new(Expression::MethodCall(
        name("Math"),
        Box::new(Expression::Text("random".to_owned())),
        Vec::new(),
    ));
    let angle = binary(
        Operator::Multiply,
        binary(Operator::Multiply, unit, number(std::f64::consts::PI)),
        number(2.0),
    );
    let functions = judged(source, vec![assign(*angle)]);
    assert_eq!(findings(&functions[0]), [Finding::Exact]);
}

#[test]
fn statements_of_names_alone_are_not_judged() {
    let source = "/// `roll()`.\nfn ours(a: f32, b: f32) -> f32 { a * b }\n";
    let functions = judged(source, vec![assign(*binary(Operator::Multiply, name("a"), name("b")))]);
    assert_eq!(findings(&functions[0]), [Finding::Nothing]);
    assert_eq!(functions[0].judged(), 0);
}

#[test]
fn the_originals_random_direction_is_ours() {
    let source = "const REACH: f32 = 30.0;\n/// `roll()`.\nfn ours(position: Vec2, random: &mut Random) -> Vec2 { position + random.direction(REACH) }\n";
    let away = Box::new(Expression::Call("crand".to_owned(), vec![Expression::Number(30.0)]));
    let other_axis = Box::new(Expression::Call("crand".to_owned(), Vec::new()));
    let functions = judged(
        source,
        vec![
            assign(*binary(Operator::Add, name("x"), away)),
            assign(*binary(Operator::Add, name("y"), other_axis)),
        ],
    );
    assert_eq!(
        findings(&functions[0]),
        [Finding::Exact, Finding::Nothing],
        "the second axis says nothing of its own"
    );
}

#[test]
fn a_sound_event_is_the_file_name_the_original_plays() {
    let source = "const SOUND_CATALOG: [(SoundEvent, &[&str]); 2] = [(SoundEvent::Holy, &[\"Holy.mp3\"]), (SoundEvent::Fired, &[\"Tears_Fire_0.mp3\", \"Tears_Fire_1.mp3\", \"Tears_Fire_2.mp3\"])];\n/// `roll()`.\nfn ours(events: &mut Vec<SoundEvent>) { events.push(SoundEvent::Holy); events.push(SoundEvent::Fired); }\n";
    let play = |name: Expression| Statement::Evaluate(Expression::Call("soundy".to_owned(), vec![name]));
    let numbered = binary(
        Operator::Add,
        binary(
            Operator::Add,
            Box::new(Expression::Text("Tears_Fire_".to_owned())),
            random(3.0),
        ),
        Box::new(Expression::Text(".mp".to_owned())),
    );
    let functions = judged(
        source,
        vec![
            play(Expression::Text("Holy.mp".to_owned())),
            play(*numbered),
            play(Expression::Text("Unlock00.wav".to_owned())),
        ],
    );
    assert_eq!(
        findings(&functions[0]),
        [Finding::Exact, Finding::Exact, Finding::NotFound],
        "`.mp` plays as `.mp3`; numbered variants are a random pick; another sound is missing"
    );
}
