use macroquad::{
    prelude::*,
    text::{Font, TextDimensions, load_ttf_font_from_bytes, measure_text},
};

use crate::{config::TEXT, screen::ScreenConfig};

const FONT_BYTES: &[u8] = include_bytes!("../assets/fonts/BlocksPixel-Regular.ttf");

/// The pixel font is drawn on an 8 pixel grid. Multiples of 8 physical pixels keep it crisp,
/// small sizes use multiples of 4 to stay readable.
/// Rounds down, so the text never gets larger than the given size.
fn pixel_font_size(size: f32, dpi_scale: f32) -> f32 {
    let physical = size * dpi_scale;
    let grid = if physical >= 32.0 { 8.0 } else { 4.0 };
    ((physical / grid).floor() * grid).max(8.0) / dpi_scale
}

/// Share of the game field width that texts on the overlay screens may use
const MAX_TEXT_WIDTH: f32 = 0.9;

pub struct FontCache {
    pub font: Font,
    pub size: f32,
    pub button_size: f32,
    pub debug_size: f32,
    pub stats_size: f32,
}

impl FontCache {
    pub fn new(screen: &ScreenConfig) -> Self {
        let mut font = load_ttf_font_from_bytes(FONT_BYTES).expect("Embedded font is valid");
        font.set_filter(FilterMode::Nearest);

        let mut cache = Self {
            font,
            size: 0.0,
            button_size: 0.0,
            debug_size: 0.0,
            stats_size: 0.0,
        };
        cache.update(screen);
        cache
    }

    pub fn update(&mut self, screen: &ScreenConfig) {
        let dpi = screen.dpi_scale;
        let base = screen_height() * 0.03;

        // The longest texts have to fit into the game field
        let size = self.max_size_to_fit(&[TEXT.gameover, TEXT.game_name], screen);
        let button_size = self.max_size_to_fit(&[TEXT.start_button, TEXT.gameover_button], screen);
        let mut stats_texts = TEXT.instructions.to_vec();
        stats_texts.push("Highscore: 9999999");
        let stats_size = self.max_size_to_fit(&stats_texts, screen);

        self.size = pixel_font_size(base.min(size), dpi);
        self.button_size = pixel_font_size((base * 0.9).min(button_size), dpi);
        self.stats_size = pixel_font_size((base * 0.7).min(stats_size), dpi);
        self.debug_size = pixel_font_size(base * 0.5, dpi);

        // Rasterize all glyphs up front instead of growing the atlas while drawing
        let characters: Vec<char> = (' '..='~').chain("äöüÄÖÜß".chars()).collect();
        for size in [
            self.size,
            self.button_size,
            self.debug_size,
            self.stats_size,
        ] {
            let physical_size = (size * dpi).ceil() as u16;
            self.font.populate_font_cache(&characters, physical_size);
        }
    }

    /// Largest font size at which all texts fit into the allowed width of the game field
    fn max_size_to_fit(&self, texts: &[&str], screen: &ScreenConfig) -> f32 {
        const REFERENCE_SIZE: f32 = 32.0;
        let widest = texts
            .iter()
            .map(|text| self.measure(text, REFERENCE_SIZE).width)
            .fold(1.0, f32::max);
        REFERENCE_SIZE * screen.field_width * MAX_TEXT_WIDTH / widest
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
