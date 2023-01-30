use crate::primitives::{
    can_pay_for_resource, pay_for_resource, print_resources, Resource, Resources, NUM_RESOURCES,
};

const FIELD_SCORE: [i32; 6] = [-1, -1, 1, 2, 3, 4];
const PASTURE_SCORE: [i32; 5] = [-1, 1, 2, 3, 4];
const GRAIN_SCORE: [i32; 9] = [-1, 1, 1, 1, 2, 2, 3, 3, 4];
const VEGETABLE_SCORE: [i32; 5] = [-1, 1, 2, 3, 4];
const SHEEP_SCORE: [i32; 9] = [-1, 1, 1, 1, 2, 2, 3, 3, 4];
const PIGS_SCORE: [i32; 8] = [-1, 1, 1, 2, 2, 3, 3, 4];
const CATTLE_SCORE: [i32; 7] = [-1, 1, 2, 2, 3, 3, 4];
const MAX_STABLES: u32 = 4;
const MAX_FAMILY_MEMBERS: u32 = 5;
const INF_RES: u32 = 1000;
lazy_static! {
    // map[idx] = (p, w) : Choice of p pastures using w wood when idx number of fences remain.
    static ref FENCING_CHOICES: [Vec<(u32, u32)>; 16] = [
        vec![],
        vec![(1, 1)],
        vec![(1, 2)],
        vec![],
        vec![(2, 4)],
        vec![(1, 3)],
        vec![(2, 5), (2, 4), (1, 2)],
        vec![],
        vec![(2, 4)],
        vec![(2, 4), (1, 3)],
        vec![],
        vec![(2, 5), (1, 3)],
        vec![],
        vec![],
        vec![],
        vec![(2, 6), (1, 4)],
    ];
}

fn calc_score(num: usize, scores: &[i32]) -> i32 {
    if num >= scores.len() {
        scores[scores.len() - 1]
    } else {
        scores[num]
    }
}

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
    Pigs,
    Cattle,
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
    fn create_new(num_spaces: u32, num_unfenced_stables: &mut u32) -> Self {
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

    fn capacity(&self) -> u32 {
        if self.stable {
            4 * self.farmyard_spaces
        } else {
            2 * self.farmyard_spaces
        }
    }
}

struct Farm {
    house: House,
    rooms: u32,
    fields: Vec<Field>,
    pastures: Vec<Pasture>,
    unfenced_stables: u32,
    fences_left: u32,
}

impl Farm {
    fn new() -> Self {
        Farm {
            house: House::Wood,
            rooms: 2,
            fields: Vec::new(),
            pastures: Vec::new(),
            unfenced_stables: 0,
            fences_left: 15,
        }
    }

    fn display(&self) {
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
            print!("[{} Fields]", self.fields.len());
        }
        let ns: u32 = self.unfenced_stables;
        if ns > 0 {
            print!("[{} UF Stables]", ns);
        }
    }

    fn num_stables(&self) -> u32 {
        let mut ret: u32 = 0;
        for p in &self.pastures {
            ret += u32::from(p.stable)
        }
        ret += self.unfenced_stables;
        ret
    }

    fn empty_farmyard_spaces(&self) -> u32 {
        let mut pasture_spaces: u32 = 0;
        for pasture in &self.pastures {
            pasture_spaces += pasture.farmyard_spaces;
        }

        15 - self.rooms - self.fields.len() as u32 - pasture_spaces - self.unfenced_stables
    }
}

pub struct Player {
    resources: Resources,
    people_placed: u32,
    family_members: u32,
    begging_tokens: u32,
    build_room_cost: Resources,
    build_stable_cost: Resources,
    renovation_cost: Resources,
    farm: Farm,
}

impl Player {
    pub fn create_new(food: u32) -> Self {
        let mut res = [0; NUM_RESOURCES];
        res[Resource::Food] = food;

        let mut room_cost = [0; NUM_RESOURCES];
        room_cost[Resource::Wood] = 5;
        room_cost[Resource::Reed] = 2;

        let mut stable_cost = [0; NUM_RESOURCES];
        stable_cost[Resource::Wood] = 2;

        let mut reno_cost = [0; NUM_RESOURCES];
        reno_cost[Resource::Clay] = 2;
        reno_cost[Resource::Reed] = 1;

        Player {
            resources: res,
            people_placed: 0,
            family_members: 2,
            begging_tokens: 0,
            build_room_cost: room_cost,
            build_stable_cost: stable_cost,
            renovation_cost: reno_cost,
            farm: Farm::new(),
        }
    }

    pub fn fence(&mut self) {
        // Follow convention as mentioned in can_fence()
        let mut recurse: bool = false;
        for (p, w) in &FENCING_CHOICES[self.farm.fences_left as usize] {
            if self.resources[Resource::Wood] >= *w {
                self.farm
                    .pastures
                    .push(Pasture::create_new(*p, &mut self.farm.unfenced_stables));
                self.farm.fences_left -= *w;
                self.resources[Resource::Wood] -= *w;
                recurse = true;
                break;
            }
        }
        if recurse {
            self.fence();
        }
    }

    pub fn can_fence(&self) -> bool {
        // No fences left
        if self.farm.fences_left == 0 {
            return false;
        }

        // No empty farmyard space or an unfenced stable
        if self.farm.empty_farmyard_spaces() == 0 && self.farm.unfenced_stables == 0 {
            return false;
        }

        // TODO this should be made very generic
        // Here we follow a convention - always aim to fence 6 spaces (if possible) and have 4 pastures with a stable in each : 2 + 2 + 1 + 1
        // Total capacity : 8 + 8 + 4 + 4 + 1 (pet) which is sufficient to max out animal score
        // Generic setup using 15 Fences with Pasture sizes [2, 2, 1, 1] :
        // + - + - +   +   +   +
        // |       |
        // + - + - +   +   +   +
        // |       |
        // + - + - +   +   +   +
        // |   |   |
        // + - + - +   +   +   +

        for (_p, w) in &FENCING_CHOICES[self.farm.fences_left as usize] {
            if self.resources[Resource::Wood] >= *w {
                return true;
            }
        }

        false
    }

    pub fn grow_family_with_room(&mut self) {
        assert!(self.can_grow_family_with_room());
        self.family_members += 1;
    }

    pub fn grow_family_without_room(&mut self) {
        assert!(self.can_grow_family_without_room());
        self.family_members += 1;
    }

    pub fn can_grow_family_with_room(&self) -> bool {
        self.family_members < MAX_FAMILY_MEMBERS && self.family_members < self.farm.rooms
    }

    pub fn can_grow_family_without_room(&self) -> bool {
        self.family_members < MAX_FAMILY_MEMBERS && self.family_members >= self.farm.rooms
    }

    pub fn renovate(&mut self) {
        assert!(self.can_renovate());
        // TODO for cards like Conservator this must be implemented in a more general way
        pay_for_resource(&self.renovation_cost, &mut self.resources);
        let current_type = &self.farm.house;
        match current_type {
            House::Wood => {
                self.farm.house = House::Clay;
                self.build_room_cost[Resource::Wood] = 0;
                self.build_room_cost[Resource::Clay] = 5;
            }
            House::Clay => {
                self.farm.house = House::Stone;
                self.build_room_cost[Resource::Clay] = 0;
                self.build_room_cost[Resource::Stone] = 5;
            }
            _ => (),
        }
    }

    pub fn build_rooms(&mut self) {
        assert!(self.can_build_rooms());
        // TODO - this function should return a Choices array
        // But we follow some convention for now
        // Build as many rooms as possible
        while self.can_build_rooms() {
            pay_for_resource(&self.build_room_cost, &mut self.resources);
            self.farm.rooms += 1;

            match self.farm.house {
                House::Wood => self.renovation_cost[Resource::Clay] += 1,
                House::Clay => self.renovation_cost[Resource::Stone] += 1,
                _ => (),
            }
        }
    }

    pub fn build_stables(&mut self) {
        assert!(self.farm.num_stables() < MAX_STABLES);
        assert!(self.can_build_stables());
        // TODO - this function should return a Choices array
        // But we follow some convention for now
        // Build as many stables as possible
        while self.can_build_stables() {
            pay_for_resource(&self.build_stable_cost, &mut self.resources);
            // Add to pasture if possible, then unfenced
            let mut found: bool = false;
            for pasture in &mut self.farm.pastures {
                if !pasture.stable {
                    pasture.stable = true;
                    found = true;
                    break;
                }
            }
            if !found {
                self.farm.unfenced_stables += 1;
            }
        }
    }

    pub fn can_renovate(&self) -> bool {
        if let House::Stone = self.farm.house {
            return false;
        }
        can_pay_for_resource(&self.renovation_cost, &self.resources)
    }

    pub fn can_build_rooms(&self) -> bool {
        if self.farm.empty_farmyard_spaces() == 0 {
            return false;
        }
        can_pay_for_resource(&self.build_room_cost, &self.resources)
    }

    pub fn can_build_stables(&self) -> bool {
        if self.farm.num_stables() >= MAX_STABLES {
            return false;
        }
        if self.farm.empty_farmyard_spaces() == 0 {
            let mut all_filled: bool = true;
            for pasture in &self.farm.pastures {
                if !pasture.stable {
                    all_filled = false;
                }
            }
            if all_filled {
                return false;
            }
        }
        can_pay_for_resource(&self.build_stable_cost, &self.resources)
    }

    fn score_plants(&self) -> i32 {
        let mut num_grain: usize = 0;
        let mut num_veg: usize = 0;
        for field in &self.farm.fields {
            match field.seed {
                PlantedSeed::Grain => num_grain += field.amount as usize,
                PlantedSeed::Vegetable => num_veg += field.amount as usize,
                _ => (),
            }
        }
        num_grain += self.resources[Resource::Grain] as usize;
        num_veg += self.resources[Resource::Vegetable] as usize;

        calc_score(num_grain, &GRAIN_SCORE) + calc_score(num_veg, &VEGETABLE_SCORE)
    }

    fn score_animals(&self) -> i32 {
        let mut num_sheep: usize = 0;
        let mut num_pigs: usize = 0;
        let mut num_cattle: usize = 0;

        for pasture in &self.farm.pastures {
            match pasture.animal {
                Animal::Sheep => num_sheep += pasture.amount as usize,
                Animal::Pigs => num_pigs += pasture.amount as usize,
                Animal::Cattle => num_cattle += pasture.amount as usize,
                _ => (),
            }
        }
        num_sheep += self.resources[Resource::Sheep] as usize;
        num_pigs += self.resources[Resource::Pigs] as usize;
        num_cattle += self.resources[Resource::Cattle] as usize;

        calc_score(num_sheep, &SHEEP_SCORE)
            + calc_score(num_pigs, &PIGS_SCORE)
            + calc_score(num_cattle, &CATTLE_SCORE)
    }

    fn score_fields(&self) -> i32 {
        calc_score(self.farm.fields.len(), &FIELD_SCORE)
    }

    fn score_pastures(&self) -> i32 {
        let mut ret: i32 = 0;
        // Number of Pastures
        ret += calc_score(self.farm.pastures.len(), &PASTURE_SCORE);
        // Number of fenced stables
        for pasture in &self.farm.pastures {
            if pasture.stable {
                ret += 1
            }
        }
        ret
    }

    fn score_house_family_empty_spaces_begging(&self) -> i32 {
        let mut ret: i32 = 0;

        // House
        match self.farm.house {
            House::Clay => ret += self.farm.rooms as i32,
            House::Stone => ret += 2 * self.farm.rooms as i32,
            _ => (),
        }

        // Family members
        ret += 3 * self.family_members as i32;

        // Empty spaces
        ret -= self.farm.empty_farmyard_spaces() as i32;

        // Begging Tokens
        ret -= 3 * self.begging_tokens as i32;
        ret
    }

    pub fn score(&self) -> i32 {
        let mut ret: i32 = 0;

        // Fields
        ret += self.score_fields();
        // Pastures
        ret += self.score_pastures();
        // Grain and Veggies
        ret += self.score_plants();
        // Animals
        ret += self.score_animals();
        // House, Family and Empty Spaces
        ret += self.score_house_family_empty_spaces_begging();
        // TODO Score Cards

        ret
    }

    pub fn take_res(&mut self, acc_res: &Resources) {
        for (i, elem) in acc_res.iter().enumerate().take(NUM_RESOURCES) {
            self.resources[i] += elem;
        }
    }

    pub fn add_new_field(&mut self) {
        let field = Field::new();
        self.farm.fields.push(field);
    }

    pub fn can_add_new_field(&self) -> bool {
        self.farm.empty_farmyard_spaces() > 0
    }

    pub fn display(&self) {
        print!(
            "Player ({}/{}) SCORE {} has ",
            self.people_placed,
            self.family_members,
            self.score()
        );
        print_resources(&self.resources);
        self.farm.display();
    }

    pub fn reset_for_next_round(&mut self) {
        self.people_placed = 0;
    }

    pub fn family_size(&self) -> u32 {
        self.family_members
    }

    pub fn increment_people_placed(&mut self) {
        self.people_placed += 1;
    }

    pub fn all_people_placed(&self) -> bool {
        self.people_placed == self.family_members
    }
}
