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
    /// Finger x position at which the last move was triggered
    anchor_x: f32,
    last_x: f32,
    last_finger_move: Time,
    start_time: Time,
    /// Direction of the last swipe, repeated while the finger rests afterwards
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

    /// `block_size` is used to move the piece one cell per block of finger travel
    pub fn update(&mut self, block_size: f32) -> InputState {
        let touch_input = self.handle_touch(block_size);
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

    fn handle_touch(&mut self, block_size: f32) -> InputState {
        let touches = touches();
        let Some(touch) = touches.first() else {
            return InputState::None;
        };
        let current_time = Time(get_time());
        // Touch positions are in physical pixels, the game works in logical ones
        let x = touch.position.x / screen_dpi_scale();

        match touch.phase {
            TouchPhase::Started => {
                self.touch = Some(TouchState {
                    start_x: x,
                    anchor_x: x,
                    last_x: x,
                    last_finger_move: current_time,
                    start_time: current_time,
                    direction: None,
                    dropping: false,
                });
            }
            TouchPhase::Moved | TouchPhase::Stationary => {
                let Some(state) = self.touch.as_mut() else {
                    return InputState::None;
                };
                if state.dropping {
                    return InputState::Drop;
                }

                if x != state.last_x {
                    state.last_x = x;
                    state.last_finger_move = current_time;
                }

                // The piece follows the finger: one cell per step of finger travel
                let step = (block_size * INPUT.swipe_step).max(1.0);
                let dx = x - state.anchor_x;
                if dx.abs() >= step {
                    state.anchor_x += step * dx.signum();
                    let direction = if dx > 0.0 {
                        InputState::MoveRight
                    } else {
                        InputState::MoveLeft
                    };
                    state.direction = Some(direction);
                    self.last_move_time = current_time;
                    return direction;
                }

                // Resting the finger after a swipe keeps moving the piece in that direction
                if let Some(direction) = state.direction {
                    let resting = current_time - state.last_finger_move > INPUT.move_cooldown_hold;
                    if resting && current_time - self.last_move_time > INPUT.move_cooldown_hold {
                        self.last_move_time = current_time;
                        return direction;
                    }
                    return InputState::None;
                }

                // Holding the finger still drops the piece, a slow swipe must not
                let still = (x - state.start_x).abs() < step * 0.3;
                if still && current_time - state.start_time > INPUT.hold_threshold {
                    state.dropping = true;
                    return InputState::Drop;
                }
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                if let Some(state) = self.touch.take() {
                    let tap = current_time - state.start_time < INPUT.touch_threshold;
                    if tap && state.direction.is_none() && !state.dropping {
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
