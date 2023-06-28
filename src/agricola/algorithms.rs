use crate::agricola::actions::Action;
use crate::agricola::mcts::GameRecord;
use crate::agricola::state::State;
use rand::Rng;
use std::collections::HashMap;
use std::io;

const NUM_GAMES_TO_SIMULATE: usize = 100000;

#[derive(Debug, Clone, Hash, PartialEq)]
pub enum Kind {
    Human,
    RandomMachine,
    UniformMachine,
    MCTSMachine,
}

struct SimulationRecord {
    games: u32,
    fitness: f32,
    action_idx: usize,
}

impl Kind {
    fn trivial_play(state: &mut State, debug: bool) -> (Option<bool>, Vec<Action>) {
        let choices = Action::next_choices(state);

        // No choices - End Game
        if choices.is_empty() {
            return (Some(false), choices);
        }

        // Only one choice, play it
        if choices.len() == 1 {
            if debug {
                choices[0].display();
            }
            choices[0].apply_choice(state);
            return (Some(true), choices);
        }

        if debug {
            state.display();
            Action::display_all(&choices);
        }

        // Multiple choices
        (None, choices)
    }

    fn play_human(state: &mut State, debug: bool) -> bool {
        let (maybe_status, choices) = Self::trivial_play(state, debug);

        if let Some(status) = maybe_status {
            return status;
        }

        print!(
            "\nEnter an action index between 0 and {} inclusive : ",
            choices.len() - 1
        );

        let stdin = io::stdin();
        let mut user_input = String::new();
        let _res = stdin.read_line(&mut user_input);

        match user_input.trim().parse::<usize>() {
            Ok(input_int) => {
                println!("Your input [{input_int}]");

                if input_int >= choices.len() {
                    println!("Invalid action .. quitting");
                    return false;
                }

                if debug {
                    choices[input_int].display();
                }
                choices[input_int].apply_choice(state);
            }
            Err(_e) => return Self::play_human(state, debug), // parsing failed - try again
        }

        true
    }

    fn play_random(state: &mut State, debug: bool) -> bool {
        let (maybe_status, choices) = Self::trivial_play(state, debug);

        if let Some(status) = maybe_status {
            return status;
        }

        // Chose a random action
        let action_idx = rand::thread_rng().gen_range(0..choices.len());
        if debug {
            choices[action_idx].display();
        }
        choices[action_idx].apply_choice(state);
        true
    }

    #[allow(clippy::cast_precision_loss)]
    fn play_machine_uniform(state: &mut State, debug: bool) -> bool {
        let (maybe_status, choices) = Self::trivial_play(state, debug);

        if let Some(status) = maybe_status {
            return status;
        }

        // 1. Simulate n games from each action with each player replaced by a random move AI.
        // 2. Compute the average score for each action from the n playouts.
        // 3. Select the move that gives rise to the maximum average score.
        let mut best_action_idx: usize = 0;
        let mut best_average: f32 = f32::NEG_INFINITY;

        let num_games_per_action: usize = NUM_GAMES_TO_SIMULATE / choices.len();

        //println!();
        // Play n simulated games for each action
        for (i, action) in choices.iter().enumerate() {
            let mut sum = 0;
            for _ in 0..num_games_per_action {
                // Clone the current game state
                let mut tmp_game = state.clone();
                // Play the current action
                action.apply_choice(&mut tmp_game);
                // Replace all players with a random move AI
                tmp_game.replace_all_players_with_random_bots();
                // Play the game out until the end
                tmp_game.play(false);
                // Sum resultant values
                sum += tmp_game.fitness()[state.current_player_idx];
            }
            let avg = sum as f32 / num_games_per_action as f32;

            print!(
                "\nAvg score from {num_games_per_action} simulated playouts for Action {action:?} is {avg}."
            );

            if avg > best_average {
                best_average = avg;
                best_action_idx = i;
            }
        }
        if debug {
            choices[best_action_idx].display();
        }
        choices[best_action_idx].apply_choice(state);
        true
    }

    fn play_machine_mcts(state: &mut State, debug: bool) -> bool {
        let (maybe_status, choices) = Self::trivial_play(state, debug);

        if let Some(status) = maybe_status {
            return status;
        }

        let mut action_hashes: Vec<u64> = vec![];
        // Add the child node hashes
        for action in &choices {
            let mut tmp_game = state.clone();
            action.apply_choice(&mut tmp_game);
            action_hashes.push(tmp_game.get_hash());
        }

        // Initialize cache
        let mut mcts_cache: HashMap<u64, GameRecord> = HashMap::new();

        for _g in 0..NUM_GAMES_TO_SIMULATE {
            let sample_idx =
                GameRecord::choose_uct(state.current_player_idx, &action_hashes, &mcts_cache);
            let mut tmp_game = state.clone();
            choices[sample_idx].apply_choice(&mut tmp_game);

            //print!("\nGame {}", g);
            let mut expand_node_hash = tmp_game.get_hash();
            let mut path: Vec<u64> = vec![expand_node_hash];

            // Expand along the tree
            while mcts_cache.contains_key(&expand_node_hash) {
                // In Agricola - since the future board state changes randomly
                // get_all_available_actions can return different actions for the same current game state.
                let sub_choices = Action::next_choices(&tmp_game);

                if sub_choices.is_empty() {
                    break;
                }

                // Generate all child hashes
                let mut sub_action_hashes: Vec<u64> = vec![];
                for sub_action in &sub_choices {
                    let mut tmp_game2 = tmp_game.clone();
                    sub_action.apply_choice(&mut tmp_game2);
                    sub_action_hashes.push(tmp_game2.get_hash());
                }
                let chosen_idx = GameRecord::choose_uct(
                    tmp_game.current_player_idx,
                    &sub_action_hashes,
                    &mcts_cache,
                );
                sub_choices[chosen_idx].apply_choice(&mut tmp_game);
                expand_node_hash = tmp_game.get_hash();
                path.push(tmp_game.get_hash());
            }

            // Add node to cache
            if let std::collections::hash_map::Entry::Vacant(e) = mcts_cache.entry(expand_node_hash)
            {
                e.insert(GameRecord::new(tmp_game.players.len()));
            }

            // Perform playout
            // Replace all players with a random move AI
            tmp_game.replace_all_players_with_random_bots();
            // Play the game out until the end
            tmp_game.play(false);
            // Calculate result and backpropagate to the root
            let res = tmp_game.fitness();
            for node in &path {
                let mut state = mcts_cache[node].clone();
                state.add_fitness(&res);
                mcts_cache.insert(*node, state);
            }
        }

        print!("\nCache has {} entries", mcts_cache.len());

        let mut records: Vec<SimulationRecord> = Vec::new();

        // Choose the best move again according to UCT
        let mut best_action_idx: usize = 0;
        let mut best_fitness: f32 = f32::NEG_INFINITY;
        let mut total_games = 0;
        for (action_idx, action) in choices.iter().enumerate() {
            let mut tmp_game = state.clone();
            action.apply_choice(&mut tmp_game);
            let action_hash = tmp_game.get_hash();

            let games: u32 = mcts_cache[&action_hash].total_games;
            total_games += games;
            let fitness: f32 = mcts_cache[&action_hash].average_fitness[state.current_player_idx];
            if fitness > best_fitness {
                best_fitness = fitness;
                best_action_idx = action_idx;
            }

            records.push(SimulationRecord {
                games,
                fitness,
                action_idx,
            });
        }

        records.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());

        for r in records {
            print!(
                "\nFitness from {} simulated playouts for Action {:?} is {:.2}.",
                r.games, choices[r.action_idx], r.fitness
            );
        }

        print!("\nTotal Games simulated {total_games}.");
        if debug {
            choices[best_action_idx].display();
        }
        choices[best_action_idx].apply_choice(state);
        true
    }

    pub fn play(&self, state: &mut State, debug: bool) -> bool {
        match self {
            Kind::Human => Kind::play_human(state, debug),
            Kind::RandomMachine => Kind::play_random(state, debug),
            Kind::UniformMachine => Kind::play_machine_uniform(state, debug),
            Kind::MCTSMachine => Kind::play_machine_mcts(state, debug),
        }
    }
}
