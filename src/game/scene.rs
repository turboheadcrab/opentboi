//! What a frame shows and plays: the room, the HUD, the minimap and overlays, the death note,
//! and the looping sounds.

use crate::game::draw::Rgba;
use crate::game::{DrawItem, Game, TextureId, VIEW_CENTER, hud, player, room};
use crate::game::{LoopingSound, SoundEvent};

/// A volume of 100 % plays a sound as recorded.
const PERCENT: f32 = 100.0;
/// Above the HUD, below the view masks: the note is part of the HUD (`hud.paus`).
const DEATH_NOTE_DEPTH: f32 = 890.0;
/// Paper white, without the art.
const DEATH_NOTE_FALLBACK: Rgba = Rgba::opaque(0.92, 0.9, 0.88);

impl Game {
    /// Sounds to keep looping this frame: the flies' buzz, louder the more of them there are
    /// (`flyss`, the root timeline's frame script).
    pub fn looping_sounds(&self) -> impl Iterator<Item = LoopingSound> {
        let flies = self.parameters.flies;
        let count = self.floor.current_room().enemies.buzzing_count();
        let percent =
            (flies.swarm_volume_base + flies.swarm_volume_per_fly * count as f32).min(flies.swarm_volume_maximum);
        (count > 0)
            .then_some(LoopingSound {
                sound: SoundEvent::FlySwarm,
                volume: percent / PERCENT,
            })
            .into_iter()
    }

    /// Appends this frame's draw list to `draw_items` (callers clear and reuse the buffer).
    pub fn fill_draw_items(&self, draw_items: &mut Vec<DrawItem>) {
        self.fill_blended_room_draw_items(draw_items);
        if let Some(frames) = self.frames_dead() {
            let scale = self.player_stats().scale;
            player::death::spotlight::append(self.player.position(), scale, frames, draw_items);
        }
        hud::append_draw_items(
            self.room_appearance,
            self.settings.hud,
            self.last_input_device,
            &self.inventory,
            &self.overlay,
            draw_items,
        );
        let death_note = self.death_note();
        // The minimap is cleared for the note.
        if death_note.is_none() {
            hud::minimap::append_draw_items(&self.floor, self.room_appearance, draw_items);
        }
        self.overlay.append_draw_items(draw_items);
        if self.settings.show_stats {
            let floor = hud::stats::FloorFacts::for_floor(&self.floor, self.seed);
            hud::stats::append_draw_items(
                &self.parameters,
                self.player_stats(),
                floor,
                self.visible_size,
                draw_items,
            );
        }
        if let Some(killer) = death_note {
            draw_items.push(DrawItem::textured(
                TextureId::DeathNote(killer),
                VIEW_CENTER,
                1.0,
                DEATH_NOTE_FALLBACK,
                DEATH_NOTE_DEPTH,
            ));
        }
        self.restart_hold.append_draw_items(draw_items);
        room::append_view_masks(hud::view_opening(self.room_appearance), draw_items);
    }
}
