use derivative::Derivative;

use super::fencing::{get_existing_pasture_capacities, PastureConfig};
use crate::agricola::fencing::get_best_fence_options;
use std::{collections::VecDeque, hash::Hash};

pub const L: usize = 5;
pub const W: usize = 3;
pub const NUM_FARMYARD_SPACES: usize = L * W;
pub const MAX_FENCES: usize = 15;
pub const MAX_STABLES: usize = 4;
pub const ROOM_INDICES: [usize; 2] = [5, 10];

#[derive(Debug, Copy, Clone, Hash, PartialEq)]
pub enum Seed {
    Grain,
    Vegetable,
}

type ContainsStable = bool;

#[derive(Copy, Clone, Debug, Default, Hash, PartialEq)]
pub enum FarmyardSpace {
    #[default]
    Empty,
    Room,
    Field(Option<(Seed, usize)>),
    UnfencedStable,
    FencedPasture(ContainsStable, usize),
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
}

impl Default for Farm {
    fn default() -> Self {
        Self::new()
    }
}

impl Farm {
    #[must_use]
    pub fn new() -> Self {
        let mut farmyard_spaces = [FarmyardSpace::Empty; NUM_FARMYARD_SPACES];
        for idx in &ROOM_INDICES {
            farmyard_spaces[*idx] = FarmyardSpace::Room;
        }

        Self {
            farmyard_spaces,
            fences_used: 0,
        }
    }

    /// Animals in order S, P, C. Capacities for various pastures. Returns leftover [S, P, C] which couldn't be accommodated
    /// # Panics
    /// If max of array fails
    #[must_use]
    pub fn accommodate_animals(&self, animals: &[usize]) -> [usize; 3] {
        let mut ret: [usize; 3] = [animals[0], animals[1], animals[2]];
        let mut capacities = get_existing_pasture_capacities(&self.farmyard_spaces);
        // Sort in descending order
        capacities.sort_by(|a, b| b.cmp(a));
        for c in &capacities {
            let max_animals_of_one_type = *ret.iter().max().unwrap();
            let max_animal_bin = ret.iter_mut().max().unwrap();
            if *max_animal_bin > 0 {
                *max_animal_bin -= c.min(&max_animals_of_one_type);
            } else {
                break;
            }
        }
        ret
    }

    fn spread(indices: &mut [bool], empty_indices: &[bool]) {
        let mut q = VecDeque::new();
        for (idx, v) in indices.iter().enumerate() {
            if *v {
                q.push_back(idx);
            }
        }
        while !q.is_empty() {
            let idx = q.pop_front().unwrap();

            indices[idx] = true;

            for &neighbor in NEIGHBOR_SPACES[idx].iter().flatten() {
                if !indices[neighbor] && empty_indices[neighbor] {
                    q.push_back(neighbor);
                }
            }
        }
    }

    #[must_use]
    pub fn flexibility(&self) -> usize {
        let mut future_room = [false; NUM_FARMYARD_SPACES];
        let mut future_field = [false; NUM_FARMYARD_SPACES];
        let mut future_stable = [false; NUM_FARMYARD_SPACES];
        let mut future_pasture = [false; NUM_FARMYARD_SPACES];
        let mut empty = [false; NUM_FARMYARD_SPACES];

        for (idx, space) in self.farmyard_spaces.iter().enumerate() {
            match space {
                FarmyardSpace::Room => future_room[idx] = true,
                FarmyardSpace::Field(_) => future_field[idx] = true,
                FarmyardSpace::UnfencedStable => future_stable[idx] = true,
                FarmyardSpace::FencedPasture(stable, _) => {
                    future_pasture[idx] = true;
                    future_stable[idx] = *stable;
                }
                FarmyardSpace::Empty => empty[idx] = true,
            }
        }

        Self::spread(&mut future_room, &empty);
        Self::spread(&mut future_field, &empty);
        Self::spread(&mut future_pasture, &empty);
        let sum_future_room = future_room.iter().filter(|&x| *x).count();
        let sum_future_field = future_field.iter().filter(|&x| *x).count();
        let sum_future_pasture = future_pasture.iter().filter(|&x| *x).count();
        sum_future_room + sum_future_field + sum_future_pasture
    }

    #[must_use]
    pub fn fencing_options(&self, cache: &[PastureConfig], wood: usize) -> Vec<PastureConfig> {
        if self.fences_used >= MAX_FENCES {
            return Vec::new();
        }
        get_best_fence_options(cache, self.fences_used, wood)
    }

    pub fn fence_spaces(&mut self, pasture_config: &PastureConfig, wood: &mut usize) {
        for (idx, pasture) in pasture_config.pastures.iter().enumerate() {
            for &space in pasture {
                match self.farmyard_spaces[space] {
                    FarmyardSpace::Empty => {
                        self.farmyard_spaces[space] = FarmyardSpace::FencedPasture(false, idx);
                    }
                    FarmyardSpace::UnfencedStable => {
                        self.farmyard_spaces[space] = FarmyardSpace::FencedPasture(true, idx);
                    }
                    FarmyardSpace::FencedPasture(stable, _) => {
                        self.farmyard_spaces[space] = FarmyardSpace::FencedPasture(stable, idx);
                    }
                    _ => (),
                }
            }
        }
        *wood += self.fences_used;
        *wood -= pasture_config.wood;
        self.fences_used = pasture_config.wood;
    }

    #[must_use]
    pub fn possible_field_positions(&self) -> Vec<usize> {
        let field_idxs = self.field_indices();

        if field_idxs.is_empty() {
            return self.empty_indices();
        }

        self.neighbor_empty_indices(&field_idxs)
    }

    #[must_use]
    pub fn possible_room_positions(&self) -> Vec<usize> {
        let room_idxs = self.room_indices();

        if room_idxs.is_empty() {
            return self.empty_indices();
        }
        self.neighbor_empty_indices(&room_idxs)
    }

    #[must_use]
    pub fn possible_stable_positions(&self) -> Vec<usize> {
        (0..NUM_FARMYARD_SPACES)
            .filter(|&i| {
                matches!(
                    self.farmyard_spaces[i],
                    FarmyardSpace::Empty | FarmyardSpace::FencedPasture(false, _)
                )
            })
            .collect()
    }

    #[must_use]
    pub fn neighbor_empty_indices(&self, indices: &[usize]) -> Vec<usize> {
        let mut ret = indices
            .iter()
            .flat_map(|i| NEIGHBOR_SPACES[*i].into_iter().flatten())
            .filter(|&i| matches!(self.farmyard_spaces[i], FarmyardSpace::Empty))
            .collect::<Vec<_>>();
        ret.dedup();
        ret
    }

    #[must_use]
    pub fn room_indices(&self) -> Vec<usize> {
        (0..NUM_FARMYARD_SPACES)
            .filter(|&i| matches!(self.farmyard_spaces[i], FarmyardSpace::Room))
            .collect()
    }

    #[must_use]
    pub fn field_indices(&self) -> Vec<usize> {
        (0..NUM_FARMYARD_SPACES)
            .filter(|&i| matches!(self.farmyard_spaces[i], FarmyardSpace::Field(_)))
            .collect()
    }

    #[must_use]
    pub fn empty_indices(&self) -> Vec<usize> {
        (0..NUM_FARMYARD_SPACES)
            .filter(|&i| matches!(self.farmyard_spaces[i], FarmyardSpace::Empty))
            .collect()
    }

    /// # Panics
    /// If cannot add field
    pub fn add_field(&mut self, idx: usize) {
        assert!(self.farmyard_spaces[idx] == FarmyardSpace::Empty);
        self.farmyard_spaces[idx] = FarmyardSpace::Field(None);
    }

    /// # Panics
    /// If cannot build room
    pub fn build_room(&mut self, idx: usize) {
        assert!(self.farmyard_spaces[idx] == FarmyardSpace::Empty);
        self.farmyard_spaces[idx] = FarmyardSpace::Room;
    }

    /// # Panics
    /// If cannot build stable
    pub fn build_stable(&mut self, idx: usize) {
        let available = matches!(
            self.farmyard_spaces[idx],
            FarmyardSpace::Empty | FarmyardSpace::FencedPasture(false, _)
        );
        assert!(available);
        assert!(self.can_build_stable());

        match self.farmyard_spaces[idx] {
            FarmyardSpace::Empty => self.farmyard_spaces[idx] = FarmyardSpace::UnfencedStable,
            FarmyardSpace::FencedPasture(false, pasture_idx) => {
                self.farmyard_spaces[idx] = FarmyardSpace::FencedPasture(true, pasture_idx);
            }
            _ => (),
        }
    }

    #[must_use]
    pub fn can_build_stable(&self) -> bool {
        let mut num_stables = 0;
        let mut candidate_spaces = 0;
        for fs in &self.farmyard_spaces {
            match *fs {
                FarmyardSpace::UnfencedStable | FarmyardSpace::FencedPasture(true, _) => {
                    num_stables += 1;
                }
                FarmyardSpace::Empty | FarmyardSpace::FencedPasture(false, _) => {
                    candidate_spaces += 1;
                }
                _ => (),
            }
        }

        if candidate_spaces > 0 && num_stables < MAX_STABLES {
            return true;
        }

        false
    }

    #[must_use]
    pub fn can_sow(&self) -> bool {
        self.farmyard_spaces
            .iter()
            .any(|f| matches!(f, FarmyardSpace::Field(None)))
    }

    /// # Panics
    /// If there are no empty fields
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
        let farm = Farm::new();
        let field_opt = farm.possible_field_positions();
        assert_eq!(field_opt, vec![0, 1, 2, 3, 4, 6, 7, 8, 9, 11, 12, 13, 14]);
    }

    #[test]
    fn test_room_options() {
        let farm = Farm::new();
        let room_opt = farm.possible_room_positions();
        //println!("{:?}", room_opt);
        assert_eq!(room_opt, [0, 6, 11]);
    }
}
