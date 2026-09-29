use std::collections::VecDeque;

use macroquad::prelude::*;

use crate::config::{INPUT, Time};

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum InputState {
    None,
    MoveLeft,
    MoveRight,
    Rotate,
    Drop,
    HardDrop,
}

struct TouchState {
    start_x: f32,
    start_time: Time,
    /// Direction of the running swipe, repeated while swiping or resting the finger
    direction: Option<InputState>,
    dropping: bool,
}

pub struct InputHandler {
    touch: Option<TouchState>,
    /// Key presses not handled yet, several keys can be pressed within one frame
    pending_keys: VecDeque<InputState>,
    last_move_time: Time,
    key_hold_start: Option<(KeyCode, Time)>,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            touch: None,
            pending_keys: VecDeque::new(),
            last_move_time: Time(0.0),
            key_hold_start: None,
        }
    }

    pub fn update(&mut self) -> InputState {
        let touch_input = self.handle_touch();
        if touch_input != InputState::None {
            return touch_input;
        }

        self.handle_keyboard()
    }

    fn handle_keyboard(&mut self) -> InputState {
        let current_time = Time(get_time());

        // Queue key presses, one of them is handled per frame
        for key in [
            KeyCode::Left,
            KeyCode::Right,
            KeyCode::Down,
            KeyCode::Up,
            KeyCode::A,
            KeyCode::D,
            KeyCode::S,
            KeyCode::W,
            KeyCode::Space,
        ] {
            if is_key_pressed(key) {
                self.key_hold_start = Some((key, current_time));
                self.last_move_time = current_time;
                self.pending_keys.push_back(match key {
                    KeyCode::Left | KeyCode::A => InputState::MoveLeft,
                    KeyCode::Right | KeyCode::D => InputState::MoveRight,
                    KeyCode::Up | KeyCode::W => InputState::Rotate,
                    KeyCode::Down | KeyCode::S => InputState::Drop,
                    _ => InputState::HardDrop,
                });
            }
        }
        if let Some(input) = self.pending_keys.pop_front() {
            return input;
        }

        // Check for held keys
        if let Some((key, start_time)) = self.key_hold_start {
            if !is_key_down(key) {
                self.key_hold_start = None;
                return InputState::None;
            }

            let repeat = current_time - start_time > INPUT.hold_threshold
                && current_time - self.last_move_time > INPUT.move_cooldown;
            match key {
                KeyCode::Down | KeyCode::S => return InputState::Drop,
                KeyCode::Left | KeyCode::A if repeat => {
                    self.last_move_time = current_time;
                    return InputState::MoveLeft;
                }
                KeyCode::Right | KeyCode::D if repeat => {
                    self.last_move_time = current_time;
                    return InputState::MoveRight;
                }
                _ => (),
            }
        }

        InputState::None
    }

    fn handle_touch(&mut self) -> InputState {
        let touches = touches();
        let Some(touch) = touches.first() else {
            return InputState::None;
        };
        let current_time = Time(get_time());
        let x = touch.position.x;

        match touch.phase {
            TouchPhase::Started => {
                self.touch = Some(TouchState {
                    start_x: x,
                    start_time: current_time,
                    direction: None,
                    dropping: false,
                });
            }
            TouchPhase::Moved => {
                let Some(state) = self.touch.as_mut() else {
                    return InputState::None;
                };
                if state.dropping {
                    return InputState::Drop;
                }

                // A short swipe starts moving the piece in that direction
                let dx = x - state.start_x;
                if dx.abs() > INPUT.swipe_threshold
                    && current_time - self.last_move_time > INPUT.move_cooldown_swipe
                {
                    self.last_move_time = current_time;
                    let direction = if dx > 0.0 {
                        InputState::MoveRight
                    } else {
                        InputState::MoveLeft
                    };
                    state.direction = Some(direction);
                    return direction;
                }
            }
            TouchPhase::Stationary => {
                let Some(state) = self.touch.as_mut() else {
                    return InputState::None;
                };

                if let Some(direction) = state.direction {
                    // Resting the finger after a swipe keeps moving the piece
                    if current_time - self.last_move_time > INPUT.move_cooldown_hold {
                        self.last_move_time = current_time;
                        return direction;
                    }
                } else if current_time - state.start_time > INPUT.hold_threshold {
                    state.dropping = true;
                    return InputState::Drop;
                }
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                if let Some(state) = self.touch.take() {
                    let tap = current_time - state.start_time < INPUT.touch_threshold;
                    if tap && state.direction.is_none() {
                        return InputState::Rotate;
                    }
                }
            }
        }
        InputState::None
    }

    pub fn reset(&mut self) {
        self.touch = None;
        self.pending_keys.clear();
        self.key_hold_start = None;
    }
}
