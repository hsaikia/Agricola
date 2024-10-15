use super::actions::Action;
use super::state::{State, MAX_NUM_PLAYERS};
use std::collections::HashMap;

const MCTS_EXPLORATION_PARAM: f64 = 2.0;
const TD_ALPHA: f64 = 0.1;
const TD_GAMMA: f64 = 0.9;

fn fitness_sum(fitness1: &[f64], fitness2: &[f64]) -> [f64; MAX_NUM_PLAYERS] {
    let mut sum = [0.0; MAX_NUM_PLAYERS];
    for ((a, b), c) in fitness1.iter().zip(fitness2.iter()).zip(sum.iter_mut()) {
        *c = *a + *b;
    }
    sum
}

fn fitness_difference(fitness1: &[f64], fitness2: &[f64]) -> [f64; MAX_NUM_PLAYERS] {
    let mut diff = [0.0; MAX_NUM_PLAYERS];
    for ((a, b), c) in fitness1.iter().zip(fitness2.iter()).zip(diff.iter_mut()) {
        *c = *a - *b;
    }
    diff
}

fn fitness_multiply(fitness: &[f64], scalar: f64) -> [f64; MAX_NUM_PLAYERS] {
    let mut res = [0.0; MAX_NUM_PLAYERS];
    for (a, b) in fitness.iter().zip(res.iter_mut()) {
        *b = *a * scalar;
    }
    res
}

#[derive(Clone)]
pub struct GameRecord {
    pub average_fitness: [f64; MAX_NUM_PLAYERS],
    pub total_games: usize,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq)]
pub enum PlayerType {
    Human,
    MctsAI,
    TdAI,
}

/// Used to store the average fitness of a node and the number of times it has been visited for all actions to be taken in the next turn
#[derive(Clone)]
pub struct SimulationRecord {
    pub games: usize,
    pub score: f64,
    pub action: Action,
    pub action_hash: u64,
}

impl Default for GameRecord {
    fn default() -> Self {
        Self::new()
    }
}

impl GameRecord {
    #[must_use]
    pub fn new() -> Self {
        GameRecord {
            average_fitness: [0.0; MAX_NUM_PLAYERS],
            total_games: 0,
        }
    }

    /// # Panics
    /// If `fitness` length doesn't match `average_fitness`
    #[allow(clippy::cast_precision_loss)]
    pub fn add_fitness_mcts(&mut self, fitness: &[f64]) {
        assert!(fitness.len() == self.average_fitness.len());

        for it in fitness.iter().zip(self.average_fitness.iter_mut()) {
            let (a, b) = it;
            *b = (*b * self.total_games as f64 + *a) / (self.total_games + 1) as f64;
        }
        self.total_games += 1;
    }
}

pub struct AI {
    pub num_games_sampled: usize,
    pub cache: HashMap<u64, GameRecord>,
}

impl Default for AI {
    fn default() -> Self {
        Self::new()
    }
}

impl AI {
    #[must_use]
    pub fn new() -> Self {
        Self {
            num_games_sampled: 0,
            cache: HashMap::new(),
        }
    }

    pub fn reset(&mut self) {
        self.num_games_sampled = 0;
    }

    #[must_use]
    pub fn get_simulation_records(state: &State) -> Vec<SimulationRecord> {
        let mut records: Vec<SimulationRecord> = Vec::new();
        let actions = Action::next_choices(state);
        for action in actions {
            let mut tmp_state = state.clone();
            action.apply_choice(&mut tmp_state);
            records.push(SimulationRecord {
                games: 0,
                score: 0.0,
                action: action.clone(),
                action_hash: tmp_state.get_hash(),
            });
        }
        records
    }

    /// # Panics
    /// If `partial_cmp` fails
    pub fn sort_records(simulation_records: &mut [SimulationRecord]) {
        simulation_records.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    }

    /// Returns index of the node to traverse
    /// # Panics
    /// If `records` is empty
    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    pub fn choose_uct(&self, player_to_play: usize, records: &[SimulationRecord]) -> Action {
        assert!(!records.is_empty());
        // Use UCT formula to sample a child node
        let mut total: usize = 0;
        let mut selected_action: Action = records[0].action.clone();
        let mut highest_uct: f64 = f64::NEG_INFINITY;
        let mut min_fitness: f64 = f64::INFINITY;
        let mut max_fitness: f64 = f64::NEG_INFINITY;

        for record in records {
            // If child key isn't in the cache, it hasn't been explored - explore it immediately
            if !self.cache.contains_key(&record.action_hash) {
                return record.action.clone();
            }
            let fitness = self.cache[&record.action_hash].average_fitness[player_to_play];
            min_fitness = fitness.min(min_fitness);
            max_fitness = fitness.max(max_fitness);
            total += self.cache[&record.action_hash].total_games;
        }

        for record in records {
            let n = self.cache[&record.action_hash].total_games;
            // If any node has never been seen before - explore it
            if n == 0 {
                return record.action.clone();
            }

            let fitness = self.cache[&record.action_hash].average_fitness[player_to_play];
            assert!(total > 0);
            let p = (fitness - min_fitness) / (max_fitness - min_fitness)
                + f64::sqrt(MCTS_EXPLORATION_PARAM * total as f64 / n as f64);

            if p > highest_uct {
                highest_uct = p;
                selected_action = record.action.clone();
            }
        }
        selected_action
    }

    pub fn trace_path_uct(&self, state: &mut State) -> Vec<(u64, [f64; MAX_NUM_PLAYERS])> {
        let mut expand_node_hash = state.get_hash();
        let mut path = vec![(expand_node_hash, state.fitness())];

        // Expand along the tree
        while self.cache.contains_key(&expand_node_hash) {
            // In Agricola - since the future board state changes randomly
            // get_all_available_actions can return different actions for the same current game state.

            let sub_choices = Action::next_choices(state);
            if sub_choices.is_empty() {
                break;
            } else if sub_choices.len() == 1 {
                sub_choices[0].apply_choice(state);
            } else {
                // Generate all child hashes
                let sub_records: Vec<SimulationRecord> = AI::get_simulation_records(state);
                let selected_sub_action = self.choose_uct(state.current_player_idx, &sub_records);
                selected_sub_action.apply_choice(state);
            }

            expand_node_hash = state.get_hash();
            path.push((expand_node_hash, state.fitness()));
        }
        path
    }

    /// # Panics
    /// Only works for AI player types, panics otherwise
    pub fn sample_once(
        &mut self,
        records: &mut [SimulationRecord],
        state: &State,
        opt_depth: Option<usize>,
    ) {
        match state.player_type(state.current_player_idx) {
            PlayerType::TdAI => self.sample_once_td(records, state, opt_depth),
            PlayerType::MctsAI => self.sample_once_mcts(records, state, opt_depth),
            PlayerType::Human => panic!("Invalid AI player type"),
        }
    }

    fn sample_once_td(
        &mut self,
        records: &mut [SimulationRecord],
        state: &State,
        opt_depth: Option<usize>,
    ) {
        let first_action = self.choose_uct(state.current_player_idx, records);
        let mut tmp_state: State = state.clone();
        first_action.apply_choice(&mut tmp_state);

        let path = self.trace_path_uct(&mut tmp_state);
        let final_node_hash = tmp_state.get_hash();

        // Perform playout - play the game out until the desired depth (if none, play until the end)
        tmp_state.play_random(opt_depth);

        // Calculate result and backpropagate to the root
        let res = tmp_state.fitness();

        let mut last_estimated_fitness =
            if let Some(game_record) = self.cache.get_mut(&final_node_hash) {
                game_record.add_fitness_mcts(&res);
                game_record.average_fitness
            } else {
                self.cache.insert(
                    final_node_hash,
                    GameRecord {
                        average_fitness: res,
                        total_games: 1,
                    },
                );
                res
            };

        for ((node1, fitness1), (_, fitness2)) in path.iter().zip(path.iter().skip(1)) {
            // For the rest of the game, use the TD update formula
            // Reward is immediate fitness reward for going from the current state to the next state
            let diff = fitness_difference(fitness2, fitness1);
            if let Some(game_record) = self.cache.get_mut(node1) {
                game_record.average_fitness = fitness_sum(
                    &fitness_multiply(&game_record.average_fitness, 1.0 - TD_ALPHA),
                    &fitness_multiply(
                        &fitness_sum(&diff, &fitness_multiply(&last_estimated_fitness, TD_GAMMA)),
                        TD_ALPHA,
                    ),
                );
                game_record.total_games += 1;
                last_estimated_fitness = game_record.average_fitness;
            } else {
                let new_fitness =
                    fitness_sum(&diff, &fitness_multiply(&last_estimated_fitness, TD_GAMMA));
                self.cache.insert(
                    *node1,
                    GameRecord {
                        average_fitness: new_fitness,
                        total_games: 1,
                    },
                );
                last_estimated_fitness = new_fitness;
            }
        }

        for rec in records.iter_mut() {
            if let Some(entry) = self.cache.get(&rec.action_hash) {
                rec.games = entry.total_games;
                rec.score = entry.average_fitness[state.current_player_idx];
            }
        }

        self.num_games_sampled += 1;
    }

    fn sample_once_mcts(
        &mut self,
        records: &mut [SimulationRecord],
        state: &State,
        opt_depth: Option<usize>,
    ) {
        let first_action = self.choose_uct(state.current_player_idx, records);
        let mut tmp_state: State = state.clone();
        first_action.apply_choice(&mut tmp_state);

        let path = self.trace_path_uct(&mut tmp_state);

        // Perform playout - play the game out until the desired depth (if none, play until the end)
        tmp_state.play_random(opt_depth);

        // Calculate result and backpropagate to the root
        let res = tmp_state.fitness();

        for (node, _) in &path {
            if let Some(game_record) = self.cache.get_mut(node) {
                game_record.add_fitness_mcts(&res);
            } else {
                self.cache.insert(
                    *node,
                    GameRecord {
                        average_fitness: res,
                        total_games: 1,
                    },
                );
            }
        }

        for rec in records.iter_mut() {
            if let Some(entry) = self.cache.get(&rec.action_hash) {
                rec.games = entry.total_games;
                rec.score = entry.average_fitness[state.current_player_idx];
            }
        }

        self.num_games_sampled += 1;
    }
}
