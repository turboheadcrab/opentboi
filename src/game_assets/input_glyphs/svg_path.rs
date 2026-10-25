//! Reads the paths of a Kenney glyph SVG into `tiny_skia` paths.

use tiny_skia::{Path, PathBuilder};

/// Numbers a command takes: a point for move and line, a control point and a point for a curve.
const MOVE_ARGUMENTS: usize = 2;
const LINE_ARGUMENTS: usize = 2;
const CURVE_ARGUMENTS: usize = 4;
const PATH_ATTRIBUTE_START: &str = " d=\"";
const FILL_ATTRIBUTE_START: &str = " fill=\"";
/// The pack draws a picture in white and marks a pressed part in another colour.
const PLAIN_FILL: &str = "#FFFFFF";
const ATTRIBUTE_END: char = '"';

/// One `<path>` of a glyph SVG.
pub(super) struct SvgPath {
    pub path: Path,
    /// Filled in a colour other than the picture's white: the pressed part of a D-pad.
    pub highlighted: bool,
}

/// Every `<path>`'s outline, in the SVG's order; `None` if there is none or one is unreadable.
pub(super) fn paths_of(svg: &str) -> Option<Vec<SvgPath>> {
    let paths: Option<Vec<SvgPath>> = svg
        .split("<path")
        .skip(1)
        .map(|element| {
            let data_start = element.find(PATH_ATTRIBUTE_START)? + PATH_ATTRIBUTE_START.len();
            let data = &element[data_start..data_start + element[data_start..].find(ATTRIBUTE_END)?];
            let fill = element.find(FILL_ATTRIBUTE_START).map(|start| {
                let rest = &element[start + FILL_ATTRIBUTE_START.len()..];
                &rest[..rest.find(ATTRIBUTE_END).unwrap_or(rest.len())]
            });
            Some(SvgPath {
                path: parse_path_data(data)?,
                highlighted: fill.is_some_and(|fill| fill != PLAIN_FILL),
            })
        })
        .collect();
    paths.filter(|paths| !paths.is_empty())
}

/// Move, line and quadratic-curve commands with absolute coordinates. As in SVG, extra pairs after a
/// command repeat it (after a move they are lines).
pub(super) fn parse_path_data(data: &str) -> Option<Path> {
    let mut builder = PathBuilder::new();
    let mut command = None;
    let mut arguments: Vec<f32> = Vec::new();
    for token in tokens(data) {
        match token {
            Token::Command(letter) => {
                if !arguments.is_empty() {
                    return None;
                }
                command = Some(letter);
            }
            Token::Number(number) => {
                arguments.push(number);
                if arguments.len() == arguments_of(command?)? {
                    command = Some(apply(&mut builder, command?, &arguments));
                    arguments.clear();
                }
            }
        }
    }
    if !arguments.is_empty() {
        return None;
    }
    builder.finish()
}

fn arguments_of(command: char) -> Option<usize> {
    match command {
        'M' => Some(MOVE_ARGUMENTS),
        'L' => Some(LINE_ARGUMENTS),
        'Q' => Some(CURVE_ARGUMENTS),
        _ => None,
    }
}

/// Adds the command to the path; returns the command that further numbers repeat.
fn apply(builder: &mut PathBuilder, command: char, arguments: &[f32]) -> char {
    match command {
        'M' => {
            builder.move_to(arguments[0], arguments[1]);
            'L'
        }
        'L' => {
            builder.line_to(arguments[0], arguments[1]);
            command
        }
        _ => {
            builder.quad_to(arguments[0], arguments[1], arguments[2], arguments[3]);
            command
        }
    }
}

enum Token {
    Command(char),
    Number(f32),
}

fn tokens(data: &str) -> impl Iterator<Item = Token> {
    let mut pieces = Vec::new();
    let mut number = String::new();
    for character in data.chars() {
        if character.is_ascii_digit() || character == '.' || character == '-' {
            number.push(character);
            continue;
        }
        if let Ok(parsed) = number.parse() {
            pieces.push(Token::Number(parsed));
        }
        number.clear();
        if character.is_ascii_alphabetic() {
            pieces.push(Token::Command(character));
        }
    }
    if let Ok(parsed) = number.parse() {
        pieces.push(Token::Number(parsed));
    }
    pieces.into_iter()
}
