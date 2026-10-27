## What this changes

## Checks
- [ ] `cargo fmt`, `cargo clippy --all-targets --all-features -- -D warnings` and `cargo test` pass.
- [ ] No file of the original (SWF, images, sounds) and no text taken from it is committed.
- [ ] The coverage map `docs/original-coverage-map.svg` was redrawn if the sources changed (the CI check "Coverage map was regenerated" fails otherwise). Without the original's scripts, ask a maintainer to push it.
