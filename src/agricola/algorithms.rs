use super::actions::Action;
use super::mcts::GameRecord;
use super::state::State;
use std::collections::HashMap;

#[derive(Debug, Clone, Hash, PartialEq)]
pub enum PlayerType {
    Human,
    MCTSMachine,
}
pub struct SimulationRecord {
    pub games: u32,
    pub fitness: f32,
    pub action_idx: usize,
}
pub struct AI {
    pub num_games_sampled: usize,
    cache: HashMap<u64, GameRecord>,
    pub records: Vec<SimulationRecord>,
}

impl AI {
    pub fn new() -> Self {
        Self {
            num_games_sampled: 0,
            cache: HashMap::new(),
            records: Vec::new(),
        }
    }

    pub fn sample_once(&mut self, state: &State) {
        let choices = Action::next_choices(state);
        if choices.len() < 2 {
            return;
        }

        let mut action_hashes: Vec<u64> = vec![];
        let mut tmp_game: State;

        // Add the child node hashes
        for action in &choices {
            tmp_game = state.clone();
            action.apply_choice(&mut tmp_game);
            action_hashes.push(tmp_game.get_hash());
        }

        let sample_idx =
            GameRecord::choose_uct(state.current_player_idx, &action_hashes, &self.cache);
        tmp_game = state.clone();
        choices[sample_idx].apply_choice(&mut tmp_game);

        //print!("\nGame {}", g);
        let mut expand_node_hash = tmp_game.get_hash();
        let mut path: Vec<u64> = vec![expand_node_hash];

        // Expand along the tree
        while self.cache.contains_key(&expand_node_hash) {
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
                &self.cache,
            );
            sub_choices[chosen_idx].apply_choice(&mut tmp_game);
            expand_node_hash = tmp_game.get_hash();
            path.push(tmp_game.get_hash());
        }

        // Add node to cache
        if let std::collections::hash_map::Entry::Vacant(e) = self.cache.entry(expand_node_hash) {
            e.insert(GameRecord::new(tmp_game.players.len()));
        }

        // Perform playout - play the game out until the end
        tmp_game.play_random();
        // Calculate result and backpropagate to the root
        let res = tmp_game.fitness();
        for node in &path {
            let mut state = self.cache[node].clone();
            state.add_fitness(&res);
            self.cache.insert(*node, state);
        }

        self.records.clear();

        // Choose the best move again according to UCT
        let mut best_fitness: f32 = f32::NEG_INFINITY;
        for (action_idx, action) in choices.iter().enumerate() {
            tmp_game = state.clone();
            action.apply_choice(&mut tmp_game);
            let action_hash = tmp_game.get_hash();
            let games: u32 = if self.cache.contains_key(&action_hash) {
                self.cache[&action_hash].total_games
            } else {
                0
            };
            let fitness: f32 = if self.cache.contains_key(&action_hash) {
                self.cache[&action_hash].average_fitness[state.current_player_idx]
            } else {
                0.0
            };
            if fitness > best_fitness {
                best_fitness = fitness;
            }

            self.records.push(SimulationRecord {
                games,
                fitness,
                action_idx,
            });
        }

        self.records
            .sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());

        self.num_games_sampled += 1;
    }
}
