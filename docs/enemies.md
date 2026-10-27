# Enemies of the original

Surveyed 2026-10-14 from the room sprites (`levs1` 6622 for the Basement and Cellar, `levs6`
6732 for special rooms), the marker clips' frame scripts, FFDec renders of the enemy clips, and
the enemy code in `DoAction.as` (`create` ~2001–2600, `alive()` ~13840–14120, `firemode()`
9515–9690, `shots()` 9397–9512). The original has no enemy names, only type numbers (`s`)
and clips named `b<type>`; the names below are ours.

## How a room places enemies
- A layout frame places marker clips; each marker's script passes its value to `spawnb()`, which
  writes it into the room grid (`levz`). Half-transparent markers lose 0.1, faint ones 0.2.
- On the room's first visit every negative cell creates type 9 minus its value (`create()`), 10 px
  below the cell's center.
- The rounding remainder picks a variant: a half-transparent -5.1 marker makes a two-shot Pooter.
- Pickups use values in (-1, 0) (type 5), grid objects positive values.
- The marker `a1`–`a3` (sprite 6591) writes 0.01–0.03, which decays to 0 within a few frames
  (DoAction 12806); it is not an enemy.

## Basement enemies (`levs1`, 186 room layouts)

| Type | Clip | Marker (value) | Rooms | Placed | Ours | Notes |
|---|---|---|---|---|---|---|
| 10 | `b10` 5051 | 6597 (-1) | 25 | 68 | `Gaper` | Gaper: small pale walker (see below) |
| 11 | `b11` 3483 | 6610 (-2) | 9 | 36 | `Walker` (`WalkerKind::Headless`) | headless body: wanders, spurts blood balls (see below) |
| 12 | `b12` 5024 | 6599 (-3) | 17 | 44 | `Walker` (`WalkerKind::Horf`) | Horf: a head that shoots |
| 13 | `b13` 1517 | 6601 (-4) | 11 | 30 | `FlyKind::Black` | Fly: harmless, drifts to breakables |
| 14 | `b14` 4990 | 6603 (-5) | 11 | 19 | `FlyKind::Pooter`, `SuperPooter` | Pooter: shoots |
| 15 | `b15` 4902 | 6618 (-6) | 9 | 13 | `Walker` (`WalkerKind::Clotty`) | Clotty: wanders and shoots four ways (see below) |
| 16 | `b16` 4932 | 6615 (-7) | 14 | 30 | `Walker` (`WalkerKind::Mulligan`) | Mulligan: runs from the player, leaves flies (see below) |
| 18 | `b18` 5053 | 6589 (-9) | 24 | 65 | `FlyKind::Attack` | Attack fly: chases, half a heart |
| 19 | `b19` 5232 | 6621 (-10) | 1 | 3 | — | the chain worm (segments linked head to tail, following the head's trail; see below) |
| 29 | `b29` 4935 | 6612 (-20) | 10 | 28 | `Walker` (`WalkerKind::Hopper`) | Hopper: leaps about (see below) |
| 61 | `b61` 4872 | 6619 (-52) | 2 | 3 | `FlyKind::Red` | red fly: chases, bursts into four shots |
| 77 | `b74` 1906 | 6613 (-68) | 6 | 24 | `Walker` (`WalkerKind::FleshMound`) | flesh mound: the last, smallest piece of the family that splits (clip shared with 74–76) |
| 80 | `b80` 4904 | 6616 (-71) | 5 | 13 | `FlyKind::Moter` | Moter: two flies stuck together, chases, splits into two attack flies |

Boss rooms (`b1-0`…`b1-4`, 6732 frames 20–24) place only Monstro's marker (6664, -11 → type 20);
the floor's other bosses come from code. Mini-boss rooms (`min0`…`min6`, frames 4–10)
place the sins, types 46–52 (Sloth `b46` 5070, Lust `b47` 5302, Wrath `b48` 5280, Gluttony `b49`
5313, Greed `b50` 5269, Envy `b51` 4180, Pride `b52` 3853), one marker each (6674 … 6694).

## Shared values
- Hit points by type: 13 → 3, 14 → 8, 18 → 5, 10 → 10, 12 → 10, 15 → 15, 16 → 13, 29 → 10, 61 → 10,
  77 → 12, 80 → 10.
- Size by type (collision radius): flies and Pooters 17 (the red fly and the Moter 13, like the
  walkers), player 10, tear 8, enemy shot 5.
- Mass by type: 13 and 18 → 3, 14 → 4, 29 → 10, 61 → 7, 77 → 3, 80 → 3.
- Death: a fatal hit plays `Death_Burst_Small_0`–`2`; the clip's `d` frame 3 plays out.

## Red fly (type 61) and Moter (type 80)
- Both take the attack fly's branch of `alive()`: the player is their goal (a red fly rests 15 px
  from him, a Moter 25), and they hurt by half a heart on touch (portraits 61 and 80). Their drift
  jitters by 0.17 as the black fly's does (the attack fly's is 0.3); like the attack fly, they do not
  damp it while approaching.
- A dying red fly shoots along the four axes at 8 px per frame (`quadf()`, 13106–13121). A Moter that
  dies with more than -20 hit points left makes two attack flies, either side of it at right angles to
  the line to the player, 10 px off and sent that way (13564–13600; the original also delays them by
  10 frames, not ported). Blown further past zero it leaves nothing.
- Clips: `b61` 4872 (1 flying, `hx` 4870 on the plain look with a 4-frame flap, 3 dying 555, 4
  appearing 4871: 25 frames), `b80` 4904 (1 two flapping 578s, 3 dying 566, 4 appearing 4903: 26).

## Hopper (type 29)
- `smarts5()` gives it `randruny()` (8662–8830). The hop clip (`d` frame 2, 4933: 26 frames) drives it:
  with no leap going it picks a spot (the player's if he, two frames ahead at his speed, is within
  130 px; else 95 px off in a random direction, kept in x 20–620, y 110–447, on a free tile, within 3
  px of its center), then each frame steps the clip: a tenth of the pull while crouching (frames 2–4),
  the full pull (a tenth of the distance, at most 3) through the air, and from frame 21 on it lands
  (40 % of its speed kept, no pull) and by frame 22 the leap ends; the next spot is picked at once.
  `Meat_impacts_0`–`2` plays on the clip's 19th frame. Each frame it keeps 70 % of its speed.
- The tests for holding the clip back in mid-air and for ending the leap early read `enfget()` of a
  sum with no second argument, which is always its standstill value; they never matter.
- Hit points 10, size 13, mass 10, drawn at 1, hurts by half a heart on touch (portrait 29).
- Clip `b29` 4935: 1 sitting (before its first leap), 2 hopping (4933, 26 frames), 3 dying (566, 36),
  4 appearing (4934, 26). Not ported: the vertical-separation quirk of `ballhit()` between airborne
  enemies (`bhh`), the eternal and `alter` variants.

## Flesh mound (type 77)
- The Fistula family (74, 75, 76, 77) shares the clip `b74` 1906 and the tail of `bawssmart2()`
  (15677–15954). 74 splits into two 75, 75 into two 76, and a dying 76 leaves a 77 (13370–13390); 77 is
  the leaf: it wanders (`randrun()`, as the headless body) in the loop of its walking clip (`d` frame 13,
  1902: 24 frames), picking a new spot each time the clip starts over, keeping 80 % of its speed each
  frame, 60 % more past the clip's 19th frame. Hit points 12, size 13, mass 3, hurts by half a heart
  on touch (portrait 77). Appearing is `d` frame 15 (1903: 25 frames), dying frame 14 (566: 36).
- Not ported: the bigger members (74–76, bosses and mini-bosses) and their splitting.

## Chain worm (type 19)
- Not ported. `smarts()`' first case (23664–24210) links the segments a layout places side by side (the
  room has three), each following the trail of the one before (`beenx`/`beeny`), the head wandering
  with `randrunx()`; a segment gains 25 hit points once linked, and a head that dies hands the lead to
  the next. It needs enemies that refer to each other.

## Flies (`flyai`: 13, 14, 18, 61, 80)
- They share `alive()`'s fly branch (13907–14118): a goal every 3rd frame, approach on even
  frames, jitter on frames 4n+1, drift kept within stage x 80–540, y 210–420 on frames 3n+2.
- Black fly and Pooter head for breakables; types 18, 61 and 80 for the player.
- At rest by a breakable, one frame in 1000 it pecks it (`killshit()`, a tear's hit); a landed peck
  turns the fly into a Pooter (two-shot one time in three), keeping its hit points.
- Every live one is counted, and the root timeline loops `insect_swarm_loop.wav` at 30 % plus 30 %
  per fly, at most 150 %.

## Pooter (type 14)
- Variant 2 (two shots, 1.5× hit points) by marker fraction, or by any of three rolls (one in 21,
  25 or 100); `hx` shows it.
- `bord()` keeps it within stage x 60–575, y 150–410.
- `firemode()`: every seventh frame (offset by its number), with the cooldown spent and the player
  within 200 px and in sight (`linechecky()`), it plays `d` frame 5, nudges its drift towards the
  player, and releases the shot 8 frames later; then waits 4 such checks. While the cooldown runs it
  keeps 80 % of its speed and 70 % of its drift each frame.
- `sideflip()` mirrors it to face the player when shooting.
- Clip `b14`: 1 flying (`hx` 4945, 2-frame flap), 3 dying (4948, 15), 4 appearing (4949, 25),
  5 shooting (`hx` 4989: 14 frames, 19 for the two-shot one).

## Gaper (type 10)
- A walker: it takes `smarts()`'s walker case, not the flies' branch. While its clip is on frame 1 or 2
  it steers with `pathfind()` towards the player at 0.6 (a calm Gaper 0.83 of that); `walkframe()`
  picks the clip (walking above 2 px per frame, standing below it; a Gaper rarely gets that fast, so
  it mostly glides on the standing clip). Hit points 10, size 13, mass 5, drawn at 0.9, touch hurts
  half a heart (death note portrait 10); the unicorn's 40 damage and blasts kill it.
- `pathfind()`: each frame it marks the cell it stands on with 0.9 (unless the cell holds 0.99 or
  more), and every sixth frame (offset by its number) a line from its tile's center to the player's
  tile's center is tested (`linecheck()`: every 6 px, at the corners of a 10 px square, with 2.5 to
  spend: a cell of 1 or more, rocks, pits, walls and doorways, ends the sight; one above 0.3 and
  below 1, a walker's mark or what a broken object leaves, costs 0.1; spikes and webs, 0.99, cost
  nothing). Its own cell is not skipped: the start cell's number (`grox`) is compared with the
  cells' values, which never equal it. `actions2()` fades every value between 0 and 0.99 by 0.1
  each third frame. With a clear line, or on the player's tile, it pulls straight at the
  player; otherwise at the next tile on the shortest way, from a flood fill over the 13×7 tiles
  (east, west, north, south as the tie-break) redone as a timer runs out (at most 20, plus 7, plus 2
  per enemy; each frame costs 3 times its pull factor, 10 while it sees the player). The pull is a
  tenth of the distance (at most 2) times the factor; the speed keeps 70 % first. With nowhere to
  go it keeps 50 %; standing on its goal 85 %.
- The marker's fraction: a plain marker's Gaper is "frantic" (`gonuts`): factor 1 (calm 0.83), 1.1
  more while it sees the player, and its head clip `h` (5043, 22 frames: 1 calm, 2 frantic, 3 and on
  the shouting) steps on each frame it sees the player; once past frame 2 it is 1.2 times faster.
- Clip `b10` 5051: 1 standing (body 4543, head `h`), 2 walking (5048, 22 frames, head included),
  3 dying (566, 36 frames, bursts far past the body: the canvas is 236×276), 4 appearing (5050,
  46 frames, `d.h` on the calm or frantic frame).
- Sounds: on a sight check that finds a clear line it growls with one chance in three, at most once
  in 40 frames (`SoundEvent::GaperGrowled`); its death is the common enemy death burst.
- Half of the Gapers a hit leaves with more than −25 hit points (a coin flip first) do not die:
  they carry on as the headless body below, at full health, without a sound or a death clip. A
  blast or the unicorn (40 damage) kills outright.
- Not ported yet: walkers slowing on tiles another walker marked and preferring free tiles in the
  flood fill (`levz` 0.9); champions.

## Headless body (type 11)
- Made by a Gaper's lethal hit (see above), keeping its position, speed, drawing scale (0.9) and
  whether it spurts (`bnuts`: one Gaper in three); a layout's own (marker 6610) spurts unless the
  marker has a fractional value, and appears first (`d` frame 4, 3482, 26 frames).
- It is `outway`: it takes no part in enemy-to-enemy contact (it pushes no one and no one pushes
  it), but touching it hurts half a heart (death note portrait 11) and tears hit it.
- `smarts()`'s case (24820–24850): each frame it keeps 75 % of its speed, and `randrun()` gives it a
  goal: a point 40 pixels off in a random direction (kept in the room's middle, x 120–540, y 210–360,
  one time in two), if the tile there is free; it pulls towards it by 0.5 per frame and gives up on
  arriving (within 3) or, when nearly stopped (below 0.2), one frame in ten. `walkframe(2)` shows it
  walking while twice its speed is above 2 pixels per frame.
- A spurting body (`firewalk()`, 9712) shoots a blood ball ahead of where it goes, every 40 frames
  plus a random 0–19: speed 5, rising at 5 and falling 0.3 faster than a Pooter's. It plays
  `bloodshoot`.
- Clip `b11` 3483: 1 standing (body 3468, the stump's blood `bb`, 10 frames, hidden unless it
  spurts), 2 walking (3481, 20 frames), 3 dying (566, 36 frames), 4 appearing (3482, 26 frames).

## Horf (type 12)
- Never moves of its own accord (each frame it keeps 70 % of any speed a hit gave it); hit points 10,
  size 13, mass 3. The shared tail of `smarts()` (25235–25285) runs `firemode(200, 4)` for it: the
  Pooter's shooting exactly (see above), one shot straight at the player, `Shakey_Kid_Roar_0`–`2`.
- Clip `b12` 5024: 1 idle (4994, 3 frames), 3 dying (566, 36 frames), 4 appearing (5013, 60 frames),
  5 shooting (5023, 20 frames; mirrored to face the player).

## Clotty (type 15)
- Hit points 15, size 13, mass 10. Each frame it keeps 90 % of its speed. Its `smarts()` case
  (24381–24470) has two modes, picked at random at first: wandering (`randrun()`, the headless body's)
  and shooting. Arriving at its spot turns wandering into shooting; shooting plays `d` frame 5
  (4900, 24 frames) and, once that clip is past its tenth frame, fires four shots along the axes at 7 px
  per frame (`quadf()`, 9180), `animal_squish0`–`2`. It goes back to wandering when the clip is over,
  or one frame in 30 once 30 frames have passed since it last wandered.
- Clip `b15` 4902: 1 idle (4877, 24 frames), 2 walking (4894, 20), 3 dying (566, 36), 4 appearing
  (4895, 27), 5 shooting (4900, 24), 6 the eternal one's (4901). Every fifth frame (offset by its
  number) it leaves a blood splat (`splater()`, scale 0.5–1.2). Its odd chance of becoming a type 17
  (which never happens in the original: the test compares a boolean to 2) is not ported.

## Mulligan (type 16)
- Hit points 13, size 13, mass 6; it keeps 80 % of its speed each frame. Touching it does not hurt
  (`ballhit()`'s list of types that hurt on touch, 7405–7500, has no 16; like the black fly and the
  Pooter it is only trampled by the unicorn). Its case (24276–24372)
  steers with `angstfind()` (9766–9921): every 14th frame (offset by its number) it looks for the
  player; one in clear view within 250 px, or hidden within 170, scares it for three looks (and in view it
  whimpers, `Scared_Whimper_0`–`2`, one time in four after a pause of 45 frames and a random 100). Scared,
  it is pushed away by 1.5 over the distance (within 500 px) and looks, up to 7 times (fewer after failed
  searches), for a spot 30–480 px off, in its view and out of the player's, then runs there at 1.3 times
  a Gaper's pull, giving the spot up every tenth frame if the player could see it or is within 200 px
  of it; calm, it wanders (`randrun()`, then keeps 90 % of its speed more). The refuges' cell checks
  (`checked`) are not ported.
- Pressed against a wall, a rock or something else (`gh`) it bursts one frame in 300: `d` frame 5
  (4930, 70 frames), and past its 20th frame it fires four shots along the axes (as the Clotty) and dies.
- Dying (a burst, or killed) it leaves four flies unless it was left below −20 hit points
  (`deathscripts()`, 13717–13741, `flya()`, 10509): each at a random spot within 6 px, a black fly
  seven times in eleven, a Pooter once and an attack fly three times.
- Clip `b16` 4932: 1 standing (a body of its own and a hand `hx`, frame 1 calm or 2 scared), 2 walking
  (4922, 22 frames, with an `hx`), 3 dying (566, 36), 4 appearing (4923, 22), 5 bursting, 6 eternal.

## Contact between enemies (`physix()`, 25625–25690)
- Every enemy not yet dying (and not `outway`) is entered into a bitmap of 100-pixel cells, ten
  slots each, in the four cells around it; `ballhit()` runs on every pair sharing a cell, once per
  frame. A full cell gives its eleventh enemy the first slot of the cell beside it, over whoever was
  there; slots beyond the bitmap are lost. A fly is `outway` on even frames, so it takes part on
  odd ones only; a fly or Pooter still appearing takes part.
- Positions part at once; the speed traded is gathered and added after the pass.
- Anything created this very frame (`fra`) touches nothing until the next: a shot that leaves in a
  frame cannot hurt the player in it. Layout enemies are made before any frame passes.

## Enemy shots (type 9, `bullet1` 1459)
- `shots()` → `ffmo()`: speed 7 towards the player; the two-shot Pooter fires two, each turned
  aside by a fifth of the aim.
- Height starts at 23 above the floor, rising at 3; each frame the rate keeps 90 % and gains 0.2,
  and the height moves by it. It lands lower than 5 above the floor (a splatter one time in two),
  and ends against walls and solid cells (`mhity()`) or on the player (half a heart).
- It leaves a `bloodsplash` (1488, 17 frames) at a random rotation.
- Its brightness pulses ×0.45, ×1, ×1.85 over three frames.
- Shadow 341 (the tear's), ball `d` (1458) authored 23.75 above the origin and moved to `dy`.

## Blood on the floor (`splater()`, ported)
- Drawn into the floor's `splat` bitmap (shown at 70 %, craters too) with the blood tint `bloc`
  (each channel 85–100 %, drawn again every seventh frame), a random look of the clip `blood` (6243;
  the red looks are frames 1–10, consecutive pairs alike), upside down one time in two, `crand(5)` off
  and 5 px up. A look of 0 keeps the clip on the frame it was last sent to. The bitmap is made anew
  for each room, so blood is gone when the room is entered again.
- Who leaves it: every dying enemy on its death clip's first three frames (scale 0.4–1.1, a Pooter
  0.7 of that; both Pooters are type 14), a Gaper losing its head (0.6–1.6), the Clotty every fifth
  frame, the headless body one frame in ten (0.4–0.9, look 0–9), an enemy shot one frame in sixteen
  in flight (0.1–0.4), and a tear or enemy shot ending, one time in thirty (0.1–0.9).
- Not ported: the other colors (`spl` 10, 20, 30: bosses, items and poison), dark rooms (`blackout`
  darkens the tint; a blast there splats instead of leaving a crater), bomb tears' splats, the other
  characters' and items' blood, and bosses'.

## Not ported yet
- Champions (one in 20 on a first visit), eternal enemies, knockback from blasts, every enemy
  marked "—" above.

## What the floor does to enemies (ported)
- `firecheck()` is called for every enemy too (`alive()`, 13844–13851): a burning fireplace within 38
  px of one of the four corners 20 px from it pushes it away by 4 px per frame and hurts it for 8
  every 10 frames; spikes hurt walkers the same way (without a push), never flies.

## Blue flies (`friends()`, 11827–12210)
- Isaac's familiars (clip `b3` 1788, frame 128, its `d` 1699 with four wing frames). One is made
  10 px above him each frame while some are owed (the Friends pill 3, a red chest 2–4); up to ten
  follow him into the next room.
- Each circles him on a ring 35 wide and 30 high (0.1 larger per fly past four), the flies spread
  over it and turning 0.03 half-turns a frame; every other frame it is pushed towards its place by
  0.15 of the gap (at most 2, doubled), and keeps 80 % of its speed. One frame in three it goes
  for the enemy hurt last instead, until that enemy is gone.
- Touching an enemy it hurts it for 1.2 times Isaac's damage plus 3 (once in 14 frames per enemy)
  and then dies one time in two; an enemy shot it meets is gone, with the same chance of dying.

## Hit flash (ported)
- A hit an enemy survives flashes it red for the damage divided by 1.5 frames, at most 6, rounded
  (`colorit()`, 14719): 3 frames for a tear's 3.5 damage. A fatal hit does not flash.
- A fly turning into a Pooter gets the new clip on its flying frame at once (`attach()`, `atta()`),
  keeping its hit points: a black fly's 3 or fewer, so one tear kills it.
- Every frame the script puts the variant clip on the variant's frame at every depth (31501): the
  Pooter's variant shows from its appearing clip on.
