#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Time(pub f64);

impl std::ops::Sub for Time {
    type Output = Time;

    fn sub(self, other: Time) -> Time {
        Time(self.0 - other.0)
    }
}

pub struct BoardDimensions {
    pub width: i32,
    /// Visible rows
    pub height: i32,
    /// Rows above the visible field where new pieces spawn
    pub hidden: i32,
}

impl BoardDimensions {
    pub const fn total_height(&self) -> i32 {
        self.height + self.hidden
    }
}

pub const BOARD: BoardDimensions = BoardDimensions {
    width: 10,
    height: 20,
    hidden: 2,
};

pub struct GameTiming {
    /// Duration of the line clear animation, the game waits meanwhile
    pub line_clear: f32,
    /// Time a piece may rest on the ground before it locks
    pub lock_delay: f32,
    /// How often moving or rotating a grounded piece restarts the lock delay
    pub max_lock_resets: u32,
    /// Soft drop falls this many times faster than the level speed
    pub soft_drop_factor: f32,
    pub max_soft_drop_interval: f32,
    /// Lines needed for the next level
    pub lines_per_level: u32,
}

pub const TIMING: GameTiming = GameTiming {
    line_clear: 0.35,
    lock_delay: 0.5,
    max_lock_resets: 15,
    soft_drop_factor: 20.0,
    max_soft_drop_interval: 0.05,
    lines_per_level: 10,
};

/// Seconds per row for each level, the last entry is used for all higher levels
pub const FALL_INTERVALS: [f32; 15] = [
    0.48, 0.42, 0.36, 0.30, 0.25, 0.20, 0.16, 0.13, 0.10, 0.08, 0.065, 0.05, 0.04, 0.03, 0.025,
];

pub fn fall_interval(level: u32) -> f32 {
    let index = (level.max(1) as usize - 1).min(FALL_INTERVALS.len() - 1);
    FALL_INTERVALS[index]
}

pub struct ScoreConfig {
    /// Points for 1 to 4 lines, multiplied by the level
    pub lines: [u32; 4],
    pub soft_drop_per_row: u32,
    pub hard_drop_per_row: u32,
    /// Points per combo step, multiplied by the level
    pub combo: u32,
}

pub const SCORE: ScoreConfig = ScoreConfig {
    lines: [100, 300, 500, 800],
    soft_drop_per_row: 1,
    hard_drop_per_row: 2,
    combo: 50,
};

pub struct KeyboardConfig {
    /// Delay before a held move key starts repeating
    pub das: Time,
    /// Interval of the repeated moves
    pub arr: Time,
}

pub const KEYBOARD: KeyboardConfig = KeyboardConfig {
    das: Time(0.15),
    arr: Time(0.05),
};

pub struct TouchConfig {
    /// Horizontal finger travel in logical pixels that starts moving the piece
    pub swipe_threshold: f32,
    /// Delay before a swipe starts repeating its move
    pub repeat_delay: Time,
    /// Interval of the repeated moves while swiping or resting the finger
    pub repeat_interval: Time,
    /// A touch shorter than this without moving is a tap
    pub tap_time: Time,
    /// Resting the finger this long soft drops
    pub hold_time: Time,
    /// A quick vertical flick longer than this drops or holds the piece
    pub flick_distance: f32,
    pub flick_time: Time,
}

pub const TOUCH: TouchConfig = TouchConfig {
    swipe_threshold: 14.0,
    repeat_delay: Time(0.18),
    repeat_interval: Time(0.09),
    tap_time: Time(0.2),
    hold_time: Time(0.25),
    flick_distance: 45.0,
    flick_time: Time(0.3),
};

pub struct UiText {
    pub title: &'static str,
    pub start_touch: &'static str,
    pub start_keys: &'static str,
    pub paused: &'static str,
    pub resume_touch: &'static str,
    pub resume_keys: &'static str,
    pub game_over: &'static str,
    pub new_record: &'static str,
    pub restart_touch: &'static str,
    pub restart_keys: &'static str,
    pub score: &'static str,
    pub record: &'static str,
    pub level: &'static str,
    pub lines: &'static str,
    pub hold: &'static str,
    pub next: &'static str,
    pub touch_help: [&'static str; 5],
    pub key_help: [&'static str; 5],
    pub clears: [&'static str; 4],
    pub back_to_back: &'static str,
    pub combo: &'static str,
    pub level_up: &'static str,
}

pub const TEXT: UiText = UiText {
    title: "BLOCKS",
    start_touch: "Tippen zum Starten",
    start_keys: "Enter zum Starten",
    paused: "Pause",
    resume_touch: "Tippen zum Weiterspielen",
    resume_keys: "Enter zum Weiterspielen",
    game_over: "Game Over",
    new_record: "Neuer Rekord!",
    restart_touch: "Tippen für neues Spiel",
    restart_keys: "Enter für neues Spiel",
    score: "Punkte",
    record: "Rekord",
    level: "Level",
    lines: "Linien",
    hold: "Halten",
    next: "Nächste",
    touch_help: [
        "Wischen: Bewegen",
        "Tippen: Drehen",
        "Halten: Schneller",
        "Nach unten: Fallen",
        "Nach oben: Halten",
    ],
    key_help: [
        "Pfeile: Bewegen",
        "Hoch/X, Z: Drehen",
        "Leertaste: Fallen",
        "C: Halten",
        "P: Pause",
    ],
    clears: ["", "DOUBLE", "TRIPLE", "TETRIS!"],
    back_to_back: "BACK TO BACK",
    combo: "COMBO",
    level_up: "LEVEL",
};
