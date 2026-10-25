//! How many frames some of the art's clips play.

/// Tear clip `b2` frames 2–17: splash against a wall or obstacle.
pub const TEAR_HIT_SPLASH_FRAME_COUNT: u8 = 16;
/// Tear clip `b2` frames 40–55 (label `groundhit`): splash when a tear falls to the floor.
pub const TEAR_GROUND_SPLASH_FRAME_COUNT: u8 = 16;

/// Fly circling a corny poop (`b96` frame 1: a 2-frame wing flap).
pub const ORBITING_FLY_FRAME_COUNT: u8 = 2;
/// A blue fly's wings (`d`, 1699, inside the friends' clip).
pub const BLUE_FLY_FRAME_COUNT: u8 = 4;
/// The splat `d` (1493) of blood creep, which wobbles through four looks.
pub const BLOOD_CREEP_LOOK_COUNT: u8 = 4;
/// Bomb clip `b4` frame 3 (2040): fireball turning into smoke. Nothing is placed after frame 26;
/// the remaining frames are empty.
pub const EXPLOSION_FRAME_COUNT: u8 = 46;
/// `locktile` frame 2 (4614); the block is gone afterwards. The last three frames are empty.
pub const KEY_BLOCK_OPENING_FRAME_COUNT: u8 = 26;
/// Bomb clip `b4` frame 1: the burning fuse (2039), with the bomb's shadow.
pub const BOMB_FUSE_FRAME_COUNT: u8 = 59;
/// `over` frame 16 holds clip 6831, 32 frames long (its last frame hides it again).
pub const ETERNAL_HEART_UNION_FRAME_COUNT: u8 = 32;
pub const CARD_NAME_FRAME_COUNT: u8 = 84;
/// The fly some shopkeeper looks carry (clip 1514) flaps between two frames.
pub const DECORATION_FRAME_COUNT: u8 = 2;
/// The overlay's frame 13 (`curs()`): The Empress's animation.
pub const EMPRESS_ANIMATION: u8 = 13;
/// Frames of `over`'s card animations (The Devil 2, Death 11, The Empress 13, The Sun 15),
/// less the last, on which the clip hides itself.
pub fn card_animation_frame_count(clip_frame: u8) -> u8 {
    if clip_frame == EMPRESS_ANIMATION { 33 } else { 31 }
}
/// Door clip frame 8: the key turns and the shop door swings open.
pub const SHOP_DOOR_UNLOCKING_FRAME_COUNT: u8 = 28;
