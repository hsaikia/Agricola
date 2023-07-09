use crate::agricola::primitives::{Resource, Resources};
use std::{collections::HashMap, hash::Hash};

pub const NUM_FARMYARD_SPACES: usize = 15;
pub const MAX_FENCES: usize = 15;
pub const MAX_STABLES: usize = 4;
const PASTURE_CAPACITY: usize = 2;
const STABLE_MULTIPLIER: usize = 2;

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
    Pigs,
    Cattle,
}

// Fence mask [North, East, West, South]
// e.g : 0000 -> Not fenced on either side
// 0110 -> Fenced on East and West sides, connected to pastures on North and South sides

type Mask = usize;
type ContainsStable = bool;

#[derive(Copy, Clone, Debug, Default, Hash, PartialEq)]
pub enum FarmyardSpace {
    #[default]
    Empty,
    Room,
    Field(Option<(Seed, usize)>),
    UnfencedStable(Option<(Animal, usize)>),
    FencedPasture(Mask, Option<(Animal, usize)>, ContainsStable),
}

// 00 01 02 03 04
// 05 06 07 08 09
// 10 11 12 13 14

// Order : NEWS
const NEIGHBOR_SPACES: [[Option<usize>; 4]; 15] = [
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

pub const MASK: [Mask; 4] = [8, 4, 2, 1];
const MASK_ALL: Mask = 15; // 8 + 4 + 2 + 1

#[derive(Debug, Clone, Hash)]
pub struct Farm {
    pub farmyard_spaces: [FarmyardSpace; NUM_FARMYARD_SPACES],
    pub pet: Option<(Animal, usize)>,
}

impl Farm {
    pub fn new() -> Self {
        let mut ret = [FarmyardSpace::Empty; NUM_FARMYARD_SPACES];
        ret[5] = FarmyardSpace::Room;
        ret[10] = FarmyardSpace::Room;
        Self {
            farmyard_spaces: ret,
            pet: None,
        }
    }

    fn candidate_pasture_spaces(&self) -> [bool; NUM_FARMYARD_SPACES] {
        let mut ret = [false; NUM_FARMYARD_SPACES];

        let mut pasture_spaces = 0;
        for idx in 0..NUM_FARMYARD_SPACES {
            if let FarmyardSpace::FencedPasture(_, _, _) = self.farmyard_spaces[idx] {
                pasture_spaces += 1;
            }
        }

        if pasture_spaces == 0 {
            for (idx, space) in self.farmyard_spaces.iter().enumerate() {
                match *space {
                    FarmyardSpace::Empty | FarmyardSpace::UnfencedStable(_) => ret[idx] = true,
                    _ => (),
                }
            }

            return ret;
        }

        for idx in 0..NUM_FARMYARD_SPACES {
            for nidx in NEIGHBOR_SPACES[idx].into_iter().flatten() {
                if let FarmyardSpace::FencedPasture(_, _, _) = self.farmyard_spaces[nidx] {
                    ret[idx] = true;
                }
            }
        }
        ret
    }

    fn pasture_position_score(&self, idx: usize) -> i32 {
        match self.farmyard_spaces[idx] {
            FarmyardSpace::Field(_) | FarmyardSpace::Room => return i32::MIN,
            _ => (),
        }

        let mut score: i32 = 0;
        for opt_i in NEIGHBOR_SPACES[idx] {
            match opt_i {
                Some(i) => match self.farmyard_spaces[i] {
                    FarmyardSpace::Empty => score += 1,
                    FarmyardSpace::FencedPasture(_, _, _) | FarmyardSpace::UnfencedStable(_) => {
                        score += 2
                    }
                    FarmyardSpace::Room | FarmyardSpace::Field(_) => score -= 1,
                },
                None => score += 2,
            }
        }
        score
    }

    fn surrounding_fences(&self, idx: usize) -> usize {
        let mut ret: usize = 0;
        if let FarmyardSpace::FencedPasture(mask, _, _) = self.farmyard_spaces[idx] {
            for i in 0..4 {
                if mask & MASK[i] > 0 {
                    ret += 1;
                }
            }
        }
        ret
    }

    fn neighbouring_fences(&self, idx: usize) -> usize {
        let mut ret: usize = 0;
        for (i, opt_nidx) in NEIGHBOR_SPACES[idx].iter().enumerate() {
            if let Some(nidx) = opt_nidx {
                if let FarmyardSpace::FencedPasture(mask, _, _) = self.farmyard_spaces[*nidx] {
                    // i is the direction, the opposite direction on the neighbor is 3 - i (NEWS)
                    // To check if a fence exists in that direction we simple bitwise AND with the mask
                    if MASK[3 - i] & mask > 0 {
                        ret += 1;
                    }
                }
            }
        }
        ret
    }

    pub fn fencing_options(&self, wood: usize) -> Vec<(Vec<usize>, usize)> {
        //let mut ret: Vec<Vec<usize>> = Vec::new();

        // Calculate all used fences
        // For all fenced pastures, count all their fences and neighboring fences, make sure a common fence is counted only once
        let mut neighbouring_fences: usize = 0;
        let mut surrounding_fences: usize = 0;
        for idx in 0..NUM_FARMYARD_SPACES {
            surrounding_fences += self.surrounding_fences(idx);
            neighbouring_fences += self.neighbouring_fences(idx);
        }

        let available_fences = wood.min(MAX_FENCES + neighbouring_fences - surrounding_fences);

        // Find single fences + wood, then for every new single fence, check other neighboring arrangements and form joint arrangements
        let mut arrangements: Vec<(Vec<usize>, usize)> = Vec::new();

        if available_fences == 0 {
            return arrangements;
        }

        let candidate_pasture_spaces = self.candidate_pasture_spaces();
        let mut mask = 0;

        for idx in 0..NUM_FARMYARD_SPACES {
            let mut can_use = 0;

            if !candidate_pasture_spaces[idx] {
                continue;
            }

            match self.farmyard_spaces[idx] {
                FarmyardSpace::Empty | FarmyardSpace::UnfencedStable(_) => {
                    // This space can be fenced
                    // Check how many neighboring spaces are fenced already
                    can_use = self.neighbouring_fences(idx);
                }
                FarmyardSpace::FencedPasture(mask_, _, _) => {
                    can_use = self.surrounding_fences(idx);
                    mask = mask_;
                }
                _ => (),
            }

            // If available_fences are not enough for this single pasture
            if can_use + available_fences < 4 {
                continue;
            }

            // Find joint fence arrangements with arrangements found earlier
            let mut new_arrangements: Vec<(Vec<usize>, usize)> = Vec::new();
            // check other arrangements for joint fencing
            for (other, w) in &arrangements {
                let mut connected_sides: usize = 0;
                for (ni, opt_nidx) in NEIGHBOR_SPACES[idx].iter().enumerate() {
                    if let Some(nidx) = opt_nidx {
                        // If fence is not present in self and present in the neighbor, it's a connecting side
                        if mask & MASK[ni] == 0 && other.contains(nidx) {
                            connected_sides += 1;
                        }
                    }
                }

                // Connected sides need not be fenced
                if connected_sides > 0 {
                    if *w + 4 < can_use + 2 * connected_sides {
                        println!("{:?}", self.farmyard_spaces);
                        println!("Was checking for idx {idx} Can use {can_use} Conn {connected_sides} Arr {:?}", other);
                    }
                    assert!(*w + 4 >= can_use + 2 * connected_sides);
                    let mut joint_idxs = other.clone();
                    joint_idxs.push(idx);
                    let joint_num_wood = *w + 4 - can_use - 2 * connected_sides;

                    if joint_num_wood <= available_fences {
                        new_arrangements.push((joint_idxs, joint_num_wood))
                    }
                }
            }

            arrangements.push((vec![idx], 4 - can_use));
            if !new_arrangements.is_empty() {
                arrangements.append(&mut new_arrangements);
            }
        }

        let mut scores: Vec<i32> = Vec::new();
        let mut best_scores_by_num_pastures: [i32; NUM_FARMYARD_SPACES] =
            [i32::MIN; NUM_FARMYARD_SPACES];
        let mut best_scores_by_num_wood: [i32; MAX_FENCES] = [i32::MIN; MAX_FENCES];

        for (arr, w) in &arrangements {
            let mut score: i32 = 0;

            for x in arr {
                score += self.pasture_position_score(*x);
            }

            scores.push(score);

            best_scores_by_num_pastures[arr.len()] =
                score.max(best_scores_by_num_pastures[arr.len()]);
            best_scores_by_num_wood[*w] = score.max(best_scores_by_num_wood[*w]);
        }

        let mut best_arrangements: Vec<(Vec<usize>, usize)> = Vec::new();

        for (i, (arr, w)) in arrangements.iter().enumerate() {
            if scores[i] == best_scores_by_num_pastures[arr.len()]
                || scores[i] == best_scores_by_num_wood[*w]
            {
                best_arrangements.push(arrangements[i].clone());
            }
        }

        //println!("{:?}", best_arrangements);
        best_arrangements
    }

    pub fn fence_spaces(&mut self, spaces: &Vec<usize>) {
        for space in spaces {
            let mut mask: usize = MASK_ALL;
            for (i, opt_nidx) in NEIGHBOR_SPACES[*space].iter().enumerate() {
                if let Some(nidx) = opt_nidx {
                    if spaces.contains(nidx) {
                        mask -= MASK[i]
                    }
                }
            }

            let space_type = self.farmyard_spaces[*space];
            match space_type {
                FarmyardSpace::Empty => {
                    self.farmyard_spaces[*space] = FarmyardSpace::FencedPasture(mask, None, false)
                }
                FarmyardSpace::UnfencedStable(animal) => {
                    self.farmyard_spaces[*space] = FarmyardSpace::FencedPasture(mask, animal, true)
                }
                FarmyardSpace::FencedPasture(_, animal, has_stable) => {
                    self.farmyard_spaces[*space] =
                        FarmyardSpace::FencedPasture(mask, animal, has_stable)
                }
                _ => (),
            }
        }
    }

    pub fn pastures_and_capacities(&self) -> Vec<(Vec<usize>, usize)> {
        let mut unfenced_stables: Vec<usize> = Vec::new(); // pet
        let mut pasture_idx = [i32::MAX; NUM_FARMYARD_SPACES];
        let mut stables: [bool; NUM_FARMYARD_SPACES] = [false; NUM_FARMYARD_SPACES];
        for idx in 0..NUM_FARMYARD_SPACES {
            match self.farmyard_spaces[idx] {
                FarmyardSpace::UnfencedStable(_) => unfenced_stables.push(idx),
                FarmyardSpace::FencedPasture(mask1, _, has_stable) => {
                    pasture_idx[idx] = pasture_idx[idx].min(idx as i32);
                    stables[idx] = has_stable;
                    for (i, opt_ni) in NEIGHBOR_SPACES[idx].iter().enumerate() {
                        if let Some(ni) = opt_ni {
                            if let FarmyardSpace::FencedPasture(mask2, _, _) =
                                self.farmyard_spaces[*ni]
                            {
                                // No fence between these two spaces -> they are part of the same pasture
                                if mask1 & MASK[i] == 0 && mask2 & MASK[3 - i] == 0 {
                                    // Assign the smallest index of all spaces in a pasture as the pasture index
                                    pasture_idx[*ni] = pasture_idx[idx].min(pasture_idx[*ni]);
                                }
                            }
                        }
                    }
                }
                _ => (),
            }
        }

        // calculate capacity
        let mut pastures: HashMap<i32, Vec<usize>> = HashMap::new();
        for (i, pi) in pasture_idx.iter().enumerate() {
            if *pi == i32::MAX {
                continue;
            }
            let values = pastures.entry(*pi).or_insert(Vec::new());
            values.push(i);
        }

        let mut ret: Vec<(Vec<usize>, usize)> = Vec::new();
        // Each UF stable can hold one animal (without card modifications)
        if !unfenced_stables.is_empty() {
            let l = unfenced_stables.len();
            ret.push((unfenced_stables, l));
        }

        for (_, v) in pastures {
            let stables = v.iter().filter(|&x| stables[*x]).count();
            let stable_multiplier = if stables > 0 {
                STABLE_MULTIPLIER * stables
            } else {
                1
            };
            let capacity = PASTURE_CAPACITY * v.len() * stable_multiplier;
            ret.push((v, capacity));
        }

        // sort according to capacity as HashMap does not guarantee ordering
        ret.sort_by(|a, b| b.1.cmp(&a.1));

        ret
    }

    pub fn farm_animals_to_resources(&mut self, res: &mut Resources) {
        // Add Pet
        if let Some((pet, amount)) = self.pet {
            match pet {
                Animal::Sheep => res[Resource::Sheep] += amount,
                Animal::Pigs => res[Resource::Pigs] += amount,
                Animal::Cattle => res[Resource::Cattle] += amount,
            }
        }

        self.pet = None;

        // Add animals from farm
        for fs in &mut self.farmyard_spaces {
            match fs {
                FarmyardSpace::UnfencedStable(animals)
                | FarmyardSpace::FencedPasture(_, animals, _) => {
                    if let Some((animal, amt)) = animals {
                        match *animal {
                            Animal::Sheep => res[Resource::Sheep] += *amt,
                            Animal::Pigs => res[Resource::Pigs] += *amt,
                            Animal::Cattle => res[Resource::Cattle] += *amt,
                        }
                        *animals = None
                    }
                }
                _ => (),
            }
        }
    }

    pub fn reorg_animals(&mut self, res: &Resources, breed: bool) -> Resources {
        let mut res = *res;

        self.farm_animals_to_resources(&mut res);

        let mut animal_count = vec![
            (Animal::Sheep, res[Resource::Sheep]),
            (Animal::Pigs, res[Resource::Pigs]),
            (Animal::Cattle, res[Resource::Cattle]),
        ];

        if breed {
            for (_, c) in &mut animal_count {
                if *c > 1 {
                    *c += 1;
                }
            }
        }

        //println!("{:?}", animal_count);

        let pastures_and_capacities = self.pastures_and_capacities();

        //println!("{:?}", pastures_and_capacities);

        for pac in pastures_and_capacities {
            animal_count.sort_by(|a, b| b.1.cmp(&a.1));

            if animal_count[0].1 == 0 {
                break;
            }

            let animal_capacity_per_pasture = pac.1 / pac.0.len();
            let animal = animal_count[0].0;
            let count = &mut animal_count[0].1;
            for idx in pac.0 {
                let num_animals_to_place = animal_capacity_per_pasture.min(*count);
                match self.farmyard_spaces[idx] {
                    FarmyardSpace::FencedPasture(mask, _, has_stable) => {
                        self.farmyard_spaces[idx] = FarmyardSpace::FencedPasture(
                            mask,
                            Some((animal, num_animals_to_place)),
                            has_stable,
                        );
                        *count -= num_animals_to_place;
                    }
                    FarmyardSpace::UnfencedStable(_) => {
                        self.farmyard_spaces[idx] =
                            FarmyardSpace::UnfencedStable(Some((animal, num_animals_to_place)));
                        *count -= num_animals_to_place;
                    }
                    _ => (),
                }
                if *count == 0 {
                    break;
                }
            }
        }

        // One more can go in to the house as a pet
        animal_count.sort_by(|a, b| b.1.cmp(&a.1));
        if animal_count[0].1 > 0 {
            self.pet = Some((animal_count[0].0, 1));
            animal_count[0].1 -= 1;
        }

        // Leftovers are returned
        for (animal, amt) in animal_count {
            match animal {
                Animal::Sheep => res[Resource::Sheep] = amt,
                Animal::Pigs => res[Resource::Pigs] = amt,
                Animal::Cattle => res[Resource::Cattle] = amt,
            }
        }

        res
    }

    pub fn best_field_positions(&self) -> Vec<usize> {
        let mut pos_and_scores: Vec<(usize, i32)> = Vec::new();

        // check if there are any fields, if not, all empty spaces must be scored
        // if not, only empty spaces adjacent to fields must be scored

        // get all field indices
        let field_idxs = self.field_indices();

        let mut max_score = i32::MIN;
        for (idx, nspace) in NEIGHBOR_SPACES.iter().enumerate() {
            if self.farmyard_spaces[idx] != FarmyardSpace::Empty {
                continue;
            }

            if !field_idxs.is_empty() {
                let mut has_adjacent_field = false;
                for nidx in nspace.iter().flatten() {
                    match self.farmyard_spaces[*nidx] {
                        FarmyardSpace::Field(_) => {
                            has_adjacent_field = true;
                            break;
                        }
                        _ => (),
                    }
                }

                if !has_adjacent_field {
                    continue;
                }
            }

            let mut score: i32 = 0;
            for opt_i in nspace {
                match opt_i {
                    Some(i) => match self.farmyard_spaces[*i] {
                        FarmyardSpace::Empty => score += 1,
                        FarmyardSpace::Field(_) => score += 2,
                        _ => score -= 1,
                    },
                    None => score += 2,
                }
            }

            max_score = max_score.max(score);
            pos_and_scores.push((idx, score));
        }

        let mut ret: Vec<usize> = Vec::new();
        for (i, s) in pos_and_scores {
            if s == max_score {
                ret.push(i);
            }
        }
        ret
    }

    pub fn field_indices(&self) -> Vec<usize> {
        (0..NUM_FARMYARD_SPACES)
            .filter(|&i| matches!(self.farmyard_spaces[i], FarmyardSpace::Field(_)))
            .collect()
    }

    pub fn best_room_positions(&self) -> Vec<usize> {
        let mut pos_and_scores: Vec<(usize, i32)> = Vec::new();
        let mut max_score = i32::MIN;

        for (idx, nspace) in NEIGHBOR_SPACES.iter().enumerate() {
            if self.farmyard_spaces[idx] != FarmyardSpace::Empty {
                continue;
            }

            let mut has_adjacent_room = false;
            for nidx in nspace.iter().flatten() {
                if self.farmyard_spaces[*nidx] == FarmyardSpace::Room {
                    has_adjacent_room = true;
                    break;
                }
            }

            if !has_adjacent_room {
                continue;
            }

            let mut score: i32 = 0;
            for opt_i in nspace {
                match opt_i {
                    Some(i) => match self.farmyard_spaces[*i] {
                        FarmyardSpace::Empty => score += 1,
                        FarmyardSpace::Room => score += 2,
                        _ => score -= 1,
                    },
                    None => score += 2,
                }
            }

            max_score = max_score.max(score);
            pos_and_scores.push((idx, score));
        }

        //println!("{:?}", pos_and_scores);

        let mut ret: Vec<usize> = Vec::new();
        for (i, s) in pos_and_scores {
            if s == max_score {
                ret.push(i);
            }
        }
        ret
    }

    pub fn best_stable_positions(&self) -> Vec<usize> {
        let mut pos_and_scores: Vec<(usize, i32)> = Vec::new();
        let mut max_score = i32::MIN;

        for (idx, nspace) in NEIGHBOR_SPACES.iter().enumerate() {
            let available = matches!(
                self.farmyard_spaces[idx],
                FarmyardSpace::Empty | FarmyardSpace::FencedPasture(_, _, false)
            );

            if !available {
                continue;
            }

            let mut score: i32 = 0;
            for opt_i in nspace {
                match opt_i {
                    Some(i) => match self.farmyard_spaces[*i] {
                        FarmyardSpace::Empty => score += 1,
                        FarmyardSpace::FencedPasture(_, _, _)
                        | FarmyardSpace::UnfencedStable(_) => score += 3,
                        _ => score -= 1,
                    },
                    None => score += 2,
                }
            }

            max_score = max_score.max(score);
            pos_and_scores.push((idx, score));
        }

        //println!("{:?}", pos_and_scores);

        let mut ret: Vec<usize> = Vec::new();
        for (i, s) in pos_and_scores {
            if s == max_score {
                ret.push(i);
            }
        }
        ret
    }

    pub fn room_indices(&self) -> Vec<usize> {
        (0..NUM_FARMYARD_SPACES)
            .filter(|&i| matches!(self.farmyard_spaces[i], FarmyardSpace::Room))
            .collect()
    }

    pub fn add_field(&mut self, idx: usize) {
        assert!(self.farmyard_spaces[idx] == FarmyardSpace::Empty);
        self.farmyard_spaces[idx] = FarmyardSpace::Field(None);
    }

    pub fn build_room(&mut self, idx: usize) {
        assert!(self.farmyard_spaces[idx] == FarmyardSpace::Empty);
        self.farmyard_spaces[idx] = FarmyardSpace::Room;
    }

    pub fn build_stable(&mut self, idx: usize) {
        let available = matches!(
            self.farmyard_spaces[idx],
            FarmyardSpace::Empty | FarmyardSpace::FencedPasture(_, _, false)
        );
        assert!(available);
        assert!(self.can_build_stable());

        match self.farmyard_spaces[idx] {
            FarmyardSpace::Empty => self.farmyard_spaces[idx] = FarmyardSpace::UnfencedStable(None),
            FarmyardSpace::FencedPasture(mask, animal, false) => {
                self.farmyard_spaces[idx] = FarmyardSpace::FencedPasture(mask, animal, true)
            }
            _ => (),
        }
    }

    pub fn can_build_stable(&self) -> bool {
        let mut num_stables = 0;
        let mut candidate_spaces = 0;
        for fs in &self.farmyard_spaces {
            match *fs {
                FarmyardSpace::UnfencedStable(_) | FarmyardSpace::FencedPasture(_, _, true) => {
                    num_stables += 1
                }
                FarmyardSpace::Empty | FarmyardSpace::FencedPasture(_, _, false) => {
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
            .find(|f| matches!(f, FarmyardSpace::Field(_)));
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
    use crate::agricola::primitives::new_res;

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn mask_test() {
        assert_eq!(1 & 2, 0);
        assert_eq!(1 | 2 | 4, 7);
        let mask = 15;
        let sides: i32 = (0..4).map(|i| if mask & MASK[i] > 0 { 1 } else { 0 }).sum();
        assert_eq!(sides, 4);
    }

    #[test]
    fn test_fencing_options() {
        let farm = Farm::new();
        let fenc_opt = farm.fencing_options(3);
        assert_eq!(fenc_opt.len(), 0);
        let fenc_opt = farm.fencing_options(4);
        assert_eq!(fenc_opt.len(), 2);
        let fenc_opt = farm.fencing_options(6);
        assert_eq!(fenc_opt.len(), 6);
        let fenc_opt = farm.fencing_options(8);
        assert_eq!(fenc_opt.len(), 9);
        let fenc_opt = farm.fencing_options(MAX_FENCES);
        assert_eq!(fenc_opt.len(), 21);
    }

    #[test]
    fn test_field_options() {
        let farm = Farm::new();
        let field_opt = farm.best_field_positions();
        //println!("{:?}", field_opt);
        assert_eq!(field_opt.len(), 2);
    }

    #[test]
    fn test_room_options() {
        let farm = Farm::new();
        let room_opt = farm.best_room_positions();
        //println!("{:?}", room_opt);
        assert_eq!(room_opt.len(), 1);
    }

    #[test]
    fn test_pastures_and_capacities() {
        let mut farm = Farm::new();
        farm.fence_spaces(&vec![3, 4, 8, 9]);
        farm.fence_spaces(&vec![8]);
        farm.build_stable(4);
        farm.build_stable(8);
        let pac = farm.pastures_and_capacities();
        assert_eq!(pac.len(), 2); // includes independent capacity (as pet, unfenced stables etc)
        assert_eq!(pac[1].0, vec![8]);
        assert_eq!(pac[1].1, 4);
        assert_eq!(pac[0].0, vec![3, 4, 9]);
        assert_eq!(pac[0].1, 12);
    }

    #[test]
    fn test_reorg_animals() {
        let mut farm = Farm::new();
        let mut res: Resources = new_res();
        farm.fence_spaces(&vec![3, 4, 8, 9]);
        farm.fence_spaces(&vec![8]);
        farm.build_stable(13);
        farm.build_stable(3);
        res[Resource::Sheep] = 4;
        let mut leftovers = farm.reorg_animals(&res, false);
        assert_eq!(leftovers[Resource::Sheep], 0);
        println!("{:?}", farm);
        res[Resource::Sheep] = 0;
        res[Resource::Cattle] = 14;
        leftovers = farm.reorg_animals(&res, false);
        println!("{:?}", farm);
        assert_eq!(leftovers[Resource::Cattle], 1);
        assert_eq!(leftovers[Resource::Sheep], 1);
    }
}
