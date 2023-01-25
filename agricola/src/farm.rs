enum House {
    Wood,
    Clay,
    Stone,
}

enum PlantedSeed {
    Empty,
    Grain,
    Vegetable,
}

enum Animal {
    Empty,
    Sheep,
    Pig,
    Cow,
}

struct Field {
    seed: PlantedSeed,
    amount: u32,
}

impl Field {
    fn new() -> Self {
        Field {
            seed: PlantedSeed::Empty,
            amount: 0,
        }
    }
}

struct Pasture {
    farmyard_spaces: u32,
    stable: bool,
    animal: Animal,
    amount: u32,
}

impl Pasture {
    fn capacity(&self) -> u32 {
        if self.stable {
            4 * self.farmyard_spaces
        } else {
            2 * self.farmyard_spaces
        }
    }
}

pub struct Farm {
    house: House,
    rooms: u32,
    fields: Vec<Field>,
    pastures: Vec<Pasture>,
    stables: u32,
    fences_left: u32,
}

impl Farm {
    pub fn new() -> Self {
        Farm {
            house: House::Wood,
            rooms: 2,
            fields: Vec::new(),
            pastures: Vec::new(),
            stables: 0,
            fences_left: 15,
        }
    }

    pub fn display(&self) {
        if !self.fields.is_empty() {
            print!("[{} Fields]", self.fields.len());
        }
    }

    pub fn add_new_field(&mut self) {
        let field = Field::new();
        self.fields.push(field);
    }
}
