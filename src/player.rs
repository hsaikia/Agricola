use crate::major_improvements::{MajorImprovement, MajorImprovementType};
use crate::primitives::{
    can_pay_for_resource, pay_for_resource, print_resources, Resource, Resources, NUM_RESOURCES, new_res,
};
use rand::Rng;
use std::cmp;

const FIELD_SCORE: [i32; 6] = [-1, -1, 1, 2, 3, 4];
const PASTURE_SCORE: [i32; 5] = [-1, 1, 2, 3, 4];
const GRAIN_SCORE: [i32; 9] = [-1, 1, 1, 1, 2, 2, 3, 3, 4];
const VEGETABLE_SCORE: [i32; 5] = [-1, 1, 2, 3, 4];
const SHEEP_SCORE: [i32; 9] = [-1, 1, 1, 1, 2, 2, 3, 3, 4];
const PIGS_SCORE: [i32; 8] = [-1, 1, 1, 2, 2, 3, 3, 4];
const CATTLE_SCORE: [i32; 7] = [-1, 1, 2, 2, 3, 3, 4];
const MAX_STABLES: u32 = 4;
const MAX_FAMILY_MEMBERS: u32 = 5;
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

    fn display(&self) {
        match self.seed {
            PlantedSeed::Empty => print!("[0]"),
            PlantedSeed::Grain => print!("[{}G]", self.amount),
            PlantedSeed::Vegetable => print!("[{}V]", self.amount),
        }
    }
}

struct Pasture {
    farmyard_spaces: u32,
    stable: bool, // TODO pastures can have multiple stables
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
    // Animals in this resources array are the ones that are pets in the house and the ones that are kept in unfenced stables
    resources: Resources,
    people_placed: u32,
    family_members: u32,
    begging_tokens: u32,
    build_room_cost: Resources,
    build_stable_cost: Resources,
    renovation_cost: Resources,
    majors: Vec<MajorImprovementType>,
    cards_score: u32,
    farm: Farm,
    promised_resources : Vec<Resources>,
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
            majors: Vec::new(),
            cards_score: 0,
            farm: Farm::new(),
            promised_resources : Vec::new(),
        }
    }

    fn num_free_animals(&self) -> u32 {
        self.resources[Resource::Sheep]
            + self.resources[Resource::Pigs]
            + self.resources[Resource::Cattle]
    }

    pub fn reorg_animals(&mut self) {
        // Can free animals be kept in the house and UF stables?
        let free_animal_spaces = self.farm.unfenced_stables + 1;
        if self.num_free_animals() <= free_animal_spaces {
            return;
        }

        // Try and fit in pastures
        if !self.farm.pastures.is_empty() {
            // Add pasture animals to the resources (free) array
            for p in &mut self.farm.pastures {
                match p.animal {
                    Animal::Sheep => self.resources[Resource::Sheep] += p.amount,
                    Animal::Pigs => self.resources[Resource::Pigs] += p.amount,
                    Animal::Cattle => self.resources[Resource::Cattle] += p.amount,
                    _ => (),
                }
                p.amount = 0;
                p.animal = Animal::Empty;
            }

            // Sorting preserves order in case of equality
            // Hence pre-sorting according to animal importance wrt scoring
            let mut free_animals = vec![
                (self.resources[Resource::Cattle], Resource::Cattle),
                (self.resources[Resource::Pigs], Resource::Pigs),
                (self.resources[Resource::Sheep], Resource::Sheep),
            ];
            let mut curr_idx = 0;

            // sort by decreasing order
            free_animals.sort_by(|a, b| b.0.cmp(&a.0));
            self.farm.pastures.sort_by_key(|k| 1000 - k.capacity());

            // Fill pastures
            // TODO this should be made generic
            for p in &mut self.farm.pastures {
                match free_animals[curr_idx].1 {
                    Resource::Sheep => p.animal = Animal::Sheep,
                    Resource::Pigs => p.animal = Animal::Pigs,
                    Resource::Cattle => p.animal = Animal::Cattle,
                    _ => (),
                }
                p.amount = cmp::min(free_animals[curr_idx].0, p.capacity());
                free_animals[curr_idx].0 -= p.amount;
                curr_idx = (curr_idx + 1) % 3;
            }

            for (a, t) in free_animals {
                self.resources[t] = a;
            }

            // Can free animals be kept in the house and UF stables?
            if self.num_free_animals() <= free_animal_spaces {
                return;
            }
        }
        // Animals are in excess, keep only the valuable ones
        let mut to_keep = free_animal_spaces;
        let mut leftover: Resources = [0; NUM_RESOURCES];

        let cattle_kept = cmp::min(self.resources[Resource::Cattle], to_keep);
        leftover[Resource::Cattle] = self.resources[Resource::Cattle] - cattle_kept;
        self.resources[Resource::Cattle] = cattle_kept;
        to_keep -= cattle_kept;

        let pigs_kept = cmp::min(self.resources[Resource::Pigs], to_keep);
        leftover[Resource::Pigs] = self.resources[Resource::Pigs] - pigs_kept;
        self.resources[Resource::Pigs] = pigs_kept;
        to_keep -= pigs_kept;

        let sheep_kept = cmp::min(self.resources[Resource::Sheep], to_keep);
        leftover[Resource::Sheep] = self.resources[Resource::Sheep] - sheep_kept;
        self.resources[Resource::Sheep] = sheep_kept;

        // Eat
        self.resources[Resource::Food] +=
            Player::convert_to_food(&self.majors, leftover, false, false);
    }

    // Takes a resource array and converts the entire array to food
    // This is done to preserve the original resource array for the player and delegate
    // the decision of which resources to convert to food to a separate function.
    fn convert_to_food(
        majors: &Vec<MajorImprovementType>,
        res: Resources,
        bake: bool,
        harvest: bool,
    ) -> u32 {
        let mut food: u32 = 0;
        let mut fp_ch_food: u32 = 0;
        let mut bake_food: u32 = 0;
        for mi in majors {
            match mi {
                MajorImprovementType::Fireplace2 | MajorImprovementType::Fireplace3 => {
                    fp_ch_food = cmp::max(
                        fp_ch_food,
                        res[Resource::Sheep] * 2
                            + res[Resource::Pigs] * 2
                            + res[Resource::Vegetable] * 2
                            + res[Resource::Cattle] * 3,
                    );
                }
                MajorImprovementType::CookingHearth4 | MajorImprovementType::CookingHearth5 => {
                    fp_ch_food = cmp::max(
                        fp_ch_food,
                        res[Resource::Sheep] * 2
                            + res[Resource::Pigs] * 3
                            + res[Resource::Vegetable] * 3
                            + res[Resource::Cattle] * 4,
                    );
                }
                MajorImprovementType::ClayOven => {
                    if bake && res[Resource::Grain] > 0 {
                        // Clay Oven bakes 1 grain for 5 food
                        // bake bread with 1 grain and eat the rest raw, 5 + n - 1 = 4 + n
                        bake_food = cmp::max(bake_food, 4 + res[Resource::Grain]);
                    }
                }
                MajorImprovementType::StoneOven => {
                    // Stone Oven bakes 1 grain for 4 food or 2 grain for 8 food
                    if bake && res[Resource::Grain] > 0 {
                        let bread = cmp::min(2, res[Resource::Grain]);
                        // bake bread with x grain and eat the rest raw, 4 * x + n - x = 3 * x - n
                        bake_food = cmp::max(bake_food, 3 * bread + res[Resource::Grain]);
                    }
                }
                MajorImprovementType::Joinery => {
                    // Joinery lets a player convert 1 wood to 2 food during harvest
                    if harvest && res[Resource::Wood] > 0 {
                        food += 2;
                    }
                }
                MajorImprovementType::Pottery => {
                    // Pottery lets a player convert 1 clay to 2 food during harvest
                    if harvest && res[Resource::Clay] > 0 {
                        food += 2;
                    }
                }
                MajorImprovementType::BasketmakersWorkshop => {
                    // BMW lets a player convert 1 reed to 3 food during harvest
                    if harvest && res[Resource::Reed] > 0 {
                        food += 3;
                    }
                }
                _ => (),
            }
        }

        if bake_food == 0 {
            // No baking improvements - eat raw grain
            bake_food = res[Resource::Grain];
        }

        if fp_ch_food == 0 {
            // No cooking improvements - eat raw veg
            fp_ch_food = res[Resource::Vegetable];
        }

        food + bake_food + fp_ch_food
    }

    // TODO - make this generic
    // Currently builds a random major
    pub fn build_major(&mut self, majors: &mut Vec<MajorImprovement>) {
        let mut available: Vec<usize> = Vec::new();
        for (i, major) in majors.iter().enumerate() {
            if can_pay_for_resource(major.cost(), &self.resources) {
                available.push(i)
            }
            // TODO if choose CH, give back FP
        }


        let idx = rand::thread_rng().gen_range(0..available.len());
        pay_for_resource(majors[available[idx]].cost(), &mut self.resources);

        let mi_type = majors[available[idx]].major_type();

        match mi_type {
            MajorImprovementType::Well => {
                while self.promised_resources.len() < 5 {
                    let res = new_res();
                    self.promised_resources.push(res);
                }
                for i in 0..5 {
                    self.promised_resources[i][Resource::Food] += 1;
                }
            }
            MajorImprovementType::ClayOven => {
                if self.resources[Resource::Grain] > 0 {
                    self.resources[Resource::Grain] -= 1;
                    self.resources[Resource::Food] += 5;
                }
            }
            MajorImprovementType::StoneOven => {
                for _ in 0..2 {
                    if self.resources[Resource::Grain] > 0 {
                        self.resources[Resource::Grain] -= 1;
                        self.resources[Resource::Food] += 4;
                    }
                }
            }
            _ => ()
        }

        self.majors.push(mi_type);
        self.cards_score += majors[available[idx]].points();
        majors.remove(available[idx]);
    }

    pub fn sow(&mut self) {
        let mut empty_field_idx: usize = 0;
        let mut grain_fields: u32 = 0;
        let mut veg_fields: u32 = 0;
        for (i, f) in &mut self.farm.fields.iter().enumerate() {
            match f.seed {
                PlantedSeed::Grain => grain_fields += 1,
                PlantedSeed::Vegetable => veg_fields += 1,
                PlantedSeed::Empty => empty_field_idx = i,
            }
        }

        let field_to_sow = &mut self.farm.fields[empty_field_idx];

        if self.resources[Resource::Vegetable] == 0
            || (self.resources[Resource::Grain] > 0 && grain_fields <= veg_fields)
        {
            assert!(self.resources[Resource::Grain] > 0);
            field_to_sow.seed = PlantedSeed::Grain;
            field_to_sow.amount = 3;
            self.resources[Resource::Grain] -= 1;
        } else if self.resources[Resource::Grain] == 0 || veg_fields < grain_fields {
            assert!(self.resources[Resource::Vegetable] > 0);
            field_to_sow.seed = PlantedSeed::Vegetable;
            field_to_sow.amount = 2;
            self.resources[Resource::Vegetable] -= 1;
        }

        if self.can_sow() {
            // Sow as much as possible
            self.sow();
        }
    }

    pub fn can_sow(&self) -> bool {
        if self.resources[Resource::Grain] == 0 && self.resources[Resource::Vegetable] == 0 {
            return false;
        }

        for f in &self.farm.fields {
            if let PlantedSeed::Empty = f.seed {
                return true;
            }
        }
        false
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

        self.reorg_animals();

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

    pub fn can_build_major(&self, majors: &Vec<MajorImprovement>) -> bool {
        for major in majors {
            if can_pay_for_resource(major.cost(), &self.resources) {
                return true;
            }
        }
        false
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
        // All animals kept as pets and in unfenced stables
        let mut num_sheep = self.resources[Resource::Sheep];
        let mut num_pigs = self.resources[Resource::Pigs];
        let mut num_cattle = self.resources[Resource::Cattle];

        for pasture in &self.farm.pastures {
            match pasture.animal {
                Animal::Sheep => num_sheep += pasture.amount,
                Animal::Pigs => num_pigs += pasture.amount,
                Animal::Cattle => num_cattle += pasture.amount,
                _ => (),
            }
        }

        calc_score(num_sheep as usize, &SHEEP_SCORE)
            + calc_score(num_pigs as usize, &PIGS_SCORE)
            + calc_score(num_cattle as usize, &CATTLE_SCORE)
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
        // Score Majors
        ret += self.cards_score as i32;

        for major in &self.majors {
            match major {
                MajorImprovementType::Joinery => {
                    if self.resources[Resource::Wood] >= 7 {
                        ret += 3;
                    } else if self.resources[Resource::Wood] >= 5 {
                        ret += 2;
                    } else if self.resources[Resource::Wood] >= 3 {
                        ret += 1;
                    }
                }
                MajorImprovementType::Pottery => {
                    if self.resources[Resource::Clay] >= 7 {
                        ret += 3;
                    } else if self.resources[Resource::Clay] >= 5 {
                        ret += 2;
                    } else if self.resources[Resource::Clay] >= 3 {
                        ret += 1;
                    }
                }
                MajorImprovementType::BasketmakersWorkshop => {
                    if self.resources[Resource::Reed] >= 5 {
                        ret += 3;
                    } else if self.resources[Resource::Reed] >= 4 {
                        ret += 2;
                    } else if self.resources[Resource::Reed] >= 2 {
                        ret += 1;
                    }
                }
                _ => (),
            }
        }

        // TODO Score minors/occs

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

        for major in &self.majors {
            match major {
                MajorImprovementType::Fireplace2 => print!("[FP2]"),
                MajorImprovementType::Fireplace3 => print!("[FP3]"),
                MajorImprovementType::CookingHearth4 => print!("[CH4]"),
                MajorImprovementType::CookingHearth5 => print!("[CH5]"),
                MajorImprovementType::Well => print!("[WL]"),
                MajorImprovementType::ClayOven => print!("[CO]"),
                MajorImprovementType::StoneOven => print!("[SO]"),
                MajorImprovementType::Joinery => print!("[JY]"),
                MajorImprovementType::Pottery => print!("[PY]"),
                MajorImprovementType::BasketmakersWorkshop => print!("[BMW]"),
            }
        }
    }

    pub fn reset_for_next_round(&mut self) {
        if let Some(res) = self.promised_resources.pop() {
            self.take_res(&res);
        }
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
