# The door clip's frames (sprite 6550, `d1`–`d4`, 41 frames)

Surveyed 2026-10-13 by rendering every frame (a throwaway probe test drawing sprite 6550 frame by
frame) and reading the door code in `DoAction.as` (room setup ~5500–5630, opening ~30560–30760).
The clip is drawn for the left wall; the game rotates it per side.

| Frames | What it shows | Used by the original | Ours |
|---|---|---|---|
| 1 | plain door, open | shop or library door once unlocked (30652) | — |
| 2 | plain door | | — |
| 3 | plain door opening: its `p` (6471) swings open over 9 frames | doors opening once a room is cleared (30773; `p` jumps to its end in the room's first second) | `DoorTexture::Opening` |
| 4 | plain door slamming shut: its clip (6472) shuts over 23 frames, `Door_Heavy_Close.mp` on frame 14 | every door on entering a room (5505); its first frame looks open | `OPEN_DOOR_FRAME`, `DoorTexture::Closing` |
| 5 | plain door in smoke (6477, 8 frames, then holds) | a shut plain door a blast reached (29772); not in the mini-boss room | `DoorTexture::Blown` |
| 6–7 | plain door | | — |
| 8 | shop door unlocking (28-frame clip 6481, ends open) | key used on the shop | `UNLOCKED_SHOP_DOOR_FRAME` |
| 9 | shop door locking (11-frame clip 6482, ends on the keyhole) | locked shop/library | `LOCKED_SHOP_DOOR_FRAME` |
| 10 | empty | | — |
| 11 | plain door | | — |
| 12 | empty (removes frame 11's parts) | devil doors are stopped here first (5617) | — |
| 13 | boss door, red glow (open) | boss doors once the room is clear, in its first second (30762) | `BOSS_DOOR_FRAME`: `DoorTexture::Open(Boss)` |
| 14 | boss door, barred (closed) | | — (the end of 16 looks the same) |
| 15 | boss door opening: its `p` (6493) fades the red glow in over 19 frames, the door already open | boss doors once the room is clear (30766) | `DoorTexture::Opening(Boss)` |
| 16 | boss door swinging shut (6494, 11 frames, silent) | every boss door on entry (5622) | `DoorTexture::Closing(Boss)` |
| 17 | blown hole in the wall | secret room walls once bombed (5570) | `BLOWN_OPEN_DOOR_FRAME` |
| 18 | dark opening appearing | devil door appearing after the boss, after the room's first second (30748) | — |
| 19 | devil door, open | devil door (at once in the room's first second), 30684 | `DEVIL_DOOR_FRAME` |
| 20–23 | dark door with red lights | 21: the arcade door once opened (30665) | — |
| 24–27 | dark barred doors | 27: the challenge room door (in the room's first half second, 30676) | — |
| 28–29 | empty | | — |
| 30–33 | golden treasure door: 30 locked, 31 the key turning (6540, 28 frames), 32 swinging shut (6541, 23 frames, `Door_Heavy_Close.mp` on 14), 33 its `p` (6542) swinging open (9 frames) | 30 or 32 on entry (32 when unlocked: always in chapter 1, 31963), 33 once clear | 32: `DoorTexture::Closing(Treasure)`, its first frame `DoorTexture::Open(Treasure)`; 33: `DoorTexture::Opening(Treasure)` |
| 34 | empty | | — |
| 35–38 | spiked door | 36: curse door shut (5569); 37: its `p` (6545) opening (9 frames); 38 (6546) unused | 36: `DoorTexture::Closing(Curse)`; 37: `DoorTexture::Opening(Curse)`, its end `DoorTexture::Open(Curse)` |
| 39 | empty | | — |
| 40 | white glow appearing | angel door appearing (after the room's first second) | — |
| 41 | white angel door with a cross | angel door open | — |

## Timing (2026-10-14)
On entry every door goes to its closing frame (4 for plain doors). While enemies live a counter
restarts each frame; once none does it counts up and the doors open at 10, ten frames after the
last enemy died (it starts at 9 when a room loads, so an empty room's doors open on its first
frame). The doors are usable as soon as no enemy lives. In the room's first second plain doors'
`p` jumps to its end, and boss doors go to 13 instead of 15.

## Blowing doors open (2026-10-14)
Only plain doors have a doorway cell a blast can clear (a value `bloww()` treats like a rock);
boss, treasure, curse, shop and library doors keep the wall's value. A cleared cell while the doors
are shut blows the door: usable at once, frame 5, and it stays in smoke when the room is cleared.
In the mini-boss room (and with Mom's parts) the doorway closes again instead.
