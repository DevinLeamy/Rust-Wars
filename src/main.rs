use game::Game;

pub mod game;
pub mod animations;
pub mod config;
pub mod player;
pub mod utils;
pub mod input;


fn main() {
    let mut game = Game::new();
    game.run();
}
