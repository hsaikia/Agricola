use crate::farm::{Animal, Field, House, Pasture, PlantedSeed};
use crate::scoring;
use crate::major_improvements::{MajorImprovement, ALL_MAJORS};
use crate::primitives::{
    can_pay_for_resource, new_res, pay_for_resource, print_resources, ConversionTime, Resource,
    ResourceConversion, Resources,
};
use rand::Rng;
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

#[derive(Clone)]
pub enum PlayerKind {
    Human,
    Machine,
    DumbMachine, // Random
}

#[derive(Clone)]
pub struct Player {
    // Animals in this resources array are the ones that are pets in the house and the ones that are kept in unfenced stables
    pub kind: PlayerKind,
    pub resources: Resources,
    people_placed: u32,
    adults: u32,
    children: u32,
    pub begging_tokens: u32,
    build_room_cost: Resources,
    build_stable_cost: Resources,
    renovation_cost: Resources,
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
    pub fn create_new(food: u32, p_kind: PlayerKind) -> Self {
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

    pub fn empty_farmyard_spaces(&self) -> u32 {
        let mut pasture_spaces: u32 = 0;
        for pasture in &self.pastures {
            pasture_spaces += pasture.farmyard_spaces;
        }

        if self.rooms + self.fields.len() as u32 + pasture_spaces + self.unfenced_stables > FARMYARD_SPACES {
            println!("\nError! {} Rooms {} fields {} pasture spaces ({} pastures) and {} UF found!", self.rooms, self.fields.len() as u32, pasture_spaces, self.pastures.len(), self.unfenced_stables);
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
                    //print!("\nGrain harvested!");
                    harvested_grain += 1;
                    f.amount -= 1;
                }
                PlantedSeed::Vegetable => {
                    //print!("\nVegetable harvested!");
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

    pub fn kind(&self) -> PlayerKind {
        self.kind.clone()
    }

    pub fn harvest(&mut self, debug : bool) {
        let food_required = 2 * self.adults + self.children;

        // Harvest grain and veggies
        let (gr, veg) = self.harvest_fields();
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
            if debug {
                print!("\nBreeding Sheep!");
            }
            self.resources[Resource::Sheep] += 1;
        }

        if self.resources[Resource::Pigs] > 1 {
            if debug {
                print!("\nBreeding Pigs!");
            }
            self.resources[Resource::Pigs] += 1;
        }

        if self.resources[Resource::Cattle] > 1 {
            if debug {
                print!("\nBreeding Cattle!");
            }
            self.resources[Resource::Cattle] += 1;
        }

        // Place animals back in pastures
        self.reorg_animals(true);

        let food_provided = cmp::min(food_required, self.resources[Resource::Food]);
        self.resources[Resource::Food] -= food_provided;
        if debug && food_required > food_provided {
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
        for p in &mut self.pastures {
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
        // Or if CH4 is already built remove CH5
        if (available[MajorImprovement::CookingHearth4.index()]
            && available[MajorImprovement::CookingHearth5.index()])
            || self.major_cards[MajorImprovement::CookingHearth4.index()]
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

        //println!("Chosen major {:?}", chosen_major);

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
            MajorImprovement::ClayOven => {
                self.bake_bread();
            }
            MajorImprovement::StoneOven => {
                self.bake_bread();
            }
            _ => (),
        }
    }

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
            if self.resources[Resource::Wood] >= *w && *p <= (self.empty_farmyard_spaces() + self.unfenced_stables) {
                let no_empty_farmyard_spaces_left : bool = self.empty_farmyard_spaces() == 0;
                self.pastures
                    .push(Pasture::create_new(*p, &mut self.unfenced_stables, no_empty_farmyard_spaces_left));
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
            }
            House::Clay => {
                self.house = House::Stone;
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
            self.rooms += 1;

            match self.house {
                House::Wood => self.renovation_cost[Resource::Clay] += 1,
                House::Clay => self.renovation_cost[Resource::Stone] += 1,
                _ => (),
            }
        }
    }

    pub fn build_stables(&mut self) {
        assert!(self.num_stables() < MAX_STABLES);
        assert!(self.can_build_stables());
        // TODO - this function should return a Choices array
        // But we follow some convention for now
        // Build as many stables as possible
        while self.can_build_stables() {
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
    }

    pub fn can_renovate(&self) -> bool {
        if let House::Stone = self.house {
            return false;
        }
        can_pay_for_resource(&self.renovation_cost, &self.resources)
    }

    pub fn can_build_rooms(&self) -> bool {
        if self.empty_farmyard_spaces() == 0 {
            return false;
        }
        can_pay_for_resource(&self.build_room_cost, &self.resources)
    }

    pub fn can_build_stables(&self) -> bool {
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

    pub fn can_build_major(&self, majors: &[bool; ALL_MAJORS.len()]) -> bool {
        !self.available_majors_to_build(majors).is_empty()
    }

    pub fn take_res(&mut self, acc_res: &Resources) {
        for it in acc_res.iter().zip(self.resources.iter_mut()) {
            let (a, b) = it;
            *b += a;
        }
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

    pub fn display(&self) {
        print!(
            "Player ({}/{}) SCORE {} has ",
            self.people_placed,
            self.adults,
            scoring::score(&self)
        );
        if self.children > 0 {
            print!("[{} Children]", self.children);
        }
        if self.begging_tokens > 0 {
            print!("[{} :(]", self.begging_tokens);
        }
        print_resources(&self.resources);
        
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
}
