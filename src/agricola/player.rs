use super::algorithms::PlayerType;
use super::farm::Farm;
use super::quantity::*;

#[derive(Clone, Hash)]
pub struct Player {
    pub player_type: PlayerType,
    pub farm: Farm,
}

impl Player {
    pub fn create_new(food: usize, player_type: PlayerType) -> Self {
        let mut res = new_res();
        res[Food.index()] = food;

        Player {
            player_type,
            farm: Farm::new(),
        }
    }

    pub fn player_type(&self) -> PlayerType {
        self.player_type.clone()
    }

    pub fn add_new_field(&mut self, idx: &usize) {
        self.farm.add_field(*idx);
    }

    pub fn field_options(&self) -> Vec<usize> {
        self.farm.possible_field_positions()
    }
}
