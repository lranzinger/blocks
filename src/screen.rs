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

        // Calculate optimal block size
        let scale_x = screen_width / BOARD.width as f32;
        let scale_y = screen_height / BOARD.height as f32;
        let block_size: f32 = scale_x.min(scale_y) * 0.95; // 95% of available space

        // Center the game field
        let offset_x = (screen_width - (BOARD.width as f32 * block_size)) / 2.0;
        let offset_y = (screen_height - (BOARD.height as f32 * block_size)) / 2.0;

        let field_width = BOARD.width as f32 * block_size;
        let field_height = BOARD.height as f32 * block_size;

        Self {
            block_size,
            offset_x,
            offset_y,
            field_width,
            field_height,
            size: (screen_width, screen_height),
            dpi_scale: screen_dpi_scale(),
        }
    }
}
