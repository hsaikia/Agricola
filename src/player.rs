use crate::algorithms::Kind;
use crate::farm::{house_emoji, Animal, Field, House, Pasture, PlantedSeed};
use crate::major_improvements::MajorImprovement;
use crate::primitives::{
    can_pay_for_resource, new_res, pay_for_resource, print_resources, Resource, ResourceExchange,
    Resources,
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
pub struct Player {
    // Animals in this resources array are the ones that are pets in the house and the ones that are kept in unfenced stables
    pub kind: Kind,
    pub resources: Resources,
    people_placed: u32,
    pub adults: u32,
    pub children: u32,
    pub begging_tokens: u32,
    build_room_cost: Resources,
    build_stable_cost: Resources,
    pub renovation_cost: Resources,
    pub major_cards: Vec<MajorImprovement>,
    pub house: House,
    pub rooms: u32,
    pub fields: Vec<Field>,
    pub pastures: Vec<Pasture>,
    pub unfenced_stables: u32,
    pub fences_left: u32,
    pub major_used_for_harvest: Vec<MajorImprovement>,
    pub harvest_paid: bool,
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
            major_cards: vec![],
            house: House::Wood,
            rooms: STARTING_ROOMS,
            fields: vec![],
            pastures: vec![],
            unfenced_stables: 0,
            fences_left: MAX_FENCES,
            major_used_for_harvest: vec![],
            harvest_paid: false,
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

        let r = self.rooms;
        let f = self.fields.len() as u32;
        let p = pasture_spaces;
        let s = self.unfenced_stables;

        if r + f + p + s > FARMYARD_SPACES {
            panic!("Error! {r} R {f} F {p} P {s} S existing together!");
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

    pub fn add_animals_in_pastures_to_resources(&mut self) {
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
            if self.major_cards.contains(&MajorImprovement::CookingHearth4)
                || self.major_cards.contains(&MajorImprovement::CookingHearth5)
            {
                self.resources[Resource::Food] += 2 * leftover[Resource::Sheep]
                    + 3 * leftover[Resource::Pigs]
                    + 4 * leftover[Resource::Cattle];
            } else if self.major_cards.contains(&MajorImprovement::Fireplace2)
                || self.major_cards.contains(&MajorImprovement::Fireplace2)
            {
                self.resources[Resource::Food] += 2 * leftover[Resource::Sheep]
                    + 2 * leftover[Resource::Pigs]
                    + 3 * leftover[Resource::Cattle];
            }
        }
    }

    pub fn can_bake_bread(&self) -> bool {
        // Check if any of the baking improvements are present
        // And at least one grain in supply
        if (self.major_cards.contains(&MajorImprovement::Fireplace2)
            || self.major_cards.contains(&MajorImprovement::Fireplace3)
            || self.major_cards.contains(&MajorImprovement::CookingHearth4)
            || self.major_cards.contains(&MajorImprovement::CookingHearth5)
            || self.major_cards.contains(&MajorImprovement::ClayOven)
            || self.major_cards.contains(&MajorImprovement::StoneOven))
            && self.resources[Resource::Grain] > 0
        {
            return true;
        }
        false
    }

    pub fn sow_field(&mut self, seed: &PlantedSeed) {
        for field in &mut self.fields {
            if field.sow(seed) {
                match seed {
                    PlantedSeed::Grain => self.resources[Resource::Grain] -= 1,
                    PlantedSeed::Vegetable => self.resources[Resource::Vegetable] -= 1,
                    _ => (),
                }
                break;
            }
        }
    }

    pub fn can_sow(&self) -> bool {
        (self.resources[Resource::Grain] > 0 || self.resources[Resource::Vegetable] > 0)
            && self
                .fields
                .iter()
                .any(|f| matches!(f.seed, PlantedSeed::Empty))
    }

    pub fn fence(&mut self) {
        assert!(self.can_fence());
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
        self.family_members() < MAX_FAMILY_MEMBERS
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
                    break;
                }
            }
            if all_filled {
                return false;
            }
        }
        can_pay_for_resource(&self.build_stable_cost, &self.resources)
    }

    pub fn add_new_field(&mut self) {
        assert!(self.can_add_new_field());
        let field = Field::new();
        self.fields.push(field);
    }

    pub fn can_add_new_field(&self) -> bool {
        self.empty_farmyard_spaces() > 0
    }

    pub fn reset_for_next_round(&mut self) {
        self.adults += self.children;
        self.children = 0;
        self.people_placed = 0;
        self.major_used_for_harvest.clear();
        self.harvest_paid = false;
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

    pub fn can_use_exchange(&self, res_ex: &ResourceExchange) -> bool {
        self.resources[res_ex.from.clone()] >= res_ex.num_from
    }

    pub fn use_exchange(&mut self, res_ex: &ResourceExchange) {
        self.resources[res_ex.from.clone()] -= res_ex.num_from;
        self.resources[res_ex.to.clone()] += res_ex.num_to;
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

        MajorImprovement::display(&self.major_cards);

        println!();
    }
}
