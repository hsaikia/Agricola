use std::time::{Instant};
use std::env;

#[macro_use]
extern crate lazy_static;

mod farm;
mod game;
mod major_improvements;
mod player;
mod primitives;
mod setup;
mod scoring;
// mod fencing;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    let num_players: usize = 4;
    let debug = true;
    let mut game = setup::get_init_state(num_players, debug);
    let start = Instant::now();
    game.play(debug);
    let duration = start.elapsed();
    println!("\nTime elapsed: {:?}", duration);
    println!("Scores {:?}", game.scores());
    println!("Fitness {:?}", game.fitness());
    // fencing::test_fencing(15);
}
