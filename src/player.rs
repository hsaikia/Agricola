use crate::farm::{house_emoji, Animal, Field, House, Pasture, PlantedSeed};
use crate::major_improvements::{MajorImprovement, ALL_MAJORS};
use crate::primitives::{
    can_pay_for_resource, new_res, pay_for_resource, print_resources, take_resource,
    ConversionTime, Resource, ResourceConversion, Resources, RESOURCE_NAMES,
};
use crate::scoring;
use std::cmp;

const MAX_STABLES: u32 = 4;
const MAX_FAMILY_MEMBERS: u32 = 5;
const FARMYARD_SPACES: u32 = 15;
const STARTING_ROOMS: u32 = 2;
const STARTING_PEOPLE: u32 = 2;
const MAX_FENCES: u32 = 15;

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

#[derive(Clone, Hash)]
pub enum Kind {
    Human,
    UniformMachine,
    RandomMachine, // Random
    MCTSMachine,
}

#[derive(Clone, Hash)]
pub struct Player {
    // Animals in this resources array are the ones that are pets in the house and the ones that are kept in unfenced stables
    pub kind: Kind,
    pub resources: Resources,
    people_placed: u32,
    adults: u32,
    children: u32,
    pub begging_tokens: u32,
    build_room_cost: Resources,
    build_stable_cost: Resources,
    pub renovation_cost: Resources,
    pub major_cards: [bool; ALL_MAJORS.len()],
    conversions: Vec<ResourceConversion>,
    pub house: House,
    pub rooms: u32,
    pub fields: Vec<Field>,
    pub pastures: Vec<Pasture>,
    pub unfenced_stables: u32,
    pub fences_left: u32,
    promised_resources: Vec<Resources>,
}

impl Player {
    pub fn create_new(food: u32, p_kind: Kind) -> Self {
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
            kind: p_kind,
            resources: res,
            people_placed: 0,
            adults: STARTING_PEOPLE,
            children: 0,
            begging_tokens: 0,
            build_room_cost: room_cost,
            build_stable_cost: stable_cost,
            renovation_cost: reno_cost,
            major_cards: [false; ALL_MAJORS.len()],
            conversions: ResourceConversion::default_conversions(),
            house: House::Wood,
            rooms: STARTING_ROOMS,
            fields: Vec::new(),
            pastures: Vec::new(),
            unfenced_stables: 0,
            fences_left: MAX_FENCES,
            promised_resources: Vec::new(),
        }
    }

    pub fn num_stables(&self) -> u32 {
        let mut ret: u32 = 0;
        for p in &self.pastures {
            ret += p.stables;
        }
        ret += self.unfenced_stables;
        ret
    }

    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
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

    pub fn harvest_fields(&mut self) {
        for f in &mut self.fields {
            match f.seed {
                PlantedSeed::Grain => {
                    //print!("\nGrain harvested!");
                    self.resources[Resource::Grain] += 1;
                    f.amount -= 1;
                }
                PlantedSeed::Vegetable => {
                    //print!("\nVegetable harvested!");
                    self.resources[Resource::Vegetable] += 1;
                    f.amount -= 1;
                }
                PlantedSeed::Empty => (),
            }
            if f.amount == 0 {
                f.seed = PlantedSeed::Empty;
            }
        }
    }

    pub fn kind(&self) -> Kind {
        self.kind.clone()
    }

    // Call after moving animals from pastures to resources and then move them back
    fn breed_animals(res: &mut Resources, debug: bool) {
        if res[Resource::Sheep] > 1 {
            if debug {
                print!("\nBreeding Sheep!");
            }
            res[Resource::Sheep] += 1;
        }

        if res[Resource::Pigs] > 1 {
            if debug {
                print!("\nBreeding Pigs!");
            }
            res[Resource::Pigs] += 1;
        }

        if res[Resource::Cattle] > 1 {
            if debug {
                print!("\nBreeding Cattle!");
            }
            res[Resource::Cattle] += 1;
        }
    }

    fn optimize_converting_resources_to_food(&mut self, debug: bool, conv_time: &ConversionTime) {
        let mut max_res_to_convert_map = new_res();
        let mut res_value_map = new_res();
        for conv in &self.conversions {
            if let Some(ret) = conv.conversion_options(&self.resources, conv_time) {
                max_res_to_convert_map[ret.0.clone()] = ret.2;
                res_value_map[ret.0.clone()] = cmp::max(res_value_map[ret.0.clone()], ret.1);
            }
        }

        // Converts resources to food, one at a time, causing least changes to total score
        let food_required = 2 * self.adults + self.children;
        while food_required > self.resources[Resource::Food] {
            let mut res_present = false;
            let mut best_score: i32 = i32::MIN;
            let mut best_resource_idx = 0;

            for i in 0..max_res_to_convert_map.len() {
                if max_res_to_convert_map[i] > 0 {
                    res_present = true;
                    // Test
                    let mut tmp_player = self.clone();
                    tmp_player.resources[Resource::Food] += res_value_map[i];
                    tmp_player.resources[i] -= 1;
                    tmp_player.breed_and_reorg_animals(false);
                    let score = scoring::score_resources(&tmp_player, false);
                    if score > best_score {
                        best_score = score;
                        best_resource_idx = i;
                    }
                }
            }

            // If no conversion possible, break
            if !res_present {
                break;
            }

            // Convert best resource
            if debug {
                print!(
                    "\nConverting 1 {} to {} Food (Best Score {best_score}).",
                    RESOURCE_NAMES[best_resource_idx], res_value_map[best_resource_idx]
                );
            }
            self.resources[best_resource_idx] -= 1;
            max_res_to_convert_map[best_resource_idx] -= 1;
            self.resources[Resource::Food] += res_value_map[best_resource_idx];
        }
    }

    pub fn harvest(&mut self, debug: bool) {
        // Harvest grain and veggies
        self.harvest_fields();

        // Move all animals to the resources array
        self.add_animals_in_pastures_to_resources();

        // Convert resources to food if required
        self.optimize_converting_resources_to_food(debug, &ConversionTime::Harvest);

        // Breed animals and reorg them
        self.breed_and_reorg_animals(debug);

        // Pay for harvest
        let food_required = 2 * self.adults + self.children;
        let food_provided = cmp::min(food_required, self.resources[Resource::Food]);

        if debug && food_required > food_provided {
            print!(
                "\nBegging Tokens as {} food is missing :( ",
                food_required - food_provided
            );
        }

        self.resources[Resource::Food] -= food_provided;
        self.begging_tokens += food_required - food_provided;
    }

    pub fn breed_and_reorg_animals(&mut self, debug: bool) {
        // Breed animals
        Self::breed_animals(&mut self.resources, debug);

        // Place animals back in pastures
        self.reorg_animals(true);
    }

    fn num_free_animals(&self) -> u32 {
        self.resources[Resource::Sheep]
            + self.resources[Resource::Pigs]
            + self.resources[Resource::Cattle]
    }

    pub fn animals_as_resources(&self) -> Resources {
        let mut res = self.resources;
        for p in &self.pastures {
            match p.animal {
                Animal::Sheep => res[Resource::Sheep] += p.amount,
                Animal::Pigs => res[Resource::Pigs] += p.amount,
                Animal::Cattle => res[Resource::Cattle] += p.amount,
                Animal::Empty => (),
            }
        }
        res
    }

    fn add_animals_in_pastures_to_resources(&mut self) {
        for p in &mut self.pastures {
            match p.animal {
                Animal::Sheep => self.resources[Resource::Sheep] += p.amount,
                Animal::Pigs => self.resources[Resource::Pigs] += p.amount,
                Animal::Cattle => self.resources[Resource::Cattle] += p.amount,
                Animal::Empty => (),
            }
            p.amount = 0;
            p.animal = Animal::Empty;
        }
    }

    pub fn reorg_animals(&mut self, discard_leftovers: bool) {
        // Can free animals be kept in the house and UF stables?
        let free_animal_spaces = self.unfenced_stables + 1;
        if self.num_free_animals() <= free_animal_spaces {
            return;
        }

        // Try and fit in pastures
        if !self.pastures.is_empty() {
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
            self.pastures.sort_by_key(|k| 1000 - k.capacity());

            // Fill pastures
            // TODO this should be made generic
            for p in &mut self.pastures {
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

    pub fn build_major(
        &mut self,
        major_idx: usize,
        majors: &mut [bool; ALL_MAJORS.len()],
        debug: bool,
    ) {
        assert!(major_idx <= ALL_MAJORS.len());
        assert!(majors[major_idx]);
        let chosen_major: MajorImprovement = ALL_MAJORS[major_idx].clone();

        if debug {
            print!("\nChosen major {chosen_major:?}");
        }

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
                } else if fp3_built {
                    self.major_cards[MajorImprovement::Fireplace3.index()] = false;
                    // Add FP3 back to board
                    majors[MajorImprovement::Fireplace3.index()] = true;
                } else {
                    pay_for_resource(&chosen_major.cost(), &mut self.resources);
                }
            }
            _ => {
                pay_for_resource(&chosen_major.cost(), &mut self.resources);
            }
        }
        // Built
        self.major_cards[major_idx] = true;

        // Remove from board
        majors[major_idx] = false;

        // Add conversions
        self.conversions.clear();

        for (i, e) in self.major_cards.iter().enumerate() {
            if !e {
                continue;
            }
            if let Some(v) = ALL_MAJORS[i].conversions_to_food() {
                self.conversions.extend(v);
            }
        }

        // Default conversions are usually pretty bad, add them only at the end
        if self.major_cards[MajorImprovement::Fireplace2.index()]
            || self.major_cards[MajorImprovement::Fireplace3.index()]
            || self.major_cards[MajorImprovement::CookingHearth4.index()]
            || self.major_cards[MajorImprovement::CookingHearth5.index()]
        {
            self.conversions
                .extend(ResourceConversion::default_grain_conversions());
        } else {
            self.conversions
                .extend(ResourceConversion::default_conversions());
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
            MajorImprovement::ClayOven | MajorImprovement::StoneOven => {
                self.bake_bread();
            }
            _ => (),
        }
    }

    // Converts as much grain into food as possible
    // TODO - this might not be the best strategy! Make generic.
    pub fn bake_bread(&mut self) {
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

    // Sows alternatively between grain and veggies
    // TODO - this might not be the best strategy! Make generic.
    pub fn sow(&mut self) {
        let mut empty_field_idx: usize = 0;
        let mut grain_fields: u32 = 0;
        let mut veg_fields: u32 = 0;
        for (i, f) in &mut self.fields.iter().enumerate() {
            match f.seed {
                PlantedSeed::Grain => grain_fields += 1,
                PlantedSeed::Vegetable => veg_fields += 1,
                PlantedSeed::Empty => empty_field_idx = i,
            }
        }

        let field_to_sow = &mut self.fields[empty_field_idx];

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

        for f in &self.fields {
            if let PlantedSeed::Empty = f.seed {
                return true;
            }
        }
        false
    }

    pub fn fence(&mut self) {
        // Follow convention as mentioned in can_fence()
        let mut recurse: bool = false;
        for (p, w) in &FENCING_CHOICES[self.fences_left as usize] {
            if self.resources[Resource::Wood] >= *w
                && *p <= (self.empty_farmyard_spaces() + self.unfenced_stables)
            {
                let no_empty_farmyard_spaces_left: bool = self.empty_farmyard_spaces() == 0;
                self.pastures.push(Pasture::create_new(
                    *p,
                    &mut self.unfenced_stables,
                    no_empty_farmyard_spaces_left,
                ));
                self.fences_left -= *w;
                self.resources[Resource::Wood] -= *w;
                recurse = true;
                break;
            }
        }

        self.reorg_animals(false);

        if recurse && self.can_fence() {
            self.fence();
        }
    }

    pub fn can_fence(&self) -> bool {
        // No fences left
        if self.fences_left == 0 {
            return false;
        }

        // No empty farmyard space or an unfenced stable
        if self.empty_farmyard_spaces() == 0 && self.unfenced_stables == 0 {
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

        for (_p, w) in &FENCING_CHOICES[self.fences_left as usize] {
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
        self.family_members() < MAX_FAMILY_MEMBERS && self.family_members() < self.rooms
    }

    pub fn can_grow_family_without_room(&self) -> bool {
        self.family_members() < MAX_FAMILY_MEMBERS && self.family_members() >= self.rooms
    }

    pub fn renovate(&mut self) {
        assert!(self.can_renovate());
        // TODO for cards like Conservator this must be implemented in a more general way
        pay_for_resource(&self.renovation_cost, &mut self.resources);
        let current_type = &self.house;
        match current_type {
            House::Wood => {
                self.house = House::Clay;
                self.build_room_cost[Resource::Wood] = 0;
                self.build_room_cost[Resource::Clay] = 5;
                self.renovation_cost[Resource::Stone] = self.rooms;
                self.renovation_cost[Resource::Clay] = 0;
            }
            House::Clay => {
                self.house = House::Stone;
                self.build_room_cost[Resource::Clay] = 0;
                self.build_room_cost[Resource::Stone] = 5;
            }
            House::Stone => (),
        }
    }

    pub fn can_renovate(&self) -> bool {
        if let House::Stone = self.house {
            return false;
        }
        can_pay_for_resource(&self.renovation_cost, &self.resources)
    }

    pub fn all_possible_room_stable_builds(&self) -> Vec<Vec<usize>> {
        let mut ret = Vec::new();
        for num_rooms in 0..FARMYARD_SPACES {
            for num_stables in 0..MAX_STABLES {
                if num_rooms == 0 && num_stables == 0 {
                    continue;
                }

                let mut tmp_player = self.clone();
                let mut rooms_failed = false;
                let mut stables_failed = false;
                for _ in 0..num_rooms {
                    if tmp_player.can_build_room() {
                        tmp_player.build_room();
                    } else {
                        rooms_failed = true;
                    }
                }
                for _ in 0..num_stables {
                    if tmp_player.can_build_stable() {
                        tmp_player.build_stable();
                    } else {
                        stables_failed = true;
                    }
                }

                if rooms_failed && stables_failed {
                    return ret;
                }
                if !rooms_failed && !stables_failed {
                    ret.push(vec![num_rooms as usize, num_stables as usize]);
                }
            }
        }
        ret
    }

    // Builds a single room
    pub fn build_room(&mut self) {
        assert!(self.can_build_room());
        pay_for_resource(&self.build_room_cost, &mut self.resources);
        self.rooms += 1;

        match self.house {
            House::Wood => self.renovation_cost[Resource::Clay] += 1,
            House::Clay => self.renovation_cost[Resource::Stone] += 1,
            House::Stone => (),
        }
    }

    pub fn can_build_room(&self) -> bool {
        if self.empty_farmyard_spaces() == 0 {
            return false;
        }
        can_pay_for_resource(&self.build_room_cost, &self.resources)
    }

    // Builds a single stable
    pub fn build_stable(&mut self) {
        assert!(self.num_stables() < MAX_STABLES);
        assert!(self.can_build_stable());
        pay_for_resource(&self.build_stable_cost, &mut self.resources);
        // Add to pasture if possible, then unfenced
        let mut found: bool = false;
        for pasture in &mut self.pastures {
            if pasture.stables == 0 {
                pasture.stables = 1;
                found = true;
                break;
            }
        }
        if !found {
            self.unfenced_stables += 1;
        }
    }

    pub fn can_build_stable(&self) -> bool {
        if self.num_stables() >= MAX_STABLES {
            return false;
        }
        if self.empty_farmyard_spaces() == 0 {
            let mut all_filled: bool = true;
            for pasture in &self.pastures {
                if pasture.stables == 0 {
                    all_filled = false;
                }
            }
            if all_filled {
                return false;
            }
        }
        can_pay_for_resource(&self.build_stable_cost, &self.resources)
    }

    pub fn add_new_field(&mut self) {
        let field = Field::new();
        self.fields.push(field);
    }

    pub fn can_add_new_field(&self) -> bool {
        self.empty_farmyard_spaces() > 0
    }

    pub fn reset_for_next_round(&mut self) {
        if let Some(res) = self.promised_resources.pop() {
            take_resource(&res, &mut self.resources);
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

    pub fn display(&self) {
        println!("Score {}", scoring::score(self, false));
        print!(
            "House and Family [{}/{}]",
            "\u{1f464}".repeat(self.people_placed as usize),
            "\u{1f464}".repeat((self.adults - self.people_placed) as usize)
        );
        if self.children > 0 {
            print!("[{}]", "\u{1f476}".repeat(self.children as usize));
        }

        print!("[{}]", house_emoji(&self.house).repeat(self.rooms as usize));
        println!();

        print!("Resources ");
        print_resources(&self.resources);
        let ns: usize = self.unfenced_stables as usize;
        if ns > 0 {
            print!("[{}]", "\u{26fa}".repeat(ns));
        }

        if self.begging_tokens > 0 {
            print!("[{}]", "\u{1f37d}".repeat(self.begging_tokens as usize));
        }

        println!();

        if !self.pastures.is_empty() {
            print!("Pastures ");
            for p in &self.pastures {
                p.display();
            }
            println!();
        }

        if !self.fields.is_empty() {
            print!("Fields ");
            for f in &self.fields {
                f.display();
            }
            println!();
        }

        for (i, e) in self.major_cards.iter().enumerate() {
            if !e {
                continue;
            }
            print!("[{}]", &ALL_MAJORS[i].name());
        }

        println!();
    }
}
