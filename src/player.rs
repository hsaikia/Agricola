use crate::farm::{Animal, Farm, Field, House, Pasture, PlantedSeed};
use crate::major_improvements::{MajorImprovement, ALL_MAJORS};
use crate::primitives::{
    can_pay_for_resource, new_res, pay_for_resource, print_resources, ConversionTime, Resource,
    ResourceConversion, Resources,
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

pub struct Player {
    // Animals in this resources array are the ones that are pets in the house and the ones that are kept in unfenced stables
    resources: Resources,
    people_placed: u32,
    adults: u32,
    children: u32,
    begging_tokens: u32,
    build_room_cost: Resources,
    build_stable_cost: Resources,
    renovation_cost: Resources,
    major_cards: [bool; ALL_MAJORS.len()],
    conversions: Vec<ResourceConversion>,
    farm: Farm,
    promised_resources: Vec<Resources>,
}

impl Player {
    pub fn create_new(food: u32) -> Self {
        let mut res = new_res();
        res[Resource::Food] = food;

        let mut room_cost = new_res();
        room_cost[Resource::Wood] = 5;
        room_cost[Resource::Reed] = 2;

        let mut stable_cost = new_res();
        stable_cost[Resource::Wood] = 2;

        let mut reno_cost = new_res();
        reno_cost[Resource::Clay] = 2;
        reno_cost[Resource::Reed] = 1;

        Player {
            resources: res,
            people_placed: 0,
            adults: 2,
            children: 0,
            begging_tokens: 0,
            build_room_cost: room_cost,
            build_stable_cost: stable_cost,
            renovation_cost: reno_cost,
            major_cards: [false; ALL_MAJORS.len()],
            conversions: ResourceConversion::default_conversions(),
            farm: Farm::new(),
            promised_resources: Vec::new(),
        }
    }

    pub fn harvest(&mut self) {
        let food_required = 2 * self.adults + self.children;

        // Harvest grain and veggies
        let (gr, veg) = self.farm.harvest_fields();
        self.resources[Resource::Grain] += gr;
        self.resources[Resource::Vegetable] += veg;

        // Move animals to the resources array if some of them need to be cooked
        if food_required > self.resources[Resource::Food] {
            self.add_animals_in_pastures_to_resources();
        }

        // If food isn't enough, try and convert some resources
        while food_required > self.resources[Resource::Food] {
            let mut found = false;
            for conv in &mut self.conversions {
                if conv.can_convert(&self.resources, &ConversionTime::Harvest) {
                    //print!("\nUsing conversion {:?}", conv);
                    conv.convert_once(&mut self.resources, &ConversionTime::Harvest);
                    found = true;
                }

                if food_required <= self.resources[Resource::Food] {
                    break;
                }
            }

            if !found {
                break;
            }
        }

        // Reset conversions
        for conv in &mut self.conversions {
            conv.reset();
        }

        // Breed animals
        if self.resources[Resource::Sheep] > 1 {
            print!("\nBreeding Sheep!");
            self.resources[Resource::Sheep] += 1;
        }

        if self.resources[Resource::Pigs] > 1 {
            print!("\nBreeding Pigs!");
            self.resources[Resource::Pigs] += 1;
        }

        if self.resources[Resource::Cattle] > 1 {
            print!("\nBreeding Cattle!");
            self.resources[Resource::Cattle] += 1;
        }

        // Place animals back in pastures
        self.reorg_animals(true);

        let food_provided = cmp::min(food_required, self.resources[Resource::Food]);
        self.resources[Resource::Food] -= food_provided;
        if food_required > food_provided {
            print!(
                "\nBegging Tokens as {} food is missing :( ",
                food_required - food_provided
            );
        }
        self.begging_tokens += food_required - food_provided;
    }

    fn num_free_animals(&self) -> u32 {
        self.resources[Resource::Sheep]
            + self.resources[Resource::Pigs]
            + self.resources[Resource::Cattle]
    }

    fn add_animals_in_pastures_to_resources(&mut self) {
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
    }

    pub fn reorg_animals(&mut self, discard_leftovers: bool) {
        // Can free animals be kept in the house and UF stables?
        let free_animal_spaces = self.farm.unfenced_stables + 1;
        if self.num_free_animals() <= free_animal_spaces {
            return;
        }

        // Try and fit in pastures
        if !self.farm.pastures.is_empty() {
            // Add pasture animals to the resources (free) array
            self.add_animals_in_pastures_to_resources();

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
        let mut leftover: Resources = new_res();

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

        if !discard_leftovers {
            // Eat the leftover animals :(
            for conv in &mut self.conversions {
                conv.convert_all(&mut leftover, &ConversionTime::Any);
                conv.reset();
            }
            self.resources[Resource::Food] += leftover[Resource::Food];
        }
    }

    fn available_majors_to_build(&self, majors: &[bool; ALL_MAJORS.len()]) -> Vec<usize> {
        // If one of the FPs are already built
        let fp2_built: bool = self.major_cards[MajorImprovement::Fireplace2.index()];
        let fp3_built: bool = self.major_cards[MajorImprovement::Fireplace3.index()];

        let mut available = [false; ALL_MAJORS.len()];

        for idx in 0..ALL_MAJORS.len() {
            if !majors[idx] {
                continue;
            }

            // If FP2 or FP3 is already built
            if fp2_built || fp3_built {
                if idx == MajorImprovement::CookingHearth4.index()
                    || idx == MajorImprovement::CookingHearth5.index()
                {
                    available[idx] = true;
                }

                if idx == MajorImprovement::Fireplace2.index()
                    || idx == MajorImprovement::Fireplace3.index()
                {
                    continue;
                }
            }
            if can_pay_for_resource(&ALL_MAJORS[idx].cost(), &self.resources) {
                available[idx] = true;
            }
        }

        // If CH4 and CH5 are both present remove the expensive one
        if available[MajorImprovement::CookingHearth4.index()]
            && available[MajorImprovement::CookingHearth5.index()]
        {
            available[MajorImprovement::CookingHearth5.index()] = false;
        }

        // If FP2 and FP3 are both present remove the expensive one
        if available[MajorImprovement::Fireplace2.index()]
            && available[MajorImprovement::Fireplace3.index()]
        {
            available[MajorImprovement::Fireplace3.index()] = false;
        }

        // Choose random index
        let mut available_indices = vec![];

        for (i, e) in available.iter().enumerate() {
            if *e {
                available_indices.push(i);
            }
        }

        available_indices
    }

    // TODO - make this generic
    // Currently builds a random major
    pub fn build_major(&mut self, majors: &mut [bool; ALL_MAJORS.len()]) {
        let available_indices = self.available_majors_to_build(majors);
        assert!(!available_indices.is_empty());

        let idx = rand::thread_rng().gen_range(0..available_indices.len());
        let chosen_major: MajorImprovement = ALL_MAJORS[available_indices[idx]].clone();

        println!("Chosen major {:?}", chosen_major);

        // If one of the FPs are already built
        let fp2_built: bool = self.major_cards[MajorImprovement::Fireplace2.index()];
        let fp3_built: bool = self.major_cards[MajorImprovement::Fireplace3.index()];

        // Build major
        match chosen_major {
            // Pay
            MajorImprovement::CookingHearth4 | MajorImprovement::CookingHearth5 => {
                if fp2_built {
                    self.major_cards[MajorImprovement::Fireplace2.index()] = false;
                    // Add FP2 back to board
                    majors[MajorImprovement::Fireplace2.index()] = true;
                }
                if fp3_built {
                    self.major_cards[MajorImprovement::Fireplace3.index()] = false;
                    // Add FP3 back to board
                    majors[MajorImprovement::Fireplace3.index()] = true;
                }
            }
            _ => {
                pay_for_resource(&chosen_major.cost(), &mut self.resources);
            }
        }
        // Built
        self.major_cards[available_indices[idx]] = true;

        // Remove from board
        majors[available_indices[idx]] = false;

        // Add conversions
        self.conversions.clear();
        self.conversions
            .extend(ResourceConversion::default_conversions());

        for (i, e) in self.major_cards.iter().enumerate() {
            if !e {
                continue;
            }
            if let Some(v) = ALL_MAJORS[i].conversions_to_food() {
                self.conversions.extend(v);
            }
        }

        // Free actions + resources
        match chosen_major {
            MajorImprovement::Well => {
                while self.promised_resources.len() < 5 {
                    let res = new_res();
                    self.promised_resources.push(res);
                }
                for i in 0..5 {
                    self.promised_resources[i][Resource::Food] += 1;
                }
            }
            MajorImprovement::ClayOven => {
                self.bake_bread();
            }
            MajorImprovement::StoneOven => {
                self.bake_bread();
            }
            _ => (),
        }
    }

    fn bake_bread(&mut self) {
        let mut best_conversion = self.resources;
        for conv in &mut self.conversions {
            let mut res = self.resources;
            conv.convert_all(&mut res, &ConversionTime::Bake);
            if res[Resource::Food] > best_conversion[Resource::Food] {
                best_conversion = res;
            }
        }
        self.resources = best_conversion;
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

        self.reorg_animals(false);

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
        self.children += 1;
    }

    pub fn grow_family_without_room(&mut self) {
        assert!(self.can_grow_family_without_room());
        self.children += 1;
    }

    pub fn can_grow_family_with_room(&self) -> bool {
        self.family_members() < MAX_FAMILY_MEMBERS && self.family_members() < self.farm.rooms
    }

    pub fn can_grow_family_without_room(&self) -> bool {
        self.family_members() < MAX_FAMILY_MEMBERS && self.family_members() >= self.farm.rooms
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

    pub fn can_build_major(&self, majors: &[bool; ALL_MAJORS.len()]) -> bool {
        !self.available_majors_to_build(majors).is_empty()
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
        ret += 3 * self.family_members() as i32;

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
        for (i, e) in self.major_cards.iter().enumerate() {
            if !e {
                continue;
            }
            ret += ALL_MAJORS[i].points() as i32;
            match ALL_MAJORS[i] {
                MajorImprovement::Joinery => {
                    if self.resources[Resource::Wood] >= 7 {
                        ret += 3;
                    } else if self.resources[Resource::Wood] >= 5 {
                        ret += 2;
                    } else if self.resources[Resource::Wood] >= 3 {
                        ret += 1;
                    }
                }
                MajorImprovement::Pottery => {
                    if self.resources[Resource::Clay] >= 7 {
                        ret += 3;
                    } else if self.resources[Resource::Clay] >= 5 {
                        ret += 2;
                    } else if self.resources[Resource::Clay] >= 3 {
                        ret += 1;
                    }
                }
                MajorImprovement::BasketmakersWorkshop => {
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
        for it in acc_res.iter().zip(self.resources.iter_mut()) {
            let (a, b) = it;
            *b += a;
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
            self.adults,
            self.score()
        );
        if self.children > 0 {
            print!("[{} Children]", self.children);
        }
        if self.begging_tokens > 0 {
            print!("[{} :(]", self.begging_tokens);
        }
        print_resources(&self.resources);
        self.farm.display();

        for (i, e) in self.major_cards.iter().enumerate() {
            if !e {
                continue;
            }
            match ALL_MAJORS[i] {
                MajorImprovement::Fireplace2 => print!("[FP2]"),
                MajorImprovement::Fireplace3 => print!("[FP3]"),
                MajorImprovement::CookingHearth4 => print!("[CH4]"),
                MajorImprovement::CookingHearth5 => print!("[CH5]"),
                MajorImprovement::Well => print!("[WL]"),
                MajorImprovement::ClayOven => print!("[CO]"),
                MajorImprovement::StoneOven => print!("[SO]"),
                MajorImprovement::Joinery => print!("[JY]"),
                MajorImprovement::Pottery => print!("[PY]"),
                MajorImprovement::BasketmakersWorkshop => print!("[BMW]"),
            }
        }
    }

    pub fn reset_for_next_round(&mut self) {
        if let Some(res) = self.promised_resources.pop() {
            self.take_res(&res);
        }
        self.adults += self.children;
        self.children = 0;
        self.people_placed = 0;
    }

    pub fn workers(&self) -> u32 {
        self.adults
    }

    pub fn family_members(&self) -> u32 {
        self.adults + self.children
    }

    pub fn increment_people_placed(&mut self) {
        self.people_placed += 1;
    }

    pub fn all_people_placed(&self) -> bool {
        self.people_placed == self.adults
    }
}
