use macroquad::prelude::*;

use crate::{
    config::{BOARD, TEXT},
    text::Text,
};

pub struct FontSizes {
    pub title: f32,
    pub heading: f32,
    pub value: f32,
    pub popup: f32,
    pub hint: f32,
    pub help: f32,
    pub label: f32,
}

impl FontSizes {
    fn all(&self) -> [f32; 7] {
        [
            self.title,
            self.heading,
            self.value,
            self.popup,
            self.hint,
            self.help,
            self.label,
        ]
    }
}

/// Stats shown next to the board, each with its area
pub struct StatSlots {
    pub score: Rect,
    pub record: Rect,
    pub level: Rect,
    pub lines: Rect,
}

/// Positions and sizes of everything on screen in logical pixels,
/// snapped to whole physical pixels
pub struct Layout {
    pub size: (f32, f32),
    pub dpi_scale: f32,
    pub portrait: bool,
    pub block: f32,
    /// Visible part of the board
    pub board: Rect,
    pub hold: Rect,
    pub next: Rect,
    pub stats: StatSlots,
    pub pause: Rect,
    pub fonts: FontSizes,
}

// Landscape: board with a panel on each side
const PANEL_WIDTH: f32 = 5.0;
const PANEL_GAP: f32 = 0.8;
const LANDSCAPE_WIDTH: f32 = BOARD.width as f32 + 2.0 * (PANEL_WIDTH + PANEL_GAP) + 1.0;
const LANDSCAPE_HEIGHT: f32 = BOARD.height as f32 + 1.2;

// Portrait: board with two rows of information above
const TOP_ROW: f32 = 1.3;
const BOX_ROW: f32 = 2.4;
const ROW_GAP: f32 = 0.25;
const PORTRAIT_HEIGHT: f32 = BOARD.height as f32 + TOP_ROW + BOX_ROW + 2.0 * ROW_GAP + 0.8;

impl Layout {
    pub fn new(text: &mut Text) -> Self {
        let (width, height) = (screen_width(), screen_height());
        let dpi_scale = screen_dpi_scale();
        let px = 1.0 / dpi_scale;
        let snap = |value: f32| (value / px).round() * px;

        let landscape_block = (width / LANDSCAPE_WIDTH).min(height / LANDSCAPE_HEIGHT);
        let portrait_block = (width * 0.96 / BOARD.width as f32).min(height / PORTRAIT_HEIGHT);
        let portrait = portrait_block > landscape_block;

        // Whole physical pixels per block, so cached and live blocks rasterize identically
        let raw_block = if portrait {
            portrait_block
        } else {
            landscape_block
        };
        let block = ((raw_block / px).floor() * px).max(px);
        let board_width = BOARD.width as f32 * block;
        let board_height = BOARD.height as f32 * block;

        let (board, hold, next, stats, pause);
        if portrait {
            let content_height = board_height + (TOP_ROW + BOX_ROW + 2.0 * ROW_GAP) * block;
            let top = snap((height - content_height) / 2.0);
            let left = snap((width - board_width) / 2.0);
            let row =
                |y: f32, h: f32| Rect::new(left, snap(top + y * block), board_width, h * block);

            let top_row = row(0.0, TOP_ROW);
            let box_row = row(TOP_ROW + ROW_GAP, BOX_ROW);
            board = Rect::new(
                left,
                snap(top + (TOP_ROW + BOX_ROW + 2.0 * ROW_GAP) * block),
                board_width,
                board_height,
            );

            pause = Rect::new(
                left + board_width - TOP_ROW * block,
                top_row.y,
                TOP_ROW * block,
                TOP_ROW * block,
            );
            let half = (board_width - pause.w - 0.3 * block) / 2.0;
            let hold_width = 2.8 * block;
            let next_width = 4.6 * block;
            hold = Rect::new(left, box_row.y, hold_width, box_row.h);
            next = Rect::new(
                left + board_width - next_width,
                box_row.y,
                next_width,
                box_row.h,
            );
            let middle_x = hold.right() + 0.2 * block;
            let middle_width = next.x - 0.2 * block - middle_x;
            stats = StatSlots {
                score: Rect::new(left, top_row.y, half, top_row.h),
                record: Rect::new(left + half, top_row.y, half, top_row.h),
                level: Rect::new(middle_x, box_row.y, middle_width, box_row.h / 2.0),
                lines: Rect::new(
                    middle_x,
                    box_row.y + box_row.h / 2.0,
                    middle_width,
                    box_row.h / 2.0,
                ),
            };
        } else {
            let left = snap((width - board_width) / 2.0);
            let top = snap((height - board_height) / 2.0);
            board = Rect::new(left, top, board_width, board_height);

            let panel_width = PANEL_WIDTH * block;
            let left_panel = board.x - (PANEL_GAP + PANEL_WIDTH) * block;
            let right_panel = board.right() + PANEL_GAP * block;
            hold = Rect::new(left_panel, top, panel_width, 4.0 * block);
            next = Rect::new(right_panel, top, panel_width, 10.5 * block);
            pause = Rect::new(
                right_panel,
                next.bottom() + 0.6 * block,
                panel_width,
                1.4 * block,
            );
            let slot = |index: f32| {
                Rect::new(
                    left_panel,
                    hold.bottom() + (0.8 + index * 1.7) * block,
                    panel_width,
                    1.5 * block,
                )
            };
            stats = StatSlots {
                score: slot(0.0),
                record: slot(1.0),
                level: slot(2.0),
                lines: slot(3.0),
            };
        }

        let overlay_width = board_width * 0.9;
        let score_width = if portrait {
            stats.score.w * 0.95
        } else {
            panel_width_for(block) * 0.95
        };
        let fonts = FontSizes {
            title: text.fit(&[TEXT.title], 1.6 * block, overlay_width),
            heading: text.fit(
                &[TEXT.game_over, TEXT.new_record, TEXT.paused],
                0.8 * block,
                overlay_width,
            ),
            value: text.fit(&["9.999.999"], 0.6 * block, score_width),
            popup: text.fit(
                &[TEXT.back_to_back, TEXT.clears[3], "COMBO x10", "+99.999"],
                0.6 * block,
                overlay_width,
            ),
            hint: text.fit(
                &[
                    TEXT.start_touch,
                    TEXT.start_keys,
                    TEXT.resume_touch,
                    TEXT.resume_keys,
                    TEXT.restart_touch,
                    TEXT.restart_keys,
                ],
                0.45 * block,
                overlay_width,
            ),
            help: text.fit(
                &TEXT
                    .touch_help
                    .iter()
                    .chain(&TEXT.key_help)
                    .copied()
                    .collect::<Vec<_>>(),
                0.4 * block,
                overlay_width,
            ),
            label: text.fit(
                &[
                    TEXT.score,
                    TEXT.record,
                    TEXT.level,
                    TEXT.lines,
                    TEXT.hold,
                    TEXT.next,
                ],
                0.35 * block,
                hold.w * 0.8,
            ),
        };
        text.prepare(&fonts.all());

        Self {
            size: (width, height),
            dpi_scale,
            portrait,
            block,
            board,
            hold,
            next,
            stats,
            pause,
            fonts,
        }
    }

    /// The screen size or scaling changed since the layout was made
    pub fn is_outdated(&self) -> bool {
        self.size != (screen_width(), screen_height()) || self.dpi_scale != screen_dpi_scale()
    }

    /// Length of one physical pixel in logical pixels
    pub fn px(&self) -> f32 {
        1.0 / self.dpi_scale
    }

    pub fn snap(&self, value: f32) -> f32 {
        (value * self.dpi_scale).round() / self.dpi_scale
    }
}

fn panel_width_for(block: f32) -> f32 {
    PANEL_WIDTH * block
}
