#[derive(Hash, PartialEq)]
pub struct GameState {
    pub current_player_idx: usize,
    pub scores: Vec<i32>,
    pub wood: Vec<u8>,
}

pub trait AccumulationSpace {
    fn update(&mut self);
}

pub trait Action {
    fn apply_choice(&self, state: &mut GameState);
}

pub struct UseDayLaborer {}

#[derive(Hash, PartialEq)]
pub struct UseForest {}

impl AccumulationSpace for UseForest {
    fn update(&mut self) {
        println!("UseForest");
    }
}

impl Action for UseDayLaborer {
    fn apply_choice(&self, state: &mut GameState) {
        state.scores[state.current_player_idx] += 1;
    }
}

impl Action for UseForest {
    fn apply_choice(&self, state: &mut GameState) {
        state.wood[state.current_player_idx] += 1;
    }
}
