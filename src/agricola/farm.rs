pub const NUM_FARMYARD_SPACES: usize = 15;
pub const MAX_FENCES: usize = 15;

#[derive(Copy, Clone, Hash)]
pub enum House {
    Wood,
    Clay,
    Stone,
}

pub fn house_emoji(house: &House) -> &str {
    match house {
        House::Wood => "\u{1f6d6}",
        House::Clay => "\u{1f3e0}",
        House::Stone => "\u{1f3f0}",
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq)]
pub enum Seed {
    Grain,
    Vegetable,
}

fn seed_emoji(seed: &Seed) -> &str {
    match seed {
        Seed::Grain => "\u{1f33e}",
        Seed::Vegetable => "\u{1f966}",
    }
}

#[derive(Copy, Clone, Hash, PartialEq)]
pub enum Animal {
    Empty,
    Sheep,
    Pigs,
    Cattle,
}

fn animal_emoji(animal: &Animal) -> &str {
    match animal {
        Animal::Sheep => "\u{1f411}",
        Animal::Pigs => "\u{1f416}",
        Animal::Cattle => "\u{1f404}",
        Animal::Empty => "",
    }
}

const PASTURE_CAPACITY: usize = 2;
const STABLE_MULTIPLIER: usize = 2;

#[derive(Clone, Hash)]
pub struct Pasture {
    pub farmyard_spaces: usize,
    pub stables: usize,
    pub animal: Animal,
    pub amount: usize,
}

impl Pasture {
    pub fn create_new(
        num_spaces: usize,
        num_unfenced_stables: &mut usize,
        no_empty_farmyard_spaces_left: bool,
    ) -> Self {
        let mut stables_to_add: usize = 0;
        if num_unfenced_stables > &mut 0 {
            if no_empty_farmyard_spaces_left {
                // The pasture should encompass as many UF stables as farmyard spaces
                stables_to_add = num_spaces;
            } else {
                stables_to_add = 1;
            }

            if stables_to_add > *num_unfenced_stables {
                println!(
                    "ERROR! Wanting to create pasture with {num_spaces} FS and {num_unfenced_stables} UFS and No empty FS {no_empty_farmyard_spaces_left}"
                );
            }
            *num_unfenced_stables -= stables_to_add;
        }
        Pasture {
            farmyard_spaces: num_spaces,
            stables: stables_to_add,
            animal: Animal::Empty,
            amount: 0,
        }
    }

    pub fn display(&self) {
        print!("[");
        print!("{}", "\u{2b55}".repeat(self.farmyard_spaces));
        if self.stables > 0 {
            print!(" ");
            for _ in 0..self.stables {
                print!("\u{26fa}");
            }
        }
        if self.amount > 0 {
            print!(" => {}", animal_emoji(&self.animal).repeat(self.amount));
        }
        print!("]");
    }

    pub fn capacity(&self) -> usize {
        if self.stables > 0 {
            return STABLE_MULTIPLIER * self.stables * PASTURE_CAPACITY * self.farmyard_spaces;
        }

        PASTURE_CAPACITY * self.farmyard_spaces
    }
}

// Fence mask [North, East, West, South]
// e.g : 0000 -> Not fenced on either side
// 0110 -> Fenced on East and West sides, connected to pastures on North and South sides

#[derive(Copy, Clone, Default, Hash, PartialEq)]
pub enum FarmyardSpace {
    #[default]
    Empty,
    Room,
    EmptyField,
    PlantedField(Seed, usize),
    UnfencedStable(Animal),
    FencedPasture(usize, Animal),
    FencedPastureWithStable(usize, Animal),
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

#[derive(Clone, Hash)]
pub struct Farm {
    pub farmyard_spaces: [FarmyardSpace; NUM_FARMYARD_SPACES],
}

impl Farm {
    pub fn new() -> Self {
        let mut ret = [FarmyardSpace::Empty; NUM_FARMYARD_SPACES];
        ret[5] = FarmyardSpace::Room;
        ret[10] = FarmyardSpace::Room;
        Self {
            farmyard_spaces: ret,
        }
    }

    fn candidate_pasture_spaces(&self) -> [bool; NUM_FARMYARD_SPACES] {
        let mut ret = [false; NUM_FARMYARD_SPACES];

        let mut pasture_spaces = 0;
        for idx in 0..NUM_FARMYARD_SPACES {
            match self.farmyard_spaces[idx] {
                FarmyardSpace::FencedPasture(_, _)
                | FarmyardSpace::FencedPastureWithStable(_, _) => pasture_spaces += 1,
                _ => (),
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
                match self.farmyard_spaces[nidx] {
                    FarmyardSpace::FencedPasture(_, _)
                    | FarmyardSpace::FencedPastureWithStable(_, _) => ret[idx] = true,
                    _ => (),
                }
            }
        }
        ret
    }

    fn pasture_position_score(&self, idx: usize) -> i32 {
        match self.farmyard_spaces[idx] {
            FarmyardSpace::EmptyField | FarmyardSpace::PlantedField(_, _) | FarmyardSpace::Room => {
                return i32::MIN
            }
            _ => (),
        }

        let mut score: i32 = 0;
        for opt_i in NEIGHBOR_SPACES[idx] {
            match opt_i {
                Some(i) => match self.farmyard_spaces[i] {
                    FarmyardSpace::Empty
                    | FarmyardSpace::FencedPasture(_, _)
                    | FarmyardSpace::UnfencedStable(_)
                    | FarmyardSpace::FencedPastureWithStable(_, _) => score += 1,
                    FarmyardSpace::Room
                    | FarmyardSpace::EmptyField
                    | FarmyardSpace::PlantedField(_, _) => score -= 1,
                },
                None => score += 2,
            }
        }
        score
    }

    fn surrounding_fences(&self, idx: usize) -> usize {
        let mut ret: usize = 0;
        match self.farmyard_spaces[idx] {
            FarmyardSpace::FencedPasture(mask, _)
            | FarmyardSpace::FencedPastureWithStable(mask, _) => {
                ret += (mask & 8) + (mask & 4) + (mask & 2) + (mask & 1);
            }
            _ => (),
        }
        ret
    }

    fn neighbouring_fences(&self, idx: usize) -> usize {
        let mut ret: usize = 0;
        for (i, opt_nidx) in NEIGHBOR_SPACES[idx].iter().enumerate() {
            if let Some(nidx) = opt_nidx {
                match self.farmyard_spaces[*nidx] {
                    FarmyardSpace::FencedPasture(mask, _)
                    | FarmyardSpace::FencedPastureWithStable(mask, _) => {
                        // i is the direction, the opposite direction on the neighbor is 3 - i (NEWS)
                        // To check if a fence exists in that direction we simple bitwise AND with the mask
                        if usize::pow(2, 3 - i as u32) & mask > 0 {
                            ret += 1;
                        }
                    }
                    _ => (),
                }
            }
        }
        ret
    }

    fn fencing_options(&self, wood: usize) -> Vec<(Vec<usize>, usize)> {
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

        let candidate_pasture_spaces = self.candidate_pasture_spaces();

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
                FarmyardSpace::FencedPasture(_, _)
                | FarmyardSpace::FencedPastureWithStable(_, _) => {
                    can_use = self.surrounding_fences(idx);
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
                for nidx in NEIGHBOR_SPACES[idx].into_iter().flatten() {
                    if other.contains(&nidx) {
                        connected_sides += 1;
                    }
                }

                // Connected sides need not be fenced
                if connected_sides > 0 {
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

    fn fence_spaces(&mut self, spaces: &Vec<usize>) {
        for space in spaces {
            let mut mask: usize = 0;
            for (i, opt_nidx) in NEIGHBOR_SPACES[*space].iter().enumerate() {
                if let Some(nidx) = opt_nidx {
                    if !spaces.contains(nidx) {
                        mask |= usize::pow(2, i as u32)
                    }
                } else {
                    mask |= usize::pow(2, i as u32)
                }
            }

            let space_type = self.farmyard_spaces[*space];
            match space_type {
                FarmyardSpace::Empty => {
                    self.farmyard_spaces[*space] = FarmyardSpace::FencedPasture(mask, Animal::Empty)
                }
                FarmyardSpace::UnfencedStable(animal) => {
                    self.farmyard_spaces[*space] =
                        FarmyardSpace::FencedPastureWithStable(mask, animal)
                }
                _ => (),
            }
        }
    }

    fn field_position_score(&self, idx: usize) -> i32 {
        if self.farmyard_spaces[idx] != FarmyardSpace::Empty {
            return i32::MIN;
        }

        let mut score: i32 = 0;
        for opt_i in NEIGHBOR_SPACES[idx] {
            match opt_i {
                Some(i) => match self.farmyard_spaces[i] {
                    FarmyardSpace::Empty
                    | FarmyardSpace::PlantedField(_, _)
                    | FarmyardSpace::EmptyField => score += 1,
                    _ => score -= 1,
                },
                None => score += 2,
            }
        }
        score
    }

    pub fn field_options(&self) -> Vec<usize> {
        let scores: Vec<i32> = (0..NUM_FARMYARD_SPACES)
            .map(|idx| self.field_position_score(idx))
            .collect();
        let max_score = *scores.iter().max().unwrap();

        if max_score == i32::MIN {
            return Vec::new();
        }

        (0..NUM_FARMYARD_SPACES)
            .filter(|i| scores[*i] == max_score)
            .collect()
    }

    pub fn num_fields(&self) -> usize {
        self.farmyard_spaces
            .iter()
            .filter(|&f| {
                matches!(
                    f,
                    FarmyardSpace::EmptyField | FarmyardSpace::PlantedField(_, _)
                )
            })
            .count()
    }

    pub fn add_field(&mut self, idx: usize) {
        assert!(self.farmyard_spaces[idx] == FarmyardSpace::Empty);
        self.farmyard_spaces[idx] = FarmyardSpace::EmptyField;
    }

    pub fn can_sow(&self) -> bool {
        self.farmyard_spaces
            .iter()
            .any(|f| matches!(f, FarmyardSpace::EmptyField))
    }

    pub fn sow_field(&mut self, seed: &Seed) {
        assert!(self.can_sow());
        let opt_empty_field = self
            .farmyard_spaces
            .iter_mut()
            .find(|f| matches!(f, FarmyardSpace::EmptyField));
        if let Some(field) = opt_empty_field {
            match *seed {
                Seed::Grain => *field = FarmyardSpace::PlantedField(Seed::Grain, 3),
                Seed::Vegetable => *field = FarmyardSpace::PlantedField(Seed::Vegetable, 2),
            }
        }
    }

    pub fn harvest_fields(&mut self) -> Vec<Seed> {
        let mut ret: Vec<Seed> = Vec::new();
        for space in &mut self.farmyard_spaces {
            if let FarmyardSpace::PlantedField(crop, amount) = space {
                ret.push(*crop);
                *amount -= 1;

                if *amount == 0 {
                    *space = FarmyardSpace::EmptyField;
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
    fn xor_test() {
        assert_eq!(1 & 2, 0);
        assert_eq!(1 | 2 | 4, 7)
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
        let field_opt = farm.field_options();
        println!("{:?}", field_opt);
        assert_eq!(field_opt.len(), 2);
    }
}
