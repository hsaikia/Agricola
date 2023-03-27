#[derive(Clone, Hash)]
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

#[derive(Clone, Hash)]
pub enum PlantedSeed {
    Empty,
    Grain,
    Vegetable,
}

fn seed_emoji(seed: &PlantedSeed) -> &str {
    match seed {
        PlantedSeed::Grain => "\u{1f33e}",
        PlantedSeed::Vegetable => "\u{1f966}",
        PlantedSeed::Empty => "\u{1f7e9}",
    }
}

#[derive(Clone, Hash)]
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

#[derive(Clone, Hash)]
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
        let reps: usize = if self.amount > 0 {
            self.amount as usize
        } else {
            1
        };
        print!("[{}]", seed_emoji(&self.seed).repeat(reps));
    }
}

#[derive(Clone, Hash)]
pub struct Pasture {
    pub farmyard_spaces: u32,
    pub stables: u32,
    pub animal: Animal,
    pub amount: u32,
}

impl Pasture {
    pub fn create_new(
        num_spaces: u32,
        num_unfenced_stables: &mut u32,
        no_empty_farmyard_spaces_left: bool,
    ) -> Self {
        let mut stables_to_add: u32 = 0;
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
        print!("{}", "\u{2b55}".repeat(self.farmyard_spaces as usize));
        if self.stables > 0 {
            print!(" ");
            for _ in 0..self.stables {
                print!("\u{26fa}");
            }
        }
        if self.amount > 0 {
            print!(
                " => {}",
                animal_emoji(&self.animal).repeat(self.amount as usize)
            );
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
