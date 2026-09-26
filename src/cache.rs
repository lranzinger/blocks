use macroquad::{
    prelude::*,
    text::{Font, TextDimensions, load_ttf_font_from_bytes, measure_text},
};

use crate::config::TEXT;

const FONT_BYTES: &[u8] = include_bytes!("../assets/fonts/BlocksPixel-Regular.ttf");

/// The pixel font is drawn on an 8 pixel grid, multiples of 8 keep it crisp
fn pixel_font_size(size: f32) -> f32 {
    ((size / 8.0).round() * 8.0).max(8.0)
}

pub struct FontCache {
    pub font: Font,
    pub size: f32,
    pub button_size: f32,
    pub debug_size: f32,
    pub stats_size: f32,
}

impl FontCache {
    pub fn new() -> Self {
        let mut font = load_ttf_font_from_bytes(FONT_BYTES).expect("Embedded font is valid");
        font.set_filter(FilterMode::Nearest);

        let mut cache = Self {
            font,
            size: 0.0,
            button_size: 0.0,
            debug_size: 0.0,
            stats_size: 0.0,
        };
        cache.update();
        cache
    }

    pub fn update(&mut self) {
        // Limited by the width as well, so the longest texts fit on narrow screens
        let base = (screen_height() * 0.03).min(screen_width() * 0.06);
        self.size = pixel_font_size(base);
        self.button_size = pixel_font_size(base * 0.9);
        self.debug_size = pixel_font_size(base * 0.5);
        self.stats_size = pixel_font_size(base * 0.7);

        // Rasterize all glyphs up front instead of growing the atlas while drawing
        let characters: Vec<char> = (' '..='~').chain("äöüÄÖÜß".chars()).collect();
        for size in [
            self.size,
            self.button_size,
            self.debug_size,
            self.stats_size,
        ] {
            let physical_size = (size * screen_dpi_scale()).ceil() as u16;
            self.font.populate_font_cache(&characters, physical_size);
        }
    }

    pub fn measure(&self, text: &str, size: f32) -> TextDimensions {
        measure_text(text, Some(&self.font), size as u16, 1.0)
    }
}

/// A number as text, only formatted and measured again when it changes
#[derive(Default)]
pub struct NumberText {
    value: Option<u32>,
    pub text: String,
    pub width: f32,
}

impl NumberText {
    pub fn set(&mut self, value: u32, font: &FontCache, size: f32) {
        if self.value != Some(value) {
            self.value = Some(value);
            self.text = value.to_string();
            self.width = font.measure(&self.text, size).width;
        }
    }

    fn invalidate(&mut self) {
        self.value = None;
    }
}

pub struct TextCache {
    pub score_label_dims: TextDimensions,
    pub level_label_dims: TextDimensions,
    pub score: NumberText,
    pub level: NumberText,
}

impl TextCache {
    pub fn new(font: &FontCache) -> Self {
        let mut cache = Self {
            score_label_dims: TextDimensions::default(),
            level_label_dims: TextDimensions::default(),
            score: NumberText::default(),
            level: NumberText::default(),
        };
        cache.update(font);
        cache
    }

    pub fn update(&mut self, font: &FontCache) {
        self.score_label_dims = font.measure(TEXT.score, font.stats_size);
        self.level_label_dims = font.measure(TEXT.level, font.stats_size);
        self.score.invalidate();
        self.level.invalidate();
    }
}
