pub const NUM_FARMYARD_SPACES: usize = 15;

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

#[derive(Debug, Copy, Clone, Hash)]
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

#[derive(Copy, Clone, Hash)]
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

#[derive(Copy, Clone, Hash)]
pub enum Field {
    Empty,
    Planted(Seed, usize),
}

const PASTURE_CAPACITY: usize = 2;
const STABLE_MULTIPLIER: usize = 2;

impl Field {
    pub fn new() -> Self {
        Self::Empty
    }

    pub fn display(&self) {
        match self {
            Self::Empty => {
                print!("[\u{1f7e9}]");
            }
            Self::Planted(seed, num) => {
                print!("[{}]", seed_emoji(seed).repeat(*num));
            }
        }
    }

    pub fn sow(&mut self, seed: &Seed) -> bool {
        if let Self::Empty = self {
            match seed {
                Seed::Grain => *self = Field::Planted(Seed::Grain, 3),
                Seed::Vegetable => *self = Field::Planted(Seed::Vegetable, 2),
            }
            return true;
        }
        false
    }

    pub fn harvest(&mut self) -> Option<Seed> {
        match self {
            Self::Planted(seed, amount) => {
                *amount -= 1;
                let ret = Some(*seed);
                if *amount == 0 {
                    *self = Field::Empty;
                }
                ret
            }
            _ => None,
        }
    }
}

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

#[derive(Copy, Clone, Default, Hash, PartialEq)]
pub enum FarmyardSpace {
    #[default]
    Empty,
    Room,
    Field,
    Pasture, // Stable or Fence
}

// 00 01 02 03 04
// 05 06 07 08 09
// 10 11 12 13 14
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

    fn neighboring_empty_spaces(&self, space_type: FarmyardSpace) -> [bool; NUM_FARMYARD_SPACES] {
        let mut ret = [false; NUM_FARMYARD_SPACES];
        for idx in 0..NUM_FARMYARD_SPACES {
            if self.farmyard_spaces[idx] != space_type {
                continue;
            }

            let r: i32 = idx as i32 / 5;
            let c: i32 = idx as i32 % 5;
            let cells = [(r - 1, c), (r, c - 1), (r + 1, c), (r, c + 1)];
            for (x, y) in cells {
                if x < 0 || y < 0 || x > 2 || y > 4 {
                    continue;
                }
                let idx_neighbor = (x * 5 + y) as usize;

                if self.farmyard_spaces[idx_neighbor] != FarmyardSpace::Empty {
                    continue;
                }

                //println!("{}", idx_neighbor);
                ret[idx_neighbor] = true;
            }
        }
        ret
    }

    fn distance(idx1: usize, idx2: usize) -> f32 {
        let x1 = idx1 as i32 / 5;
        let x2 = idx2 as i32 / 5;
        let y1 = idx1 as i32 % 5;
        let y2 = idx2 as i32 % 5;

        let dx = (x2 - x1) as f32;
        let dy = (y2 - y1) as f32;

        f32::sqrt((dx * dx) + (dy * dy))
    }

    // Finds the best position of the given type on the farm. Tries to keep different regions as farther apart as possible.
    pub fn best_spaces(&self, num_positions: usize, space_type: &FarmyardSpace) -> Vec<usize> {
        let mut ret: Vec<usize> = Vec::new();

        assert!(*space_type != FarmyardSpace::Empty);

        let mut curr_farm = self.clone();
        for _ in 0..num_positions {
            let mut candidate_spaces = curr_farm.neighboring_empty_spaces(*space_type);

            let num_occ_spaces = curr_farm
                .farmyard_spaces
                .iter()
                .filter(|&x| x == space_type)
                .count();
            let num_cand_spaces = candidate_spaces.iter().filter(|&x| *x).count();

            if num_occ_spaces > 0 && num_cand_spaces == 0 {
                // Blocked on all sides, can't expand
                break;
            }

            // If no such farmyard space exists yet, can place anywhere on an empty space
            if num_cand_spaces == 0 {
                for (i, fys) in curr_farm.farmyard_spaces.iter().enumerate() {
                    candidate_spaces[i] = fys == &FarmyardSpace::Empty;
                }
            }

            if candidate_spaces.iter().filter(|&x| *x).count() == 0 {
                break;
            }

            let mut best_dist: f32 = 0.0;
            let mut best_idx: usize = 0;
            for (idx1, cs) in candidate_spaces.iter().enumerate() {
                if !cs {
                    continue;
                }

                let mut sum_dist: f32 = 0.0;

                for idx2 in 0..NUM_FARMYARD_SPACES {
                    if idx1 == idx2
                        || curr_farm.farmyard_spaces[idx2] == *space_type
                        || curr_farm.farmyard_spaces[idx2] == FarmyardSpace::Empty
                    {
                        continue;
                    }

                    sum_dist += Farm::distance(idx1, idx2);
                }

                if sum_dist > best_dist {
                    best_dist = sum_dist;
                    best_idx = idx1;
                }
            }

            ret.push(best_idx);
            curr_farm.farmyard_spaces[best_idx] = *space_type;
        }

        ret
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_farm_neighboring_spaces() {
        let farm = Farm::new();
        assert_eq!(
            farm.neighboring_empty_spaces(FarmyardSpace::Room)
                .iter()
                .filter(|&x| *x)
                .count(),
            3
        );
        assert_eq!(
            farm.neighboring_empty_spaces(FarmyardSpace::Field)
                .iter()
                .filter(|&x| *x)
                .count(),
            0
        );
        assert_eq!(
            farm.neighboring_empty_spaces(FarmyardSpace::Pasture)
                .iter()
                .filter(|&x| *x)
                .count(),
            0
        );
    }

    #[test]
    fn test_best_spaces() {
        let farm = Farm::new();
        assert_eq!(farm.best_spaces(1, &FarmyardSpace::Room), vec![0]);
        assert_eq!(farm.best_spaces(1, &FarmyardSpace::Field), vec![4]);
        assert_eq!(farm.best_spaces(2, &FarmyardSpace::Pasture), vec![4, 9]);
    }
}
