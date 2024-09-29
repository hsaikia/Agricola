use std::env;

use agricola_game::agricola::{actions::Action, algorithms::PlayerType, state::State};
use rand::Rng;

#[derive(Debug, Eq, PartialEq, Hash)]
enum Pattern {
    Build,
    Grow,
}

impl Pattern {
    fn matching(&self, state: &State, action: &Action) -> bool {
        match self {
            Self::Build => {
                if state.can_build_room() {
                    matches!(action, Action::BuildRoom(_))
                } else {
                    false
                }
            }
            Self::Grow => {
                if state.can_grow_family_with_room() || state.can_grow_family_without_room() {
                    matches!(action, Action::GrowFamily(_))
                } else {
                    false
                }
            }
        }
    }
}

#[derive(Debug)]
struct Statistics {
    pattern: Pattern,
    appeared: Vec<usize>,
    average_fitness: Vec<f64>,
}

fn empty_stats(num_players: usize) -> Vec<Statistics> {
    vec![
        Statistics {
            pattern: Pattern::Build,
            appeared: vec![0; num_players],
            average_fitness: vec![0.0; num_players],
        },
        Statistics {
            pattern: Pattern::Grow,
            appeared: vec![0; num_players],
            average_fitness: vec![0.0; num_players],
        },
    ]
}

#[allow(clippy::cast_precision_loss)]
fn merge_stats(overall: &mut [Statistics], new_stat: &[Statistics]) {
    for (a, b) in overall.iter_mut().zip(new_stat.iter()) {
        for idx in 0..a.appeared.len() {
            a.average_fitness[idx] = a.average_fitness[idx] * a.appeared[idx] as f64
                + b.average_fitness[idx] * b.appeared[idx] as f64;
            a.appeared[idx] += b.appeared[idx];
            if a.appeared[idx] > 0 {
                a.average_fitness[idx] /= a.appeared[idx] as f64;
            }
        }
    }
}

fn add_to_stats(statistics: &mut Vec<Statistics>, state: &State, action: &Action) {
    for stat in statistics {
        if stat.pattern.matching(state, action) {
            stat.appeared[state.current_player_idx] += 1;
        }
    }
}

fn sim_one_game(players: &[PlayerType]) -> Vec<Statistics> {
    let opt_state = State::new(players);
    let mut state = opt_state.unwrap();
    let mut statistics = empty_stats(players.len());

    loop {
        let choices = Action::next_choices(&state);
        if choices.is_empty() {
            break;
        }

        let action_idx = if choices.len() == 1 {
            // Only one choice, play it
            0
        } else {
            // Chose a random action
            rand::thread_rng().gen_range(0..choices.len())
        };

        choices[action_idx].apply_choice(&mut state);
        add_to_stats(&mut statistics, &state, &choices[action_idx]);
    }

    let fitness = state.fitness();

    for stat in &mut statistics {
        stat.average_fitness.clone_from(&fitness);
    }

    statistics
}

fn main() {
    const NUM_SIMS: usize = 100;
    env::set_var("RUN_BACKTRACE", "1");
    let players = vec![PlayerType::MCTSMachine, PlayerType::MCTSMachine];

    let mut statistics = empty_stats(players.len());

    for i in 0..NUM_SIMS {
        println!("#{i} {statistics:?}");
        let stat = sim_one_game(&players);
        merge_stats(&mut statistics, &stat);
    }

    println!("{statistics:?}");
}
