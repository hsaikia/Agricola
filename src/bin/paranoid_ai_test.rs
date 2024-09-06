use std::env;

use agricola_game::agricola::{algorithms::PlayerType, paranoid_ai::best_move, state::State};

fn main() {
    env::set_var("RUN_BACKTRACE", "1");
    let opt_state = State::new(&[PlayerType::MCTSMachine, PlayerType::MCTSMachine]);
    let mut state = opt_state.unwrap();

    loop {
        let opt_action = best_move(&state);
        if let Some(action) = opt_action {
            action.apply_choice(&mut state);
        } else {
            println!("GAME OVER");
            break;
        }
    }

    println!("Scores {:?}", state.scores());
}
