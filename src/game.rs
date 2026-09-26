use macroquad::prelude::*;

use crate::{
    effects::{Backdrop, Effects},
    input::{Action, InputHandler},
    logic::{Event, GameState, GameStatus},
    renderer::Renderer,
    storage,
};

/// A frame taking longer than this means the tab was hidden or the game stalled
const STALL_TIME: f32 = 0.3;
/// Restarting is possible only after this delay, so a running swipe doesn't restart
const RESTART_DELAY: f64 = 0.8;

pub struct Game {
    state: GameState,
    renderer: Renderer,
    input: InputHandler,
    effects: Effects,
    backdrop: Backdrop,
    game_over_time: f64,
}

impl Game {
    pub fn new() -> Self {
        Self {
            state: GameState::new(storage::get_high_score()),
            renderer: Renderer::new(),
            input: InputHandler::new(),
            effects: Effects::new(),
            backdrop: Backdrop::new(),
            game_over_time: 0.0,
        }
    }

    pub fn frame(&mut self) {
        self.renderer.update_layout();
        let now = get_time();
        let delta = get_frame_time();

        if self.state.status == GameStatus::Playing && delta > STALL_TIME {
            self.state.toggle_pause();
        }
        let delta = delta.min(0.1);

        let input = self.input.update(self.renderer.layout.pause);
        self.state.soft_drop = input.soft_drop;
        for action in input.actions {
            self.handle(action, now);
        }

        self.state.update(delta);
        for event in self.state.take_events() {
            match event {
                Event::Locked { .. } => self.input.reset_touch_gesture(),
                Event::LinesCleared { rows, .. } => {
                    storage::vibrate(if rows.len() == 4 { 40 } else { 15 })
                }
                Event::GameOver { .. } => {
                    storage::vibrate(80);
                    self.game_over_time = now;
                }
                _ => {}
            }
            self.effects.handle(&event, &self.state.board);
        }

        self.effects.update(delta);
        if self.state.status == GameStatus::Start {
            self.backdrop.update(delta);
        }

        let touch = self.input.touch_used || self.renderer.layout.portrait;
        self.renderer
            .draw(&self.state, &self.effects, &self.backdrop, touch, now);
    }

    fn handle(&mut self, action: Action, now: f64) {
        let state = &mut self.state;
        match state.status {
            GameStatus::Start => {
                if matches!(action, Action::Confirm | Action::Tap | Action::HardDrop) {
                    self.effects.clear();
                    state.start();
                }
            }
            GameStatus::Playing => match action {
                Action::MoveLeft => {
                    state.move_horizontal(-1);
                }
                Action::MoveRight => {
                    state.move_horizontal(1);
                }
                Action::RotateClockwise | Action::Tap => {
                    state.rotate(true);
                }
                Action::RotateCounterClockwise => {
                    state.rotate(false);
                }
                Action::HardDrop => state.hard_drop(),
                Action::Hold => state.hold(),
                Action::Pause => state.toggle_pause(),
                Action::Confirm => {}
            },
            GameStatus::Paused => {
                if matches!(action, Action::Pause | Action::Confirm | Action::Tap) {
                    state.toggle_pause();
                }
            }
            GameStatus::GameOver => {
                let ready = now - self.game_over_time > RESTART_DELAY;
                if ready && matches!(action, Action::Confirm | Action::Tap | Action::HardDrop) {
                    self.effects.clear();
                    state.start();
                }
            }
        }
    }
}
