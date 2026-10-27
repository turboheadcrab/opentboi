/// Something the player should hear. The game only reports what happened; frontends choose and
/// play the matching audio.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SoundEvent {
    TearFired,
    /// A tear splashed against a wall or fell at the end of its range.
    TearImpact,
    /// A poop's last hit (`plop.wav`).
    PoopDestroyed,
    /// A fireplace put out by tears (`steam_halfsec.wav`).
    FireExtinguished,
    /// A TNT barrel or bomb going off (`boss1_explosions` 0–2).
    Explosion,
    /// A bomb placed (`Fetus_Land` 0–1).
    BombPlaced,
    /// A rock blown apart (`Rock_crumble` 0–2).
    RockCrumbled,
    /// Entering the treasure room for the first time (`treasure room enter.mp3`).
    TreasureRoomEntered,
    /// Entering a room for the first time: the boss room's portcullis (`castleportcullis.wav`),
    /// the mini-boss room (`miniboss.mp3`), the devil room (`Satan find.mp3`).
    BossRoomEntered,
    /// An enemy killed (`Death_Burst_Small_` 0–2).
    EnemyDied,
    /// A Gaper sighting the player (`Zombie_Walker_Kid`).
    GaperGrowled,
    /// A Pooter's or the headless body's blood ball leaving (`bloodshoot`).
    BloodShot,
    /// Horf's shot leaving (`Shakey_Kid_Roar`).
    HorfShot,
    /// A Clotty's shot squelching out (`animal_squish`, 0–2).
    ClottySquished,
    /// A Mulligan sighting the player and whimpering (`Scared_Whimper_`, 0–2).
    MulliganWhimpered,
    /// A Hopper landing from a leap (`Meat_impacts_`, 0–2).
    HopperLanded,
    /// Doors slamming shut on a room with enemies (`Door_Heavy_Close.mp`) and opening once it is
    /// cleared (`Door_Heavy_Open.mp`).
    DoorsClosed,
    DoorsOpened,
    MiniBossRoomEntered,
    DevilRoomEntered,
    /// Bombing open the floor's secret room (`secret room find.mp3`).
    SecretRoomFound,
    /// Bombing open the super secret room (`isaacxpsecretroomreveal2.mp3`).
    SuperSecretRoomFound,
    /// A key used on a key block (`Unlock00.wav`).
    KeyUsed,
    /// The key block breaking open (`metal_blockbreak` 0–1).
    KeyBlockBroken,
    /// Coins taken (`coincol()`): `pennypickup.mp`, `nickelpickup.mp`, `dimepickup.wav`.
    PennyPickedUp,
    NickelPickedUp,
    DimePickedUp,
    /// `KeyPickup_Gauntlet.wav`; the golden key plays `goldenkey.wav`.
    KeyPickedUp,
    GoldenKeyPickedUp,
    /// `fetus_feet` 0–1.
    BombPickedUp,
    /// A red heart (`boss2_bubbles` 0–1), a soul heart (`Holy.mp`), an eternal heart
    /// (`superholy.wav`).
    HeartPickedUp,
    SoulHeartPickedUp,
    EternalHeartPickedUp,
    /// `Isaac_Hurt_Grunt` 0–2.
    PlayerHurt,
    /// `isaacdies.wav`.
    PlayerDied,
    /// An item taken from its pedestal: a choir (other rooms pick other jingles, not made yet).
    ItemTaken,
    /// A cleared room charged the active item (`beep.wav`), or charged it full
    /// (`itemrecharge.wav`).
    ItemChargeGained,
    ItemRecharged,
    /// `Chest_Open.mp`.
    ChestOpened,
    /// Touching a burning fireplace (`Firedeath_hiss.wav`).
    FireHiss,
    /// Pickups hitting the floor: a heart (`Meat_Feet_slow0.wav`), coins (`pennydrop.mp`,
    /// `nickeldrop.mp`, `dimedrop.wav`), a key (`Key_drop0.wav`), a bomb (`Fetus_Land_1.wav`) and a
    /// chest (`Chest_Drop.mp`).
    HeartLanded,
    PennyLanded,
    NickelLanded,
    DimeLanded,
    KeyLanded,
    BombLanded,
    ChestDropped,
    /// Gambling: a coin into a slot or fortune machine (`Coin_Slot.mp`), the reels stopping
    /// (`slot touched.wav`), blood given (`bloodbank touched.wav`), a machine paying out
    /// (`slotspawn` 0–2, the blood bank's `bloodbank spawn` 0–1), Isaac's thumbs up or down
    /// (`thumbsup.wav`, `thumbs down.wav`), a beggar leaving (`bum shoot off.wav`), the shells
    /// shuffled (`shellgame` 0–1) and lifted (`shell game.wav`), and a lost shell game
    /// (`boss2intro_errorbuzz.wav`).
    CoinInserted,
    ReelsStopped,
    BloodGiven,
    MachinePaidOut,
    BloodBankPaidOut,
    ThumbsUp,
    ThumbsDown,
    BeggarLeft,
    ShellsShuffled,
    ShellsLifted,
    ShellGameLost,
    /// Cards: taken (`Book Page Turn 12.wav`), a teleport starting and ending (`hell_portal2.wav`,
    /// `hell_portal1.wav`), something summoned (`summonsound.wav`), The Devil (`Devil_Card.mp`),
    /// Death (`Death_Card.mp`), The Chariot's tune (`isaacunicorn.mp`).
    CardPickedUp,
    /// A card landing (`scamper` 0–1).
    CardLanded,
    /// A pill landing (`Fetus_Jump_` 0–1).
    PillLanded,
    /// A pill taken makes the shell game's shuffle (`shellgame` 0–1, [`Self::ShellsShuffled`]); a
    /// good one used says so (`Powerup_spewer.wav`), `derp.mp` goes with the pill-eater's face.
    GoodPill,
    Derp,
    TeleportStarted,
    TeleportArrived,
    Summoned,
    DevilCard,
    DeathCard,
    ChariotTune,
    /// Health gained by a trinket (`Vamp_Gulp.mp`).
    HealthGulped,
    /// Bad Gas (`fart.mp`).
    Fart,
    /// The Empress (`curs()`: `isaacsatanitemget.mp`).
    EmpressCard,
    /// Flies buzzing (`insect_swarm_loop.wav`): never an event, only a [`LoopingSound`].
    FlySwarm,
    /// An enemy shot landing on the floor (`splatter` 0–2).
    ShotSplattered,
}

/// A sound the frontend keeps looping while the game reports it, at `volume` (1 is the
/// recording's own; the original's 100 %).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoopingSound {
    pub sound: SoundEvent,
    pub volume: f32,
}

impl SoundEvent {
    /// A jingle that takes the music down while it plays (the original keeps it as `mmus`): the
    /// first visit to the treasure, mini-boss and devil rooms, a secret room found, the Empress
    /// and the Chariot's tune.
    pub fn ducks_music(self) -> bool {
        matches!(
            self,
            Self::TreasureRoomEntered
                | Self::MiniBossRoomEntered
                | Self::DevilRoomEntered
                | Self::SecretRoomFound
                | Self::SuperSecretRoomFound
                | Self::EmpressCard
                | Self::ChariotTune
        )
    }
}
