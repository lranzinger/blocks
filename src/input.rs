use macroquad::{input::utils, prelude::*};

use crate::{
    config::{KEYBOARD, TOUCH, Time},
    storage,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    MoveLeft,
    MoveRight,
    RotateClockwise,
    RotateCounterClockwise,
    HardDrop,
    Hold,
    Pause,
    /// Enter or a mouse click outside of the buttons
    Confirm,
    /// Short touch outside of the buttons
    Tap,
}

#[derive(Default)]
pub struct FrameInput {
    pub actions: Vec<Action>,
    pub soft_drop: bool,
}

struct HeldMove {
    direction: i32,
    key: KeyCode,
    pressed: Time,
    last_move: Time,
}

struct TouchState {
    id: u64,
    start: Vec2,
    /// Latest position, used to update the gesture in frames without touch events
    position: Vec2,
    start_time: Time,
    /// Touch started on the pause button
    on_button: bool,
    /// Direction of the running swipe
    direction: Option<i32>,
    direction_time: Time,
    last_move: Time,
    /// A flick was recognized, the rest of the touch is ignored
    flicked: bool,
    dropping: bool,
}

/// Collects all touch events of a frame in their order
#[derive(Default)]
struct TouchEvents(Vec<(miniquad::TouchPhase, u64, Vec2)>);

impl miniquad::EventHandler for TouchEvents {
    fn update(&mut self) {}
    fn draw(&mut self) {}
    fn touch_event(&mut self, phase: miniquad::TouchPhase, id: u64, x: f32, y: f32) {
        self.0.push((phase, id, vec2(x, y)));
    }
}

pub struct InputHandler {
    held_move: Option<HeldMove>,
    touch: Option<TouchState>,
    /// Receives every touch event, `touches()` only has the last state per frame
    event_subscriber: usize,
    /// Touch is the current input method, used to show the matching help texts.
    /// Starts with the device type and follows the input used last.
    pub touch_mode: bool,
}

const LEFT_KEYS: [KeyCode; 2] = [KeyCode::Left, KeyCode::A];
const RIGHT_KEYS: [KeyCode; 2] = [KeyCode::Right, KeyCode::D];

impl InputHandler {
    pub fn new() -> Self {
        // Touches are handled here, simulated mouse clicks would count twice
        simulate_mouse_with_touch(false);
        Self {
            held_move: None,
            touch: None,
            event_subscriber: utils::register_input_subscriber(),
            touch_mode: storage::is_touch_device(),
        }
    }

    /// Collects the input of this frame. `pause_button` is in logical pixels.
    pub fn update(&mut self, pause_button: Rect) -> FrameInput {
        let now = Time(get_time());
        let mut input = FrameInput::default();
        self.keyboard(now, &mut input);
        self.mouse(pause_button, &mut input);
        self.touch(now, pause_button, &mut input);
        input
    }

    /// Stops a running move or drop, for example when a new piece spawns
    pub fn reset_touch_gesture(&mut self) {
        if let Some(touch) = &mut self.touch {
            touch.direction = None;
            touch.dropping = false;
            touch.flicked = true;
        }
    }

    fn keyboard(&mut self, now: Time, input: &mut FrameInput) {
        if get_last_key_pressed().is_some() {
            self.touch_mode = false;
        }
        let pressed = |keys: &[KeyCode]| keys.iter().find(|key| is_key_pressed(**key)).copied();

        if let Some(key) = pressed(&LEFT_KEYS) {
            self.start_move(-1, key, now, input);
        }
        if let Some(key) = pressed(&RIGHT_KEYS) {
            self.start_move(1, key, now, input);
        }
        self.repeat_move(now, input);

        let actions = [
            (
                &[KeyCode::Up, KeyCode::W, KeyCode::X][..],
                Action::RotateClockwise,
            ),
            (
                &[
                    KeyCode::Z,
                    KeyCode::Y,
                    KeyCode::LeftControl,
                    KeyCode::RightControl,
                ][..],
                Action::RotateCounterClockwise,
            ),
            (&[KeyCode::Space][..], Action::HardDrop),
            (
                &[KeyCode::C, KeyCode::LeftShift, KeyCode::RightShift][..],
                Action::Hold,
            ),
            (&[KeyCode::P, KeyCode::Escape][..], Action::Pause),
            (&[KeyCode::Enter, KeyCode::KpEnter][..], Action::Confirm),
        ];
        for (keys, action) in actions {
            if pressed(keys).is_some() {
                input.actions.push(action);
            }
        }

        input.soft_drop |= is_key_down(KeyCode::Down) || is_key_down(KeyCode::S);
    }

    fn start_move(&mut self, direction: i32, key: KeyCode, now: Time, input: &mut FrameInput) {
        input.actions.push(move_action(direction));
        self.held_move = Some(HeldMove {
            direction,
            key,
            pressed: now,
            last_move: now,
        });
    }

    /// Auto repeat: after a delay (DAS) a held key moves the piece in short intervals (ARR)
    fn repeat_move(&mut self, now: Time, input: &mut FrameInput) {
        let Some(held) = &mut self.held_move else {
            return;
        };

        if !is_key_down(held.key) {
            // Continue with the other direction if its key is still held
            let other = if held.direction < 0 {
                (1, RIGHT_KEYS)
            } else {
                (-1, LEFT_KEYS)
            };
            self.held_move = other
                .1
                .iter()
                .find(|key| is_key_down(**key))
                .map(|key| HeldMove {
                    direction: other.0,
                    key: *key,
                    pressed: now,
                    last_move: now,
                });
            return;
        }

        if now - held.pressed > KEYBOARD.das && now - held.last_move >= KEYBOARD.arr {
            held.last_move = now;
            input.actions.push(move_action(held.direction));
        }
    }

    fn mouse(&mut self, pause_button: Rect, input: &mut FrameInput) {
        if is_mouse_button_pressed(MouseButton::Left) {
            let position = Vec2::from(mouse_position());
            input.actions.push(if pause_button.contains(position) {
                Action::Pause
            } else {
                Action::Confirm
            });
        }
    }

    /// Handles every touch event in order. With a low frame rate, for example in power
    /// saving mode, starting, moving and lifting a finger can happen within one frame.
    fn touch(&mut self, now: Time, pause_button: Rect, input: &mut FrameInput) {
        let mut events = TouchEvents::default();
        utils::repeat_all_miniquad_input(&mut events, self.event_subscriber);
        if !events.0.is_empty() {
            self.touch_mode = true;
        }

        let mut updated = false;
        for (phase, id, position) in events.0 {
            // Touch positions are in physical pixels, the layout uses logical ones
            let position = position / screen_dpi_scale();
            match phase {
                miniquad::TouchPhase::Started => {
                    // Further fingers are ignored while one is down
                    if self.touch.is_none() {
                        self.touch = Some(TouchState {
                            id,
                            start: position,
                            position,
                            start_time: now,
                            on_button: pause_button.contains(position),
                            direction: None,
                            direction_time: now,
                            last_move: now,
                            flicked: false,
                            dropping: false,
                        });
                    }
                }
                miniquad::TouchPhase::Moved => {
                    if let Some(state) = self.touch.as_mut().filter(|state| state.id == id) {
                        state.position = position;
                        state.gesture(now, input);
                        updated = true;
                    }
                }
                miniquad::TouchPhase::Ended | miniquad::TouchPhase::Cancelled => {
                    if self.touch.as_ref().is_some_and(|state| state.id == id) {
                        let state = self.touch.take().expect("checked above");
                        state.end(position, now, pause_button, input);
                    }
                }
            }
        }

        // Time based parts of the gesture, like resting the finger, also run without events
        if !updated && let Some(state) = &mut self.touch {
            state.gesture(now, input);
        }
    }
}

impl TouchState {
    fn end(self, position: Vec2, now: Time, pause_button: Rect, input: &mut FrameInput) {
        if self.on_button {
            if pause_button.contains(position) {
                input.actions.push(Action::Pause);
            }
            return;
        }
        let distance = (position - self.start).length();
        let tap = now - self.start_time < TOUCH.tap_time
            && distance < TOUCH.swipe_threshold
            && self.direction.is_none()
            && !self.flicked
            && !self.dropping;
        if tap {
            input.actions.push(Action::Tap);
        }
    }

    fn gesture(&mut self, now: Time, input: &mut FrameInput) {
        if self.on_button || self.flicked {
            return;
        }
        let delta = self.position - self.start;
        let elapsed = now - self.start_time;

        // Quick vertical flick: down drops the piece, up holds it
        let vertical = delta.y.abs() > 2.0 * delta.x.abs();
        if self.direction.is_none()
            && !self.dropping
            && vertical
            && elapsed < TOUCH.flick_time
            && delta.y.abs() > TOUCH.flick_distance
        {
            self.flicked = true;
            input.actions.push(if delta.y > 0.0 {
                Action::HardDrop
            } else {
                Action::Hold
            });
            return;
        }

        // Horizontal swipe: move once, then keep moving while swiping or resting the finger
        let horizontal = delta.x.abs() > TOUCH.swipe_threshold && delta.x.abs() > delta.y.abs();
        if horizontal || (self.direction.is_some() && delta.x.abs() > TOUCH.swipe_threshold) {
            let direction = delta.x.signum() as i32;
            if self.direction != Some(direction) {
                self.direction = Some(direction);
                self.direction_time = now;
                self.last_move = now;
                input.actions.push(move_action(direction));
            } else if now - self.direction_time > TOUCH.repeat_delay
                && now - self.last_move > TOUCH.repeat_interval
            {
                self.last_move = now;
                input.actions.push(move_action(direction));
            }
            return;
        }
        if self.direction.is_some() {
            // Finger went back to the start: stop moving
            self.direction = None;
            return;
        }

        // Resting the finger or slowly dragging down drops faster
        let still = delta.length() < TOUCH.swipe_threshold && elapsed > TOUCH.hold_time;
        let dragging_down =
            vertical && delta.y > TOUCH.swipe_threshold && elapsed > TOUCH.flick_time;
        if still || dragging_down {
            self.dropping = true;
        }
        input.soft_drop |= self.dropping;
    }
}

fn move_action(direction: i32) -> Action {
    if direction < 0 {
        Action::MoveLeft
    } else {
        Action::MoveRight
    }
}
