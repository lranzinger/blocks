use macroquad::prelude::*;

use crate::config::BOARD;

pub struct ScreenConfig {
    pub block_size: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub field_width: f32,
    pub field_height: f32,
    pub size: (f32, f32),
    pub dpi_scale: f32,
}

impl ScreenConfig {
    pub fn new() -> Self {
        let screen_width = screen_width();
        let screen_height = screen_height();
        let dpi_scale = screen_dpi_scale();

        // Calculate optimal block size, 95% of available space.
        // Sizes and positions are snapped to whole physical pixels, so the cached board and
        // the falling piece rasterize identically, also with fractional scaling like 150%.
        let scale_x = screen_width / BOARD.width as f32;
        let scale_y = screen_height / BOARD.height as f32;
        let block_size = (scale_x.min(scale_y) * 0.95 * dpi_scale).floor().max(1.0) / dpi_scale;

        let field_width = BOARD.width as f32 * block_size;
        let field_height = BOARD.height as f32 * block_size;

        // Center the game field
        let offset_x = ((screen_width - field_width) / 2.0 * dpi_scale).round() / dpi_scale;
        let offset_y = ((screen_height - field_height) / 2.0 * dpi_scale).round() / dpi_scale;

        Self {
            block_size,
            offset_x,
            offset_y,
            field_width,
            field_height,
            size: (screen_width, screen_height),
            dpi_scale,
        }
    }

    /// Rounds a length to whole physical pixels
    pub fn snap(&self, length: f32) -> f32 {
        (length * self.dpi_scale).round() / self.dpi_scale
    }
}
