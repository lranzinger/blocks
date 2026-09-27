use crate::{
    config::{BOARD, SCORE, TIMING, fall_interval},
    platform,
    tetromino::{Bag, Rotation, Tetromino},
};

const WIDTH: usize = BOARD.width as usize;
const TOTAL_HEIGHT: usize = BOARD.total_height() as usize;
const PREVIEW_COUNT: usize = 3;

/// All rows including the hidden ones above the visible field
pub type Board = [[Option<Tetromino>; WIDTH]; TOTAL_HEIGHT];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameStatus {
    Start,
    Playing,
    Paused,
    GameOver,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Piece {
    pub kind: Tetromino,
    pub rotation: Rotation,
    /// Position of the bounding box on the board
    pub x: i32,
    pub y: i32,
}

impl Piece {
    fn spawn(kind: Tetromino) -> Self {
        Self {
            kind,
            rotation: Rotation::Zero,
            x: 3,
            y: 0,
        }
    }

    /// Board positions of the four blocks
    pub fn cells(&self) -> [(i32, i32); 4] {
        self.kind
            .cells(self.rotation)
            .map(|(x, y)| (self.x + x, self.y + y))
    }
}

/// Set of board rows stored as bit mask
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Rows(u32);

impl Rows {
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
}

pub struct LineClear {
    pub rows: Rows,
    /// Progress of the animation from 0 to 1
    pub progress: f32,
}

/// Things that happened during an update, used for effects and sounds
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Locked {
        cells: [(i32, i32); 4],
    },
    HardDrop {
        kind: Tetromino,
        cells: [(i32, i32); 4],
        rows: i32,
    },
    LinesCleared {
        rows: Rows,
        points: u32,
        back_to_back: bool,
        combo: u32,
    },
    LevelUp(u32),
    GameOver {
        new_record: bool,
    },
}

pub struct GameState {
    pub status: GameStatus,
    pub board: Board,
    pub piece: Piece,
    pub hold: Option<Tetromino>,
    hold_used: bool,
    pub next: [Tetromino; PREVIEW_COUNT],
    bag: Bag,
    pub score: u32,
    pub high_score: u32,
    pub lines: u32,
    pub level: u32,
    /// Consecutive pieces clearing lines, -1 without a running combo
    combo: i32,
    /// The last line clear was a tetris
    back_to_back: bool,
    pub soft_drop: bool,
    fall_timer: f32,
    lock_timer: f32,
    lock_resets: u32,
    lowest_y: i32,
    pub line_clear: Option<LineClear>,
    pub new_record: bool,
    events: Vec<Event>,
    /// Incremented on every change of the locked blocks
    pub board_version: u32,
}

impl GameState {
    pub fn new(high_score: u32) -> Self {
        let mut bag = Bag::new();
        let next = [bag.next(), bag.next(), bag.next()];
        Self {
            status: GameStatus::Start,
            board: [[None; WIDTH]; TOTAL_HEIGHT],
            // Placeholder until the first piece spawns
            piece: Piece::spawn(Tetromino::O),
            hold: None,
            hold_used: false,
            next,
            bag,
            score: 0,
            high_score,
            lines: 0,
            level: 1,
            combo: -1,
            back_to_back: false,
            soft_drop: false,
            fall_timer: 0.0,
            lock_timer: 0.0,
            lock_resets: 0,
            lowest_y: 0,
            line_clear: None,
            new_record: false,
            events: Vec::new(),
            board_version: 0,
        }
    }

    pub fn start(&mut self) {
        *self = GameState::new(self.high_score);
        self.status = GameStatus::Playing;
        self.spawn_next();
    }

    pub fn toggle_pause(&mut self) {
        self.status = match self.status {
            GameStatus::Playing => GameStatus::Paused,
            GameStatus::Paused => GameStatus::Playing,
            status => status,
        };
    }

    pub fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    fn accepts_input(&self) -> bool {
        self.status == GameStatus::Playing && self.line_clear.is_none()
    }

    /// Advances the game by `delta` seconds
    pub fn update(&mut self, delta: f32) {
        if self.status != GameStatus::Playing {
            return;
        }

        if let Some(clear) = &mut self.line_clear {
            clear.progress += delta / TIMING.line_clear;
            if clear.progress >= 1.0 {
                let rows = clear.rows;
                self.line_clear = None;
                self.remove_rows(rows);
                self.spawn_next();
            }
            return;
        }

        let interval = if self.soft_drop {
            (fall_interval(self.level) / TIMING.soft_drop_factor).min(TIMING.max_soft_drop_interval)
        } else {
            fall_interval(self.level)
        };

        self.fall_timer += delta;
        // Several rows per frame are possible at high speeds
        let mut steps = 0;
        while self.fall_timer >= interval && steps < BOARD.total_height() {
            self.fall_timer -= interval;
            steps += 1;
            if !self.try_move(0, 1) {
                self.fall_timer = 0.0;
                break;
            }
            if self.soft_drop {
                self.score += SCORE.soft_drop_per_row;
            }
        }

        if self.is_grounded() {
            self.lock_timer += delta;
            if self.lock_timer >= TIMING.lock_delay {
                self.lock();
            }
        }
    }

    pub fn move_horizontal(&mut self, dx: i32) -> bool {
        if !self.accepts_input() || !self.try_move(dx, 0) {
            return false;
        }
        self.on_manipulated();
        true
    }

    pub fn rotate(&mut self, clockwise: bool) -> bool {
        if !self.accepts_input() {
            return false;
        }
        let from = self.piece.rotation;
        let to = if clockwise {
            from.clockwise()
        } else {
            from.counter_clockwise()
        };

        for (dx, dy) in self.piece.kind.kicks(from, to) {
            let candidate = Piece {
                rotation: to,
                x: self.piece.x + dx,
                y: self.piece.y + dy,
                ..self.piece
            };
            if self.fits(&candidate) {
                self.piece = candidate;
                self.on_manipulated();
                return true;
            }
        }
        false
    }

    pub fn hard_drop(&mut self) {
        if !self.accepts_input() {
            return;
        }
        let rows = self.drop_distance();
        self.piece.y += rows;
        self.score += rows as u32 * SCORE.hard_drop_per_row;
        self.events.push(Event::HardDrop {
            kind: self.piece.kind,
            cells: self.piece.cells(),
            rows,
        });
        self.lock();
    }

    /// Swaps the current piece with the held one, once per piece
    pub fn hold(&mut self) {
        if !self.accepts_input() || self.hold_used {
            return;
        }
        let current = self.piece.kind;
        match self.hold.replace(current) {
            Some(held) => self.spawn(held),
            None => self.spawn_next(),
        }
        self.hold_used = true;
    }

    /// Rows the current piece can fall until it lands
    pub fn drop_distance(&self) -> i32 {
        let mut rows = 0;
        while self.fits(&Piece {
            y: self.piece.y + rows + 1,
            ..self.piece
        }) {
            rows += 1;
        }
        rows
    }

    fn fits(&self, piece: &Piece) -> bool {
        piece.cells().iter().all(|&(x, y)| {
            (0..BOARD.width).contains(&x)
                && (0..BOARD.total_height()).contains(&y)
                && self.board[y as usize][x as usize].is_none()
        })
    }

    fn try_move(&mut self, dx: i32, dy: i32) -> bool {
        let moved = Piece {
            x: self.piece.x + dx,
            y: self.piece.y + dy,
            ..self.piece
        };
        if !self.fits(&moved) {
            return false;
        }
        self.piece = moved;
        if self.piece.y > self.lowest_y {
            // Reaching a new lowest row gives the full lock delay again
            self.lowest_y = self.piece.y;
            self.lock_resets = 0;
            self.lock_timer = 0.0;
        }
        true
    }

    fn is_grounded(&self) -> bool {
        !self.fits(&Piece {
            y: self.piece.y + 1,
            ..self.piece
        })
    }

    /// Moving or rotating a grounded piece restarts the lock delay a limited number of times
    fn on_manipulated(&mut self) {
        if self.lock_resets < TIMING.max_lock_resets {
            self.lock_timer = 0.0;
            self.lock_resets += 1;
        }
    }

    fn spawn_next(&mut self) {
        let kind = self.next[0];
        self.next.rotate_left(1);
        self.next[PREVIEW_COUNT - 1] = self.bag.next();
        self.spawn(kind);
        self.hold_used = false;
    }

    fn spawn(&mut self, kind: Tetromino) {
        self.piece = Piece::spawn(kind);
        self.fall_timer = 0.0;
        self.lock_timer = 0.0;
        self.lock_resets = 0;
        self.lowest_y = self.piece.y;

        // Block out: no room for the new piece
        if !self.fits(&self.piece) {
            self.end_game();
            return;
        }
        // Enter the visible field right away
        self.try_move(0, 1);
    }

    fn lock(&mut self) {
        let cells = self.piece.cells();
        for &(x, y) in &cells {
            self.board[y as usize][x as usize] = Some(self.piece.kind);
        }
        self.board_version += 1;
        self.events.push(Event::Locked { cells });

        // Lock out: the piece is completely above the visible field
        if cells.iter().all(|&(_, y)| y < BOARD.hidden) {
            self.end_game();
            return;
        }

        let mut rows = Rows::default();
        for (y, row) in self.board.iter().enumerate() {
            if row.iter().all(Option::is_some) {
                rows.insert(y);
            }
        }

        if rows.is_empty() {
            self.combo = -1;
            self.spawn_next();
        } else {
            self.score_lines(rows);
            self.line_clear = Some(LineClear {
                rows,
                progress: 0.0,
            });
        }
    }

    fn score_lines(&mut self, rows: Rows) {
        let count = rows.len();
        let tetris = count == 4;
        let back_to_back = tetris && self.back_to_back;
        self.back_to_back = tetris;
        self.combo += 1;

        let mut points = SCORE.lines[count as usize - 1] * self.level;
        if back_to_back {
            points += points / 2;
        }
        points += SCORE.combo * self.combo as u32 * self.level;
        self.score += points;
        self.events.push(Event::LinesCleared {
            rows,
            points,
            back_to_back,
            combo: self.combo as u32,
        });

        self.lines += count;
        let level = self.lines / TIMING.lines_per_level + 1;
        if level > self.level {
            self.level = level;
            self.events.push(Event::LevelUp(level));
        }
    }

    fn remove_rows(&mut self, rows: Rows) {
        let mut new_board = [[None; WIDTH]; TOTAL_HEIGHT];
        let mut target = TOTAL_HEIGHT;
        for y in (0..TOTAL_HEIGHT).rev() {
            if !rows.contains(y) {
                target -= 1;
                new_board[target] = self.board[y];
            }
        }
        self.board = new_board;
        self.board_version += 1;
    }

    fn end_game(&mut self) {
        self.status = GameStatus::GameOver;
        self.new_record = self.score > self.high_score;
        if self.new_record {
            self.high_score = self.score;
            platform::update_high_score(self.score);
        }
        self.events.push(Event::GameOver {
            new_record: self.new_record,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOTTOM: usize = TOTAL_HEIGHT - 1;

    fn playing() -> GameState {
        let mut state = GameState::new(0);
        state.start();
        state.take_events();
        state
    }

    fn set_piece(state: &mut GameState, kind: Tetromino, rotation: Rotation, x: i32, y: i32) {
        state.piece = Piece {
            kind,
            rotation,
            x,
            y,
        };
        state.lowest_y = y;
    }

    fn fill_row_except(state: &mut GameState, row: usize, gap: std::ops::Range<usize>) {
        for x in 0..WIDTH {
            if !gap.contains(&x) {
                state.board[row][x] = Some(Tetromino::O);
            }
        }
    }

    #[test]
    fn bag_contains_every_piece_once() {
        let mut bag = Bag::new();
        for _ in 0..3 {
            let mut pieces: Vec<_> = (0..7).map(|_| bag.next() as u8).collect();
            pieces.sort();
            assert_eq!(pieces, (0..7).collect::<Vec<u8>>());
        }
    }

    #[test]
    fn new_piece_enters_the_visible_field() {
        let state = playing();
        let top = state.piece.cells().iter().map(|&(_, y)| y).max().unwrap();
        assert!(top >= BOARD.hidden);
    }

    #[test]
    fn preview_advances_on_spawn() {
        let mut state = playing();
        let expected = state.next[0];
        state.hard_drop();
        assert_eq!(state.piece.kind, expected);
    }

    #[test]
    fn hard_drop_scores_rows_and_locks() {
        let mut state = playing();
        set_piece(&mut state, Tetromino::I, Rotation::Zero, 0, 0);
        state.hard_drop();

        assert!((0..4).all(|x| state.board[BOTTOM][x] == Some(Tetromino::I)));
        let rows = BOTTOM as u32 - 1;
        assert_eq!(state.score, rows * SCORE.hard_drop_per_row);
        let events = state.take_events();
        assert!(matches!(events[0], Event::HardDrop { .. }));
        assert!(matches!(events[1], Event::Locked { .. }));
    }

    #[test]
    fn piece_locks_only_after_the_lock_delay() {
        let mut state = playing();
        set_piece(
            &mut state,
            Tetromino::O,
            Rotation::Zero,
            3,
            BOTTOM as i32 - 1,
        );
        state.update(TIMING.lock_delay / 2.0);
        assert!(state.board[BOTTOM].iter().all(Option::is_none));

        // Moving restarts the delay
        assert!(state.move_horizontal(1));
        state.update(TIMING.lock_delay * 0.75);
        assert!(state.board[BOTTOM].iter().all(Option::is_none));

        state.update(TIMING.lock_delay);
        assert!(state.board[BOTTOM].iter().any(Option::is_some));
    }

    #[test]
    fn lock_resets_are_limited() {
        let mut state = playing();
        set_piece(
            &mut state,
            Tetromino::O,
            Rotation::Zero,
            3,
            BOTTOM as i32 - 1,
        );
        for i in 0..40 {
            let dx = if i % 2 == 0 { 1 } else { -1 };
            state.move_horizontal(dx);
            state.update(TIMING.lock_delay * 0.6);
            if state.board[BOTTOM].iter().any(Option::is_some) {
                return;
            }
        }
        panic!("piece never locked");
    }

    #[test]
    fn single_line_clear_scores_and_removes_row_after_animation() {
        let mut state = playing();
        fill_row_except(&mut state, BOTTOM, 0..4);
        state.board[BOTTOM - 1][9] = Some(Tetromino::T);
        set_piece(&mut state, Tetromino::I, Rotation::Zero, 0, 0);
        state.hard_drop();
        let drop_points = state.score - SCORE.lines[0];

        assert!(state.line_clear.is_some());
        assert_eq!(state.score, drop_points + SCORE.lines[0]);

        // Input is ignored during the animation
        let piece = state.piece;
        state.hard_drop();
        assert_eq!(state.piece, piece);

        state.update(TIMING.line_clear * 1.1);
        assert!(state.line_clear.is_none());
        assert_eq!(state.board[BOTTOM][9], Some(Tetromino::T));
        assert!(state.board[BOTTOM - 1].iter().all(Option::is_none));
        assert_eq!(state.lines, 1);
    }

    #[test]
    fn back_to_back_tetris_and_combo_bonus() {
        let mut state = playing();
        let clear_tetris = |state: &mut GameState| {
            for row in BOTTOM - 3..=BOTTOM {
                fill_row_except(state, row, 9..10);
            }
            set_piece(state, Tetromino::I, Rotation::Right, 7, 0);
            state.hard_drop();
            state.update(TIMING.line_clear * 1.1);
        };

        clear_tetris(&mut state);
        let events = state.take_events();
        assert!(events.contains(&Event::LinesCleared {
            rows: {
                let mut rows = Rows::default();
                (BOTTOM - 3..=BOTTOM).for_each(|row| rows.insert(row));
                rows
            },
            points: SCORE.lines[3],
            back_to_back: false,
            combo: 0,
        }));

        clear_tetris(&mut state);
        let points = state
            .take_events()
            .iter()
            .find_map(|event| match event {
                Event::LinesCleared {
                    points,
                    back_to_back: true,
                    combo: 1,
                    ..
                } => Some(*points),
                _ => None,
            })
            .expect("back to back tetris with combo");
        assert_eq!(points, SCORE.lines[3] * 3 / 2 + SCORE.combo);
    }

    #[test]
    fn level_increases_every_ten_lines() {
        let mut state = playing();
        state.lines = TIMING.lines_per_level - 1;
        fill_row_except(&mut state, BOTTOM, 0..4);
        set_piece(&mut state, Tetromino::I, Rotation::Zero, 0, 0);
        state.hard_drop();

        assert_eq!(state.level, 2);
        assert!(state.take_events().contains(&Event::LevelUp(2)));
    }

    #[test]
    fn hold_swaps_once_per_piece() {
        let mut state = playing();
        let first = state.piece.kind;
        let second = state.next[0];
        state.hold();
        assert_eq!(state.hold, Some(first));
        assert_eq!(state.piece.kind, second);

        // Second hold for the same piece is ignored
        state.hold();
        assert_eq!(state.piece.kind, second);

        state.hard_drop();
        state.hold();
        assert_eq!(state.piece.kind, first);
    }

    #[test]
    fn rotation_kicks_off_the_wall() {
        let mut state = playing();
        // Vertical I piece against the right wall
        set_piece(&mut state, Tetromino::I, Rotation::Right, 7, 5);
        assert!(state.rotate(true));
        assert!(
            state
                .piece
                .cells()
                .iter()
                .all(|&(x, _)| (0..BOARD.width).contains(&x))
        );
    }

    #[test]
    fn counter_clockwise_rotation_reverses_clockwise() {
        let mut state = playing();
        set_piece(&mut state, Tetromino::T, Rotation::Zero, 3, 8);
        assert!(state.rotate(true));
        assert!(state.rotate(false));
        assert_eq!(state.piece.rotation, Rotation::Zero);
        assert_eq!((state.piece.x, state.piece.y), (3, 8));
    }

    #[test]
    fn soft_drop_falls_faster_and_scores() {
        let mut state = playing();
        let start_y = state.piece.y;
        state.soft_drop = true;
        for _ in 0..5 {
            state.update(TIMING.max_soft_drop_interval);
        }
        assert!(state.piece.y >= start_y + 9);
        assert!(state.score >= 9 * SCORE.soft_drop_per_row);
    }

    #[test]
    fn blocked_spawn_ends_the_game_and_keeps_record() {
        let mut state = playing();
        state.score = 1234;
        for row in 0..TOTAL_HEIGHT {
            fill_row_except(&mut state, row, 0..1);
        }
        state.hard_drop();

        assert_eq!(state.status, GameStatus::GameOver);
        assert!(state.new_record);
        assert_eq!(state.high_score, state.score);
    }

    #[test]
    fn pause_stops_the_game() {
        let mut state = playing();
        let piece = state.piece;
        state.toggle_pause();
        state.update(10.0);
        assert!(!state.move_horizontal(1));
        assert_eq!(state.piece, piece);
        state.toggle_pause();
        assert_eq!(state.status, GameStatus::Playing);
    }

    /// Runs the game for `seconds` in steps of `delta`
    fn run(state: &mut GameState, seconds: f32, delta: f32) {
        let steps = (seconds / delta).round() as usize;
        for _ in 0..steps {
            state.update(delta);
        }
    }

    /// Frame rates of fast devices, power saving mode and slow devices
    const FRAME_TIMES: [f32; 3] = [1.0 / 60.0, 1.0 / 30.0, 0.1];

    #[test]
    fn fall_speed_does_not_depend_on_the_frame_rate() {
        for soft_drop in [false, true] {
            let rows: Vec<i32> = FRAME_TIMES
                .iter()
                .map(|&delta| {
                    let mut state = playing();
                    set_piece(&mut state, Tetromino::T, Rotation::Zero, 3, 2);
                    state.soft_drop = soft_drop;
                    let seconds = if soft_drop { 0.3 } else { 3.0 };
                    run(&mut state, seconds, delta);
                    state.piece.y
                })
                .collect();
            let spread = rows.iter().max().unwrap() - rows.iter().min().unwrap();
            assert!(spread <= 1, "soft drop {soft_drop}: rows {rows:?}");
        }
    }

    #[test]
    fn lock_delay_does_not_depend_on_the_frame_rate() {
        for delta in FRAME_TIMES {
            let mut state = playing();
            set_piece(
                &mut state,
                Tetromino::O,
                Rotation::Zero,
                3,
                BOTTOM as i32 - 1,
            );
            run(&mut state, TIMING.lock_delay * 0.8, delta);
            assert!(
                state.board[BOTTOM].iter().all(Option::is_none),
                "locked too early at {delta}"
            );
            run(&mut state, TIMING.lock_delay * 0.4, delta);
            assert!(
                state.board[BOTTOM].iter().any(Option::is_some),
                "not locked at {delta}"
            );
        }
    }

    #[test]
    fn line_clear_animation_does_not_depend_on_the_frame_rate() {
        for delta in FRAME_TIMES {
            let mut state = playing();
            fill_row_except(&mut state, BOTTOM, 0..4);
            set_piece(&mut state, Tetromino::I, Rotation::Zero, 0, 0);
            state.hard_drop();
            run(&mut state, TIMING.line_clear * 0.7, delta);
            assert!(state.line_clear.is_some(), "finished too early at {delta}");
            run(&mut state, TIMING.line_clear * 0.5, delta);
            assert!(state.line_clear.is_none(), "not finished at {delta}");
        }
    }
}
