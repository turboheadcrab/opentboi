//! Enforces the project's source file size limit so modules stay small enough to review.

use std::path::{Path, PathBuf};

const MAXIMUM_LINES_PER_SOURCE_FILE: usize = 250;
const SOURCE_DIRECTORIES: [&str; 2] = ["src", "tests"];
const RUST_SOURCE_EXTENSION: &str = "rs";

#[test]
fn every_source_file_is_within_the_line_limit() {
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut source_files = Vec::new();
    for directory in SOURCE_DIRECTORIES {
        collect_rust_files(&project_root.join(directory), &mut source_files);
    }
    assert!(
        !source_files.is_empty(),
        "no Rust sources found under {}",
        project_root.display()
    );

    let oversized_files: Vec<String> = source_files
        .iter()
        .filter_map(|path| {
            let line_count = std::fs::read_to_string(path).ok()?.lines().count();
            (line_count > MAXIMUM_LINES_PER_SOURCE_FILE).then(|| format!("{} ({line_count} lines)", path.display()))
        })
        .collect();
    assert!(
        oversized_files.is_empty(),
        "split these files to stay within {MAXIMUM_LINES_PER_SOURCE_FILE} lines: {oversized_files:?}"
    );
}

fn collect_rust_files(directory: &Path, found_files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, found_files);
        } else if path
            .extension()
            .is_some_and(|extension| extension == RUST_SOURCE_EXTENSION)
        {
            found_files.push(path);
        }
    }
}
