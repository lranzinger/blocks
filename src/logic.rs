use crate::{
    config::{BOARD, LEVEL_CONFIGS, SCORE, TIMING},
    input::InputState,
    state::{GameState, GameStatus},
    storage,
    tetromino::{RotationState, Tetromino},
};

/// Fall interval while the drop input is held
const SOFT_DROP_INTERVAL: f32 = 0.05;

/// Horizontal offsets tried when a rotation collides (simple wall kick)
const KICK_OFFSETS: [i32; 5] = [0, -1, 1, -2, 2];

fn rotated_shape(typ: Tetromino, rotation: RotationState) -> [(i32, i32); 4] {
    let shape = typ.shape();
    let pivot = (1, 1);

    let moved_center = shape.map(|(x, y)| (x - pivot.0, y - pivot.1));

    match rotation {
        RotationState::Zero => shape,
        RotationState::Right => moved_center.map(|(x, y)| (-y + pivot.0, x + pivot.1)),
        RotationState::Two => moved_center.map(|(x, y)| (-x + pivot.0, -y + pivot.1)),
        RotationState::Left => moved_center.map(|(x, y)| (y + pivot.0, -x + pivot.1)),
    }
}

impl GameState {
    pub fn start(&mut self) {
        self.dummy_board = None;
        self.status = GameStatus::Playing;
        self.spawn_piece();
        self.events.board_changed = true;
    }

    pub fn restart(&mut self) {
        let high_score = self.score.highest;
        *self = GameState::new();
        self.score.highest = high_score;
        self.start();
    }

    /// Advances the game by `delta` seconds
    pub fn update(&mut self, delta: f32) {
        if self.status != GameStatus::Playing {
            return;
        }

        // Handle line clear animation, the game is paused meanwhile
        if !self.board.flashing_lines.is_empty() {
            self.timing.line_clear_timer -= delta;
            if self.timing.line_clear_timer <= 0.0 {
                self.remove_flashing_lines();
                self.spawn_piece();
            }
            return;
        }

        // Handle automatic piece falling
        self.timing.fall_timer += delta;
        if self.timing.fall_timer >= self.timing.fall_interval {
            // Keep the remainder for a steady speed, but never catch up more than one step
            self.timing.fall_timer =
                (self.timing.fall_timer - self.timing.fall_interval).min(self.timing.fall_interval);
            if self.can_move(0, 1) {
                self.piece.position.1 += 1;
            } else {
                self.settle_piece();
            }
        }
    }

    pub fn handle_input(&mut self, input: InputState) {
        if self.status != GameStatus::Playing || !self.board.flashing_lines.is_empty() {
            return;
        }

        self.timing.fall_interval = LEVEL_CONFIGS[self.level.current].fall_interval;

        match input {
            InputState::MoveLeft => {
                if self.can_move(-1, 0) {
                    self.piece.position.0 -= 1;
                }
            }
            InputState::MoveRight => {
                if self.can_move(1, 0) {
                    self.piece.position.0 += 1;
                }
            }
            InputState::Rotate => self.try_rotation(),
            InputState::Drop => self.timing.fall_interval = SOFT_DROP_INTERVAL,
            InputState::HardDrop => self.hard_drop(),
            InputState::None => (),
        }
    }

    fn spawn_piece(&mut self) {
        self.piece.typ = self.bag.next();
        self.piece.rotation = RotationState::Zero;
        self.piece.rotated = self.piece.typ.shape();

        let shape = self.piece.rotated;
        let piece_width = shape.iter().map(|(x, _)| x).max().unwrap()
            - shape.iter().map(|(x, _)| x).min().unwrap()
            + 1;
        self.piece.position = (BOARD.width / 2 - piece_width / 2, -1);
        self.timing.fall_timer = 0.0;

        // Block out: the new piece overlaps with existing blocks
        if !self.can_move(0, 0) {
            self.end_game();
        }
    }

    fn can_move(&self, dx: i32, dy: i32) -> bool {
        self.piece.rotated.iter().all(|&(x, y)| {
            let new_x = self.piece.position.0 + x + dx;
            let new_y = self.piece.position.1 + y + dy;
            (0..BOARD.width).contains(&new_x)
                && new_y < BOARD.height
                && (new_y < 0 || self.board.cells[new_y as usize][new_x as usize].is_none())
        })
    }

    fn hard_drop(&mut self) {
        while self.can_move(0, 1) {
            self.piece.position.1 += 1;
        }
        self.settle_piece();
    }

    /// Writes the current piece into the board.
    /// Returns `true` if part of the piece is above the visible field (lock out).
    fn lock_piece(&mut self) -> bool {
        let mut locked_out = false;
        for &(x, y) in &self.piece.rotated {
            let board_x = self.piece.position.0 + x;
            let board_y = self.piece.position.1 + y;
            if board_y >= 0 {
                self.board.cells[board_y as usize][board_x as usize] = Some(self.piece.typ.color());
            } else {
                locked_out = true;
            }
        }
        self.events.piece_locked = true;
        self.events.board_changed = true;
        locked_out
    }

    fn settle_piece(&mut self) {
        if self.lock_piece() {
            self.end_game();
            return;
        }

        self.clear_lines();
        // With cleared lines the next piece spawns after the flash animation
        if self.board.flashing_lines.is_empty() {
            self.spawn_piece();
        }
    }

    fn end_game(&mut self) {
        let last_highscore = storage::get_high_score();
        if self.score.highest > last_highscore {
            storage::update_high_score(self.score.highest);
        }
        self.status = GameStatus::GameOver;
    }

    fn clear_lines(&mut self) {
        for (y, row) in self.board.cells.iter().enumerate() {
            if row.iter().all(|cell| cell.is_some()) {
                self.board.flashing_lines.insert(y);
            }
        }

        let lines_cleared = self.board.flashing_lines.len();
        if lines_cleared == 0 {
            return;
        }

        // Start line clear animation
        self.timing.line_clear_timer = TIMING.line_clearing;

        self.score.current += self.calculate_score(lines_cleared);
        self.score.highest = self.score.highest.max(self.score.current);
        self.level.total_lines_cleared += lines_cleared;
        self.update_level();
    }

    fn remove_flashing_lines(&mut self) {
        let mut new_board = [[None; BOARD.width as usize]; BOARD.height as usize];
        let mut new_row = BOARD.height as usize;

        // Copy the board from the bottom up, skipping the cleared lines
        for y in (0..BOARD.height as usize).rev() {
            if !self.board.flashing_lines.contains(y) {
                new_row -= 1;
                new_board[new_row] = self.board.cells[y];
            }
        }

        self.board.cells = new_board;
        self.board.flashing_lines.clear();
        self.events.board_changed = true;
    }

    fn try_rotation(&mut self) {
        if self.piece.typ == Tetromino::O {
            return;
        }

        let original_x = self.piece.position.0;
        let original_rotation = self.piece.rotation;

        self.piece.rotation = self.piece.rotation.next();
        self.piece.rotated = rotated_shape(self.piece.typ, self.piece.rotation);

        for offset in KICK_OFFSETS {
            self.piece.position.0 = original_x + offset;
            if self.can_move(0, 0) {
                return;
            }
        }

        // Restore original position and rotation if no valid position found
        self.piece.position.0 = original_x;
        self.piece.rotation = original_rotation;
        self.piece.rotated = rotated_shape(self.piece.typ, self.piece.rotation);
    }

    fn update_level(&mut self) {
        let current_level = self.level.current;
        if self.level.total_lines_cleared >= LEVEL_CONFIGS[current_level].lines_required
            && current_level < LEVEL_CONFIGS.len() - 1
        {
            self.level.current = current_level + 1;
        }
    }

    fn calculate_score(&self, lines_cleared: u32) -> u32 {
        let base_score = match lines_cleared {
            1 => SCORE.single,
            2 => SCORE.double,
            3 => SCORE.triple,
            4 => SCORE.tetris,
            _ => 0,
        };

        (base_score as f32 * LEVEL_CONFIGS[self.level.current].score_multiplier) as u32
    }

    /// Returns the events since the last call and resets them
    pub fn take_events(&mut self) -> crate::state::Events {
        std::mem::take(&mut self.events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{state::FlashingLines, tetromino::Bag};
    use macroquad::color::GRAY;

    const BOTTOM: usize = BOARD.height as usize - 1;

    fn playing_state() -> GameState {
        let mut state = GameState::new();
        state.start();
        state.take_events();
        state
    }

    /// Replaces the current piece with a horizontal I piece at the given position
    fn set_i_piece(state: &mut GameState, x: i32, y: i32) {
        state.piece.typ = Tetromino::I;
        state.piece.rotation = RotationState::Zero;
        state.piece.rotated = Tetromino::I.shape();
        state.piece.position = (x, y);
    }

    /// Fills the given row except for the four cells starting at `gap_x`
    fn fill_row_except_four(state: &mut GameState, row: usize, gap_x: usize) {
        for x in 0..BOARD.width as usize {
            if !(gap_x..gap_x + 4).contains(&x) {
                state.board.cells[row][x] = Some(GRAY);
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
    fn flashing_lines_bit_mask() {
        let mut lines = FlashingLines::default();
        assert!(lines.is_empty());
        lines.insert(0);
        lines.insert(19);
        assert!(lines.contains(0) && lines.contains(19) && !lines.contains(5));
        assert_eq!(lines.len(), 2);
        lines.clear();
        assert!(lines.is_empty());
    }

    #[test]
    fn piece_stays_inside_the_walls() {
        let mut state = playing_state();
        for _ in 0..20 {
            state.handle_input(InputState::MoveLeft);
        }
        let min_x = state
            .piece
            .rotated
            .iter()
            .map(|(x, _)| state.piece.position.0 + x)
            .min()
            .unwrap();
        assert_eq!(min_x, 0);
    }

    #[test]
    fn hard_drop_locks_piece_at_the_bottom() {
        let mut state = playing_state();
        set_i_piece(&mut state, 0, 0);
        state.handle_input(InputState::HardDrop);

        assert!((0..4).all(|x| state.board.cells[BOTTOM][x].is_some()));
        assert!(state.take_events().piece_locked);
    }

    #[test]
    fn cleared_line_is_scored_once_and_removed_after_animation() {
        let mut state = playing_state();
        fill_row_except_four(&mut state, BOTTOM, 0);
        state.board.cells[BOTTOM - 1][9] = Some(GRAY);
        set_i_piece(&mut state, 0, 0);
        state.handle_input(InputState::HardDrop);

        assert_eq!(state.score.current, SCORE.single);
        assert!(state.board.flashing_lines.contains(BOTTOM));

        // Gravity and input are paused during the animation
        let position = state.piece.position;
        state.handle_input(InputState::HardDrop);
        state.update(TIMING.line_clearing / 2.0);
        assert_eq!(state.score.current, SCORE.single);
        assert_eq!(state.piece.position, position);

        // After the animation the row above moved down and a new piece spawned
        state.update(TIMING.line_clearing);
        assert!(state.board.flashing_lines.is_empty());
        assert!(state.board.cells[BOTTOM][9].is_some());
        assert!(state.board.cells[BOTTOM - 1].iter().all(|c| c.is_none()));
        assert_eq!(state.piece.position.1, -1);
        assert_eq!(state.level.total_lines_cleared, 1);
    }

    #[test]
    fn four_lines_score_a_tetris() {
        let mut state = playing_state();
        for row in BOTTOM - 3..=BOTTOM {
            for x in 0..BOARD.width as usize - 1 {
                state.board.cells[row][x] = Some(GRAY);
            }
        }
        // Vertical I piece in the last column
        state.piece.typ = Tetromino::I;
        state.piece.rotation = RotationState::Right;
        state.piece.rotated = rotated_shape(Tetromino::I, RotationState::Right);
        state.piece.position = (BOARD.width - 2, 0);
        state.handle_input(InputState::HardDrop);

        assert_eq!(state.score.current, SCORE.tetris);
        assert_eq!(state.board.flashing_lines.len(), 4);
    }

    #[test]
    fn level_increases_after_enough_lines() {
        let mut state = playing_state();
        state.level.total_lines_cleared = LEVEL_CONFIGS[0].lines_required - 1;
        fill_row_except_four(&mut state, BOTTOM, 0);
        set_i_piece(&mut state, 0, 0);
        state.handle_input(InputState::HardDrop);

        assert_eq!(state.level.current, 1);
    }

    #[test]
    fn rotation_kicks_away_from_the_wall() {
        let mut state = playing_state();
        state.piece.typ = Tetromino::I;
        state.piece.rotation = RotationState::Right;
        state.piece.rotated = rotated_shape(Tetromino::I, RotationState::Right);
        // Vertical I piece in the rightmost column
        state.piece.position = (BOARD.width - 2, 5);
        state.handle_input(InputState::Rotate);

        assert!(matches!(state.piece.rotation, RotationState::Two));
        assert!(
            state
                .piece
                .rotated
                .iter()
                .all(|(x, _)| (0..BOARD.width).contains(&(state.piece.position.0 + x)))
        );
    }

    #[test]
    fn locking_above_the_field_ends_the_game() {
        let mut state = playing_state();
        for row in 0..BOARD.height as usize {
            state.board.cells[row][0] = Some(GRAY);
        }
        state.board.cells[0][0] = None;
        state.board.cells[1][0] = None;
        state.piece.typ = Tetromino::I;
        state.piece.rotation = RotationState::Right;
        state.piece.rotated = rotated_shape(Tetromino::I, RotationState::Right);
        state.piece.position = (-1, -2);
        state.handle_input(InputState::HardDrop);

        assert_eq!(state.status, GameStatus::GameOver);
    }

    #[test]
    fn soft_drop_falls_faster() {
        let mut state = playing_state();
        let start_y = state.piece.position.1;
        for _ in 0..10 {
            state.handle_input(InputState::Drop);
            state.update(SOFT_DROP_INTERVAL);
        }
        assert_eq!(state.piece.position.1, start_y + 10);
    }

    #[test]
    fn restart_keeps_high_score() {
        let mut state = playing_state();
        state.score.current = 500;
        state.score.highest = 500;
        state.restart();

        assert_eq!(state.score.current, 0);
        assert_eq!(state.score.highest, 500);
        assert_eq!(state.status, GameStatus::Playing);
    }
}
