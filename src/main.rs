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
    // env::set_var("RUST_BACKTRACE", "1");
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Please enter the number of players in [1, 4] and the default AI id in [0, 2]. ");
        return;
    }

    // First arg is the binary
    // Second arg is the number of players
    let num_players: usize = match &args[1].parse::<usize>() {
        Ok(num) => *num,
        Err(e) => {
            println!("Couldn't parse number of players, please enter a number between 1 and 4 inclusive. {e:?}");
            return;
        }
    };

    // Third arg is the kind of machine player to be used as default
    let default_ai_id: usize = match &args[2].parse::<usize>() {
        Ok(num) => {
            if *num > 2 {
                println!("Use a number in [0, 1, 2] to signify the default AI. 0 -> Random, 1 -> Uniform, 2 -> MCTS");
                return;
            }
            *num
        }
        Err(_e) => {
            println!("Couldn't parse default AI id. Use a number in [0, 1, 2] to signify the default AI. 0 -> Random, 1 -> Uniform, 2 -> MCTS");
            return;
        }
    };

    let mut human_player = false;
    if args.len() == 4 {
        // Make the first player Human
        human_player = true;
    }

    let debug = true;
    let mut game = setup::get_init_state(num_players, human_player, default_ai_id, debug);
    let start = Instant::now();
    game.play(debug);
    let duration = start.elapsed();
    println!("\nTime elapsed: {duration:?}");
    println!("Scores {:?}", game.scores());
    println!("Fitness {:?}", game.fitness());
    // fencing::test_fencing(15);
}
