#[derive(Clone)]
pub enum House {
    Wood,
    Clay,
    Stone,
}

#[derive(Clone)]
pub enum PlantedSeed {
    Empty,
    Grain,
    Vegetable,
}

#[derive(Clone)]
pub enum Animal {
    Empty,
    Sheep,
    Pigs,
    Cattle,
}

#[derive(Clone)]
pub struct Field {
    pub seed: PlantedSeed,
    pub amount: u32,
}

const PASTURE_CAPACITY: u32 = 2;
const STABLE_MULTIPLIER: u32 = 2;

impl Field {
    pub fn new() -> Self {
        Field {
            seed: PlantedSeed::Empty,
            amount: 0,
        }
    }

    pub fn display(&self) {
        match self.seed {
            PlantedSeed::Empty => print!("[0]"),
            PlantedSeed::Grain => print!("[{}G]", self.amount),
            PlantedSeed::Vegetable => print!("[{}V]", self.amount),
        }
    }
}

#[derive(Clone)]
pub struct Pasture {
    pub farmyard_spaces: u32,
    pub stables: u32,
    pub animal: Animal,
    pub amount: u32,
}

impl Pasture {
    pub fn create_new(num_spaces: u32, num_unfenced_stables: &mut u32, no_empty_farmyard_spaces_left : bool) -> Self {
        let mut stables_to_add : u32 = 0;
        if num_unfenced_stables > &mut 0 {
            if no_empty_farmyard_spaces_left {
                // The pasture should encompass as many UF stables as farmyard spaces
                stables_to_add = num_spaces;    
            } else {
                stables_to_add = 1;
            }

            if stables_to_add > *num_unfenced_stables {
                println!("ERROR! Wanting to create pasture with {} FS and {} UFS and No empty FS {}", num_spaces, num_unfenced_stables, no_empty_farmyard_spaces_left);
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
        print!("{}", self.farmyard_spaces);
        if self.stables > 0 {
            print!(" + ");
            for _ in 0..self.stables {
                print!("S");
            }
        }
        if self.amount > 0 {
            match self.animal {
                Animal::Sheep => print!(" => {} Sheep", self.amount),
                Animal::Pigs => print!(" => {} Pig(s)", self.amount),
                Animal::Cattle => print!(" => {} Cow(s)", self.amount),
                _ => (),
            }
        }
        print!("]");
    }

    pub fn capacity(&self) -> u32 {
        if self.stables > 0 {
            return STABLE_MULTIPLIER * self.stables * PASTURE_CAPACITY * self.farmyard_spaces;
        }

        PASTURE_CAPACITY * self.farmyard_spaces
    }
}
