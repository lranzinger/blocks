use macroquad::color::Color;

use crate::{
    config::{BOARD, LEVEL_CONFIGS},
    dummy_board::DummyBoard,
    storage,
    tetromino::{Bag, RotationState, Tetromino},
};

pub type Board = [[Option<Color>; BOARD.width as usize]; BOARD.height as usize];

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GameStatus {
    Start,
    Playing,
    GameOver,
}

pub struct PieceState {
    pub typ: Tetromino,
    pub position: (i32, i32),
    pub rotated: [(i32, i32); 4],
    pub rotation: RotationState,
}

pub struct TimingState {
    pub fall_interval: f32,
    pub fall_timer: f32,
    pub line_clear_timer: f32,
}

/// Set of board rows stored as bit mask, one bit per row
#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct FlashingLines(u32);

impl FlashingLines {
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub fn len(&self) -> u32 {
        self.0.count_ones()
    }

    pub fn contains(&self, row: usize) -> bool {
        self.0 & (1 << row) != 0
    }

    pub fn insert(&mut self, row: usize) {
        self.0 |= 1 << row;
    }

    pub fn clear(&mut self) {
        self.0 = 0;
    }
}

pub struct BoardState {
    pub cells: Board,
    pub flashing_lines: FlashingLines,
}

pub struct ScoreState {
    pub current: u32,
    pub highest: u32,
}

pub struct LevelState {
    pub current: usize,
    pub total_lines_cleared: u32,
}

/// Things that happened during a logic update which other parts of the game react to
#[derive(Default)]
pub struct Events {
    pub board_changed: bool,
    pub piece_locked: bool,
}

pub struct GameState {
    pub status: GameStatus,
    pub score: ScoreState,
    pub dummy_board: Option<DummyBoard>,
    pub board: BoardState,
    pub piece: PieceState,
    pub timing: TimingState,
    pub level: LevelState,
    pub bag: Bag,
    pub events: Events,
}

impl GameState {
    pub fn new() -> Self {
        // Placeholder until the game starts and the first piece is spawned
        let initial_piece = Tetromino::O;
        Self {
            status: GameStatus::Start,
            score: ScoreState {
                current: 0,
                highest: storage::get_high_score(),
            },
            dummy_board: Some(DummyBoard::new()),
            board: BoardState {
                cells: [[None; BOARD.width as usize]; BOARD.height as usize],
                flashing_lines: FlashingLines::default(),
            },
            piece: PieceState {
                typ: initial_piece,
                rotated: initial_piece.shape(),
                position: (BOARD.width / 2 - 2, -1),
                rotation: RotationState::Zero,
            },
            timing: TimingState {
                fall_timer: 0.0,
                fall_interval: LEVEL_CONFIGS[0].fall_interval,
                line_clear_timer: 0.0,
            },
            level: LevelState {
                current: 0,
                total_lines_cleared: 0,
            },
            bag: Bag::new(),
            events: Events::default(),
        }
    }
}
