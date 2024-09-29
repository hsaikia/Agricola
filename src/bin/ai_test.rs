use indicatif::ProgressBar;
use std::{env, time::Instant};

use agricola_game::agricola::{
    actions::Action,
    algorithms::{PlayerType, AI},
    state::State,
};

fn main() {
    const NUM_GAMES_TO_SIMULATE: usize = 100;
    const OPT_DEPTH: Option<usize> = None;
    env::set_var("RUN_BACKTRACE", "1");
    let start = Instant::now();
    let opt_state = State::new(&[PlayerType::MCTSMachine, PlayerType::MCTSMachine]);
    let mut ai_agent = AI::new();
    let mut state = opt_state.unwrap();

    loop {
        let actions = Action::next_choices(&state);
        if actions.is_empty() {
            println!("GAME OVER");
            break;
        }

        if actions.len() == 1 {
            actions[0].apply_choice(&mut state);
            println!("Auto-choosing single action [{:?}]", actions[0]);
            continue;
        }

        ai_agent.init_records(&actions, &state);
        let bar = ProgressBar::new(NUM_GAMES_TO_SIMULATE as u64);
        for _ in 0..NUM_GAMES_TO_SIMULATE {
            bar.inc(1);
            ai_agent.sample_once(&state, OPT_DEPTH, false);
        }
        bar.finish();

        println!("Scores {:?}", state.scores());
        let records = ai_agent.sorted_records();
        records[0].action.apply_choice(&mut state);
        println!(
            "Player {} chose action [{:?}]",
            state.current_player_idx, records[0].action
        );
    }
    let duration = start.elapsed();
    println!(
        "Time taken in a {} player MCTS AI game (Simulated Games {}, Depth {:?}): {:?}",
        state.num_players, NUM_GAMES_TO_SIMULATE, OPT_DEPTH, duration
    );
}
