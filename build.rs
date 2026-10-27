//! Regenerates `THIRD-PARTY-LICENSES` with `cargo about` whenever the dependencies or the notice
//! settings change, so the licence texts we must ship never fall behind `Cargo.lock`.
//!
//! Needs `cargo install cargo-about --locked --features cli`. Without it, or when generation
//! fails, the build carries on with a warning, so a contributor without the tool can still compile.
//! Set `OPENTBOI_REQUIRE_LICENSES=1` (release and CI builds) to make that a build error instead.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const OUTPUT_FILE: &str = "THIRD-PARTY-LICENSES";
const TEMPLATE_FILE: &str = "about.hbs";
const SETTINGS_FILE: &str = "about.toml";
const LOCK_FILE: &str = "Cargo.lock";
const BUILD_SCRIPT_FILE: &str = "build.rs";
const REQUIRE_VARIABLE: &str = "OPENTBOI_REQUIRE_LICENSES";
const CARGO_VARIABLE: &str = "CARGO";
const MANIFEST_DIRECTORY_VARIABLE: &str = "CARGO_MANIFEST_DIR";
const DEFAULT_CARGO: &str = "cargo";
const SOURCE_DIRECTORY: &str = "src";
const COVERAGE_SCRIPT: &str = "scripts/original-coverage.py";
/// The one place the map lives; tracked, since it holds only function names and counts.
const COVERAGE_MAP: &str = "docs/original-coverage-map.svg";
const PYTHON: &str = "python3";
const DECOMPILED_SCRIPTS: [&str; 2] = [
    ".cache/opentboi/decompiled/scripts/DefineSprite_6765/frame_1/DoAction.as",
    ".cache/opentboi/decompiled/scripts/frame_1/DoAction.as",
];
const HOME_VARIABLE: &str = "HOME";

fn main() {
    let inputs = [
        BUILD_SCRIPT_FILE,
        LOCK_FILE,
        SETTINGS_FILE,
        TEMPLATE_FILE,
        SOURCE_DIRECTORY,
        COVERAGE_SCRIPT,
    ];
    for input in inputs {
        println!("cargo:rerun-if-changed={input}");
    }
    println!("cargo:rerun-if-env-changed={REQUIRE_VARIABLE}");

    if let Err(reason) = generate_licenses() {
        let required = std::env::var_os(REQUIRE_VARIABLE).is_some_and(|value| !value.is_empty() && value != "0");
        assert!(!required, "{OUTPUT_FILE} could not be generated: {reason}");
        println!("cargo:warning={OUTPUT_FILE} was not regenerated: {reason}");
    }
    if let Err(reason) = generate_coverage_map() {
        println!("cargo:warning={COVERAGE_MAP} was not regenerated: {reason}");
    }
}

/// Redraws the map of how much of the original's script the sources cover, whenever the sources
/// change. Without the original's scripts (made from the user's copy of the game) there is nothing
/// to map, and the build goes on silently: the pull request check "Coverage map was regenerated"
/// (`.github/workflows/coverage-map.yml`) is what makes sure someone who has them redraws it.
fn generate_coverage_map() -> Result<(), String> {
    let home = std::env::var_os(HOME_VARIABLE)
        .map(PathBuf::from)
        .ok_or("HOME is not set")?;
    if !DECOMPILED_SCRIPTS.iter().all(|script| home.join(script).exists()) {
        return Ok(());
    }
    let status = Command::new(PYTHON)
        .args(["-I", COVERAGE_SCRIPT, "--svg", COVERAGE_MAP])
        .stdout(Stdio::null())
        .status()
        .map_err(|error| format!("{PYTHON} could not be started: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{COVERAGE_SCRIPT} {status}"))
    }
}

fn cargo_about(arguments: &[&str]) -> Command {
    let cargo = std::env::var_os(CARGO_VARIABLE).unwrap_or_else(|| DEFAULT_CARGO.into());
    let mut command = Command::new(cargo);
    command.arg("about").args(arguments);
    command
}

fn generate_licenses() -> Result<(), String> {
    let manifest_directory = std::env::var_os(MANIFEST_DIRECTORY_VARIABLE)
        .ok_or_else(|| format!("{MANIFEST_DIRECTORY_VARIABLE} is not set"))?;
    let manifest_directory = Path::new(&manifest_directory);

    let installed = cargo_about(&["--version"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success());
    if !installed {
        return Err("cargo-about is not installed (cargo install cargo-about --locked --features cli)".to_owned());
    }

    // Offline first (no network needed once the crates are downloaded); a clean machine has only
    // downloaded the crates of the features it builds, while the notice covers all features, so
    // then let cargo fetch the rest.
    let mut last_status = None;
    for offline in [true, false] {
        let mut arguments = vec!["generate", "--config", SETTINGS_FILE, "--all-features"];
        if offline {
            arguments.push("--offline");
        }
        arguments.extend(["--output-file", OUTPUT_FILE, TEMPLATE_FILE]);
        let status = cargo_about(&arguments)
            .current_dir(manifest_directory)
            .stderr(if offline { Stdio::null() } else { Stdio::inherit() })
            .status()
            .map_err(|error| format!("cargo about could not be started: {error}"))?;
        if status.success() {
            return Ok(());
        }
        last_status = Some(status);
    }
    Err(format!(
        "cargo about {}",
        last_status.map_or_else(String::new, |status| status.to_string())
    ))
}
