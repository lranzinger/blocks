use macroquad::{color::Color, rand::gen_range};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tetromino {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Rotation {
    Zero,
    Right,
    Two,
    Left,
}

impl Rotation {
    pub fn clockwise(self) -> Self {
        match self {
            Rotation::Zero => Rotation::Right,
            Rotation::Right => Rotation::Two,
            Rotation::Two => Rotation::Left,
            Rotation::Left => Rotation::Zero,
        }
    }

    pub fn counter_clockwise(self) -> Self {
        match self {
            Rotation::Zero => Rotation::Left,
            Rotation::Left => Rotation::Two,
            Rotation::Two => Rotation::Right,
            Rotation::Right => Rotation::Zero,
        }
    }

    fn index(self) -> usize {
        self as usize
    }
}

type Cells = [(i32, i32); 4];

/// Cells of each piece and rotation inside its bounding box, following the
/// Super Rotation System (SRS). x grows to the right, y downwards.
const SHAPES: [[Cells; 4]; 7] = [
    // I
    [
        [(0, 1), (1, 1), (2, 1), (3, 1)],
        [(2, 0), (2, 1), (2, 2), (2, 3)],
        [(0, 2), (1, 2), (2, 2), (3, 2)],
        [(1, 0), (1, 1), (1, 2), (1, 3)],
    ],
    // O
    [
        [(1, 0), (2, 0), (1, 1), (2, 1)],
        [(1, 0), (2, 0), (1, 1), (2, 1)],
        [(1, 0), (2, 0), (1, 1), (2, 1)],
        [(1, 0), (2, 0), (1, 1), (2, 1)],
    ],
    // T
    [
        [(1, 0), (0, 1), (1, 1), (2, 1)],
        [(1, 0), (1, 1), (2, 1), (1, 2)],
        [(0, 1), (1, 1), (2, 1), (1, 2)],
        [(1, 0), (0, 1), (1, 1), (1, 2)],
    ],
    // S
    [
        [(1, 0), (2, 0), (0, 1), (1, 1)],
        [(1, 0), (1, 1), (2, 1), (2, 2)],
        [(1, 1), (2, 1), (0, 2), (1, 2)],
        [(0, 0), (0, 1), (1, 1), (1, 2)],
    ],
    // Z
    [
        [(0, 0), (1, 0), (1, 1), (2, 1)],
        [(2, 0), (1, 1), (2, 1), (1, 2)],
        [(0, 1), (1, 1), (1, 2), (2, 2)],
        [(1, 0), (0, 1), (1, 1), (0, 2)],
    ],
    // J
    [
        [(0, 0), (0, 1), (1, 1), (2, 1)],
        [(1, 0), (2, 0), (1, 1), (1, 2)],
        [(0, 1), (1, 1), (2, 1), (2, 2)],
        [(1, 0), (1, 1), (0, 2), (1, 2)],
    ],
    // L
    [
        [(2, 0), (0, 1), (1, 1), (2, 1)],
        [(1, 0), (1, 1), (1, 2), (2, 2)],
        [(0, 1), (1, 1), (2, 1), (0, 2)],
        [(0, 0), (1, 0), (1, 1), (1, 2)],
    ],
];

/// SRS wall kick offsets for J, L, S, T and Z, indexed by the start rotation for
/// clockwise rotations. Counter-clockwise uses the negated offsets of the reverse
/// rotation. y points downwards.
const KICKS_JLSTZ: [[(i32, i32); 5]; 4] = [
    [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],  // 0 -> R
    [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],    // R -> 2
    [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],     // 2 -> L
    [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)], // L -> 0
];

const KICKS_I: [[(i32, i32); 5]; 4] = [
    [(0, 0), (-2, 0), (1, 0), (-2, 1), (1, -2)], // 0 -> R
    [(0, 0), (-1, 0), (2, 0), (-1, -2), (2, 1)], // R -> 2
    [(0, 0), (2, 0), (-1, 0), (2, -1), (-1, 2)], // 2 -> L
    [(0, 0), (1, 0), (-2, 0), (1, 2), (-2, -1)], // L -> 0
];

impl Tetromino {
    pub const ALL: [Tetromino; 7] = [
        Tetromino::I,
        Tetromino::O,
        Tetromino::T,
        Tetromino::S,
        Tetromino::Z,
        Tetromino::J,
        Tetromino::L,
    ];

    pub fn cells(self, rotation: Rotation) -> Cells {
        SHAPES[self as usize][rotation.index()]
    }

    /// Kick offsets to try when rotating from `from` to `to`
    pub fn kicks(self, from: Rotation, to: Rotation) -> [(i32, i32); 5] {
        let table = match self {
            Tetromino::O => return [(0, 0); 5],
            Tetromino::I => &KICKS_I,
            _ => &KICKS_JLSTZ,
        };
        if to == from.clockwise() {
            table[from.index()]
        } else {
            // Reverse of the clockwise rotation to -> from
            table[to.index()].map(|(x, y)| (-x, -y))
        }
    }

    pub fn color(self) -> Color {
        match self {
            Tetromino::I => Color::from_rgba(0x3c, 0xcf, 0xf4, 0xff),
            Tetromino::O => Color::from_rgba(0xf7, 0xd0, 0x38, 0xff),
            Tetromino::T => Color::from_rgba(0xae, 0x6b, 0xf0, 0xff),
            Tetromino::S => Color::from_rgba(0x4c, 0xd9, 0x7b, 0xff),
            Tetromino::Z => Color::from_rgba(0xf2, 0x55, 0x5c, 0xff),
            Tetromino::J => Color::from_rgba(0x45, 0x7b, 0xf5, 0xff),
            Tetromino::L => Color::from_rgba(0xf8, 0x94, 0x2e, 0xff),
        }
    }

    pub fn random() -> Self {
        Self::ALL[gen_range(0, Self::ALL.len())]
    }
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
