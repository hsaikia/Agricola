pub enum House {
    Wood,
    Clay,
    Stone,
}

pub enum PlantedSeed {
    Empty,
    Grain,
    Vegetable,
}

pub enum Animal {
    Empty,
    Sheep,
    Pigs,
    Cattle,
}

pub struct Field {
    pub seed: PlantedSeed,
    pub amount: u32,
}

const FARMYARD_SPACES: u32 = 15;
const PASTURE_CAPACITY: u32 = 2;
const STABLE_MULTIPLIER: u32 = 2;

impl Field {
    pub fn new() -> Self {
        Field {
            seed: PlantedSeed::Empty,
            amount: 0,
        }
    }

    fn display(&self) {
        match self.seed {
            PlantedSeed::Empty => print!("[0]"),
            PlantedSeed::Grain => print!("[{}G]", self.amount),
            PlantedSeed::Vegetable => print!("[{}V]", self.amount),
        }
    }
}

pub struct Pasture {
    pub farmyard_spaces: u32,
    pub stable: bool, // TODO pastures can have multiple stables
    pub animal: Animal,
    pub amount: u32,
}

impl Pasture {
    pub fn create_new(num_spaces: u32, num_unfenced_stables: &mut u32) -> Self {
        let mut around_stable: bool = false;
        if num_unfenced_stables > &mut 0 {
            around_stable = true;
            *num_unfenced_stables -= 1;
        }
        Pasture {
            farmyard_spaces: num_spaces,
            stable: around_stable,
            animal: Animal::Empty,
            amount: 0,
        }
    }

    fn display(&self) {
        print!("[");
        print!("{}", self.farmyard_spaces);
        if self.stable {
            print!(" + S");
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
        if self.stable {
            return STABLE_MULTIPLIER * PASTURE_CAPACITY * self.farmyard_spaces;
        }

        PASTURE_CAPACITY * self.farmyard_spaces
    }
}

pub struct Farm {
    pub house: House,
    pub rooms: u32,
    pub fields: Vec<Field>,
    pub pastures: Vec<Pasture>,
    pub unfenced_stables: u32,
    pub fences_left: u32,
}

impl Farm {
    pub fn new() -> Self {
        Farm {
            house: House::Wood,
            rooms: 2,
            fields: Vec::new(),
            pastures: Vec::new(),
            unfenced_stables: 0,
            fences_left: 15,
        }
    }

    pub fn display(&self) {
        print!("[{} Room ", self.rooms);
        match self.house {
            House::Wood => print!("Wood"),
            House::Clay => print!("Clay"),
            House::Stone => print!("Stone"),
        }
        print!(" House]");

        if !self.pastures.is_empty() {
            print!("[Pastures ");
            for p in &self.pastures {
                p.display();
            }
            print!("]");
        }

        if !self.fields.is_empty() {
            print!("[Fields ");
            for f in &self.fields {
                f.display();
            }
            print!("]");
        }
        let ns: u32 = self.unfenced_stables;
        if ns > 0 {
            print!("[{} UF Stables]", ns);
        }
    }

    pub fn num_stables(&self) -> u32 {
        let mut ret: u32 = 0;
        for p in &self.pastures {
            ret += u32::from(p.stable)
        }
        ret += self.unfenced_stables;
        ret
    }

    pub fn empty_farmyard_spaces(&self) -> u32 {
        let mut pasture_spaces: u32 = 0;
        for pasture in &self.pastures {
            pasture_spaces += pasture.farmyard_spaces;
        }

        FARMYARD_SPACES
            - self.rooms
            - self.fields.len() as u32
            - pasture_spaces
            - self.unfenced_stables
    }

    pub fn harvest_fields(&mut self) -> (u32, u32) {
        let mut harvested_grain = 0;
        let mut harvested_veg = 0;
        for f in &mut self.fields {
            match f.seed {
                PlantedSeed::Grain => {
                    print!("\nGrain harvested!");
                    harvested_grain += 1;
                    f.amount -= 1;
                }
                PlantedSeed::Vegetable => {
                    print!("\nVegetable harvested!");
                    harvested_veg += 1;
                    f.amount -= 1;
                }
                _ => (),
            }
            if f.amount == 0 {
                f.seed = PlantedSeed::Empty;
            }
        }
        (harvested_grain, harvested_veg)
    }
}
