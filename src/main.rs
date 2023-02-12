use std::env;
use std::time::Instant;

#[macro_use]
extern crate lazy_static;

mod farm;
mod game;
mod major_improvements;
mod player;
mod primitives;
mod scoring;
mod setup;
// mod fencing;

fn main() {
    //env::set_var("RUST_BACKTRACE", "1");
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Please enter the number of players.");
        return;
    }

    // First arg is the binary
    let num_players: usize = match &args[1].parse::<usize>() {
        Ok(num) => *num,
        Err(e) => {
            println!("Couldn't parse number of players, please enter a number between 1 and 4 inclusive. {:?}", e);
            return;
        }
    };

    let mut human_player = false;
    if args.len() == 3 {
        // Make the first player Human
        human_player = true;
    }

    let debug = true;
    let mut game = setup::get_init_state(num_players, human_player, debug);
    let start = Instant::now();
    game.play(debug);
    let duration = start.elapsed();
    println!("\nTime elapsed: {:?}", duration);
    println!("Scores {:?}", game.scores());
    println!("Fitness {:?}", game.fitness());
    // fencing::test_fencing(15);
}
