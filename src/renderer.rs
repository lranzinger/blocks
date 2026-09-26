use crate::{
    cache::{FontCache, TextCache},
    config::{BOARD, TEXT, TIMING},
    screen::ScreenConfig,
    state::{Board, GameState, GameStatus, PieceState},
};
use macroquad::prelude::*;

struct ButtonBounds {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

pub struct Renderer {
    game_field: RenderTarget,
    placed_pieces: RenderTarget,
    pub screen: ScreenConfig,
    text: TextCache,
    font: FontCache,
    last_fps_update: f64,
    current_fps: i32,
    board_dirty: bool,
    flashing: bool,
}

impl Renderer {
    pub fn new() -> Self {
        let font = FontCache::new();
        let screen = ScreenConfig::new();

        let mut renderer = Self {
            game_field: render_target(0, 0),
            placed_pieces: render_target(0, 0),
            screen,
            text: TextCache::new(font.stats_size as u16),
            font,
            last_fps_update: 0.0,
            current_fps: 0,
            board_dirty: false,
            flashing: false,
        };
        renderer.set_render_targets();
        renderer
    }

    pub fn draw(&mut self, state: &GameState) {
        let current_size = (screen_width(), screen_height());
        if self.screen.size != current_size || self.screen.dpi_scale != screen_dpi_scale() {
            self.screen = ScreenConfig::new();
            self.font.update();
            self.text.update(self.font.stats_size as u16);
            self.set_render_targets();
        }

        let new_flashing = if state.board.flashing_lines.is_empty() {
            false
        } else {
            (get_time() * TIMING.flashing_intervall) as i32 % 2 == 0
        };

        if self.flashing != new_flashing {
            self.board_dirty = true;
            self.flashing = new_flashing;
        }

        // Update placed pieces if needed
        if self.board_dirty {
            match (&state.status, &state.dummy_board) {
                (GameStatus::Start, Some(dummy_board)) => {
                    self.update_placed_pieces(&dummy_board.cells, &[], false);
                }
                _ => self.update_placed_pieces(
                    &state.board.cells,
                    &state.board.flashing_lines,
                    self.flashing,
                ),
            }
        }

        // Draw game field
        self.draw_field_texture(&self.game_field.texture);

        // Draw placed pieces
        self.draw_field_texture(&self.placed_pieces.texture);

        match state.status {
            GameStatus::Start => {
                self.draw_start_screen();
            }
            GameStatus::Playing => {
                // The locked piece is part of the board while lines are flashing
                if state.board.flashing_lines.is_empty() {
                    self.draw_current_piece(&state.piece);
                }
                self.draw_stats(state.score.current, state.level.current);
            }
            GameStatus::GameOver => {
                self.draw_game_over(
                    state.score.current,
                    state.score.highest,
                    state.level.current,
                );
            }
        }
        self.draw_debug_info();
    }

    fn draw_field_texture(&self, texture: &Texture2D) {
        draw_texture_ex(
            texture,
            self.screen.offset_x,
            self.screen.offset_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(self.screen.field_width, self.screen.field_height)),
                ..Default::default()
            },
        );
    }

    fn update_game_field(&mut self) {
        let screen = &self.screen;
        set_camera(&Camera2D {
            zoom: vec2(2.0 / screen.field_width, 2.0 / screen.field_height),
            target: vec2(screen.field_width * 0.5, screen.field_height * 0.5),
            render_target: Some(self.game_field.clone()),
            ..Default::default()
        });
        clear_background(BLANK);
        self.draw_game_field(screen);
        set_default_camera();
    }

    fn update_placed_pieces(&mut self, cells: &Board, flashing_lines: &[u8], flashing: bool) {
        set_camera(&Camera2D {
            zoom: vec2(
                2.0 / self.screen.field_width,
                2.0 / self.screen.field_height,
            ),
            target: vec2(
                self.screen.field_width * 0.5,
                self.screen.field_height * 0.5,
            ),
            render_target: Some(self.placed_pieces.clone()),
            ..Default::default()
        });
        clear_background(BLANK);
        self.draw_placed_pieces(cells, flashing_lines, flashing);
        set_default_camera();
        self.board_dirty = false;
    }

    fn set_render_targets(&mut self) {
        // Create new render targets at new size in physical pixels,
        // otherwise the field gets upscaled and blurry on high DPI screens
        let width = (self.screen.field_width * self.screen.dpi_scale)
            .round()
            .max(1.0) as u32;
        let height = (self.screen.field_height * self.screen.dpi_scale)
            .round()
            .max(1.0) as u32;
        self.game_field = render_target(width, height);
        self.placed_pieces = render_target(width, height);

        // Set filtering mode
        self.game_field.texture.set_filter(FilterMode::Nearest);
        self.placed_pieces.texture.set_filter(FilterMode::Nearest);

        // Update game field with new size
        self.update_game_field();
        self.board_dirty = true;
    }

    pub fn mark_board_dirty(&mut self) {
        self.board_dirty = true;
    }

    fn draw_block(&self, x: f32, y: f32, color: Color, offset: bool) {
        let (offset_x, offset_y) = if offset {
            (self.screen.offset_x, self.screen.offset_y)
        } else {
            (0.0, 0.0)
        };

        let size = self.screen.block_size;
        let pos_x = offset_x + x * size;
        let pos_y = offset_y + y * size;

        // Draw block face
        draw_rectangle(pos_x, pos_y, size, size, color);

        // Draw inner shading for 3D effect
        let darker = Color::new(color.r * 0.8, color.g * 0.8, color.b * 0.8, 1.0);
        draw_rectangle(
            pos_x + size * 0.1,
            pos_y + size * 0.1,
            size * 0.8,
            size * 0.8,
            darker,
        );

        // Draw smooth outline
        draw_rectangle_lines(
            pos_x,
            pos_y,
            size,
            size,
            size * 0.1,                     // Thicker lines
            Color::new(0.0, 0.0, 0.0, 0.5), // Semi-transparent black
        );
    }

    fn get_button_bounds(&self, button_text: &str, font_size: f32) -> ButtonBounds {
        let button_dims = measure_text(button_text, None, font_size as u16, 1.0);

        let width = button_dims.width + screen_width() * 0.01;
        let height = button_dims.height + screen_height() * 0.02;

        ButtonBounds {
            x: screen_width() / 2.0 - width / 2.0,
            y: screen_height() / 2.0 + screen_height() * 0.02,
            width,
            height,
        }
    }

    fn draw_start_screen(&mut self) {
        let instructions = [
            "Links/Rechts: Bewegen",
            "Tippen: Drehen",
            "Halten: Fallen lassen",
        ];

        self.draw_overlay_screen(TEXT.start, TEXT.start_button, instructions);
    }

    fn draw_game_over(&mut self, score: u32, high_score: u32, level: usize) {
        let score_text = [TEXT.score, &score.to_string()].join("");
        let highscore_text = [TEXT.highscore, &high_score.to_string()].join("");
        let level_text = [TEXT.level, &(level + 1).to_string()].join("");
        let scores = [
            score_text.as_str(),
            level_text.as_str(),
            highscore_text.as_str(),
        ];

        self.draw_overlay_screen(TEXT.gameover, TEXT.gameover_button, scores);
    }

    fn draw_overlay_screen(&mut self, title: &str, button_text: &str, subtext: [&str; 3]) {
        let screen_w = screen_width();
        let screen_h = screen_height();
        let center_x = screen_w / 2.0;
        let center_y = screen_h / 2.0;
        let spacing = screen_h * 0.05;

        // Background
        draw_rectangle(0.0, 0.0, screen_w, screen_h, Color::new(0.0, 0.0, 0.0, 0.7));

        // Title
        let title_dims = measure_text(title, None, self.font.size as u16, 1.0);
        draw_text(
            title,
            center_x - title_dims.width / 2.0,
            center_y - spacing,
            self.font.size,
            WHITE,
        );

        // Button
        let button = self.get_button_bounds(button_text, self.font.button_size);
        draw_rectangle(button.x, button.y, button.width, button.height, DARKGRAY);
        let button_dims = measure_text(button_text, None, self.font.button_size as u16, 1.0);
        draw_text(
            button_text,
            button.x + (button.width - button_dims.width) / 2.0,
            button.y + (button.height + button_dims.height) / 2.0,
            self.font.button_size,
            WHITE,
        );

        // Subtext
        let mut y = center_y + spacing * 3.0;
        for text in subtext {
            let dims = measure_text(text, None, self.font.stats_size as u16, 1.0);
            draw_text(
                text,
                center_x - dims.width / 2.0,
                y,
                self.font.stats_size,
                WHITE,
            );
            y += spacing;
        }
    }

    pub fn check_click(&self, status: GameStatus) -> bool {
        if !is_mouse_button_pressed(MouseButton::Left) {
            return false;
        }

        let button_size = self.font.button_size;
        let button_text = match status {
            GameStatus::Start => TEXT.start_button,
            GameStatus::GameOver => TEXT.gameover_button,
            _ => return false,
        };
        let (mouse_x, mouse_y) = mouse_position();
        let button = self.get_button_bounds(button_text, button_size);

        mouse_x >= button.x
            && mouse_x <= button.x + button.width
            && mouse_y >= button.y
            && mouse_y <= button.y + button.height
    }

    fn draw_stats(&self, current_score: u32, level: usize) {
        let font_size = self.font.stats_size;
        let padding = 10.0;

        // Score drawing
        let score_num_width = self.text.get_number_width(current_score);
        let x_score = if self.screen.offset_x
            > self.text.score_label_dims.width + score_num_width + padding * 3.0
        {
            self.screen.offset_x
                - self.text.score_label_dims.width
                - score_num_width
                - padding * 2.0
        } else {
            padding
        };

        // Level drawing
        let level_num_width = self.text.get_number_width((level + 1) as u32);
        let total_level_width = self.text.level_label_dims.width + level_num_width;
        let game_field_right = self.screen.offset_x + (BOARD.width as f32 * self.screen.block_size);
        let x_level = if screen_width() > game_field_right + total_level_width + padding * 3.0 {
            game_field_right + padding * 2.0
        } else {
            screen_width() - total_level_width - padding
        };

        let y = self.text.level_label_dims.height + padding;

        // Draw texts
        draw_text(TEXT.score, x_score, y, font_size, WHITE);
        draw_text(
            &current_score.to_string(),
            x_score + self.text.score_label_dims.width,
            y,
            font_size,
            WHITE,
        );
        draw_text(TEXT.level, x_level, y, font_size, WHITE);
        draw_text(
            &(level + 1).to_string(),
            x_level + self.text.level_label_dims.width,
            y,
            font_size,
            WHITE,
        );
    }

    fn draw_game_field(&self, screen: &ScreenConfig) {
        // Draw border
        draw_rectangle_lines(
            0.0,
            0.0,
            screen.field_width,
            screen.field_height,
            3.0,
            DARKGRAY,
        );

        // Vertical lines
        for x in 1..BOARD.width {
            let thickness = if x % 2 == 0 { 1.5 } else { 1.0 };
            draw_line(
                x as f32 * screen.block_size,
                0.0,
                x as f32 * screen.block_size,
                screen.field_height,
                thickness,
                DARKGRAY,
            );
        }

        // Horizontal lines
        for y in 1..BOARD.height {
            let thickness = if y % 2 == 0 { 2.0 } else { 1.0 };
            draw_line(
                0.0,
                y as f32 * screen.block_size,
                screen.field_width,
                y as f32 * screen.block_size,
                thickness,
                DARKGRAY,
            );
        }
    }

    fn draw_placed_pieces(&mut self, cells: &Board, flashing_lines: &[u8], flashing: bool) {
        for y in 0..BOARD.height as u8 {
            let is_line_flashing = flashing_lines.contains(&y);
            for x in 0..BOARD.width as u8 {
                if let Some(color) = cells[y as usize][x as usize] {
                    let draw_color = if flashing && is_line_flashing {
                        WHITE
                    } else {
                        color
                    };

                    self.draw_block(x as f32, y as f32, draw_color, false);
                }
            }
        }
    }

    fn draw_current_piece(&self, piece: &PieceState) {
        for &(x, y) in &piece.rotated {
            let draw_x = piece.position.0 + x;
            let draw_y = piece.position.1 + y;
            if draw_y >= 0 {
                self.draw_block(draw_x as f32, draw_y as f32, piece.typ.color(), true);
            }
        }
    }

    fn draw_debug_info(&mut self) {
        if !cfg!(debug_assertions) {
            return;
        }

        let current_time = get_time();

        // Update FPS once per second
        if current_time - self.last_fps_update >= 1.0 {
            self.current_fps = get_fps();
            self.last_fps_update = current_time;
        }

        let fps_text = self.current_fps.to_string();
        let padding: f32 = 10.0;

        let font_size = self.font.debug_size;
        let text_dims = measure_text(&fps_text, None, font_size as u16, 1.0);
        let x = screen_width() - text_dims.width - padding;
        let y = 2.5 * (text_dims.height + padding);

        draw_text(&fps_text, x, y, font_size, WHITE);
    }
}
