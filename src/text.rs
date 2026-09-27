use std::collections::HashSet;

use macroquad::{
    prelude::*,
    text::{Font, TextDimensions, load_ttf_font_from_bytes, measure_text},
};

const FONT_BYTES: &[u8] = include_bytes!("../assets/fonts/BlocksPixel-Regular.ttf");
/// Size for measuring text widths. The pixel font scales linearly, a small size keeps the
/// glyphs rasterized for measuring small.
const REFERENCE_SIZE: f32 = 8.0;

/// Characters of numbers and the "H" used for vertical centering, needed in every size
const COMMON_CHARACTERS: &str = "0123456789.+x !H";

pub struct Text {
    font: Font,
    /// Characters already rasterized, per physical font size
    prepared: HashSet<(u16, char)>,
}

impl Text {
    pub fn new() -> Self {
        let mut font = load_ttf_font_from_bytes(FONT_BYTES).expect("Embedded font is valid");
        font.set_filter(FilterMode::Nearest);
        Self {
            font,
            prepared: HashSet::new(),
        }
    }

    /// The pixel font is drawn on an 8 pixel grid. Multiples of 8 physical pixels keep it
    /// crisp, small sizes use multiples of 4 to stay readable. Rounds down to whole logical
    /// pixels, as macroquad takes font sizes as integers: fractional sizes would be drawn
    /// in a different size than the prepared glyphs.
    pub fn pixel_size(size: f32) -> f32 {
        let dpi = screen_dpi_scale();
        let physical = size * dpi;
        let grid = if physical >= 32.0 { 8.0 } else { 4.0 };
        let physical = ((physical / grid).floor() * grid).max(8.0);
        (physical / dpi).floor().max((8.0 / dpi).ceil())
    }

    /// Largest pixel size up to `preferred` at which all texts fit into `max_width`
    pub fn fit(&self, texts: &[&str], preferred: f32, max_width: f32) -> f32 {
        let widest = texts
            .iter()
            .map(|text| self.measure(text, REFERENCE_SIZE).width)
            .fold(1.0, f32::max);
        Self::pixel_size(preferred.min(REFERENCE_SIZE * max_width / widest))
    }

    /// Rasterizes the glyphs of the given texts up front, so the glyph atlas does not grow
    /// while drawing. Only the characters used in each size, a large atlas is slow to upload
    /// and takes a lot of memory on high resolution screens.
    pub fn prepare(&mut self, size: f32, texts: &[&str]) {
        // Same rounding as macroquad when drawing
        let physical = (size as u16 as f32 * screen_dpi_scale()).ceil() as u16;
        let characters: Vec<char> = texts
            .iter()
            .flat_map(|text| text.chars())
            .chain(COMMON_CHARACTERS.chars())
            .filter(|character| self.prepared.insert((physical, *character)))
            .collect();
        if !characters.is_empty() {
            self.font.populate_font_cache(&characters, physical);
        }
    }

    pub fn measure(&self, text: &str, size: f32) -> TextDimensions {
        measure_text(text, Some(&self.font), size as u16, 1.0)
    }

    /// Draws text with its left edge at `x` and its vertical center at `center_y`
    pub fn draw(&self, text: &str, x: f32, center_y: f32, size: f32, color: Color) {
        let cap_height = self.measure("H", size).offset_y;
        let px = 1.0 / screen_dpi_scale();
        // Whole physical pixel positions keep the pixel font crisp
        let snap = |value: f32| (value / px).round() * px;
        draw_text_ex(
            text,
            snap(x),
            snap(center_y + cap_height / 2.0),
            TextParams {
                font: Some(&self.font),
                font_size: size as u16,
                color,
                ..Default::default()
            },
        );
    }

    pub fn draw_centered(&self, text: &str, center_x: f32, center_y: f32, size: f32, color: Color) {
        let width = self.measure(text, size).width;
        self.draw(text, center_x - width / 2.0, center_y, size, color);
    }

    /// Centered text with a dark shadow for readability on busy backgrounds
    pub fn draw_shadowed(&self, text: &str, center_x: f32, center_y: f32, size: f32, color: Color) {
        let offset = (size / 8.0).max(1.0);
        let shadow = Color::new(0.0, 0.0, 0.0, 0.6 * color.a);
        self.draw_centered(text, center_x + offset, center_y + offset, size, shadow);
        self.draw_centered(text, center_x, center_y, size, color);
    }

    pub fn draw_right(&self, text: &str, right_x: f32, center_y: f32, size: f32, color: Color) {
        let width = self.measure(text, size).width;
        self.draw(text, right_x - width, center_y, size, color);
    }
}
