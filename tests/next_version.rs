//! `scripts/next-version.sh` works the release version out from the titles of the commits since the
//! last tag; this runs it on throwaway repositories.

use std::path::{Path, PathBuf};
use std::process::Command;

const SCRIPT: &str = "scripts/next-version.sh";
const NO_RELEASE_STATUS: i32 = 3;

struct Repository {
    directory: PathBuf,
}

impl Repository {
    fn new(name: &str) -> Self {
        let directory = std::env::temp_dir().join(format!("opentboi-next-version-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).expect("cannot create the repository directory");
        let repository = Self { directory };
        repository.git(&["init", "--quiet", "--initial-branch=main"]);
        std::fs::write(
            repository.directory.join("Cargo.toml"),
            "[package]\nname = \"x\"\nversion = \"0.1.0\"\n",
        )
        .expect("cannot write Cargo.toml");
        // A tag needs something to point at.
        repository.git(&["add", "Cargo.toml"]);
        repository.commit("chore: start");
        repository
    }

    fn git(&self, arguments: &[&str]) {
        let status = Command::new("git")
            .args([
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.com",
                "-c",
                "commit.gpgsign=false",
            ])
            .args(arguments)
            .current_dir(&self.directory)
            .status()
            .expect("git is not available");
        assert!(status.success(), "git {arguments:?} failed");
    }

    fn commit(&self, subject: &str) {
        self.git(&["commit", "--quiet", "--allow-empty", "-m", subject]);
    }

    fn tag(&self, name: &str) {
        self.git(&["tag", name]);
    }

    /// What the script prints, or `None` when it says nothing is to be released.
    fn next_version(&self) -> Option<String> {
        let script = Path::new(env!("CARGO_MANIFEST_DIR")).join(SCRIPT);
        let output = Command::new("bash")
            .arg(script)
            .current_dir(&self.directory)
            .output()
            .expect("bash is not available");
        match output.status.code() {
            Some(0) => Some(String::from_utf8_lossy(&output.stdout).trim().to_owned()),
            Some(NO_RELEASE_STATUS) => None,
            other => panic!(
                "the script failed ({other:?}): {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        }
    }
}

impl Drop for Repository {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}

#[test]
fn the_first_release_is_the_version_cargo_names() {
    let repository = Repository::new("first");
    repository.commit("feature: something (#1)");
    assert_eq!(repository.next_version().as_deref(), Some("0.1.0"));
}

#[test]
fn a_fix_makes_the_next_patch_a_feature_the_next_minor() {
    let repository = Repository::new("small");
    repository.commit("feature: first (#1)");
    repository.tag("v1.2.3");
    repository.commit("fix: a bug (#2)");
    assert_eq!(repository.next_version().as_deref(), Some("1.2.4"));
    repository.commit("feature: a thing (#3)");
    assert_eq!(repository.next_version().as_deref(), Some("1.3.0"));
}

#[test]
fn a_breaking_change_makes_the_next_major_but_only_the_next_minor_before_version_one() {
    let repository = Repository::new("breaking");
    repository.tag("v0.4.2");
    repository.commit("fix: a bug (#1)");
    repository.commit("breaking: the save format changes (#2)");
    repository.commit("feature: a thing (#3)");
    assert_eq!(repository.next_version().as_deref(), Some("0.5.0"));
    repository.tag("v2.0.0");
    repository.commit("breaking(game): another (#4)");
    assert_eq!(repository.next_version().as_deref(), Some("3.0.0"));
}

#[test]
fn chores_and_untitled_commits_call_for_no_release() {
    let repository = Repository::new("chores");
    repository.tag("v1.0.0");
    repository.commit("chore: bump a crate (#1)");
    repository.commit("Merge something with no tag");
    assert_eq!(repository.next_version(), None);
}
