use macroquad::prelude::*;
use rand::gen_range;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tetromino {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

impl Tetromino {
    pub fn shape(&self) -> [(i32, i32); 4] {
        match self {
            Tetromino::I => [(0, 1), (1, 1), (2, 1), (3, 1)],
            Tetromino::O => [(0, 0), (1, 0), (0, 1), (1, 1)],
            Tetromino::T => [(1, 0), (0, 1), (1, 1), (2, 1)],
            Tetromino::S => [(1, 0), (2, 0), (0, 1), (1, 1)],
            Tetromino::Z => [(0, 0), (1, 0), (1, 1), (2, 1)],
            Tetromino::J => [(0, 0), (0, 1), (1, 1), (2, 1)],
            Tetromino::L => [(2, 0), (0, 1), (1, 1), (2, 1)],
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Tetromino::I => BLUE,
            Tetromino::O => YELLOW,
            Tetromino::T => PURPLE,
            Tetromino::S => GREEN,
            Tetromino::Z => RED,
            Tetromino::J => ORANGE,
            Tetromino::L => PINK,
        }
    }

    pub fn random() -> Self {
        Self::ALL[gen_range(0, Self::ALL.len())]
    }

    const ALL: [Tetromino; 7] = [
        Tetromino::I,
        Tetromino::O,
        Tetromino::T,
        Tetromino::S,
        Tetromino::Z,
        Tetromino::J,
        Tetromino::L,
    ];
}

/// 7-bag randomizer: every piece appears exactly once per bag of seven,
/// which prevents long droughts of a single piece.
pub struct Bag {
    pieces: [Tetromino; 7],
    next: usize,
}

impl Bag {
    pub fn new() -> Self {
        Self {
            pieces: Tetromino::ALL,
            next: Tetromino::ALL.len(),
        }
    }

    pub fn next(&mut self) -> Tetromino {
        if self.next >= self.pieces.len() {
            // Fisher-Yates shuffle
            for i in (1..self.pieces.len()).rev() {
                self.pieces.swap(i, gen_range(0, i + 1));
            }
            self.next = 0;
        }
        let piece = self.pieces[self.next];
        self.next += 1;
        piece
    }
}

#[derive(Clone, Copy)]
pub enum RotationState {
    Zero = 0,
    Right = 1,
    Two = 2,
    Left = 3,
}

impl RotationState {
    pub fn next(&self) -> Self {
        match self {
            RotationState::Zero => RotationState::Right,
            RotationState::Right => RotationState::Two,
            RotationState::Two => RotationState::Left,
            RotationState::Left => RotationState::Zero,
        }
    }
}
