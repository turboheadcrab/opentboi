# Button pictures

The SVG files here are the shoulder buttons, left triggers, D-pad directions and face buttons of Kenney's "Input Prompts"
pack, version 1.5a, which Kenney dedicated to the public domain (CC0 1.0; see `License.txt`).

- Download: <https://kenney.nl/assets/input-prompts> (`kenney_input-prompts_1.5.zip`)
- SHA-256 of that zip: `ac2fcf599080b0f3ba2d174c9474db6df1a0e96ff0662580e2da79a122ab78a1`
- Files are copied unchanged from the pack's `Vector` folders:

  | File here | Pack folder |
  |---|---|
  | `xbox_lb_outline.svg` | `Xbox Series/Vector` |
  | `playstation_trigger_l1_outline.svg` | `PlayStation Series/Vector` |
  | `switch_button_l_outline.svg` | `Nintendo Switch/Vector` |
  | `steamdeck_button_l1_outline.svg` | `Steam Deck/Vector` |
  | `xbox_rb_outline.svg` | `Xbox Series/Vector` |
  | `playstation_trigger_r1_outline.svg` | `PlayStation Series/Vector` |
  | `switch_button_r_outline.svg` | `Nintendo Switch/Vector` |
  | `steamdeck_button_r1_outline.svg` | `Steam Deck/Vector` |
  | `xbox_lt_outline.svg` | `Xbox Series/Vector` |
  | `playstation_trigger_l2_outline.svg` | `PlayStation Series/Vector` |
  | `switch_button_zl_outline.svg` | `Nintendo Switch/Vector` |
  | `steamdeck_button_l2_outline.svg` | `Steam Deck/Vector` |
  | `xbox_dpad_{up,left,down,right}.svg` | `Xbox Series/Vector` |
  | `playstation_dpad_{up,left,down,right}_outline.svg` | `PlayStation Series/Vector` |
  | `switch_dpad_{up,left,down,right}.svg` | `Nintendo Switch/Vector` |
  | `xbox_button_{a,b,x,y}_outline.svg` | `Xbox Series/Vector` |
  | `playstation_button_{triangle,square,cross,circle}_outline.svg` | `PlayStation Series/Vector` |

The Xbox and Switch D-pads are the filled variants (PlayStation's is outlined: its four separate
arrows look heavy filled): the pad in white and the pressed arm in red. The game draws in
one ink, so it fills the pad and outlines the pressed arm, as Rebirth's hints do.

The pack's Steam Deck D-pad is the Xbox one, and its lettered face buttons are the same picture for
Xbox, Switch and the Steam Deck (checked byte for byte), so only the Xbox files are kept for those.

They are compiled into the binary (`src/game_assets/input_glyphs.rs`). The console shapes are used only
to name a button; this project is not affiliated with Sony, Nintendo, Microsoft or Valve.
