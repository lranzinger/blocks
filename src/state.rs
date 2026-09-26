use macroquad::color::Color;
use smallvec::SmallVec;

use crate::{
    config::{BOARD, LEVEL_CONFIGS},
    dummy_board::DummyBoard,
    storage,
    tetromino::{RotationState, Tetromino},
};

pub type Board = [[Option<Color>; BOARD.width as usize]; BOARD.height as usize];

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

pub struct BoardState {
    pub cells: Board,
    pub flashing_lines: SmallVec<[u8; 4]>,
}

pub struct ScoreState {
    pub current: u32,
    pub highest: u32,
}

pub struct LevelState {
    pub current: usize,
    pub total_lines_cleared: u32,
}

pub struct GameState {
    pub status: GameStatus,
    pub score: ScoreState,
    pub dummy_board: Option<DummyBoard>,
    pub board: BoardState,
    pub piece: PieceState,
    pub timing: TimingState,
    pub level: LevelState,
}

impl GameState {
    pub fn new() -> Self {
        let initial_piece = Tetromino::random();
        Self {
            status: GameStatus::Start,
            score: ScoreState {
                current: 0,
                highest: storage::get_high_score(),
            },
            dummy_board: Some(DummyBoard::new()),
            board: BoardState {
                cells: [[None; BOARD.width as usize]; BOARD.height as usize],
                flashing_lines: SmallVec::new(),
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
        }
    }
}
