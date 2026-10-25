//! Tears in flight: they slow over spider webs and splash against walls, obstacles or the floor.

use crate::game::Game;
use crate::game::attacks::splash::{Splash, SplashKind};
use crate::game::attacks::tear::TearStatus;
use crate::game::original_units::Frames;
use crate::game::room::grid_object::{GridObject, HitStrength};
use crate::game::sound_event::SoundEvent;

impl Game {
    pub(crate) fn advance_tears(&mut self, elapsed: Frames) {
        let tear_blockers = self.floor.current_room().tear_blockers();
        let blocker_bounds: Vec<_> = tear_blockers.iter().map(|(_, tile)| *tile).collect();
        let spider_webs = self.floor.current_room().bounds_where(GridObject::is_spider_web);
        let mut still_flying = Vec::with_capacity(self.tears.len());
        for mut tear in std::mem::take(&mut self.tears) {
            tear.advance(elapsed);
            tear.slow_down_if_over(&spider_webs);
            let hit_bomb = !tear.is_new() && self.bombs.hit_by_tear(tear.floor_point(), tear.velocity(), elapsed);
            let hit_enemy = !tear.is_new()
                && !hit_bomb
                && self.floor.current_room_mut().enemies.hit_by_tear(
                    tear.floor_point(),
                    tear.velocity(),
                    tear.damage(),
                    elapsed,
                    &mut self.random,
                    &mut self.pending_sound_events,
                );
            let status = if hit_bomb || hit_enemy {
                TearStatus::HitWall
            } else {
                tear.status(&blocker_bounds)
            };
            let splash_kind = match status {
                TearStatus::Flying => {
                    still_flying.push(tear);
                    continue;
                }
                TearStatus::Landed => SplashKind::Ground,
                TearStatus::HitWall => SplashKind::Hit,
                TearStatus::HitSolidTile(blocker_index) => {
                    let object_index = tear_blockers[blocker_index].0;
                    self.floor.current_room_mut().hit(
                        object_index,
                        HitStrength::Tear,
                        &mut self.random,
                        &mut self.pending_sound_events,
                    );
                    SplashKind::Hit
                }
            };
            let splash = Splash::new(splash_kind, tear.drawn_center(), tear.scale(), &mut self.random);
            self.splashes.push(splash);
            self.pending_sound_events.push(SoundEvent::TearImpact);
        }
        self.tears = still_flying;
    }
}
