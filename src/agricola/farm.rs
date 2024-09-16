use derivative::Derivative;

use super::fencing::PastureConfig;
use crate::agricola::fencing::{get_all_pasture_configs, get_rand_fence_options};
use std::hash::Hash;

const L: usize = 5;
const W: usize = 3;
pub const NUM_FARMYARD_SPACES: usize = L * W;
pub const MAX_FENCES: usize = 15;
pub const MAX_STABLES: usize = 4;
pub const ROOM_INDICES: [usize; 2] = [5, 10];

#[derive(Copy, Clone, Hash)]
pub enum House {
    Wood,
    Clay,
    Stone,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq)]
pub enum Seed {
    Grain,
    Vegetable,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq)]
pub enum Animal {
    Sheep,
    Boar,
    Cattle,
}

type ContainsStable = bool;

#[derive(Copy, Clone, Debug, Default, Hash, PartialEq)]
pub enum FarmyardSpace {
    #[default]
    Empty,
    Room,
    Field(Option<(Seed, usize)>),
    UnfencedStable(Option<(Animal, usize)>),
    FencedPasture(Option<(Animal, usize)>, ContainsStable, usize),
}

// Farmyard spaces
// 00 01 02 03 04
// 05 06 07 08 09
// 10 11 12 13 14

// Order : NEWS
pub const NEIGHBOR_SPACES: [[Option<usize>; 4]; NUM_FARMYARD_SPACES] = [
    [None, Some(1), None, Some(5)],
    [None, Some(2), Some(0), Some(6)],
    [None, Some(3), Some(1), Some(7)],
    [None, Some(4), Some(2), Some(8)],
    [None, None, Some(3), Some(9)],
    [Some(0), Some(6), None, Some(10)],
    [Some(1), Some(7), Some(5), Some(11)],
    [Some(2), Some(8), Some(6), Some(12)],
    [Some(3), Some(9), Some(7), Some(13)],
    [Some(4), None, Some(8), Some(14)],
    [Some(5), Some(11), None, None],
    [Some(6), Some(12), Some(10), None],
    [Some(7), Some(13), Some(11), None],
    [Some(8), Some(14), Some(12), None],
    [Some(9), None, Some(13), None],
];

#[derive(Derivative, Debug, Clone, Hash)]
pub struct Farm {
    pub farmyard_spaces: [FarmyardSpace; NUM_FARMYARD_SPACES],
    pub fences_used: usize,
    pub pet: Option<(Animal, usize)>,
    #[derivative(Hash = "ignore")]
    pub fence_options_cache: Vec<PastureConfig>,
}

impl Default for Farm {
    fn default() -> Self {
        Self::new()
    }
}

impl Farm {
    pub fn new() -> Self {
        let mut farmyard_spaces = [FarmyardSpace::Empty; NUM_FARMYARD_SPACES];
        for idx in ROOM_INDICES.iter() {
            farmyard_spaces[*idx] = FarmyardSpace::Room;
        }

        Self {
            farmyard_spaces,
            fences_used: 0,
            pet: None,
            fence_options_cache: get_all_pasture_configs(&farmyard_spaces),
        }
    }

    pub fn fencing_options(&self, wood: usize) -> Vec<PastureConfig> {
        get_rand_fence_options(&self.fence_options_cache, self.fences_used, wood)
    }

    pub fn fence_spaces(&mut self, pasture_config: &PastureConfig, wood: &mut usize) {
        for (idx, pasture) in pasture_config.pastures.iter().enumerate() {
            for &space in pasture {
                match self.farmyard_spaces[space] {
                    FarmyardSpace::Empty => {
                        self.farmyard_spaces[space] =
                            FarmyardSpace::FencedPasture(None, false, idx);
                    }
                    FarmyardSpace::UnfencedStable(animal) => {
                        self.farmyard_spaces[space] =
                            FarmyardSpace::FencedPasture(animal, true, idx);
                    }
                    _ => (),
                }
            }
        }
        *wood += self.fences_used;
        *wood -= pasture_config.wood;
        self.fences_used = pasture_config.wood;
        self.fence_options_cache = get_all_pasture_configs(&self.farmyard_spaces);
    }

    pub fn possible_field_positions(&self) -> Vec<usize> {
        let field_idxs = self.field_indices();

        if field_idxs.is_empty() {
            return self.empty_indices();
        }

        self.neighbor_empty_indices(&field_idxs)
    }

    pub fn possible_room_positions(&self) -> Vec<usize> {
        let room_idxs = self.room_indices();

        if room_idxs.is_empty() {
            return self.empty_indices();
        }
        self.neighbor_empty_indices(&room_idxs)
    }

    pub fn possible_stable_positions(&self) -> Vec<usize> {
        (0..NUM_FARMYARD_SPACES)
            .filter(|&i| {
                matches!(
                    self.farmyard_spaces[i],
                    FarmyardSpace::Empty | FarmyardSpace::FencedPasture(_, false, _)
                )
            })
            .collect()
    }

    pub fn neighbor_empty_indices(&self, indices: &[usize]) -> Vec<usize> {
        indices
            .iter()
            .flat_map(|i| NEIGHBOR_SPACES[*i].into_iter().flatten())
            .filter(|&i| matches!(self.farmyard_spaces[i], FarmyardSpace::Empty))
            .collect::<Vec<_>>()
    }

    pub fn room_indices(&self) -> Vec<usize> {
        (0..NUM_FARMYARD_SPACES)
            .filter(|&i| matches!(self.farmyard_spaces[i], FarmyardSpace::Room))
            .collect()
    }

    pub fn field_indices(&self) -> Vec<usize> {
        (0..NUM_FARMYARD_SPACES)
            .filter(|&i| matches!(self.farmyard_spaces[i], FarmyardSpace::Field(_)))
            .collect()
    }

    pub fn empty_indices(&self) -> Vec<usize> {
        (0..NUM_FARMYARD_SPACES)
            .filter(|&i| matches!(self.farmyard_spaces[i], FarmyardSpace::Empty))
            .collect()
    }

    pub fn add_field(&mut self, idx: usize) {
        assert!(self.farmyard_spaces[idx] == FarmyardSpace::Empty);
        self.farmyard_spaces[idx] = FarmyardSpace::Field(None);
        self.fence_options_cache = get_all_pasture_configs(&self.farmyard_spaces);
    }

    pub fn build_room(&mut self, idx: usize) {
        assert!(self.farmyard_spaces[idx] == FarmyardSpace::Empty);
        self.farmyard_spaces[idx] = FarmyardSpace::Room;
        self.fence_options_cache = get_all_pasture_configs(&self.farmyard_spaces);
    }

    pub fn build_stable(&mut self, idx: usize) {
        let available = matches!(
            self.farmyard_spaces[idx],
            FarmyardSpace::Empty | FarmyardSpace::FencedPasture(_, false, _)
        );
        assert!(available);
        assert!(self.can_build_stable());

        match self.farmyard_spaces[idx] {
            FarmyardSpace::Empty => self.farmyard_spaces[idx] = FarmyardSpace::UnfencedStable(None),
            FarmyardSpace::FencedPasture(animal, false, pasture_idx) => {
                self.farmyard_spaces[idx] = FarmyardSpace::FencedPasture(animal, true, pasture_idx)
            }
            _ => (),
        }
    }

    pub fn can_build_stable(&self) -> bool {
        let mut num_stables = 0;
        let mut candidate_spaces = 0;
        for fs in &self.farmyard_spaces {
            match *fs {
                FarmyardSpace::UnfencedStable(_) | FarmyardSpace::FencedPasture(_, true, _) => {
                    num_stables += 1
                }
                FarmyardSpace::Empty | FarmyardSpace::FencedPasture(_, false, _) => {
                    candidate_spaces += 1
                }
                _ => (),
            }
        }

        if candidate_spaces > 0 && num_stables < MAX_STABLES {
            return true;
        }

        false
    }

    pub fn can_sow(&self) -> bool {
        self.farmyard_spaces
            .iter()
            .any(|f| matches!(f, FarmyardSpace::Field(None)))
    }

    pub fn sow_field(&mut self, seed: &Seed) {
        assert!(self.can_sow());
        let opt_empty_field = self
            .farmyard_spaces
            .iter_mut()
            .find(|f| matches!(f, FarmyardSpace::Field(None)));
        if let Some(field) = opt_empty_field {
            match *seed {
                Seed::Grain => *field = FarmyardSpace::Field(Some((Seed::Grain, 3))),
                Seed::Vegetable => *field = FarmyardSpace::Field(Some((Seed::Vegetable, 2))),
            }
        }
    }

    pub fn harvest_fields(&mut self) -> Vec<Seed> {
        let mut ret: Vec<Seed> = Vec::new();
        for space in &mut self.farmyard_spaces {
            if let FarmyardSpace::Field(Some((crop, amount))) = space {
                ret.push(*crop);
                *amount -= 1;

                if *amount == 0 {
                    *space = FarmyardSpace::Field(None);
                }
            }
        }
        ret
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_field_options() {
        let mut farm = Farm::new();
        let field_opt = farm.possible_field_positions();
        assert_eq!(field_opt, vec![0, 1, 2, 3, 4, 6, 7, 8, 9, 11, 12, 13, 14]);

        farm.add_field(0);
        let field_opt = farm.possible_field_positions();
        assert_eq!(field_opt, vec![1]);

        farm = Farm::new();
        farm.add_field(2);
        let field_opt = farm.possible_field_positions();
        assert_eq!(field_opt, vec![3, 1, 7]);

        farm = Farm::new();
        farm.add_field(7);
        let field_opt = farm.possible_field_positions();
        assert_eq!(field_opt, vec![2, 8, 6, 12]);
    }

    #[test]
    fn test_room_options() {
        let farm = Farm::new();
        let room_opt = farm.possible_room_positions();
        //println!("{:?}", room_opt);
        assert_eq!(room_opt, [0, 6, 11]);
    }
}
