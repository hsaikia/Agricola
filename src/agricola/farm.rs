use super::primitives::*;
use std::{collections::HashMap, collections::VecDeque, hash::Hash};

const L: usize = 5;
const W: usize = 3;
pub const NUM_FARMYARD_SPACES: usize = L * W;
pub const MAX_FENCES: usize = 15;
pub const MAX_STABLES: usize = 4;
const PASTURE_CAPACITY: usize = 2;
const STABLE_MULTIPLIER: usize = 2;
const NUM_FENCE_INDICES: usize = 77; // (5 + 6) * (3 + 4)

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
    Cow,
}

type ContainsStable = bool;

#[derive(Copy, Clone, Debug, Default, Hash, PartialEq)]
pub enum FarmyardSpace {
    #[default]
    Empty,
    Room,
    Field(Option<(Seed, usize)>),
    UnfencedStable(Option<(Animal, usize)>),
    FencedPasture(Option<(Animal, usize)>, ContainsStable),
}

// Farmyard spaces
// 00 01 02 03 04
// 05 06 07 08 09
// 10 11 12 13 14

// Order : NEWS
const NEIGHBOR_SPACES: [[Option<usize>; 4]; NUM_FARMYARD_SPACES] = [
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

// Fence positions
// 00  01  02  03  04  05  06  07  08  09  10
// 11 [12] 13 [14] 15 [16] 17 [18] 19 [20] 21
// 22  23  24  25  26  27  28  29  30  31  32
// 33 [34] 35 [36] 37 [38] 39 [40] 41 [42] 43
// 44  45  46  47  48  49  50  51  52  53  54
// 55 [56] 57 [58] 59 [60] 61 [62] 63 [64] 65
// 66  67  68  69  70  71  72  73  74  75  76

// Order : NEWS
pub const FENCE_INDICES: [[usize; 4]; NUM_FARMYARD_SPACES] = [
    [1, 13, 11, 23],
    [3, 15, 13, 25],
    [5, 17, 15, 27],
    [7, 19, 17, 29],
    [9, 21, 19, 31],
    [23, 35, 33, 45],
    [25, 37, 35, 47],
    [27, 39, 37, 49],
    [29, 41, 39, 51],
    [31, 43, 41, 53],
    [45, 57, 55, 67],
    [47, 59, 57, 69],
    [49, 61, 59, 71],
    [51, 63, 61, 73],
    [53, 65, 63, 75],
];

#[derive(Debug, Clone, Hash)]
pub struct Farm {
    pub farmyard_spaces: [FarmyardSpace; NUM_FARMYARD_SPACES],
    pub fences: [bool; NUM_FENCE_INDICES], // (5 + 6) * (3 + 4)
    pub pet: Option<(Animal, usize)>,
}

impl Default for Farm {
    fn default() -> Self {
        Self::new()
    }
}

impl Farm {
    pub fn new() -> Self {
        let mut farmyard_spaces = [FarmyardSpace::Empty; NUM_FARMYARD_SPACES];
        farmyard_spaces[5] = FarmyardSpace::Room;
        farmyard_spaces[10] = FarmyardSpace::Room;

        Self {
            farmyard_spaces,
            fences: [false; NUM_FENCE_INDICES],
            pet: None,
        }
    }

    pub fn format_fence_layout(layout: &[usize]) -> String {
        let mut ret: String = String::new();
        let l = 2 * L + 1;
        let w = 2 * W + 1;
        for i in 0..w {
            for j in 0..l {
                let idx = i * l + j;
                if i % 2 == 0 {
                    if j % 2 == 0 {
                        ret = format!("{ret}+");
                    } else if layout.contains(&idx) {
                        ret = format!("{ret} - ");
                    } else {
                        ret = format!("{ret}   ");
                    }
                } else if j % 2 == 0 {
                    if layout.contains(&idx) {
                        ret = format!("{ret}|");
                    } else {
                        ret = format!("{ret} ");
                    }
                } else {
                    ret = format!("{ret}   ");
                }
            }
            ret = format!("{ret}\n");
        }
        ret = format!("{ret}\n\n");
        ret
    }

    fn available_fence_spots(&self) -> [bool; NUM_FENCE_INDICES] {
        // Mark all to false
        let mut available_fence_spots = [false; NUM_FENCE_INDICES];
        for (i, space) in self.farmyard_spaces.iter().enumerate() {
            match space {
                FarmyardSpace::Empty
                | FarmyardSpace::FencedPasture(_, _)
                | FarmyardSpace::UnfencedStable(_) => {
                    // Mark all unused spots next to empty spaces, fenced spaces, and spaces with unfenced stables
                    for spot in FENCE_INDICES[i] {
                        if !self.fences[spot] {
                            available_fence_spots[spot] = true;
                        }
                    }
                }
                _ => (),
            }
        }
        available_fence_spots
    }

    fn fence_graph() -> HashMap<usize, Vec<(usize, usize)>> {
        let mut ret: HashMap<usize, Vec<(usize, usize)>> = HashMap::new();
        let l = 2 * L + 1;
        let w = 2 * W + 1;

        // Horizontal connections
        for i in 0..w {
            if i % 2 == 1 {
                continue;
            }
            for j in 0..l - 2 {
                if j % 2 == 1 {
                    continue;
                }

                let edge_idx = i * l + j + 1;
                let idx_from = i * l + j;
                let idx_to = i * l + j + 2;

                ret.entry(idx_from).or_default().push((idx_to, edge_idx));
                ret.entry(idx_to).or_default().push((idx_from, edge_idx));
            }
        }

        // Vertical connections
        for i in 0..w - 2 {
            if i % 2 == 1 {
                continue;
            }
            for j in 0..l {
                if j % 2 == 1 {
                    continue;
                }

                let edge_idx = i * l + j + l;
                let idx_from = i * l + j;
                let idx_to = (i + 2) * l + j;

                ret.entry(idx_from).or_default().push((idx_to, edge_idx));
                ret.entry(idx_to).or_default().push((idx_from, edge_idx));
            }
        }
        ret
    }

    pub fn fencing_option_score(&self) -> [i32; NUM_FENCE_INDICES] {
        let mut ret = [0; NUM_FENCE_INDICES];
        let mut count = [0; NUM_FENCE_INDICES];
        for idx in 0..NUM_FARMYARD_SPACES {
            match self.farmyard_spaces[idx] {
                FarmyardSpace::Empty
                | FarmyardSpace::UnfencedStable(_)
                | FarmyardSpace::FencedPasture(_, _) => {
                    for i in 0..4 {
                        ret[FENCE_INDICES[idx][i]] += 1;
                        count[FENCE_INDICES[idx][i]] += 1;
                    }
                }
                FarmyardSpace::Room | FarmyardSpace::Field(_) => {
                    for i in 0..4 {
                        ret[FENCE_INDICES[idx][i]] -= 1;
                        count[FENCE_INDICES[idx][i]] += 1;
                    }
                }
            }
        }
        for idx in 0..NUM_FENCE_INDICES {
            ret[idx] += 3 * (count[idx] % 2);
        }
        ret
    }

    // Starts a BFS from nodes (corners of farmyard spaces) and outputs all paths that end at terminal nodes (nodes that are connected to at least one fence)
    pub fn fencing_options(&self, wood: usize) -> Vec<Vec<usize>> {
        let graph = Self::fence_graph();
        let available_fence_spots = self.available_fence_spots();
        let total_fences_used = self.total_fences_used();
        let available_wood = wood.min(MAX_FENCES - total_fences_used);
        let fencing_option_scores = self.fencing_option_score();

        //println!("{graph:?}");
        //println!("Total fences used {}", self.total_fences_used());

        // (to, from, edge)
        let mut queue: VecDeque<(usize, Vec<(usize, usize)>)> = VecDeque::new();
        let mut terminal_nodes: Vec<usize> = Vec::new();

        // Check for terminal nodes (i.e., whether there are fences already)
        for (k, v) in &graph {
            // Add possible terminal nodes
            //println!("{k} => {v:?}");
            if v.iter().any(|&x| self.fences[x.1]) {
                //println!("{k} is a terminal node");
                terminal_nodes.push(*k);
            }
        }

        //println!("TNs {terminal_nodes:?}");

        if terminal_nodes.is_empty() {
            // No fences, add all nodes to the queue
            for (k, v) in &graph {
                for elem in v {
                    if !available_fence_spots[elem.1] {
                        continue;
                    }
                    queue.push_back((*k, vec![*elem]));
                }
            }
        } else {
            // Only add nodes that are terminal
            for k in &terminal_nodes {
                for elem in &graph[k] {
                    if !available_fence_spots[elem.1] {
                        continue;
                    }
                    queue.push_back((*k, vec![*elem]));
                }
            }
        }

        // Map of wood -> (arrangement, score)
        let mut best_arrangements: HashMap<usize, (Vec<Vec<usize>>, i32)> = HashMap::new();

        while !queue.is_empty() {
            let top = queue.pop_front();
            if let Some((idx1, v)) = top {
                if v.len() > available_wood {
                    continue;
                }

                let mut arr: Vec<usize> = Vec::new();

                // Starting and ending at the same terminal node - does not share an exisiting fence => Invalid
                if v.iter()
                    .any(|&y| y.0 == idx1 && terminal_nodes.contains(&y.0))
                {
                    continue;
                }

                if v.iter()
                    .any(|&y| y.0 == idx1 || terminal_nodes.contains(&y.0))
                {
                    // Found a loop, this is a valid fence arrangement
                    // Add all fence positions from the start up until this point
                    for elem in &v {
                        arr.push(elem.1);
                        if elem.0 == idx1 || terminal_nodes.contains(&elem.0) {
                            break;
                        }
                    }
                }

                if !arr.is_empty() {
                    arr.sort();

                    // score the arrangement
                    let score = arr.iter().map(|&x| fencing_option_scores[x]).sum();
                    //println!("Arr {arr:?} scored {score}");

                    let w: usize = arr.len();

                    if let Some(val) = best_arrangements.get_mut(&w) {
                        if val.1 == score && !val.0.contains(&arr) {
                            val.0.push(arr);
                        } else if val.1 < score {
                            best_arrangements.insert(w, (vec![arr], score));
                        }
                    } else {
                        best_arrangements.insert(w, (vec![arr], score));
                    }
                } else if let Some((last_idx, e)) = v.last() {
                    for neighbors in &graph[last_idx] {
                        if !available_fence_spots[neighbors.1] {
                            continue;
                        }

                        // Skip if edge is the same
                        if *e == neighbors.1 {
                            continue;
                        }

                        // Don't proceed if node was already seen in the sequence
                        if v.iter().any(|&x| x.0 == neighbors.0) {
                            continue;
                        }

                        // Add the node to the sequence to make a new sequence
                        let mut vv = v.clone();
                        vv.push(*neighbors);
                        queue.push_back((idx1, vv));
                    }
                }
            }
        }

        // Add all the best arrangements
        let mut arrangements: Vec<Vec<usize>> = Vec::new();
        for (_, v) in best_arrangements {
            arrangements.extend(v.0);
        }

        arrangements.sort();
        arrangements
    }

    fn total_fences_used(&self) -> usize {
        self.fences.iter().filter(|&x| *x).count()
    }

    fn mark_fenced_spaces(&self) -> [bool; NUM_FARMYARD_SPACES] {
        let mut fenced_spaces: [bool; NUM_FARMYARD_SPACES] = [true; NUM_FARMYARD_SPACES];
        // Start at the room space 5
        let mut queue: VecDeque<usize> = VecDeque::new();
        queue.push_back(5);
        while !queue.is_empty() {
            let top = queue.pop_front();
            if let Some(idx) = top {
                if !fenced_spaces[idx] {
                    continue;
                }
                fenced_spaces[idx] = false;

                for i in 0..4 {
                    if let Some(nidx) = NEIGHBOR_SPACES[idx][i] {
                        if !self.fences[FENCE_INDICES[idx][i]] {
                            queue.push_back(nidx);
                        }
                    }
                }
            }
        }
        fenced_spaces
    }

    pub fn fence_spaces(&mut self, fence_spots: &Vec<usize>) {
        for spot in fence_spots {
            self.fences[*spot] = true;
        }

        let fenced_farmyard_spaces = self.mark_fenced_spaces();

        for (idx, space) in fenced_farmyard_spaces.iter().enumerate() {
            if !space {
                continue;
            }
            match self.farmyard_spaces[idx] {
                FarmyardSpace::Empty => {
                    self.farmyard_spaces[idx] = FarmyardSpace::FencedPasture(None, false)
                }
                FarmyardSpace::UnfencedStable(animal) => {
                    self.farmyard_spaces[idx] = FarmyardSpace::FencedPasture(animal, true)
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
                FarmyardSpace::FencedPasture(_, has_stable) => {
                    pasture_idx[idx] = pasture_idx[idx].min(idx as i32);
                    stables[idx] = has_stable;
                    for (i, opt_ni) in NEIGHBOR_SPACES[idx].iter().enumerate() {
                        if let Some(ni) = opt_ni {
                            if let FarmyardSpace::FencedPasture(_, _) = self.farmyard_spaces[*ni] {
                                // No fence between these two spaces -> they are part of the same pasture
                                if !self.fences[FENCE_INDICES[idx][i]] {
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
            let values = pastures.entry(*pi).or_default();
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
                Animal::Sheep => res[Sheep.index()] += amount,
                Animal::Boar => res[Boar.index()] += amount,
                Animal::Cow => res[Cow.index()] += amount,
            }
        }

        self.pet = None;

        // Add animals from farm
        for fs in &mut self.farmyard_spaces {
            match fs {
                FarmyardSpace::UnfencedStable(animals)
                | FarmyardSpace::FencedPasture(animals, _) => {
                    if let Some((animal, amt)) = animals {
                        match *animal {
                            Animal::Sheep => res[Sheep.index()] += *amt,
                            Animal::Boar => res[Boar.index()] += *amt,
                            Animal::Cow => res[Cow.index()] += *amt,
                        }
                        *animals = None
                    }
                }
                _ => (),
            }
        }
    }

    pub fn reorg_animals(&mut self, res: &mut Resources, breed: bool) {
        self.farm_animals_to_resources(res);

        let mut animal_count = vec![
            (Animal::Sheep, res[Sheep.index()]),
            (Animal::Boar, res[Boar.index()]),
            (Animal::Cow, res[Cow.index()]),
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
                    FarmyardSpace::FencedPasture(_, has_stable) => {
                        self.farmyard_spaces[idx] = FarmyardSpace::FencedPasture(
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

        // Leftovers are put back in the resources array
        for (animal, amt) in animal_count {
            match animal {
                Animal::Sheep => res[Sheep.index()] = amt,
                Animal::Boar => res[Boar.index()] = amt,
                Animal::Cow => res[Cow.index()] = amt,
            }
        }
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
                    if let FarmyardSpace::Field(_) = self.farmyard_spaces[*nidx] {
                        has_adjacent_field = true;
                        break;
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
                FarmyardSpace::Empty | FarmyardSpace::FencedPasture(_, false)
            );

            if !available {
                continue;
            }

            let mut score: i32 = 0;
            for opt_i in nspace {
                match opt_i {
                    Some(i) => match self.farmyard_spaces[*i] {
                        FarmyardSpace::Empty => score += 1,
                        FarmyardSpace::FencedPasture(_, _) | FarmyardSpace::UnfencedStable(_) => {
                            score += 3
                        }
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
            FarmyardSpace::Empty | FarmyardSpace::FencedPasture(_, false)
        );
        assert!(available);
        assert!(self.can_build_stable());

        match self.farmyard_spaces[idx] {
            FarmyardSpace::Empty => self.farmyard_spaces[idx] = FarmyardSpace::UnfencedStable(None),
            FarmyardSpace::FencedPasture(animal, false) => {
                self.farmyard_spaces[idx] = FarmyardSpace::FencedPasture(animal, true)
            }
            _ => (),
        }
    }

    pub fn can_build_stable(&self) -> bool {
        let mut num_stables = 0;
        let mut candidate_spaces = 0;
        for fs in &self.farmyard_spaces {
            match *fs {
                FarmyardSpace::UnfencedStable(_) | FarmyardSpace::FencedPasture(_, true) => {
                    num_stables += 1
                }
                FarmyardSpace::Empty | FarmyardSpace::FencedPasture(_, false) => {
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
    use crate::agricola::primitives::new_res;

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_fencing_options() {
        let mut farm = Farm::new();
        let fenc_opt = farm.fencing_options(3);
        assert_eq!(fenc_opt.len(), 0);
        let fenc_opt = farm.fencing_options(4);
        assert_eq!(fenc_opt.len(), 2);
        let fenc_opt = farm.fencing_options(6);
        assert_eq!(fenc_opt.len(), 6);
        let fenc_opt = farm.fencing_options(8);
        assert_eq!(fenc_opt.len(), 7);
        let fenc_opt = farm.fencing_options(MAX_FENCES);
        assert_eq!(fenc_opt.len(), 10);

        farm.fence_spaces(&vec![9, 19, 21, 31]);
        let fenc_opt = farm.fencing_options(5);
        println!("{fenc_opt:?}");
        assert_eq!(fenc_opt.len(), 3);

        farm = Farm::new();
        farm.fence_spaces(&vec![9, 19, 21, 41, 43, 53]);
        let fenc_opt = farm.fencing_options(1);
        assert_eq!(fenc_opt.len(), 1);
    }

    #[test]
    fn test_fence_functions() {
        let mut farm = Farm::new();

        farm.fences[FENCE_INDICES[3][0]] = true;
        farm.fences[FENCE_INDICES[3][1]] = true;
        farm.fences[FENCE_INDICES[3][2]] = true;
        farm.fences[FENCE_INDICES[3][3]] = true;
        farm.fences[FENCE_INDICES[4][0]] = true;
        farm.fences[FENCE_INDICES[4][1]] = true;
        farm.fences[FENCE_INDICES[4][2]] = true;
        farm.fences[FENCE_INDICES[9][1]] = true;
        farm.fences[FENCE_INDICES[9][2]] = true;
        farm.fences[FENCE_INDICES[9][3]] = true;

        farm.farmyard_spaces[3] = FarmyardSpace::FencedPasture(None, false);
        farm.farmyard_spaces[4] = FarmyardSpace::FencedPasture(None, false);
        farm.farmyard_spaces[9] = FarmyardSpace::FencedPasture(None, false);

        let fences_used = farm.total_fences_used();
        assert_eq!(fences_used, 9);
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
        farm.fence_spaces(&vec![7, 9, 17, 21, 39, 43, 51, 53]);
        farm.fence_spaces(&vec![29, 39, 41, 51]);
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
        farm.fence_spaces(&vec![7, 9, 17, 21, 39, 43, 51, 53]);
        farm.fence_spaces(&vec![29, 39, 41, 51]);
        farm.build_stable(13);
        farm.build_stable(3);
        res[Sheep.index()] = 4;
        farm.reorg_animals(&mut res, false);
        assert_eq!(res[Sheep.index()], 0);
        println!("{:?}", farm);
        res[Sheep.index()] = 0;
        res[Cow.index()] = 14;
        farm.reorg_animals(&mut res, false);
        println!("{:?}", farm);
        assert_eq!(res[Cow.index()], 1);
        assert_eq!(res[Sheep.index()], 1);
    }
}
