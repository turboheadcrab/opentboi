# Taking gameplay values and formulas from the game itself

Goal: the values in `GameParameters`, and the formulas they feed, are read from whichever SWF the
player has (the Steam release, Wrath of the Lamb or an earlier version, the browser demo) instead
of being transcribed into the source. Supporting a new version should mean data, not code.

## What exists
- `ParameterProvider` (`src/game/parameters/mod.rs`) with one provider, `TranscribedOriginal`:
  the Steam version's values, each with a citation of where it comes from.
- The `swf` crate we already parse with can read AVM1 bytecode (`swf::avm1::read`).

## Plan
1. **Read the main script's bytecode** (DoAction of sprite 6765) into a small expression tree:
   AVM1 is a stack machine (`Push`, `GetVariable`, `GetMember`, `Multiply`, `SetMember`, …), so
   replaying the stack per statement rebuilds a speed multiplied by a constant as an assignment with a literal.
2. **Anchors instead of code**: each parameter gets an anchor saying where its value lives, e.g.
   "in function `aicol()`, the literal in the first multiplication of a speed" or "element 5 of the array
   assigned to `sizes`". Anchors are a table, one per game version if versions differ, and
   tests check that the Steam version's anchors give exactly today's transcribed values.
3. **Formulas by running the original**: where versions differ in a formula's shape rather than
   its numbers (`slotf()`, chest rolls, payouts), run the original function's bytecode in a small
   AVM1 interpreter limited to arithmetic, comparisons, branches, arrays, `random()` (fed from our
   `Random`) and reads/writes of a few named variables. Our code then calls "the version's
   `slotf()`" instead of a Rust transcription; transcriptions stay as the fallback and as tests.
4. **Report the source of each value** (`ParameterSource` per value: extracted, run, or
   transcribed), and print what fell back, so a new version shows at once what is missing.

## Order
Constants first (movement, tears, pickups: the bulk of `GameParameters`), then the interpreter
for self-contained roll functions (`slotf()`, `sloty()`, chest contents), then larger logic only
where versions are known to differ. Getting a demo or pre-Wrath of the Lamb SWF early would tell
which parts actually differ.

## Status (2026-10-12)
- `src/game_assets/script/`: reads every DoAction/DoInitAction (main timeline and sprites) with
  `swf::avm1`, replays the stack machine into statements (`Assign`, `Evaluate`, `BranchUnless`,
  `Return`) and functions. Jumps are not followed yet.
- Steam SWF: 244 functions; the size table is read exactly as transcribed.
- Browser demo (`downloads/`, never committed): SWF 8, protected by "Secure SWF" (tags 253, 255):
  only 62 functions are found and their names are obfuscated, so anchors by name will not work
  there. Anchors have to match structure (statement shapes, constant patterns) instead, and the
  demo's code may be partly hidden from a plain read. Character ids differ too, so the texture
  catalog's hardcoded ids (3376, 2246, …) must also be found by name or structure to support it.

