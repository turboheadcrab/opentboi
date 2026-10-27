//! Names of the artwork the game draws, each rendered to exactly `world_size()` around its origin point.

use crate::game::floor::MapPiece;
use crate::game::hud::{HudHeart, held_trinket::SlotView};
use crate::game::input_state::InputDevice;
pub use crate::game::pickups::phase::PickupPhase;
use crate::game::pickups::{kind::PickupKind, pill::PillLook, trinket::Trinket};
pub use crate::game::room::doors::DoorStyle;
use crate::game::room::grid_kind::{BreakableKind, TileKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextureId {
    /// Walls and floor of a room, covering [`crate::game::room::ROOM_SIZE`].
    RoomBackground(RoomArt),
    /// The in-game HUD, covering the whole stage.
    Hud,
    /// The death note (`hud.paus` frame 3) naming what killed Isaac, covering the stage.
    DeathNote(crate::game::player::death::Killer),
    /// The beam of light over dying Isaac, at fade step 0 (none) to 16 (full).
    DeathSpotlight(u8),
    /// The pattern around the stage on screens of another shape, with a stage-sized hole.
    StageSurround,
    /// The HUD's coin, bomb and key counter showing `0..=99` (`"0" + n` below 10).
    HudCounter(u8),
    /// One heart of the HUD's life bar (`lif.h0`, ...).
    HudHeart(HudHeart),
    /// Frame `0..ETERNAL_HEART_UNION_FRAME_COUNT` of the full-screen animation of two eternal
    /// halves becoming a heart (`over` frame 16).
    EternalHeartUnion(u8),
    /// The HUD's active item box: frame `0..ITEM_BOX_FRAME_COUNT`, with the item's picture.
    ActiveItemBox {
        item: Option<crate::game::pickups::item::Item>,
        frame: u8,
    },
    /// The HUD's key icon (`hud.kep`): golden once the golden key is found.
    HudKeyIcon {
        golden: bool,
    },
    /// A frame of a pickup, from 0 up to the phase's frame count for its kind.
    Pickup {
        kind: PickupKind,
        phase: PickupPhase,
        frame: u8,
    },
    /// A frame of a pill lying about, in the look this run gives it (see `PillLooks`).
    PillPickup(PillLook, PickupPhase, u8),
    /// A minimap room or icon (clip `mpiece`).
    MapPiece(MapPiece),
    /// The start room's floor drawing of the controls, the size of [`Self::RoomBackground`].
    StartRoomControls,
    /// The bomb control inside [`Self::StartRoomControls`]'s gap, named for the device it is for.
    StartRoomHint(crate::game::floor::StartRoomControl, InputDevice),
    /// The label above the active item's box in the HUD, naming the item button of a device.
    ActiveItemLabel(InputDevice),
    /// The badge on the held card's name strip, naming the card button of a device.
    HeldCardLabel(InputDevice),
    Tear,
    /// The tear the devil's boost gives: dark.
    DarkTear,
    TearShadow,
    Tile(TileKind),
    KeyBlock,
    /// What a blast leaves of a rock (rubble) or a spider web (torn strands), or a bridged pit.
    ClearedTile(TileKind),
    /// The floor a blast broke between two pits (`breakall()`, tile sheet frame 67).
    BrokenFloor,
    /// A stage the kind has, and a frame of that stage's idle loop.
    BreakableStage {
        kind: BreakableKind,
        stage: u8,
        idle_frame: u8,
    },
    /// `frame < kind.destroyed_frame_count()`.
    BreakableDestroyed {
        kind: BreakableKind,
        frame: u8,
    },
    /// `frame < kind.regrow_frame_count()`.
    BreakableRegrow {
        kind: BreakableKind,
        frame: u8,
    },
    /// Frame `0..KEY_BLOCK_OPENING_FRAME_COUNT`.
    KeyBlockOpening(u8),
    /// Frame `0..ORBITING_FLY_FRAME_COUNT`.
    OrbitingFly(u8),
    /// Frame `0..EXPLOSION_FRAME_COUNT`.
    Explosion(u8),
    /// A door (clip 6550), drawn on the right wall and turned for the others.
    Door(DoorTexture),
    /// Frame `0..BOMB_FUSE_FRAME_COUNT`.
    BombFuse(u8),
    /// Scorch mark an explosion leaves on the floor (`crater`).
    ExplosionScorch,
    /// Frame `0..TEAR_HIT_SPLASH_FRAME_COUNT`.
    TearHitSplash(u8),
    /// Frame `0..TEAR_GROUND_SPLASH_FRAME_COUNT`.
    TearGroundSplash(u8),
    /// An enemy shot's blood ball at brightness step `0..ENEMY_SHOT_BRIGHTNESS_STEPS`.
    EnemyShot(u8),
    /// Frame `0..BLOOD_SPLASH_FRAME_COUNT` of an enemy shot's splash.
    BloodSplash(u8),
    /// Isaac's head or body, in a ride color or his own.
    Isaac {
        part: IsaacPart,
        tint: Option<RideColor>,
    },
    /// Frame `0..frame_count()` of an animation Isaac plays as a whole.
    IsaacAnimation {
        animation: crate::game::player::animation::PlayerAnimation,
        frame: u8,
    },
    /// A figure a layout places (the devil statue, shopkeepers), in look `look` from 0; `frame`
    /// animates what loops on it (the flies on some shopkeepers).
    Decoration {
        decoration: crate::game::room::layout::Decoration,
        look: u8,
        frame: u8,
    },
    /// A fly: `frame` of `animation`; `wing` is the black fly's wings' frame while flying.
    Fly {
        kind: FlyKind,
        animation: FlyAnimation,
        frame: u8,
        wing: u8,
        /// Flashing red from a hit (`hurtcol`).
        flashing: bool,
    },
    /// A Gaper: `frame` of `animation`.
    Gaper {
        animation: GaperAnimation,
        frame: u8,
        /// Flashing red from a hit (`hurtcol`).
        flashing: bool,
    },
    /// A headless body (a Gaper's, or a layout's own): `frame` of `animation`.
    Headless {
        animation: HeadlessAnimation,
        frame: u8,
        /// Flashing red from a hit (`hurtcol`).
        flashing: bool,
    },
    /// Horf: `frame` of `animation`.
    Horf {
        animation: HorfAnimation,
        frame: u8,
        /// Flashing red from a hit (`hurtcol`).
        flashing: bool,
    },
    /// One of Isaac's blue flies, on `frame` of its four wing frames.
    BlueFly(u8),
    /// A blood creep puddle (`bloo`, 1494) in one of its looks.
    BloodCreep(u8),
    /// A blood splat on the floor (`blood`, 6243) in one of its red looks.
    BloodSplat(u8),
    /// A Clotty: `frame` of `animation`.
    Clotty {
        animation: ClottyAnimation,
        frame: u8,
        /// Flashing red from a hit (`hurtcol`).
        flashing: bool,
    },
    /// The Hopper or the flesh mound.
    Enemy(EnemyTexture),
    /// A Mulligan: `frame` of `animation`; `scared` shows its hand scared (it is running from the
    /// player).
    Mulligan {
        animation: MulliganAnimation,
        frame: u8,
        scared: bool,
        /// Flashing red from a hit (`hurtcol`).
        flashing: bool,
    },
    /// A character of the HUD font (one of `HUD_GLYPHS`), for the stats panel.
    HudGlyph(u8),
    /// The minimap's mark for what is left in a room (`itmc` frame 1–10, see `map_mark`).
    MapPickupMark(u8),
    /// A shop ware: pickup clip frame 15 showing ware frame `n` (see `shop::Ware::clip_frame`).
    ShopWare(u8),
    /// The pill ware (frame 3, or 7 on sale) showing the pill or card with this `col`.
    ShopConsumableWare {
        color: u8,
        on_sale: bool,
    },
    /// Frame `0..MESSAGE_FRAME_COUNT` of a pickup message.
    PickupMessage {
        message: crate::game::hud::message::PickupMessage,
        frame: u8,
    },
    /// The HUD's picture of the card held, and its name.
    HeldCard(crate::game::pickups::card::Card),
    HeldCardName(crate::game::pickups::card::Card),
    /// The same for a pill held (by its look), and the name of its effect (or the unknown one).
    HeldPill(PillLook),
    HeldPillName(crate::game::hud::pill_text::PillText),
    /// The HUD's corner with a trinket held: picture, name and, unless it animates, the ring.
    TrinketSlot(Trinket, SlotView),
    /// The HUD's card (or pill) slot animating: taken (`pilll` frame 2) or used (frame 3).
    CardSlotAnimation {
        used: bool,
        frame: u8,
    },
    /// Frame `0..CARD_NAME_FRAME_COUNT` of the banner (`hud.st1`) showing a card's or an item's name.
    NameBanner {
        name: crate::game::hud::name_banner::BannerName,
        frame: u8,
    },
    /// Frame `0..card_animation_frame_count()` of `over` frame `clip_frame` (The Devil 2,
    /// Death 11, The Sun 15).
    CardAnimation {
        clip_frame: u8,
        frame: u8,
    },
    /// Frame `0..BUBBLE_FRAME_COUNT` of the bubble round Isaac while a trinket shields him.
    ShieldBubble(u8),
    /// A gambling machine or beggar: machine clip `frame` (see `machines::clip`), its animation's
    /// frame `animation` from 0. Rendered on first use only; [`TextureId::on_demand_textures`]
    /// lists them.
    Machine {
        /// The fortune machine's or the devil beggar's look.
        alternate: bool,
        frame: u8,
        /// While shuffling: the shell game's prize (`itt`).
        prize: u8,
        animation: u8,
    },
}

mod all;
mod all_enemies;
mod clotty;
mod enemy;
mod frame_counts;
mod gaper;
mod headless;
mod horf;
mod isaac;
mod motions;
mod mulligan;
#[cfg(test)]
mod on_demand;
mod room_art;
mod size;
pub use {clotty::*, enemy::*, frame_counts::*, gaper::*, headless::*, horf::*, isaac::*, motions::*, mulligan::*};
pub use {room_art::RoomArt, size::CARD_NAME_TEXTURE_CENTER_X};
