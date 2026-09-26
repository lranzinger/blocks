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
    pub height: i32,
}

pub const BOARD: BoardDimensions = BoardDimensions {
    width: 10,
    height: 20,
};

pub struct GameTiming {
    pub line_clearing: f32,
    pub flashing_intervall: f64,
}

pub const TIMING: GameTiming = GameTiming {
    line_clearing: 0.5,
    flashing_intervall: 10.0,
};

pub struct InputConfig {
    /// Horizontal finger travel per moved cell, relative to the block size
    pub swipe_step: f32,
    pub hold_threshold: Time,
    pub move_cooldown: Time,
    pub touch_threshold: Time,
}

pub const INPUT: InputConfig = InputConfig {
    swipe_step: 0.8,
    hold_threshold: Time(0.2),
    move_cooldown: Time(0.1),
    touch_threshold: Time(0.15),
};

pub struct UiText {
    pub game_name: &'static str,
    pub start: &'static str,
    pub start_button: &'static str,
    pub gameover: &'static str,
    pub gameover_button: &'static str,
    pub score: &'static str,
    pub level: &'static str,
    pub highscore: &'static str,
    pub instructions: [&'static str; 3],
}

pub const TEXT: UiText = UiText {
    game_name: "Blocks",
    start: "",
    start_button: "Start",
    gameover: "Spiel vorbei",
    gameover_button: "Neu starten",
    score: "Score: ",
    level: "Level: ",
    highscore: "Highscore: ",
    instructions: [
        "Links/Rechts: Bewegen",
        "Tippen: Drehen",
        "Halten: Fallen lassen",
    ],
};

pub struct ScoreConfig {
    pub single: u32,
    pub double: u32,
    pub triple: u32,
    pub tetris: u32,
}

pub const SCORE: ScoreConfig = ScoreConfig {
    single: 100,
    double: 300,
    triple: 500,
    tetris: 800,
};

pub struct LevelConfig {
    pub fall_interval: f32,
    pub lines_required: u32,
    pub score_multiplier: f32,
}

pub const LEVEL_CONFIGS: [LevelConfig; 10] = [
    LevelConfig {
        fall_interval: 0.48,
        lines_required: 10,
        score_multiplier: 1.0,
    }, // Level 1
    LevelConfig {
        fall_interval: 0.40,
        lines_required: 20,
        score_multiplier: 1.5,
    }, // Level 2
    LevelConfig {
        fall_interval: 0.32,
        lines_required: 30,
        score_multiplier: 2.0,
    }, // Level 3
    LevelConfig {
        fall_interval: 0.24,
        lines_required: 40,
        score_multiplier: 2.5,
    }, // Level 4
    LevelConfig {
        fall_interval: 0.16,
        lines_required: 50,
        score_multiplier: 3.0,
    }, // Level 5
    LevelConfig {
        fall_interval: 0.14,
        lines_required: 60,
        score_multiplier: 3.5,
    }, // Level 6
    LevelConfig {
        fall_interval: 0.12,
        lines_required: 70,
        score_multiplier: 4.0,
    }, // Level 7
    LevelConfig {
        fall_interval: 0.11,
        lines_required: 80,
        score_multiplier: 4.5,
    }, // Level 8
    LevelConfig {
        fall_interval: 0.10,
        lines_required: 90,
        score_multiplier: 4.7,
    }, // Level 9
    LevelConfig {
        fall_interval: 0.10,
        lines_required: 100,
        score_multiplier: 5.0,
    }, // Level 10
];
