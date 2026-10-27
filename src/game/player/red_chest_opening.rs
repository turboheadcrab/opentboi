//! Opening a red chest: what `chestopen`'s `c2` branch does with its contents.

use glam::Vec2;

use crate::game::Game;
use crate::game::floor::DEVIL_CELL;
use crate::game::pickups::kind::{BombKind, PickupKind};
use crate::game::pickups::pill::Pill;
use crate::game::pickups::red_chest::{MEGA_TROLL_BOMB_RADIUS, RedChestContents, red_chest_contents};
use crate::game::room::contents::SpawOptions;

impl Game {
    pub(crate) fn open_red_chest(&mut self, center: Vec2) {
        let room = self.floor.current_room_mut();
        match red_chest_contents(&mut self.random) {
            RedChestContents::Item => room.pickups.place(PickupKind::ItemPedestal, center),
            RedChestContents::DevilRoom => self.teleport(DEVIL_CELL),
            RedChestContents::MegaTrollBomb => {
                let spot = room.spaw(center, MEGA_TROLL_BOMB_RADIUS, SpawOptions::default(), &mut self.random);
                room.pickups
                    .spawn(PickupKind::Bomb(BombKind::MegaTroll), spot, Vec2::ZERO);
            }
            RedChestContents::Thrown {
                kind: Some(kind),
                count,
            } => {
                for _ in 0..count {
                    room.spawn_pickup(kind, center, true, &mut self.random);
                }
            }
            RedChestContents::Thrown { kind: None, count } => {
                for _ in 0..count {
                    let pill = PickupKind::Pill(Pill::roll(&mut self.random));
                    self.floor
                        .current_room_mut()
                        .spawn_pickup(pill, center, true, &mut self.random);
                }
            }
            RedChestContents::BlueFlies => {
                let flies = self.parameters.blue_flies;
                self.blue_flies
                    .owe(flies.chest_base + u32::from(self.random.below_u8(flies.chest_range)));
            }
            RedChestContents::Spiders => {}
        }
    }
}
