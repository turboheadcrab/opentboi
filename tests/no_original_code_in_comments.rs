//! Comments and docs describe the original game's behaviour in our own words and cite its
//! functions by name (`name()`); they never quote its code. This test fails on the code's most
//! telling marks in comments (Rust) and anywhere in Markdown, so a quote cannot slip back in.

use std::path::{Path, PathBuf};

const SCANNED_DIRECTORIES: [&str; 3] = ["src", "tests", "docs"];
const ROOT_DOCUMENTS: [&str; 4] = ["README.md", "AGENTS.md", "TODO.md", "VERIFIED.md"];
const RUST_EXTENSION: &str = "rs";
const MARKDOWN_EXTENSION: &str = "md";
const LINE_COMMENT: &str = "//";
/// This file names the marks it looks for.
const THIS_FILE: &str = "no_original_code_in_comments.rs";
/// Marks of the original's code that our own prose never needs: its root and object prefixes,
/// its locals, its frame jumps and its speed fields.
const ORIGINAL_CODE_MARKS: [&str; 10] = [
    "_root.",
    "trg.",
    "highs.",
    "lows.",
    "_loc1_",
    "_loc2_",
    "gotoAndStop(",
    "gotoAndPlay(",
    "xbew",
    "ybew",
];

#[test]
fn comments_and_docs_do_not_quote_the_original() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files: Vec<PathBuf> = ROOT_DOCUMENTS.iter().map(|name| root.join(name)).collect();
    for directory in SCANNED_DIRECTORIES {
        collect_files(&root.join(directory), &mut files);
    }
    let mut quotes = Vec::new();
    for path in files.iter().filter(|path| !path.ends_with(THIS_FILE)) {
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        let rust = path.extension().is_some_and(|extension| extension == RUST_EXTENSION);
        for (index, line) in text.lines().enumerate() {
            let checked = if rust {
                line.find(LINE_COMMENT).map_or("", |start| &line[start..])
            } else {
                line
            };
            if let Some(mark) = ORIGINAL_CODE_MARKS.iter().find(|mark| contains_mark(checked, mark)) {
                quotes.push(format!("{}:{}: {mark}", path.display(), index + 1));
            }
        }
    }
    assert!(quotes.is_empty(), "describe these in words instead: {quotes:#?}");
}

/// `mark` in `text` as a word of its own: not the end of a longer word ("allows." is not "lows.").
fn contains_mark(text: &str, mark: &str) -> bool {
    text.match_indices(mark).any(|(start, _)| {
        text[..start]
            .chars()
            .next_back()
            .is_none_or(|before| !before.is_alphanumeric() && before != '_')
    })
}

fn collect_files(directory: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, found);
        } else if path
            .extension()
            .is_some_and(|extension| extension == RUST_EXTENSION || extension == MARKDOWN_EXTENSION)
        {
            found.push(path);
        }
    }
}
