use std::env;

use agricola_game::agricola::{state::State, algorithms::{PlayerType, AI}, actions::Action};

fn main() {
    env::set_var("RUN_BACKTRACE", "1");
    let opt_state = State::new(&vec![PlayerType::MCTSMachine]);
    let mut ai_agent = AI::new();
    let mut state = opt_state.unwrap();
    const NUM_GAMES_TO_SIMULATE : usize = 100;

    loop {
        let actions = Action::next_choices(&state);
        if actions.is_empty() {
            println!("GAME OVER");
            break;
        }

        if actions.len() == 1 {
            actions[0].apply_choice(&mut state);
            println!("{:?}", actions[0]);
            continue;
        }

        ai_agent.init_records(&actions, &state);
        for _ in 0..NUM_GAMES_TO_SIMULATE {
            ai_agent.sample_once(&state, false);
        }
    
        let records = ai_agent.sorted_records();
        records[0].action.apply_choice(&mut state);
        println!("{:?}", records[0].action);
    }
}