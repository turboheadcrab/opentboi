//! The sound effects: SWF export names of each event's variants.

use crate::game::SoundEvent;

/// SWF export names (all MP3) of each event's variants; the original picks one per playback.
pub(super) const SOUND_CATALOG: [(SoundEvent, &[&str]); 74] = [
    (
        SoundEvent::TearFired,
        &["Tears_Fire_0.mp3", "Tears_Fire_1.mp3", "Tears_Fire_2.mp3"],
    ),
    (
        SoundEvent::TearImpact,
        &["TearImpacts0.mp3", "TearImpacts1.mp3", "TearImpacts2.mp3"],
    ),
    // Exported with a .wav name but stored as MP3, like most of the game's sounds.
    (SoundEvent::PoopDestroyed, &["plop.wav"]),
    (SoundEvent::FireExtinguished, &["steam_halfsec.wav"]),
    (
        SoundEvent::Explosion,
        &[
            "boss1_explosions0.wav",
            "boss1_explosions1.wav",
            "boss1_explosions2.wav",
        ],
    ),
    (SoundEvent::KeyUsed, &["Unlock00.wav"]),
    (SoundEvent::TreasureRoomEntered, &["treasure room enter.mp3"]),
    (SoundEvent::BossRoomEntered, &["castleportcullis.wav"]),
    (
        SoundEvent::EnemyDied,
        &[
            "Death_Burst_Small_0.mp3",
            "Death_Burst_Small_1.mp3",
            "Death_Burst_Small_2.mp3",
        ],
    ),
    (SoundEvent::GaperGrowled, &["Zombie_Walker_Kid.mp3"]),
    (
        SoundEvent::BloodShot,
        &["bloodshoot0.wav", "bloodshoot1.wav", "bloodshoot2.wav"],
    ),
    (
        SoundEvent::HorfShot,
        &[
            "Shakey_Kid_Roar_0.mp3",
            "Shakey_Kid_Roar_1.mp3",
            "Shakey_Kid_Roar_2.mp3",
        ],
    ),
    (
        SoundEvent::ClottySquished,
        &["animal_squish0.wav", "animal_squish1.wav", "animal_squish2.wav"],
    ),
    (
        SoundEvent::MulliganWhimpered,
        &["Scared_Whimper_0.mp3", "Scared_Whimper_1.mp3", "Scared_Whimper_2.mp3"],
    ),
    (
        SoundEvent::HopperLanded,
        &["Meat_impacts_0.wav", "Meat_impacts_1.wav", "Meat_impacts_2.wav"],
    ),
    (SoundEvent::DoorsClosed, &["Door_Heavy_Close.mp3"]),
    (SoundEvent::DoorsOpened, &["Door_Heavy_Open.mp3"]),
    (SoundEvent::MiniBossRoomEntered, &["miniboss.mp3"]),
    (SoundEvent::DevilRoomEntered, &["Satan find.mp3"]),
    (SoundEvent::SecretRoomFound, &["secret room find.mp3"]),
    (SoundEvent::SuperSecretRoomFound, &["isaacxpsecretroomreveal2.mp3"]),
    (SoundEvent::BombPlaced, &["Fetus_Land_0.wav", "Fetus_Land_1.wav"]),
    (
        SoundEvent::RockCrumbled,
        &["Rock_crumble 0.wav", "Rock_crumble 1.wav", "Rock_crumble 2.wav"],
    ),
    (
        SoundEvent::KeyBlockBroken,
        &["metal_blockbreak0.wav", "metal_blockbreak1.wav"],
    ),
    // `soundy()` turns a `.mp` ending into `.mp3`.
    (SoundEvent::PennyPickedUp, &["pennypickup.mp3"]),
    (SoundEvent::NickelPickedUp, &["nickelpickup.mp3"]),
    (SoundEvent::DimePickedUp, &["dimepickup.wav"]),
    (SoundEvent::KeyPickedUp, &["KeyPickup_Gauntlet.wav"]),
    (SoundEvent::GoldenKeyPickedUp, &["goldenkey.wav"]),
    (SoundEvent::BombPickedUp, &["fetus_feet0.wav", "fetus_feet1.wav"]),
    (SoundEvent::HeartPickedUp, &["boss2_bubbles0.wav", "boss2_bubbles1.wav"]),
    (SoundEvent::SoulHeartPickedUp, &["Holy.mp3"]),
    (SoundEvent::EternalHeartPickedUp, &["superholy.wav"]),
    (SoundEvent::ItemTaken, &["Choir_unlock.wav"]),
    (SoundEvent::ItemChargeGained, &["beep.wav"]),
    (SoundEvent::ItemRecharged, &["itemrecharge.wav"]),
    (
        SoundEvent::PlayerHurt,
        &[
            "Isaac_Hurt_Grunt0.mp3",
            "Isaac_Hurt_Grunt1.mp3",
            "Isaac_Hurt_Grunt2.mp3",
        ],
    ),
    (SoundEvent::PlayerDied, &["isaacdies.wav"]),
    (SoundEvent::ChestOpened, &["Chest_Open.mp3"]),
    (SoundEvent::FireHiss, &["Firedeath_hiss.wav"]),
    (SoundEvent::HeartLanded, &["Meat_Feet_slow0.wav"]),
    (SoundEvent::PennyLanded, &["pennydrop.mp3"]),
    (SoundEvent::NickelLanded, &["nickeldrop.mp3"]),
    (SoundEvent::DimeLanded, &["dimedrop.wav"]),
    (SoundEvent::KeyLanded, &["Key_drop0.wav"]),
    (SoundEvent::BombLanded, &["Fetus_Land_1.wav"]),
    (SoundEvent::ChestDropped, &["Chest_Drop.mp3"]),
    (SoundEvent::CoinInserted, &["Coin_Slot.mp3"]),
    (SoundEvent::ReelsStopped, &["slot touched.wav"]),
    (SoundEvent::BloodGiven, &["bloodbank touched.wav"]),
    (
        SoundEvent::MachinePaidOut,
        &["slotspawn0.wav", "slotspawn1.wav", "slotspawn2.wav"],
    ),
    (
        SoundEvent::BloodBankPaidOut,
        &["bloodbank spawn0.wav", "bloodbank spawn1.wav"],
    ),
    (SoundEvent::ThumbsUp, &["thumbsup.wav"]),
    (SoundEvent::ThumbsDown, &["thumbs down.wav"]),
    (SoundEvent::BeggarLeft, &["bum shoot off.wav"]),
    (SoundEvent::ShellsShuffled, &["shellgame0.wav", "shellgame1.wav"]),
    (SoundEvent::ShellsLifted, &["shell game.wav"]),
    (SoundEvent::ShellGameLost, &["boss2intro_errorbuzz.wav"]),
    (SoundEvent::CardPickedUp, &["Book Page Turn 12.wav"]),
    (SoundEvent::CardLanded, &["scamper0.wav", "scamper1.wav"]),
    (SoundEvent::PillLanded, &["Fetus_Jump_0.wav", "Fetus_Jump_1.wav"]),
    (SoundEvent::GoodPill, &["Powerup_spewer.wav"]),
    (SoundEvent::Derp, &["derp.mp3"]),
    (SoundEvent::TeleportStarted, &["hell_portal2.wav"]),
    (SoundEvent::TeleportArrived, &["hell_portal1.wav"]),
    (SoundEvent::Summoned, &["summonsound.wav"]),
    (SoundEvent::DevilCard, &["Devil_Card.mp3"]),
    (SoundEvent::DeathCard, &["Death_Card.mp3"]),
    (SoundEvent::ChariotTune, &["isaacunicorn.mp3"]),
    (SoundEvent::HealthGulped, &["Vamp_Gulp.mp3"]),
    (SoundEvent::Fart, &["Fart.mp3"]),
    (SoundEvent::EmpressCard, &["isaacsatanitemget.mp3"]),
    (SoundEvent::FlySwarm, &["insect_swarm_loop.wav"]),
    (
        SoundEvent::ShotSplattered,
        &["splatter00.wav", "splatter01.wav", "splatter02.wav"],
    ),
];
