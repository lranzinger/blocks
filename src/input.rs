use std::collections::VecDeque;

use macroquad::{input::utils, prelude::*};

use crate::{
    config::{INPUT, Time},
    storage,
};

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
    id: u64,
    start_x: f32,
    start_time: Time,
    /// Exact time the finger touched down, if the browser reported it
    start_exact: Option<Time>,
    /// Direction of the running swipe, repeated while swiping or resting the finger
    direction: Option<InputState>,
    dropping: bool,
}

/// Collects all touch events of a frame in their order
#[derive(Default)]
struct TouchEvents(Vec<(miniquad::TouchPhase, u64, f32)>);

impl miniquad::EventHandler for TouchEvents {
    fn update(&mut self) {}
    fn draw(&mut self) {}
    fn touch_event(&mut self, phase: miniquad::TouchPhase, id: u64, x: f32, _y: f32) {
        self.0.push((phase, id, x));
    }
}

pub struct InputHandler {
    touch: Option<TouchState>,
    /// Receives every touch event, `touches()` only has the last state per frame
    event_subscriber: usize,
    /// Touch actions not handled yet, a frame can contain several touch events
    pending_touch: VecDeque<InputState>,
    /// Key presses not handled yet, several keys can be pressed within one frame
    pending_keys: VecDeque<InputState>,
    last_move_time: Time,
    key_hold_start: Option<(KeyCode, Time)>,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            touch: None,
            event_subscriber: utils::register_input_subscriber(),
            pending_touch: VecDeque::new(),
            pending_keys: VecDeque::new(),
            last_move_time: Time(0.0),
            key_hold_start: None,
        }
    }

    pub fn update(&mut self) -> InputState {
        self.handle_touch();
        if let Some(input) = self.pending_touch.pop_front() {
            return input;
        }
        if self.touch.as_ref().is_some_and(|touch| touch.dropping) {
            return InputState::Drop;
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

    /// Handles every touch event in order. With a low frame rate, for example in power
    /// saving mode, starting, moving and lifting a finger can happen within one frame.
    /// Taps are measured with the exact event times, as a frame can take longer than a tap.
    /// Everything else works with the frame time like before.
    fn handle_touch(&mut self) {
        let current_time = Time(get_time());
        let mut events = TouchEvents::default();
        utils::repeat_all_miniquad_input(&mut events, self.event_subscriber);

        let mut moved = false;
        for (phase, id, x) in events.0 {
            let exact = storage::next_touch_time(id).map(Time);
            match phase {
                miniquad::TouchPhase::Started => {
                    // Further fingers are ignored while one is down
                    if self.touch.is_none() {
                        self.touch = Some(TouchState {
                            id,
                            start_x: x,
                            start_time: current_time,
                            start_exact: exact,
                            direction: None,
                            dropping: false,
                        });
                        moved = true;
                    }
                }
                miniquad::TouchPhase::Moved => {
                    if self.touch.as_ref().is_some_and(|touch| touch.id == id) {
                        self.touch_moved(x, current_time);
                        moved = true;
                    }
                }
                miniquad::TouchPhase::Ended | miniquad::TouchPhase::Cancelled => {
                    if self.touch.as_ref().is_some_and(|touch| touch.id == id) {
                        let state = self.touch.take().expect("checked above");
                        let duration = match (exact, state.start_exact) {
                            (Some(end), Some(start)) => end - start,
                            _ => current_time - state.start_time,
                        };
                        let tap = duration < INPUT.touch_threshold;
                        if tap && state.direction.is_none() {
                            self.pending_touch.push_back(InputState::Rotate);
                        }
                    }
                }
            }
        }
        storage::clear_touch_times();

        // A frame without events of the finger means it rests
        if !moved {
            self.touch_resting(current_time);
        }
    }

    fn touch_moved(&mut self, x: f32, current_time: Time) {
        let Some(state) = self.touch.as_mut() else {
            return;
        };
        if state.dropping {
            return;
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
            self.pending_touch.push_back(direction);
        }
    }

    fn touch_resting(&mut self, current_time: Time) {
        let Some(state) = self.touch.as_mut() else {
            return;
        };

        if let Some(direction) = state.direction {
            // Resting the finger after a swipe keeps moving the piece
            if current_time - self.last_move_time > INPUT.move_cooldown_hold {
                self.last_move_time = current_time;
                self.pending_touch.push_back(direction);
            }
        } else if current_time - state.start_time > INPUT.hold_threshold {
            state.dropping = true;
        }
    }

    pub fn reset(&mut self) {
        self.touch = None;
        self.pending_touch.clear();
        self.pending_keys.clear();
        // Events of the current touches, e.g. the tap on the start button, are handled
        utils::repeat_all_miniquad_input(&mut TouchEvents::default(), self.event_subscriber);
        storage::clear_touch_times();
        self.key_hold_start = None;
    }
}
