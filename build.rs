//! Regenerates `THIRD-PARTY-LICENSES` with `cargo about` whenever the dependencies or the notice
//! settings change, so the licence texts we must ship never fall behind `Cargo.lock`.
//!
//! Needs `cargo install cargo-about --locked --features cli`. Without it, or when generation
//! fails, the build carries on with a warning, so a contributor without the tool can still compile.
//! Set `OPENTBOI_REQUIRE_LICENSES=1` (release and CI builds) to make that a build error instead.
//!
//! Also redraws the coverage map `docs/original-coverage-map.svg` (see `generate_coverage_map`),
//! which CI requires to be up to date with `OPENTBOI_REQUIRE_COVERAGE_MAP=1`.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const OUTPUT_FILE: &str = "THIRD-PARTY-LICENSES";
const TEMPLATE_FILE: &str = "about.hbs";
const SETTINGS_FILE: &str = "about.toml";
const LOCK_FILE: &str = "Cargo.lock";
const BUILD_SCRIPT_FILE: &str = "build.rs";
const REQUIRE_VARIABLE: &str = "OPENTBOI_REQUIRE_LICENSES";
const REQUIRE_MAP_VARIABLE: &str = "OPENTBOI_REQUIRE_COVERAGE_MAP";
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
    println!("cargo:rerun-if-env-changed={REQUIRE_MAP_VARIABLE}");

    if let Err(reason) = generate_licenses() {
        assert!(
            !is_required(REQUIRE_VARIABLE),
            "{OUTPUT_FILE} could not be generated: {reason}"
        );
        println!("cargo:warning={OUTPUT_FILE} was not regenerated: {reason}");
    }
    if let Err(reason) = generate_coverage_map() {
        assert!(
            !is_required(REQUIRE_MAP_VARIABLE),
            "{COVERAGE_MAP} is out of date: {reason}"
        );
        println!("cargo:warning={COVERAGE_MAP} is out of date: {reason}");
    }
}

/// Redraws the map of how much of the original's script the sources cover, whenever the sources
/// change. Without the original's scripts (made from the user's copy of the game) it cannot be
/// redrawn, so it is checked instead: the map carries a digest of the sources it was drawn from,
/// which needs nothing of the original, and a map drawn from other sources is out of date. That is
/// a warning for a contributor and, with `OPENTBOI_REQUIRE_COVERAGE_MAP=1` (CI), a build error.
fn generate_coverage_map() -> Result<(), String> {
    let home = std::env::var_os(HOME_VARIABLE)
        .map(PathBuf::from)
        .ok_or("HOME is not set")?;
    let redrawable = DECOMPILED_SCRIPTS.iter().all(|script| home.join(script).exists());
    let arguments = if redrawable {
        ["--svg", COVERAGE_MAP]
    } else {
        ["--check-svg", COVERAGE_MAP]
    };
    let status = Command::new(PYTHON)
        .arg("-I")
        .arg(COVERAGE_SCRIPT)
        .args(arguments)
        .stdout(Stdio::null())
        .status()
        .map_err(|error| format!("{PYTHON} could not be started: {error}"))?;
    match (status.success(), redrawable) {
        (true, _) => Ok(()),
        (false, true) => Err(format!("{COVERAGE_SCRIPT} {status}")),
        (false, false) => Err(
            "it was drawn from other sources than these, and redrawing it needs the original's \
             scripts in ~/.cache/opentboi (a maintainer who has them can push the redrawn map)"
                .to_owned(),
        ),
    }
}

/// Whether the environment variable `name` is set to something other than empty or `0`.
fn is_required(name: &str) -> bool {
    std::env::var_os(name).is_some_and(|value| !value.is_empty() && value != "0")
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
