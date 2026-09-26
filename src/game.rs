use crate::{
    input::InputHandler,
    renderer::Renderer,
    state::{GameState, GameStatus},
};
use macroquad::prelude::*;

pub struct Game {
    pub state: GameState,
    pub renderer: Renderer,
    pub input: InputHandler,
}

impl Game {
    pub fn new() -> Self {
        Self {
            state: GameState::new(),
            renderer: Renderer::new(),
            input: InputHandler::new(),
        }
    }

    pub fn update(&mut self) {
        match self.state.status {
            GameStatus::Start => {
                if self.renderer.check_click(GameStatus::Start) {
                    self.state.start();
                    // Prevent the button tap from rotating the first piece
                    self.input.reset();
                }
            }
            GameStatus::Playing => {
                let input = self.input.update(self.renderer.screen.block_size);
                self.state.handle_input(input);
                self.state.update(get_frame_time());
            }
            GameStatus::GameOver => {
                if self.renderer.check_click(GameStatus::GameOver) {
                    self.state.restart();
                    self.input.reset();
                }
            }
        }

        let events = self.state.take_events();
        if events.piece_locked {
            // Don't carry a held drop or move over to the next piece
            self.input.reset();
        }
        if events.board_changed {
            self.renderer.mark_board_dirty();
        }
    }
}
