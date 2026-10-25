# Tracking how much of the original we port

**Update 2026-10-25.** The design below was not built as written: the Python script stayed and is
what the project uses. It reads the game's script and the main timeline's (cited as `DoAction N` and
`Timeline N` in comments), counts the gameplay lines our sources cite or use, and draws the map in
[`original-coverage-map.svg`](original-coverage-map.svg) (see the end). Its numbers are evidence of a
port, not proof.

Status of the design: **design**, apart from primitive matching (built 2026-10-15, judged per
function since 2026-10-17; see the end). It replaces the idea behind
`scripts/original-coverage.py`, which counts quoted expressions in comments against a text dump of
the original's script. That script depends on one build, on one tool's output and on quoting the
original, none of which we want: committed files should not quote the original.

## What the process must do
1. Give one answer per supported build of the original (Steam release, browser demo, others), not
   one number.
2. Tell apart, per original function, whether our code is
   - **`port`**: the same steps in the same order (a 1:1 port),
   - **`equivalent`**: the same outcome, a different implementation (tables instead of branches,
     our own structures, our own random-number plumbing),
   - **`partial`**: only some branches or cases are done, and say which,
   - **`missing`**, or **`skipped`** on purpose (Flash user interface, advertising, things our
     frontends replace), with a reason.
3. Be replicable: anyone with their own copy of a build gets the same table from one command, and
   the table says which build it was made from (SHA-256).
4. Keep every original expression out of committed files: only function names, positions, counts and
   hashes are committed (names and hashes are facts about a file, not expressions from it).
5. Notice when a build changes a function we ported, instead of silently keeping an old claim.

## Two separate axes
Status above is *what we say we did*. It must be paired with *how well we know it*, because a
claimed 1:1 port is not proof:

| Evidence | Meaning | How it is obtained |
|---|---|---|
| 0 claimed | a tag in the code says so | the author |
| 1 values checked | the numbers our code uses equal the ones read from this build's script | `SwfParameters`/anchors test per build (`docs/parameter-extraction.md`) |
| 2 behaviour checked | our function and the original's give the same results for the same seed and input | differential or trace test (below) |
| 3 every build | evidence 2 passes in every supported build | the same test run per build |

`equivalent` is only accepted at evidence 2 or higher; without a comparison "same outcome" is
just an opinion. A `port` may stay at 0 or 1 (cheap), and the report shows the level.

## How units are identified
The unit is an original function, plus the script blocks that run outside functions (the main
timeline's and each sprite's frame scripts). Inside a function, the sub-units are the branches
of a top-level `switch` or `if` chain, so `partial` can name them.

Our decoder (`src/game_assets/script`) already reads every build we have: the Steam release gives 244
functions, the demo 62 (measured 2026-10-15). In the demo every function has the same name (two
control characters), so **names cannot be the key**. Identity per build is therefore:
- **structure hash (S-hash)**: the function's operations with names, constants and jump offsets
  removed (jumps become statement indices). It survives renaming and obfuscation of names, and
  changes when the logic changes.
- **value hash (V-hash)**: the same with the constants kept. S equal and V different means "same
  logic, other numbers", which is a parameter table, not a rewrite.
- **name**, where the build keeps names. A name is a hint; the S-hash confirms it.
- **statement position**: the index in our decoder's output, plus the statement's S-hash
  (`balljunk#412:9f3a`), as in `TODO.md` ("Citing the original without quoting it").

Matching a unit from the reference build (the Steam release) into another build: same name and
same S-hash; else a unique S-hash; else the S-hash plus the S-hashes of the functions it calls
(the call graph survives obfuscation); else report **ambiguous** and let a person decide.

## Where the data lives
1. **Tags in the Rust code, next to the code that ports** (the source of truth for what is ported).
   One machine-readable comment per ported unit, in our own words around it:
   `// original: <function>[#first-last] <port|equivalent|partial> [test=<name>]`.
   A function ported in several places gets several tags; a tag range is what makes `partial`
   countable. Moving or refactoring code keeps its tag; nothing in a tag quotes the original.
2. **`coverage/lock.toml`** (committed, like `Cargo.lock`): for every tag, the S-hashes of the
   statements it claimed in the reference build, and that build's SHA-256. Hashes only. When a build
   disagrees, the tag is *stale*, and a person re-reads the function and re-locks it.
3. **`coverage/skipped.toml`** (committed): units deliberately not ported, each with a reason in
   words ("Flash-only ad screen", "our frontends draw this").
4. **`docs/original-builds.md`** (committed): the registry of builds (name, origin, SHA-256,
   obfuscated or not). Already planned in `TODO.md`.
5. **Generated, never committed:** the per-statement report (it would list the original's structure
   in detail, question 64). **Committed:** the aggregate table (builds × statuses × percentages) in
   `VERIFIED.md`, with the build hash and who ran it.

## The tool
A Rust dev binary (`cargo run --features dev-tools --bin coverage -- …`, or a separate `xtask`
crate) built on the existing decoder, so it reads any build, needs no FFDec and replaces the
Python script. Commands:
- `scan`: read the tags from `src/` and list them (syntax errors, duplicates, ranges that overlap).
- `report --build <steam|demo|PATH>`: resolve every unit in that build and print, per unit:
  found or absent, **same** (S and V equal), **values differ** (S equal, V different),
  **structure differs** (the port may be stale or the build behaves differently), or **ambiguous**;
  next to our status and evidence. Totals: statements covered / total gameplay statements, with the
  denominator excluding `skipped`.
- `lock`: record the S-hashes of the tagged statements (after a human reviewed them).
- `check`: for CI and for pull requests: tags parse, every `test=` names an existing test, every
  `skipped` has a reason, and — only where a build is available — the lock still matches.
- `unported <function> --build …`: **local only**; prints the statements of the function that no
  tag covers, with their text from the user's own SWF, to the terminal. Never writes a file.

What CI can check without any game file: tag syntax, the lock file's shape, and the hashing and
matching code on SWFs the tests build themselves (the `swf` crate can write them), including a
renamed copy to prove the match survives obfuscation. Real-build reports are `#[ignore]`d tests or
local runs, recorded in `VERIFIED.md` (who, which hash).

## Getting evidence 2 (differential testing)
Two ways, to decide after trying the first:
- **Own AVM1 runner for pure functions** (`slotf()`, roll and payout functions), as in
  `docs/parameter-extraction.md` step 3: arithmetic, comparisons, branches, arrays, `random()` from
  our generator. Small and deterministic; cannot run the whole game.
- **Ruffle as a dev dependency** (we already depend on its `swf` and `ruffle_render` crates, MIT or
  Apache-2.0): step the real movie headless with scripted input and log chosen variables per frame,
  then compare with our traces. Needed for functions that touch the display list or timeline.
  Whether Ruffle's script engine can be driven and seeded that way is **not verified**.

## Adding a function, or a build
- *A function:* `unported` to read it locally; write the Rust with a tag; add a test (evidence 1
  or 2); `lock`; `check`; the aggregate table is regenerated.
- *A build:* register it; run `report --build`; **same** and **values differ** are handled by
  data (parameters, anchors), **structure differs** means a new option next to the existing
  original-versus-Rebirth ones or an "unsupported in this build" entry; **absent** is expected for
  features a build does not have. Code finds things by data (label or linkage name exists), never by
  version number.

## Primitive matching (built; a floor of evidence, no tags needed)
`cargo test original_arithmetic_in_our_code -- --ignored --nocapture` (code in
`src/game_assets/script/primitives/`) reads the original's bytecode with our own decoder and our Rust
sources with `syn`, and reduces both to the same name-free *shape*: operators and numbers stay,
every variable, member, call, field and method becomes a wildcard. Equivalences both sides share are
applied first (`a > b` is `b < a`, `<=`/`>=`/`!=` are negations, constants fold, commutative operands
are ordered); named constants and parameter fields given a number (`Frames::new(30.0)`) count as that
number. A statement of the original whose shape also occurs in our code is reported as found.
- **Only telling shapes count.** Two or more operators and a number other than -1, 0, 1, 2, or five
  or more operators; a shape found in more than three of our functions is reported as ambiguous.
  `switch` cases (a negated equals) and a plain comparison with a small number say nothing and are not judged. "Same structure with other
  numbers" is only tried for shapes with three or more operators.
- **Findings:** whole computation found; a telling part found; same structure with other numbers (a
  parameter or unit we changed); ambiguous; not found; nothing telling.
- **Output** names original functions and counts, never expressions. `OPENTBOI_PRIMITIVES_FUNCTION=name`
  lists one function's statements by shape, for local reading only; do not paste it into files.
- **What it is:** evidence that the same computation exists in our code, independent of comments or
  citations, so it also catches code that cites nothing. It is not proof: a shape can coincide, and
  a shape that is *not* found is not a missing port. Result on the Steam build (245 functions,
  2026-10-18, after the decoder fix, the random direction and the sound names below; random rolls
  no longer count as telling here): 857 statements are telling enough to judge (of about 23,700);
  49 are found with equal numbers, 9 more in part, 81 have the same structure with other numbers,
  10 are too common, 708 are not found. No
  function reaches the "nearly complete" bar (3 or more judged statements, 75 % found); short
  functions are judged per function instead (below).
  Our ports use vector maths and typed units, so a sum of products often looks different from the
  original's per-axis lines (`ffmo`: both additions found, both subtractions not). The "best fit"
  column names the Rust function holding most of the found shapes; with only one or two shapes it
  can be a coincidence (`onEnterFrame` → `read_names`), so read it only when the count is high.
- **Per function, against its counterparts (built 2026-10-17).** The whole-code search cannot judge
  short functions: most of their statements are one comparison with one number, which matches
  something anywhere. So each original function is also judged against its *counterparts*: the Rust
  functions whose doc comment names it (`` `slotf()` ``, a name, never an expression), plus the
  function holding most of its telling computations if that is at least three of them and 30 % of
  its judged statements (fewer is coincidence: random rolls are in every big function). Inside that
  scope every statement with a number, a random roll or two operators is judged: found with equal
  numbers, found with other numbers, found in part, or missing. Statements of names alone are not
  judged (a flag that only has a name cannot be seen, so a function can look more complete than it
  is by that much).
- **Equivalences both sides share**, added for this: a test counts whichever way round it is written
  (a condition and its negation decide the same thing; negations are pushed through `&&` and `||`);
  a range constant's `contains` is the two comparisons it makes; constants in a sum or product are
  regrouped and folded (a full turn written as two factors or as one constant); numbers are compared
  at `f32` precision (the game's); AVM1's random-number instruction is our `Random::below`, and the
  random fraction is our `Random::next_unit`; the original's own helper for a distance in a random
  direction (called about 160 times, each time followed by a second call for the other axis) is our
  `Random::direction`, and the second call says nothing of its own; picking from a list
  (`Random::index`) is the random-number instruction too; `Type::from(x)`, `Type::new(x)` and a
  tuple struct's `Type(x)` are `x`, and a type's own constants (`Hearts::ONE`) are numbers; only the
  gameplay parameter structs' fields count as constants (other structs start state off, such as a
  count at 0, which is not a constant); a `SoundEvent` is the file name the sound catalog gives it,
  built the way the original builds names (a start, a random number and an end for numbered variants, and a
  name ending in `.mp` played as `.mp3`), and a sound's file name is telling on its own; the standard library's `PI` and
  `TAU` are numbers. "Other numbers" is accepted for a one-operator test only when the numbers differ
  by a power of ten (a percentage written as a fraction, 30 and 0.30), for longer shapes in any
  number, and never when our shape is already the exact counterpart of another statement of the same
  function (two branches that differ only in a number are not both found).
- **Decoder fix found by this.** Locals and parameters live in AVM1 registers; the decoder used to
  paste the last value stored in a register into every later read, also across branches, and
  dropped the store itself. Short functions came out as computations the original never makes. A
  store whose value is dropped at once is now an assignment, and a register read stays a name.
- **Calibration** (2026-10-17, Steam build), on functions read by hand: `ender()` and `crand()` are
  complete in our code and come out 1 of 1 each (a random index into the dead ends; a random angle of
  a full turn). `spawnb()` comes out 10 of 11 (two of the ten with other numbers: we keep alpha as a
  fraction); the missing statement is the grid's lazy creation, which our data model does not need.
  `slotf()` comes out 7 of 8, and the missing statement is exactly the Lucky Foot's roll, which we
  do not port (the arcade's flag in the same function is a name alone, so it is not seen). `pffx()`
  comes out 5 of 7: the two missing are where our floor grid holds booleans instead of numbers, an
  equivalent rather than a port. `tater()` comes out 1 found and 2 in part: we pick an index into
  our card list where the original picks a colour number. Those last two are the `equivalent` cases
  of the design above: same outcome, other representation.
- **Known blind spots:** vector maths written per axis in the original (a sum of products, a
  rotation) does not look like `aim * forward + aim.perp() * side`; array lookups by computed index
  hide their index arithmetic; and a function cited by a doc comment that ports only one branch
  shows as mostly missing, which is right but says nothing about that branch alone.
- **Next:** name the branch a counterpart ports (the design's `#first-last` ranges) so partial ports
  are judged against their part; feed the result into the `port`/`equivalent` tags as the evidence
  level "same primitives"; run it on other builds.

## Order and rough size
1. Tags plus `scan` and `report` for the Steam build, `skipped.toml`; convert the quoted
   comments to tags (about 120 lines in 59 files); retire the Python script. (a few days)
2. S-hash and V-hash, matching, `report` for the demo (the matching is the hard part). (a few days)
3. `lock` and drift detection in `check` and CI. (about a day)
4. Differential tests: an own runner for pure functions first. (days per family; open-ended)
5. A test rejecting ActionScript tokens in comments and docs (already on the list in `TODO.md`).

## Honest limits
- S-hash equality does not prove equal behaviour when behaviour comes from data (frame labels,
  sprite contents, parameters in other scripts). For those the check is a trace comparison.
- Statement counts weigh a one-line getter like a 40-line branch; the report shows both counts
  and the unit list, and the headline should be units by status, not one percentage.
- The hashing and matching are not written yet, so it is unknown how many demo functions match
  uniquely by structure; that number decides whether step 2 is easy.

## The coverage map
`docs/original-coverage-map.svg` is the only copy, and it is tracked. It has one square per gameplay
line of the original and a block per function (the main timeline's functions are labeled in blue):
green where our code cites the line or uses its values, amber where only the function is named
in a comment of a file that uses the line's numbers, grey where nothing does. It holds function
names and counts, no code. `build.rs` redraws it whenever `src/` or the script changes, if the
original's scripts are in your cache (they come from your own copy of the game, so a contributor
without them leaves the file as it is). Redraw it by hand with
`scripts/original-coverage.py --svg docs/original-coverage-map.svg`.
