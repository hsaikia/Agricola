use super::actions::Action;
use super::mcts::GameRecord;
use super::state::State;
use std::collections::HashMap;

const DEPTH : usize = 100;

#[derive(Debug, Clone, Hash, PartialEq)]
pub enum PlayerType {
    Human,
    MCTSMachine,
}
#[derive(Clone)]
pub struct SimulationRecord {
    pub games: usize,
    pub fitness: f32,
    pub action: Action,
    pub action_hash: u64,
}

pub struct AI {
    pub num_games_sampled: usize,
    pub cache: HashMap<u64, GameRecord>,
    pub records: Vec<SimulationRecord>,
}

impl Default for AI {
    fn default() -> Self {
        Self::new()
    }
}

impl AI {
    pub fn new() -> Self {
        Self {
            num_games_sampled: 0,
            cache: HashMap::new(),
            records: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.num_games_sampled = 0;
        self.records.clear();
    }

    pub fn init_records(&mut self, actions: &Vec<Action>, state: &State) {
        self.reset();
        for action in actions {
            let mut tmp_game = state.clone();
            action.apply_choice(&mut tmp_game);
            self.records.push(SimulationRecord {
                games: 0,
                fitness: 0.0,
                action: action.clone(),
                action_hash: tmp_game.get_hash(),
            });
        }
    }

    pub fn sorted_records(&self) -> Vec<SimulationRecord> {
        let mut records = self.records.clone();
        records.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
        records
    }

    pub fn sample_once(&mut self, state: &State, debug: bool) {
        let mut tmp_game: State;
        let selected_action =
            GameRecord::choose_uct(state.current_player_idx, &self.records, &self.cache);
        tmp_game = state.clone();
        selected_action.apply_choice(&mut tmp_game);

        if debug {
            print!(
                "\nSample {} : {:?} | ",
                self.num_games_sampled, selected_action
            );
        }
        let mut expand_node_hash = tmp_game.get_hash();
        let mut path: Vec<u64> = vec![expand_node_hash];

        // Expand along the tree
        while self.cache.contains_key(&expand_node_hash) {
            // In Agricola - since the future board state changes randomly
            // get_all_available_actions can return different actions for the same current game state.

            let sub_choices = Action::next_choices(&tmp_game);
            if sub_choices.is_empty() {
                break;
            } else if sub_choices.len() == 1 {
                sub_choices[0].apply_choice(&mut tmp_game);
            } else {
                // Generate all child hashes
                let mut sub_records: Vec<SimulationRecord> = Vec::new();

                for sub_action in &sub_choices {
                    let mut tmp_game2 = tmp_game.clone();
                    sub_action.apply_choice(&mut tmp_game2);
                    let child_hash = tmp_game2.get_hash();
                    sub_records.push(SimulationRecord {
                        games: 0,
                        fitness: 0.0,
                        action: sub_action.clone(),
                        action_hash: child_hash,
                    });
                }
                let selected_sub_action =
                    GameRecord::choose_uct(tmp_game.current_player_idx, &sub_records, &self.cache);
                if debug {
                    print!("{:?}", selected_sub_action);
                }
                selected_sub_action.apply_choice(&mut tmp_game);
            }

            expand_node_hash = tmp_game.get_hash();
            path.push(expand_node_hash);
        }

        // Perform playout - play the game out until the end
        tmp_game.play_random(&mut path, DEPTH);
        // Calculate result and backpropagate to the root
        let res = tmp_game.fitness();
        if debug {
            println!(": {}", res[state.current_player_idx]);
        }

        for node in &path {
            if let Some(game_record) = self.cache.get_mut(node) {
                game_record.add_fitness(&res);
            } else {
                self.cache.insert(
                    *node,
                    GameRecord {
                        average_fitness: res.clone(),
                        total_games: 1,
                    },
                );
            }
        }

        // Choose the best move again according to UCT
        for rec in &mut self.records {
            if let Some(entry) = self.cache.get(&rec.action_hash) {
                rec.games = entry.total_games;
                rec.fitness = entry.average_fitness[state.current_player_idx];
            }
        }

        self.num_games_sampled += 1;
    }
}
