use crate::{
    cache::{FontCache, TextCache},
    config::{BOARD, TEXT, TIMING},
    screen::ScreenConfig,
    state::{Board, FlashingLines, GameState, GameStatus, PieceState},
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
        let screen = ScreenConfig::new();
        let font = FontCache::new(&screen);

        let mut renderer = Self {
            game_field: render_target(0, 0),
            placed_pieces: render_target(0, 0),
            screen,
            text: TextCache::new(&font),
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
            self.font.update(&self.screen);
            self.text.update(&self.font);
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
                    self.update_placed_pieces(&dummy_board.cells, FlashingLines::default(), false);
                }
                _ => self.update_placed_pieces(
                    &state.board.cells,
                    state.board.flashing_lines,
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

    fn set_field_camera(&self, target: &RenderTarget) {
        let screen = &self.screen;
        set_camera(&Camera2D {
            zoom: vec2(2.0 / screen.field_width, 2.0 / screen.field_height),
            target: vec2(screen.field_width * 0.5, screen.field_height * 0.5),
            render_target: Some(target.clone()),
            ..Default::default()
        });
    }

    fn update_game_field(&mut self) {
        self.set_field_camera(&self.game_field);
        clear_background(BLANK);
        self.draw_game_field();
        set_default_camera();
    }

    fn update_placed_pieces(
        &mut self,
        cells: &Board,
        flashing_lines: FlashingLines,
        flashing: bool,
    ) {
        self.set_field_camera(&self.placed_pieces);
        clear_background(BLANK);
        self.draw_placed_pieces(cells, flashing_lines, flashing);
        set_default_camera();
        self.board_dirty = false;
    }

    fn set_render_targets(&mut self) {
        // Create new render targets at new size in physical pixels,
        // otherwise the field gets upscaled and blurry on high DPI screens
        let width = (self.screen.field_width * self.screen.dpi_scale).round() as u32;
        let height = (self.screen.field_height * self.screen.dpi_scale).round() as u32;
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

        // Whole physical pixel insets, so the block rasterizes identically on screen and in
        // the board render target
        let shading_inset = self.screen.snap(size * 0.1);
        let outline_width = self
            .screen
            .snap(size * 0.05)
            .max(1.0 / self.screen.dpi_scale);

        // Draw block face
        draw_rectangle(pos_x, pos_y, size, size, color);

        // Draw inner shading for 3D effect
        let darker = Color::new(color.r * 0.8, color.g * 0.8, color.b * 0.8, 1.0);
        draw_rectangle(
            pos_x + shading_inset,
            pos_y + shading_inset,
            size - 2.0 * shading_inset,
            size - 2.0 * shading_inset,
            darker,
        );

        // Draw outline, the look of semi-transparent black on the block face.
        // It has to be opaque: drawn into the board render target, a transparent
        // outline would get blended twice and look darker than on the falling piece.
        draw_rectangle_lines(
            pos_x,
            pos_y,
            size,
            size,
            outline_width * 2.0, // Lines are drawn with half the thickness inside
            Color::new(color.r * 0.5, color.g * 0.5, color.b * 0.5, 1.0),
        );
    }

    fn draw_text(&self, text: &str, x: f32, y: f32, font_size: f32) {
        // Whole pixel positions keep the pixel font crisp
        draw_text_ex(
            text,
            x.round(),
            y.round(),
            TextParams {
                font: Some(&self.font.font),
                font_size: font_size as u16,
                color: WHITE,
                ..Default::default()
            },
        );
    }

    fn get_button_bounds(&self, button_text: &str, font_size: f32) -> ButtonBounds {
        let button_dims = self.font.measure(button_text, font_size);

        let width = button_dims.width + font_size;
        let height = button_dims.height + font_size * 0.8;

        ButtonBounds {
            x: screen_width() / 2.0 - width / 2.0,
            y: screen_height() / 2.0 + screen_height() * 0.02,
            width,
            height,
        }
    }

    fn draw_start_screen(&mut self) {
        self.draw_overlay_screen(TEXT.start, TEXT.start_button, TEXT.instructions);
    }

    fn draw_game_over(&mut self, score: u32, high_score: u32, level: usize) {
        let score_text = format!("{}{}", TEXT.score, score);
        let highscore_text = format!("{}{}", TEXT.highscore, high_score);
        let level_text = format!("{}{}", TEXT.level, level + 1);
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
        let title_dims = self.font.measure(title, self.font.size);
        self.draw_text(
            title,
            center_x - title_dims.width / 2.0,
            center_y - spacing,
            self.font.size,
        );

        // Button
        let button = self.get_button_bounds(button_text, self.font.button_size);
        draw_rectangle(button.x, button.y, button.width, button.height, DARKGRAY);
        let button_dims = self.font.measure(button_text, self.font.button_size);
        self.draw_text(
            button_text,
            button.x + (button.width - button_dims.width) / 2.0,
            button.y + (button.height - button_dims.height) / 2.0 + button_dims.offset_y,
            self.font.button_size,
        );

        // Subtext
        let mut y = center_y + spacing * 3.0;
        for text in subtext {
            let dims = self.font.measure(text, self.font.stats_size);
            self.draw_text(text, center_x - dims.width / 2.0, y, self.font.stats_size);
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

    fn draw_stats(&mut self, current_score: u32, level: usize) {
        let font_size = self.font.stats_size;
        let padding = 10.0;

        self.text.score.set(current_score, &self.font, font_size);
        self.text.level.set(level as u32 + 1, &self.font, font_size);

        // Score drawing
        let score_width = self.text.score_label_dims.width + self.text.score.width;
        let x_score = if self.screen.offset_x > score_width + padding * 3.0 {
            self.screen.offset_x - score_width - padding * 2.0
        } else {
            padding
        };

        // Level drawing
        let total_level_width = self.text.level_label_dims.width + self.text.level.width;
        let game_field_right = self.screen.offset_x + self.screen.field_width;
        let x_level = if screen_width() > game_field_right + total_level_width + padding * 3.0 {
            game_field_right + padding * 2.0
        } else {
            screen_width() - total_level_width - padding
        };

        let y = self.text.level_label_dims.height + padding;

        // Draw texts
        self.draw_text(TEXT.score, x_score, y, font_size);
        self.draw_text(
            &self.text.score.text,
            x_score + self.text.score_label_dims.width,
            y,
            font_size,
        );
        self.draw_text(TEXT.level, x_level, y, font_size);
        self.draw_text(
            &self.text.level.text,
            x_level + self.text.level_label_dims.width,
            y,
            font_size,
        );
    }

    fn draw_game_field(&self) {
        let screen = &self.screen;

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

    fn draw_placed_pieces(&self, cells: &Board, flashing_lines: FlashingLines, flashing: bool) {
        for (y, row) in cells.iter().enumerate() {
            let is_line_flashing = flashing && flashing_lines.contains(y);
            for (x, cell) in row.iter().enumerate() {
                if let Some(color) = *cell {
                    let draw_color = if is_line_flashing { WHITE } else { color };
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
        let text_dims = self.font.measure(&fps_text, font_size);
        let x = screen_width() - text_dims.width - padding;
        let y = 2.5 * (text_dims.height + padding);

        self.draw_text(&fps_text, x, y, font_size);
    }
}
