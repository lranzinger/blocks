mod cache;
mod config;
mod dummy_board;
mod game;
mod input;
mod logic;
mod renderer;
mod screen;
mod state;
mod storage;
mod tetromino;

use config::TEXT;
use game::Game;
use macroquad::prelude::*;
use miniquad::date;

fn window_conf() -> Conf {
    Conf {
        window_title: TEXT.game_name.to_string(),
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
        game.update();
        game.renderer.draw(&game.state);
        next_frame().await;
    }
}
