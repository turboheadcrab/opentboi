//! Before the game: the intro plays while loading finishes, then the game starts.

use sdl3::render::WindowCanvas;

use super::Session;
use crate::frontend::sdl3_frontend::intro_player::{IntroPlayer, intro_render_width};
use crate::game::Game;

impl Session<'_> {
    /// Starts the intro (`--intro`) once its art is parsed, sized for the visible window.
    pub(super) fn start_intro_when_ready(&mut self, canvas: &WindowCanvas) -> Result<(), String> {
        let Some((renderer, intro_music)) = self.intro_art.take() else {
            return Ok(());
        };
        let (output_width, output_height) = canvas.output_size().map_err(|error| error.to_string())?;
        let intro_width = intro_render_width(output_width, output_height);
        self.intro = Some(IntroPlayer::start(renderer, intro_width, intro_music));
        Ok(())
    }

    /// Plays the intro; once it is over (or skipped) and loading has finished, the game starts.
    pub(super) fn advance_loading_screen(&mut self, skip_requested: bool) {
        if let Some(intro) = &mut self.intro {
            if skip_requested {
                intro.skip();
            }
            intro.update(self.creator, self.sounds.as_mut());
        }
        let intro_over = self.intro.as_ref().is_none_or(IntroPlayer::is_over);
        if let (true, Some(loaded)) = (intro_over, &self.loaded) {
            let mut game = Game::new(
                loaded.room_appearance,
                &self.parameters,
                &loaded.room_layouts,
                self.seed,
            );
            game.apply_settings(self.game_settings);
            game.set_jingle_lengths(loaded.sound_bank.jingle_lengths());
            self.game = Some(game);
            self.intro = None;
            if let Some(sounds) = &mut self.sounds {
                sounds.fade_out_music();
            }
            println!("Game started after {} ms", self.process_started.elapsed().as_millis());
        }
    }
}
