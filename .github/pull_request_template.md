## What this changes

## Checks
- [ ] `cargo fmt`, `cargo clippy --all-targets --all-features -- -D warnings` and `cargo test` pass.
- [ ] No file of the original (SWF, images, sounds) and no text taken from it is committed.
- [ ] The coverage map `docs/original-coverage-map.svg` is redrawn (`cargo build` does it where the original's scripts are in your cache). CI's build fails when it is out of date; without the scripts, ask a maintainer to push it.
