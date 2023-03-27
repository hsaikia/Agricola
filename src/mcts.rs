use std::collections::HashMap;

const MCTS_EXPLORATION_PARAM: f32 = 2.0;

#[derive(Clone)]
pub struct GameRecord {
    pub average_fitness: Vec<f32>, // Wins for each player
    pub total_games: u32,
}

impl GameRecord {
    pub fn new(num_players: usize) -> Self {
        GameRecord {
            average_fitness: vec![0.0; num_players],
            total_games: 0,
        }
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn add_fitness(&mut self, fitness: &Vec<i32>) {
        assert!(fitness.len() == self.average_fitness.len());

        for it in fitness.iter().zip(self.average_fitness.iter_mut()) {
            let (a, b) = it;
            *b = (*b * self.total_games as f32 + *a as f32) / (self.total_games + 1) as f32;
        }
        self.total_games += 1;
    }

    // Returns index of the node to traverse
    #[allow(clippy::cast_precision_loss)]
    pub fn choose_uct(
        player_to_play: usize,
        nodes: &Vec<u64>,
        mcts_cache: &HashMap<u64, GameRecord>,
    ) -> usize {
        assert!(!nodes.is_empty());
        // Use UCT formula to sample a child node
        let mut total: u32 = 0;
        let mut idx: usize = 0;
        let mut highest_uct: f32 = 0.0;
        let mut min_fitness: f32 = f32::INFINITY;
        let mut max_fitness: f32 = f32::NEG_INFINITY;

        for (i, node) in nodes.iter().enumerate() {
            // If child key isn't in the cache, it hasn't been explored - explore it immediately
            if !mcts_cache.contains_key(node) {
                return i;
            }
            let fitness = mcts_cache[node].average_fitness[player_to_play];
            min_fitness = fitness.min(min_fitness);
            max_fitness = fitness.max(max_fitness);
            total += mcts_cache[node].total_games;
        }

        for (i, node) in nodes.iter().enumerate() {
            let n: u32 = mcts_cache[node].total_games;
            // If any node has never been seen before - explore it
            if n == 0 {
                return i;
            }
            let fitness = mcts_cache[node].average_fitness[player_to_play];
            assert!(total > 0);
            let p = (fitness - min_fitness) / (max_fitness - min_fitness)
                + f32::sqrt(MCTS_EXPLORATION_PARAM * total as f32 / n as f32);

            if p > highest_uct {
                highest_uct = p;
                idx = i;
            }
        }
        idx
    }
}
