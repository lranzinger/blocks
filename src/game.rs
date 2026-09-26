use crate::{
    config::{BOARD, LEVEL_CONFIGS, SCORE, TIMING},
    input::{InputHandler, InputState},
    renderer::Renderer,
    state::{GameState, GameStatus},
    storage,
    tetromino::{RotationState, Tetromino},
};
use macroquad::prelude::*;
use smallvec::SmallVec;

pub struct Game {
    pub state: GameState,
    pub renderer: Renderer,
    pub input: InputHandler,
}
impl Game {
    pub fn new() -> Self {
        Self {
            state: GameState::new(),
            renderer: Renderer::new(),
            input: InputHandler::new(),
        }
    }

    fn spawn_piece(&mut self) {
        self.state.piece.typ = Tetromino::random();
        let shape = self.state.piece.typ.shape();
        let piece_width = shape.iter().map(|(x, _)| x).max().unwrap()
            - shape.iter().map(|(x, _)| x).min().unwrap()
            + 1;
        self.state.piece.position = (BOARD.width / 2 - piece_width / 2, -1);
        self.state.piece.rotation = RotationState::Zero;
        self.state.piece.rotated = self.get_rotated_shape();

        if self.is_game_over() {
            self.end_game();
        }
    }

    fn get_rotated_shape(&self) -> [(i32, i32); 4] {
        let shape = self.state.piece.typ.shape();
        let pivot = (1, 1);

        // Using array methods directly
        let moved_center = shape.map(|(x, y)| (x - pivot.0, y - pivot.1));

        match self.state.piece.rotation {
            RotationState::Zero => shape,
            RotationState::Right => moved_center.map(|(x, y)| (-y + pivot.0, x + pivot.1)),
            RotationState::Two => moved_center.map(|(x, y)| (-x + pivot.0, -y + pivot.1)),
            RotationState::Left => moved_center.map(|(x, y)| (y + pivot.0, -x + pivot.1)),
        }
    }

    fn can_move(&self, dx: i32, dy: i32) -> bool {
        for &(x, y) in &self.state.piece.rotated {
            let new_x = self.state.piece.position.0 + x + dx;
            let new_y = self.state.piece.position.1 + y + dy;
            if !(0..BOARD.width).contains(&new_x)
                || new_y >= BOARD.height
                || (new_y >= 0 && self.state.board.cells[new_y as usize][new_x as usize].is_some())
            {
                return false;
            }
        }
        true
    }

    /// Writes the current piece into the board.
    /// Returns `true` if part of the piece is above the visible field (lock out).
    fn lock_piece(&mut self) -> bool {
        let mut locked_out = false;
        for &(x, y) in &self.state.piece.rotated {
            let board_x = self.state.piece.position.0 + x;
            let board_y = self.state.piece.position.1 + y;
            if board_y >= 0 {
                self.state.board.cells[board_y as usize][board_x as usize] =
                    Some(self.state.piece.typ.color());
            } else {
                locked_out = true;
            }
        }
        self.input.reset();
        locked_out
    }

    fn settle_piece(&mut self) {
        let locked_out = self.lock_piece();
        self.renderer.mark_board_dirty();
        if locked_out {
            self.end_game();
            return;
        }

        self.clear_lines();
        // With cleared lines the next piece spawns after the flash animation
        if self.state.board.flashing_lines.is_empty() {
            self.spawn_piece();
        }
    }

    fn end_game(&mut self) {
        let last_highscore = storage::get_high_score();
        let new_highscore = self.state.score.highest;
        if new_highscore > last_highscore {
            storage::update_high_score(new_highscore);
        }
        self.state.status = GameStatus::GameOver;
    }

    fn clear_lines(&mut self) {
        let mut lines_to_clear = SmallVec::new();

        // Identify full lines
        for y in 0..BOARD.height as u8 {
            if self.state.board.cells[y as usize]
                .iter()
                .all(|&cell| cell.is_some())
            {
                lines_to_clear.push(y);
            }
        }

        if !lines_to_clear.is_empty() {
            let num_of_lines_to_clear = lines_to_clear.len() as u32;

            // Start line clear animation
            self.state.board.flashing_lines = lines_to_clear;
            self.state.timing.line_clear_timer = TIMING.line_clearing;

            // Calculate scrore
            let score = self.calculate_score(num_of_lines_to_clear);
            self.state.score.current += score;
            self.state.score.highest = self.state.score.highest.max(self.state.score.current);
            self.state.level.total_lines_cleared += num_of_lines_to_clear;

            // Update level
            self.update_level();
        }
    }

    fn is_game_over(&self) -> bool {
        // Check if new piece overlaps with existing pieces
        for &(x, y) in &self.state.piece.rotated {
            let board_x = self.state.piece.position.0 + x;
            let board_y = self.state.piece.position.1 + y;
            if board_y >= 0 && self.state.board.cells[board_y as usize][board_x as usize].is_some()
            {
                return true;
            }
        }
        false
    }

    pub fn update_logic(&mut self) {
        match self.state.status {
            GameStatus::Start => {
                if self.renderer.check_click(GameStatus::Start) {
                    self.state.dummy_board = None;
                    self.state.status = GameStatus::Playing;
                    self.renderer.mark_board_dirty();
                    // Prevent the button tap from rotating the first piece
                    self.input.reset();
                }
            }
            GameStatus::Playing => self.update_gameplay(),
            GameStatus::GameOver => {
                if self.renderer.check_click(GameStatus::GameOver) {
                    self.restart();
                    self.state.status = GameStatus::Playing;
                    self.input.reset();
                }
            }
        }
    }

    fn update_gameplay(&mut self) {
        let delta = get_frame_time();

        // Handle line clear animation, the game is paused meanwhile
        if !self.state.board.flashing_lines.is_empty() {
            self.state.timing.line_clear_timer -= delta;
            if self.state.timing.line_clear_timer <= 0.0 {
                // Remove lines after flashing
                self.remove_flashing_lines();
                self.state.board.flashing_lines.clear();
                self.spawn_piece();
            }
            return;
        }

        // Update timers
        self.state.timing.fall_timer += delta;

        // Handle automatic piece falling
        if self.state.timing.fall_timer >= self.state.timing.fall_interval {
            self.state.timing.fall_timer = 0.0;
            if self.can_move(0, 1) {
                self.state.piece.position.1 += 1;
            } else {
                self.settle_piece();
            }
        }
    }

    pub fn handle_input(&mut self, input: InputState) {
        if !matches!(self.state.status, GameStatus::Playing)
            || !self.state.board.flashing_lines.is_empty()
        {
            return;
        }

        match input {
            InputState::MoveLeft => {
                if self.can_move(-1, 0) {
                    self.state.piece.position.0 -= 1;
                }
            }
            InputState::MoveRight => {
                if self.can_move(1, 0) {
                    self.state.piece.position.0 += 1;
                }
            }
            InputState::Rotate => {
                self.try_rotation();
            }
            InputState::Drop => {
                self.state.timing.fall_interval = 0.05; // Increase fall speed when dropping
            }
            InputState::None => {
                self.state.timing.fall_interval =
                    LEVEL_CONFIGS[self.state.level.current].fall_interval;
            }
        }
    }

    fn try_rotation(&mut self) {
        if self.state.piece.typ == Tetromino::O {
            return;
        }

        let original_x = self.state.piece.position.0;

        let offsets = [0, -1, 1, -2, 2];

        let temp_rotation = self.state.piece.rotation;
        self.state.piece.rotation = self.state.piece.rotation.next();
        self.state.piece.rotated = self.get_rotated_shape();

        for &offset in &offsets {
            self.state.piece.position.0 = original_x + offset;
            if self.can_move(0, 0) {
                return;
            }
        }

        // Restore original position and rotation if no valid position found
        self.state.piece.position.0 = original_x;
        self.state.piece.rotation = temp_rotation;
        self.state.piece.rotated = self.get_rotated_shape();
    }

    fn restart(&mut self) {
        let high_score = self.state.score.highest;
        let mut new_state = GameState::new();
        self.renderer.mark_board_dirty();
        new_state.score.highest = high_score;
        self.state = new_state;
    }

    fn remove_flashing_lines(&mut self) {
        let mut new_board = [[None; BOARD.width as usize]; BOARD.height as usize];
        let mut new_row = BOARD.height as usize - 1;

        // Copy the board, skipping the lines that were cleared
        for y in (0..BOARD.height as u8).rev() {
            if !self.state.board.flashing_lines.contains(&y) {
                new_board[new_row] = self.state.board.cells[y as usize];
                new_row = new_row.saturating_sub(1);
            }
        }

        self.renderer.mark_board_dirty();
        self.state.board.cells = new_board;
    }

    fn update_level(&mut self) {
        let current_level = self.state.level.current;
        let current_config = &LEVEL_CONFIGS[current_level];

        if self.state.level.total_lines_cleared >= current_config.lines_required
            && current_level < LEVEL_CONFIGS.len() - 1
        {
            let next_level = current_level + 1;
            self.state.level.current = next_level;
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

        (base_score as f32 * LEVEL_CONFIGS[self.state.level.current].score_multiplier) as u32
    }
}
