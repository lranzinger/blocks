mod config;
mod effects;
mod game;
mod input;
mod layout;
mod logic;
mod platform;
mod renderer;
mod tetromino;
mod text;

use config::TEXT;
use game::Game;
use macroquad::prelude::*;
use miniquad::date;

fn window_conf() -> Conf {
    Conf {
        window_title: TEXT.title.to_string(),
        high_dpi: true,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Set seed for random generator
    rand::srand(date::now() as u64);

    let mut game = Game::new();
    loop {
        game.frame();
        next_frame().await;
    }
}
