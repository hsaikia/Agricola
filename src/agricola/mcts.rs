use std::collections::HashMap;

use super::{actions::Action, algorithms::SimulationRecord};

const MCTS_EXPLORATION_PARAM: f64 = 2.0;

#[derive(Clone)]
pub struct GameRecord {
    pub average_fitness: Vec<f64>, // Wins for each player
    pub total_games: usize,
}

impl GameRecord {
    #[must_use]
    pub fn new(num_players: usize) -> Self {
        GameRecord {
            average_fitness: vec![0.0; num_players],
            total_games: 0,
        }
    }

    /// # Panics
    /// If `fitness` length doesn't match `average_fitness`
    #[allow(clippy::cast_precision_loss)]
    pub fn add_fitness(&mut self, fitness: &[f64]) {
        assert!(fitness.len() == self.average_fitness.len());

        for it in fitness.iter().zip(self.average_fitness.iter_mut()) {
            let (a, b) = it;
            *b = (*b * self.total_games as f64 + *a) / (self.total_games + 1) as f64;
        }
        self.total_games += 1;
    }

    /// Returns index of the node to traverse
    /// # Panics
    /// If `records` is empty
    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    pub fn choose_uct(
        player_to_play: usize,
        records: &Vec<SimulationRecord>,
        mcts_cache: &HashMap<u64, GameRecord>,
    ) -> Action {
        assert!(!records.is_empty());
        // Use UCT formula to sample a child node
        let mut total: usize = 0;
        let mut selected_action: Action = records[0].action.clone();
        let mut highest_uct: f64 = f64::NEG_INFINITY;
        let mut min_fitness: f64 = f64::INFINITY;
        let mut max_fitness: f64 = f64::NEG_INFINITY;

        for record in records {
            // If child key isn't in the cache, it hasn't been explored - explore it immediately
            if !mcts_cache.contains_key(&record.action_hash) {
                return record.action.clone();
            }
            let fitness = mcts_cache[&record.action_hash].average_fitness[player_to_play];
            min_fitness = fitness.min(min_fitness);
            max_fitness = fitness.max(max_fitness);
            total += mcts_cache[&record.action_hash].total_games;
        }

        for record in records {
            let n = mcts_cache[&record.action_hash].total_games;
            // If any node has never been seen before - explore it
            if n == 0 {
                return record.action.clone();
            }
            let fitness = mcts_cache[&record.action_hash].average_fitness[player_to_play];
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
}
