use macroquad::{
    color::{Color, WHITE},
    math::{Vec2, vec2},
    rand::gen_range,
};

use crate::{
    config::{BOARD, TEXT},
    logic::{Board, Event},
    tetromino::{Rotation, Tetromino},
};

/// Positions are in board cells of the visible field: x from the left, y from the top.
pub struct Particle {
    pub position: Vec2,
    velocity: Vec2,
    pub color: Color,
    pub size: f32,
    age: f32,
    lifetime: f32,
}

impl Particle {
    /// Remaining life from 1 to 0
    pub fn life(&self) -> f32 {
        1.0 - self.age / self.lifetime
    }
}

pub struct Popup {
    pub lines: Vec<(String, Color)>,
    pub age: f32,
    pub duration: f32,
}

pub struct Trail {
    /// Column and the topmost row of the landed piece in it
    pub columns: Vec<(i32, f32)>,
    /// Rows the piece fell
    pub rows: f32,
    pub color: Color,
    pub age: f32,
}

pub const TRAIL_DURATION: f32 = 0.25;
pub const FLASH_DURATION: f32 = 0.18;

pub struct Flash {
    pub cells: [(i32, i32); 4],
    pub age: f32,
}

pub struct Effects {
    pub particles: Vec<Particle>,
    pub popups: Vec<Popup>,
    pub trails: Vec<Trail>,
    pub flashes: Vec<Flash>,
    shake: f32,
}

const GRAVITY: f32 = 30.0;
const SHAKE_DURATION: f32 = 0.3;

fn visible_row(y: i32) -> f32 {
    (y - BOARD.hidden) as f32
}

impl Effects {
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
            popups: Vec::new(),
            trails: Vec::new(),
            flashes: Vec::new(),
            shake: 0.0,
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn handle(&mut self, event: &Event, board: &Board) {
        match event {
            Event::Locked { cells } => self.flashes.push(Flash {
                cells: *cells,
                age: 0.0,
            }),
            Event::HardDrop { kind, cells, rows } => {
                let mut columns: Vec<(i32, f32)> = Vec::new();
                for &(x, y) in cells {
                    let top = visible_row(y);
                    match columns.iter_mut().find(|column| column.0 == x) {
                        Some(column) => column.1 = column.1.min(top),
                        None => columns.push((x, top)),
                    }
                }
                self.trails.push(Trail {
                    columns,
                    rows: *rows as f32,
                    color: kind.color(),
                    age: 0.0,
                });
            }
            Event::LinesCleared {
                rows,
                points,
                back_to_back,
                combo,
            } => {
                let count = rows.len();
                for (y, row) in board.iter().enumerate() {
                    if !rows.contains(y) {
                        continue;
                    }
                    for (x, cell) in row.iter().enumerate() {
                        let color = cell.map_or(WHITE, Tetromino::color);
                        self.burst(x as f32 + 0.5, visible_row(y as i32) + 0.5, color, count);
                    }
                }

                let mut lines = Vec::new();
                let label = TEXT.clears[count as usize - 1];
                if !label.is_empty() {
                    lines.push((label.to_string(), clear_color(count)));
                }
                if *back_to_back {
                    lines.push((TEXT.back_to_back.to_string(), GOLD));
                }
                if *combo > 0 {
                    lines.push((format!("{} x{}", TEXT.combo, combo + 1), MINT));
                }
                lines.push((format!("+{}", format_number(*points)), WHITE));
                self.popups.push(Popup {
                    lines,
                    age: 0.0,
                    duration: 1.1,
                });

                if count == 4 {
                    self.shake = SHAKE_DURATION;
                }
            }
            Event::LevelUp(level) => self.popups.push(Popup {
                lines: vec![(format!("{} {}", TEXT.level_up, level), GOLD)],
                age: -0.5,
                duration: 1.3,
            }),
            Event::GameOver { .. } => {}
        }
    }

    fn burst(&mut self, x: f32, y: f32, color: Color, lines: u32) {
        let count = 1 + lines as usize;
        for _ in 0..count {
            let lifetime = gen_range(0.45, 0.9);
            self.particles.push(Particle {
                position: vec2(x + gen_range(-0.3, 0.3), y + gen_range(-0.3, 0.3)),
                velocity: vec2(gen_range(-5.0, 5.0), gen_range(-9.0, -2.0)),
                color,
                size: gen_range(0.12, 0.28),
                age: 0.0,
                lifetime,
            });
        }
    }

    pub fn update(&mut self, delta: f32) {
        for particle in &mut self.particles {
            particle.age += delta;
            particle.velocity.y += GRAVITY * delta;
            particle.position += particle.velocity * delta;
        }
        self.particles
            .retain(|particle| particle.age < particle.lifetime);

        for popup in &mut self.popups {
            popup.age += delta;
        }
        self.popups.retain(|popup| popup.age < popup.duration);

        for trail in &mut self.trails {
            trail.age += delta;
        }
        self.trails.retain(|trail| trail.age < TRAIL_DURATION);

        for flash in &mut self.flashes {
            flash.age += delta;
        }
        self.flashes.retain(|flash| flash.age < FLASH_DURATION);

        self.shake = (self.shake - delta).max(0.0);
    }

    /// Offset of the board in cells for the screen shake
    pub fn shake_offset(&self) -> Vec2 {
        if self.shake <= 0.0 {
            return Vec2::ZERO;
        }
        let strength = 0.18 * self.shake / SHAKE_DURATION;
        vec2(
            gen_range(-strength, strength),
            gen_range(-strength, strength),
        )
    }
}

pub const GOLD: Color = Color::new(1.0, 0.82, 0.3, 1.0);
pub const MINT: Color = Color::new(0.45, 0.95, 0.75, 1.0);

fn clear_color(lines: u32) -> Color {
    match lines {
        4 => Tetromino::I.color(),
        3 => Tetromino::T.color(),
        _ => Tetromino::S.color(),
    }
}

/// Formats a number with dots as thousands separators, like 12.345
pub fn format_number(value: u32) -> String {
    let digits = value.to_string();
    let mut result = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, digit) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            result.push('.');
        }
        result.push(digit);
    }
    result
}

/// Pieces slowly falling behind the start screen
pub struct Backdrop {
    pub pieces: Vec<BackdropPiece>,
}

pub struct BackdropPiece {
    pub kind: Tetromino,
    pub rotation: Rotation,
    /// Position relative to the screen size, 0 to 1
    pub x: f32,
    pub y: f32,
    speed: f32,
    /// Size relative to the screen height
    pub scale: f32,
}

impl Backdrop {
    pub fn new() -> Self {
        let pieces = (0..14)
            .map(|_| {
                let mut piece = BackdropPiece::random();
                piece.y = gen_range(-0.2, 1.0);
                piece
            })
            .collect();
        Self { pieces }
    }

    pub fn update(&mut self, delta: f32) {
        for piece in &mut self.pieces {
            piece.y += piece.speed * delta;
            if piece.y > 1.1 {
                *piece = BackdropPiece::random();
            }
        }
    }
}

impl BackdropPiece {
    fn random() -> Self {
        let rotations = [
            Rotation::Zero,
            Rotation::Right,
            Rotation::Two,
            Rotation::Left,
        ];
        Self {
            kind: Tetromino::random(),
            rotation: rotations[gen_range(0, 4)],
            x: gen_range(0.0, 1.0),
            y: gen_range(-0.35, -0.15),
            speed: gen_range(0.03, 0.08),
            scale: gen_range(0.025, 0.045),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::format_number;

    #[test]
    fn numbers_get_thousands_separators() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(999), "999");
        assert_eq!(format_number(1000), "1.000");
        assert_eq!(format_number(1234567), "1.234.567");
    }
}
