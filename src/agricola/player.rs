use super::algorithms::PlayerType;
use super::quantity::*;

#[derive(Clone, Hash)]
pub struct Player {
    pub player_type: PlayerType,
}

impl Player {
    pub fn create_new(food: usize, player_type: PlayerType) -> Self {
        let mut res = new_res();
        res[Food.index()] = food;

        Player { player_type }
    }

    pub fn player_type(&self) -> PlayerType {
        self.player_type.clone()
    }
}
