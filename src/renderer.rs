use macroquad::{
    models::{Mesh, Vertex, draw_mesh},
    prelude::*,
};

use crate::{
    config::{BOARD, TEXT},
    effects::{Backdrop, Effects, FLASH_DURATION, GOLD, TRAIL_DURATION, format_number},
    layout::Layout,
    logic::{Board, GameState, GameStatus, Piece, Rows},
    tetromino::{Rotation, Tetromino},
    text::Text,
};

const BACKGROUND_TOP: Color = Color::new(0.043, 0.055, 0.118, 1.0);
const BACKGROUND_BOTTOM: Color = Color::new(0.102, 0.055, 0.18, 1.0);
const BOARD_FILL: Color = Color::new(0.035, 0.047, 0.094, 1.0);
const GRID: Color = Color::new(0.09, 0.11, 0.2, 1.0);
const FRAME: Color = Color::new(0.2, 0.24, 0.4, 1.0);
const PANEL_FILL: Color = Color::new(0.06, 0.075, 0.15, 1.0);
const MUTED: Color = Color::new(0.55, 0.6, 0.78, 1.0);

/// What the cached board texture currently shows
#[derive(PartialEq, Clone, Copy)]
struct BoardCacheKey {
    version: u32,
    hidden_rows: Rows,
}

pub struct Renderer {
    text: Text,
    pub layout: Layout,
    board_target: RenderTarget,
    cache: Option<BoardCacheKey>,
}

impl Renderer {
    pub fn new() -> Self {
        let mut text = Text::new();
        let layout = Layout::new(&mut text);
        let mut renderer = Self {
            text,
            layout,
            board_target: render_target(1, 1),
            cache: None,
        };
        renderer.create_board_target();
        renderer
    }

    /// Recalculates the layout when the window size changed
    pub fn update_layout(&mut self) {
        if self.layout.is_outdated() {
            self.layout = Layout::new(&mut self.text);
            self.create_board_target();
        }
    }

    fn create_board_target(&mut self) {
        let board = self.layout.board;
        let dpi = self.layout.dpi_scale;
        self.board_target = render_target(
            (board.w * dpi).round() as u32,
            (board.h * dpi).round() as u32,
        );
        self.board_target.texture.set_filter(FilterMode::Nearest);
        self.cache = None;
    }

    pub fn draw(
        &mut self,
        state: &GameState,
        effects: &Effects,
        backdrop: &Backdrop,
        touch: bool,
        time: f64,
    ) {
        let layout = &self.layout;
        clear_background(BACKGROUND_TOP);
        gradient_rect(
            Rect::new(0.0, 0.0, layout.size.0, layout.size.1),
            BACKGROUND_TOP,
            BACKGROUND_BOTTOM,
        );

        if state.status == GameStatus::Start {
            self.draw_backdrop(backdrop);
        }

        self.draw_panels(state);
        self.draw_board(state, effects);

        match state.status {
            GameStatus::Start => self.draw_start_screen(state, touch, time),
            GameStatus::Paused => self.draw_pause_screen(touch, time),
            GameStatus::GameOver => self.draw_game_over(state, touch, time),
            GameStatus::Playing => self.draw_popups(effects),
        }
    }

    // Board

    fn draw_board(&mut self, state: &GameState, effects: &Effects) {
        let block = self.layout.block;
        let px = self.layout.px();
        let shake = effects.shake_offset() * block;
        let origin = vec2(
            self.layout.snap(self.layout.board.x + shake.x),
            self.layout.snap(self.layout.board.y + shake.y),
        );
        let board = Rect::new(origin.x, origin.y, self.layout.board.w, self.layout.board.h);

        // Frame and grid
        let frame = 2.0 * px.max(self.layout.snap(block * 0.06));
        draw_rectangle(
            board.x - frame,
            board.y - frame,
            board.w + 2.0 * frame,
            board.h + 2.0 * frame,
            FRAME,
        );
        draw_rectangle(board.x, board.y, board.w, board.h, BOARD_FILL);
        for x in 1..BOARD.width {
            draw_rectangle(board.x + x as f32 * block, board.y, px, board.h, GRID);
        }
        for y in 1..BOARD.height {
            draw_rectangle(board.x, board.y + y as f32 * block, board.w, px, GRID);
        }

        if state.status == GameStatus::Paused {
            // Hide the board while paused
            return;
        }

        // Locked blocks, cached in a texture and only redrawn when they change
        let clearing = state
            .line_clear
            .as_ref()
            .map(|clear| clear.rows)
            .unwrap_or_default();
        let key = BoardCacheKey {
            version: state.board_version,
            hidden_rows: clearing,
        };
        if self.cache != Some(key) {
            self.render_board_cache(&state.board, clearing);
            self.cache = Some(key);
        }
        draw_texture_ex(
            &self.board_target.texture,
            board.x,
            board.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(board.w, board.h)),
                ..Default::default()
            },
        );

        let cell = |x: i32, y: i32| {
            vec2(
                board.x + x as f32 * block,
                board.y + (y - BOARD.hidden) as f32 * block,
            )
        };

        if let Some(clear) = &state.line_clear {
            self.draw_clearing_rows(&state.board, clear.rows, clear.progress, &cell);
        }

        if state.status == GameStatus::Playing && state.line_clear.is_none() {
            for (x, y) in visible_cells(&state.piece) {
                let position = cell(x, y);
                draw_block(
                    position.x,
                    position.y,
                    block,
                    state.piece.kind.color(),
                    &self.layout,
                );
            }
        }

        self.draw_effects(effects, board, &cell);
    }

    fn render_board_cache(&mut self, cells: &Board, hidden_rows: Rows) {
        let board = self.layout.board;
        set_camera(&Camera2D {
            zoom: vec2(2.0 / board.w, 2.0 / board.h),
            target: vec2(board.w / 2.0, board.h / 2.0),
            render_target: Some(self.board_target.clone()),
            ..Default::default()
        });
        clear_background(BLANK);
        let block = self.layout.block;
        for (y, row) in cells.iter().enumerate().skip(BOARD.hidden as usize) {
            if hidden_rows.contains(y) {
                continue;
            }
            for (x, cell) in row.iter().enumerate() {
                if let Some(kind) = cell {
                    let top = (y as i32 - BOARD.hidden) as f32 * block;
                    draw_block(x as f32 * block, top, block, kind.color(), &self.layout);
                }
            }
        }
        set_default_camera();
    }

    fn draw_clearing_rows(
        &self,
        cells: &Board,
        rows: Rows,
        progress: f32,
        cell: &dyn Fn(i32, i32) -> Vec2,
    ) {
        let block = self.layout.block;
        const FLASH: f32 = 0.35;
        for (y, row) in cells.iter().enumerate() {
            if !rows.contains(y) {
                continue;
            }
            for (x, kind) in row.iter().enumerate() {
                let Some(kind) = kind else { continue };
                let position = cell(x as i32, y as i32);
                if progress < FLASH {
                    let color = mix(kind.color(), WHITE, progress / FLASH);
                    draw_block(position.x, position.y, block, color, &self.layout);
                } else {
                    // Shrink and fade out
                    let t = (progress - FLASH) / (1.0 - FLASH);
                    let size = block * (1.0 - t);
                    let offset = (block - size) / 2.0;
                    draw_rectangle(
                        position.x + offset,
                        position.y + offset,
                        size,
                        size,
                        Color::new(1.0, 1.0, 1.0, 1.0 - t),
                    );
                }
            }
        }
    }

    fn draw_effects(&self, effects: &Effects, board: Rect, cell: &dyn Fn(i32, i32) -> Vec2) {
        let block = self.layout.block;

        for trail in &effects.trails {
            let fade = 1.0 - trail.age / TRAIL_DURATION;
            for &(x, top) in &trail.columns {
                let bottom = board.y + top * block;
                let height = (trail.rows * block).min(bottom - board.y);
                let rect = Rect::new(
                    board.x + (x as f32 + 0.2) * block,
                    bottom - height,
                    block * 0.6,
                    height,
                );
                let color = trail.color;
                gradient_rect(
                    rect,
                    Color::new(color.r, color.g, color.b, 0.0),
                    Color::new(color.r, color.g, color.b, 0.45 * fade),
                );
            }
        }

        for flash in &effects.flashes {
            let alpha = 0.6 * (1.0 - flash.age / FLASH_DURATION);
            for &(x, y) in &flash.cells {
                if y >= BOARD.hidden {
                    let position = cell(x, y);
                    draw_rectangle(
                        position.x,
                        position.y,
                        block,
                        block,
                        Color::new(1.0, 1.0, 1.0, alpha),
                    );
                }
            }
        }

        for particle in &effects.particles {
            let size = particle.size * block;
            let color = particle.color;
            draw_rectangle(
                board.x + particle.position.x * block - size / 2.0,
                board.y + particle.position.y * block - size / 2.0,
                size,
                size,
                Color::new(color.r, color.g, color.b, particle.life()),
            );
        }
    }

    fn draw_popups(&self, effects: &Effects) {
        let board = self.layout.board;
        let size = self.layout.fonts.popup;
        for popup in &effects.popups {
            if popup.age < 0.0 {
                continue;
            }
            let fade_in = (popup.age / 0.1).min(1.0);
            let fade_out = ((popup.duration - popup.age) / 0.3).min(1.0);
            let alpha = fade_in.min(fade_out);
            let rise = popup.age * self.layout.block * 1.2;
            let line_height = size * 1.5;
            let top = board.y + board.h * 0.32
                - rise
                - (popup.lines.len() as f32 - 1.0) * line_height / 2.0;
            for (i, (line, color)) in popup.lines.iter().enumerate() {
                self.text.draw_shadowed(
                    line,
                    board.center().x,
                    top + i as f32 * line_height,
                    size,
                    Color::new(color.r, color.g, color.b, alpha),
                );
            }
        }
    }

    // Panels

    fn draw_panels(&self, state: &GameState) {
        let layout = &self.layout;
        let fonts = &layout.fonts;
        let block = layout.block;
        let showing_game = state.status != GameStatus::Start;

        // Hold
        let hold_content = self.draw_box(layout.hold, TEXT.hold);
        if let (Some(kind), true) = (state.hold, showing_game) {
            let size = if layout.portrait { 0.5 } else { 0.8 } * block;
            draw_preview(kind, hold_content.center(), size, layout);
        }

        // Next pieces: the first one larger
        let next_content = self.draw_box(layout.next, TEXT.next);
        if showing_game {
            if layout.portrait {
                let first = Rect::new(
                    next_content.x,
                    next_content.y,
                    next_content.w * 0.55,
                    next_content.h,
                );
                draw_preview(state.next[0], first.center(), 0.5 * block, layout);
                let small = 0.32 * block;
                for (i, kind) in state.next[1..].iter().enumerate() {
                    let center = vec2(
                        next_content.x + next_content.w * 0.8,
                        next_content.y + next_content.h * (0.28 + 0.44 * i as f32),
                    );
                    draw_preview(*kind, center, small, layout);
                }
            } else {
                let slots = [(0.18, 0.85), (0.5, 0.65), (0.78, 0.65)];
                for (kind, (y, scale)) in state.next.iter().zip(slots) {
                    let center = vec2(next_content.center().x, next_content.y + next_content.h * y);
                    draw_preview(*kind, center, scale * block, layout);
                }
            }
        }

        self.draw_pause_button(state.status == GameStatus::Playing);
        if !showing_game {
            return;
        }

        // Stats
        let stats = &layout.stats;
        let entries = [
            (stats.score, TEXT.score, format_number(state.score), WHITE),
            (
                stats.record,
                TEXT.record,
                format_number(state.high_score.max(state.score)),
                GOLD,
            ),
            (stats.level, TEXT.level, state.level.to_string(), WHITE),
            (stats.lines, TEXT.lines, state.lines.to_string(), WHITE),
        ];
        for (index, (rect, label, value, color)) in entries.iter().enumerate() {
            let value_size = if layout.portrait && index >= 2 {
                fonts.label
            } else {
                fonts.value
            };
            let right_aligned = layout.portrait && index == 1;
            let label_y = rect.y + rect.h * 0.28;
            let value_y = rect.y + rect.h * 0.7;
            if right_aligned {
                let right = rect.right() - 0.3 * block;
                self.text
                    .draw_right(label, right, label_y, fonts.label, MUTED);
                self.text
                    .draw_right(value, right, value_y, value_size, *color);
            } else if layout.portrait && index >= 2 {
                self.text
                    .draw_centered(label, rect.center().x, label_y, fonts.label, MUTED);
                self.text
                    .draw_centered(value, rect.center().x, value_y, value_size, *color);
            } else {
                self.text.draw(label, rect.x, label_y, fonts.label, MUTED);
                self.text.draw(value, rect.x, value_y, value_size, *color);
            }
        }
    }

    /// Draws a panel box with a label and returns the area for its content
    fn draw_box(&self, rect: Rect, label: &str) -> Rect {
        let layout = &self.layout;
        let border = layout.px().max(layout.snap(layout.block * 0.05));
        draw_rectangle(rect.x, rect.y, rect.w, rect.h, FRAME);
        draw_rectangle(
            rect.x + border,
            rect.y + border,
            rect.w - 2.0 * border,
            rect.h - 2.0 * border,
            PANEL_FILL,
        );
        let label_height = layout.fonts.label * 2.0;
        self.text.draw_centered(
            label,
            rect.center().x,
            rect.y + label_height * 0.55,
            layout.fonts.label,
            MUTED,
        );
        Rect::new(rect.x, rect.y + label_height, rect.w, rect.h - label_height)
    }

    fn draw_pause_button(&self, active: bool) {
        let layout = &self.layout;
        let rect = layout.pause;
        let color = if active {
            MUTED
        } else {
            Color::new(0.3, 0.33, 0.45, 1.0)
        };
        let border = layout.px().max(layout.snap(layout.block * 0.05));
        draw_rectangle(rect.x, rect.y, rect.w, rect.h, FRAME);
        draw_rectangle(
            rect.x + border,
            rect.y + border,
            rect.w - 2.0 * border,
            rect.h - 2.0 * border,
            PANEL_FILL,
        );

        let icon_height = layout.snap(rect.h.min(rect.w) * 0.4);
        let bar = layout.snap(icon_height * 0.28);
        let icon_x = if layout.portrait {
            rect.center().x - 1.5 * bar
        } else {
            rect.x + rect.h * 0.45
        };
        let icon_y = layout.snap(rect.center().y - icon_height / 2.0);
        draw_rectangle(layout.snap(icon_x), icon_y, bar, icon_height, color);
        draw_rectangle(
            layout.snap(icon_x + 2.0 * bar),
            icon_y,
            bar,
            icon_height,
            color,
        );
        if !layout.portrait {
            self.text.draw(
                TEXT.paused,
                icon_x + 4.0 * bar,
                rect.center().y,
                layout.fonts.label,
                color,
            );
        }
    }

    // Screens

    fn dim_board(&self, alpha: f32) {
        let board = self.layout.board;
        draw_rectangle(
            board.x,
            board.y,
            board.w,
            board.h,
            Color::new(0.02, 0.03, 0.07, alpha),
        );
    }

    fn draw_backdrop(&self, backdrop: &Backdrop) {
        let (width, height) = self.layout.size;
        for piece in &backdrop.pieces {
            let size = self.layout.snap(piece.scale * height);
            let color = mix(piece.kind.color(), BACKGROUND_BOTTOM, 0.8);
            let origin = vec2(piece.x * width, piece.y * height);
            for (x, y) in piece.kind.cells(piece.rotation) {
                draw_block(
                    self.layout.snap(origin.x + x as f32 * size),
                    self.layout.snap(origin.y + y as f32 * size),
                    size,
                    color,
                    &self.layout,
                );
            }
        }
    }

    fn draw_start_screen(&self, state: &GameState, touch: bool, time: f64) {
        let layout = &self.layout;
        let board = layout.board;
        let fonts = &layout.fonts;
        let center_x = board.center().x;
        self.dim_board(0.55);

        // Title with one color per letter, gently bouncing
        let title_y = board.y + board.h * 0.24;
        let width = self.text.measure(TEXT.title, fonts.title).width;
        let letter_width = width / TEXT.title.len() as f32;
        let colors = [
            Tetromino::Z,
            Tetromino::L,
            Tetromino::O,
            Tetromino::S,
            Tetromino::I,
            Tetromino::T,
        ];
        for (i, letter) in TEXT.title.chars().enumerate() {
            let bounce = ((time * 2.5 + i as f64 * 0.6).sin() as f32) * layout.block * 0.12;
            let x = center_x - width / 2.0 + (i as f32 + 0.5) * letter_width;
            let color = colors[i % colors.len()].color();
            self.text
                .draw_shadowed(&letter.to_string(), x, title_y + bounce, fonts.title, color);
        }

        let record = format!("{} {}", TEXT.record, format_number(state.high_score));
        self.text.draw_shadowed(
            &record,
            center_x,
            board.y + board.h * 0.36,
            fonts.hint,
            GOLD,
        );

        let help = if touch {
            TEXT.touch_help
        } else {
            TEXT.key_help
        };
        let top = board.y + board.h * 0.48;
        for (i, line) in help.iter().enumerate() {
            let y = top + i as f32 * fonts.help * 2.0;
            self.text
                .draw_shadowed(line, center_x, y, fonts.help, MUTED);
        }

        let hint = if touch {
            TEXT.start_touch
        } else {
            TEXT.start_keys
        };
        self.text.draw_shadowed(
            hint,
            center_x,
            board.y + board.h * 0.84,
            fonts.hint,
            blink(time),
        );
    }

    fn draw_pause_screen(&self, touch: bool, time: f64) {
        let board = self.layout.board;
        let fonts = &self.layout.fonts;
        self.dim_board(0.4);
        self.text.draw_shadowed(
            TEXT.paused,
            board.center().x,
            board.y + board.h * 0.42,
            fonts.heading,
            WHITE,
        );
        let hint = if touch {
            TEXT.resume_touch
        } else {
            TEXT.resume_keys
        };
        self.text.draw_shadowed(
            hint,
            board.center().x,
            board.y + board.h * 0.52,
            fonts.hint,
            blink(time),
        );
    }

    fn draw_game_over(&self, state: &GameState, touch: bool, time: f64) {
        let layout = &self.layout;
        let board = layout.board;
        let fonts = &layout.fonts;
        let center_x = board.center().x;
        self.dim_board(0.85);

        self.text.draw_shadowed(
            TEXT.game_over,
            center_x,
            board.y + board.h * 0.3,
            fonts.heading,
            Tetromino::Z.color(),
        );
        if state.new_record {
            let pulse = 0.75 + 0.25 * (time * 5.0).sin() as f32;
            self.text.draw_shadowed(
                TEXT.new_record,
                center_x,
                board.y + board.h * 0.38,
                fonts.hint,
                Color::new(GOLD.r, GOLD.g, GOLD.b, pulse),
            );
        }
        self.text.draw_shadowed(
            TEXT.score,
            center_x,
            board.y + board.h * 0.47,
            fonts.label,
            MUTED,
        );
        self.text.draw_shadowed(
            &format_number(state.score),
            center_x,
            board.y + board.h * 0.52,
            fonts.heading,
            WHITE,
        );
        let details = format!(
            "{} {}   {} {}",
            TEXT.level, state.level, TEXT.lines, state.lines
        );
        self.text.draw_shadowed(
            &details,
            center_x,
            board.y + board.h * 0.6,
            fonts.help,
            MUTED,
        );

        let hint = if touch {
            TEXT.restart_touch
        } else {
            TEXT.restart_keys
        };
        self.text.draw_shadowed(
            hint,
            center_x,
            board.y + board.h * 0.78,
            fonts.hint,
            blink(time),
        );
    }
}

/// Cells of a piece inside the visible field
fn visible_cells(piece: &Piece) -> impl Iterator<Item = (i32, i32)> {
    piece
        .cells()
        .into_iter()
        .filter(|&(_, y)| y >= BOARD.hidden)
}

/// Block with a light upper left and a dark lower right edge. Opaque and snapped to
/// physical pixels, so it looks the same in the cached board texture and on screen.
fn draw_block(x: f32, y: f32, size: f32, color: Color, layout: &Layout) {
    let bevel = layout.snap(size * 0.14).max(layout.px());
    draw_rectangle(x, y, size, size, mix(color, BLACK, 0.4));
    draw_rectangle(x, y, size - bevel, size - bevel, mix(color, WHITE, 0.4));
    draw_rectangle(
        x + bevel,
        y + bevel,
        size - 2.0 * bevel,
        size - 2.0 * bevel,
        color,
    );
}

/// Draws a piece centered at `center` with the given cell size
fn draw_preview(kind: Tetromino, center: Vec2, cell: f32, layout: &Layout) {
    let cell = layout.snap(cell);
    let cells = kind.cells(Rotation::Zero);
    let min_x = cells.iter().map(|c| c.0).min().unwrap_or(0);
    let max_x = cells.iter().map(|c| c.0).max().unwrap_or(0);
    let min_y = cells.iter().map(|c| c.1).min().unwrap_or(0);
    let max_y = cells.iter().map(|c| c.1).max().unwrap_or(0);
    let width = (max_x - min_x + 1) as f32 * cell;
    let height = (max_y - min_y + 1) as f32 * cell;
    let left = layout.snap(center.x - width / 2.0);
    let top = layout.snap(center.y - height / 2.0);
    for (x, y) in cells {
        draw_block(
            left + (x - min_x) as f32 * cell,
            top + (y - min_y) as f32 * cell,
            cell,
            kind.color(),
            layout,
        );
    }
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

fn blink(time: f64) -> Color {
    let alpha = 0.55 + 0.45 * (time * 3.0).sin().abs() as f32;
    Color::new(1.0, 1.0, 1.0, alpha)
}

/// Rectangle with a vertical color gradient
fn gradient_rect(rect: Rect, top: Color, bottom: Color) {
    let vertex = |x: f32, y: f32, color: Color| Vertex::new(x, y, 0.0, 0.0, 0.0, color);
    let mesh = Mesh {
        vertices: vec![
            vertex(rect.x, rect.y, top),
            vertex(rect.right(), rect.y, top),
            vertex(rect.right(), rect.bottom(), bottom),
            vertex(rect.x, rect.bottom(), bottom),
        ],
        indices: vec![0, 1, 2, 0, 2, 3],
        texture: None,
    };
    draw_mesh(&mesh);
}
