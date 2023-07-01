use crate::agricola::algorithms::Kind;
use crate::agricola::farm::{Animal, Farm, House, Pasture, Seed, MAX_FENCES, NUM_FARMYARD_SPACES};
use crate::agricola::major_improvements::{Cheaper, MajorImprovement};
use crate::agricola::occupations::Occupation;
use crate::agricola::primitives::{
    can_pay_for_resource, new_res, pay_for_resource, print_resources, Resource, ResourceExchange,
    Resources,
};
use crate::agricola::scoring;
use rand::seq::SliceRandom;
use std::cmp;

const MAX_STABLES: usize = 4;
const MAX_FAMILY_MEMBERS: usize = 5;
const STARTING_PEOPLE: usize = 2;

lazy_static! {
    // map[idx] = (p, w) : Choice of p pastures using w wood when idx number of fences remain.
    static ref FENCING_CHOICES: [Vec<(usize, usize)>; 16] = [
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
    pub people_placed: usize,
    pub adults: usize,
    pub children: usize,
    pub begging_tokens: usize,
    build_room_cost: Resources,
    build_stable_cost: Resources,
    pub renovation_cost: Resources,
    pub major_cards: Vec<MajorImprovement>,
    pub house: House,
    pub pastures: Vec<Pasture>,
    pub unfenced_stables: usize,
    pub fences_left: usize,
    pub majors_used_for_harvest: Vec<MajorImprovement>,
    pub occupations: Vec<Occupation>,
    pub harvest_paid: bool,
    pub before_round_start: bool,
    pub farm: Farm,
}

impl Player {
    pub fn create_new(food: usize, p_kind: Kind) -> Self {
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
            pastures: vec![],
            unfenced_stables: 0,
            fences_left: MAX_FENCES,
            majors_used_for_harvest: vec![],
            occupations: vec![],
            harvest_paid: false,
            before_round_start: true,
            farm: Farm::new(),
        }
    }

    pub fn num_stables(&self) -> usize {
        let mut ret: usize = 0;
        for p in &self.pastures {
            ret += p.stables;
        }
        ret += self.unfenced_stables;
        ret
    }

    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    pub fn empty_farmyard_spaces(&self) -> usize {
        let mut pasture_spaces: usize = 0;
        for pasture in &self.pastures {
            pasture_spaces += pasture.farmyard_spaces;
        }

        NUM_FARMYARD_SPACES
            - self.farm.room_indices().len()
            - self.farm.field_indices().len()
            - pasture_spaces
            - self.unfenced_stables
    }

    pub fn harvest_fields(&mut self) {
        let crops = self.farm.harvest_fields();
        for crop in crops {
            match crop {
                Seed::Grain => self.resources[Resource::Grain] += 1,
                Seed::Vegetable => self.resources[Resource::Vegetable] += 1,
            }
        }
    }

    pub fn got_enough_food(&self) -> bool {
        2 * self.adults + self.children <= self.resources[Resource::Food]
    }

    pub fn kind(&self) -> Kind {
        self.kind.clone()
    }

    // Call after moving animals from pastures to resources and then move them back
    fn breed_animals(res: &mut Resources) {
        if res[Resource::Sheep] > 1 {
            res[Resource::Sheep] += 1;
        }

        if res[Resource::Pigs] > 1 {
            res[Resource::Pigs] += 1;
        }

        if res[Resource::Cattle] > 1 {
            res[Resource::Cattle] += 1;
        }
    }

    pub fn breed_and_reorg_animals(&mut self) {
        // Breed animals
        Self::breed_animals(&mut self.resources);

        // Place animals back in pastures
        self.reorg_animals(true);
    }

    fn num_free_animals(&self) -> usize {
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
            if self
                .major_cards
                .contains(&MajorImprovement::CookingHearth(Cheaper(true)))
                || self
                    .major_cards
                    .contains(&MajorImprovement::CookingHearth(Cheaper(false)))
            {
                self.resources[Resource::Food] += 2 * leftover[Resource::Sheep]
                    + 3 * leftover[Resource::Pigs]
                    + 4 * leftover[Resource::Cattle];
            } else if self
                .major_cards
                .contains(&MajorImprovement::Fireplace(Cheaper(true)))
                || self
                    .major_cards
                    .contains(&MajorImprovement::Fireplace(Cheaper(false)))
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
        if (self
            .major_cards
            .contains(&MajorImprovement::Fireplace(Cheaper(true)))
            || self
                .major_cards
                .contains(&MajorImprovement::Fireplace(Cheaper(false)))
            || self
                .major_cards
                .contains(&MajorImprovement::CookingHearth(Cheaper(true)))
            || self
                .major_cards
                .contains(&MajorImprovement::CookingHearth(Cheaper(false)))
            || self.major_cards.contains(&MajorImprovement::ClayOven)
            || self.major_cards.contains(&MajorImprovement::StoneOven))
            && self.resources[Resource::Grain] > 0
        {
            return true;
        }
        false
    }

    pub fn sow_field(&mut self, seed: &Seed) {
        self.farm.sow_field(seed);
        match seed {
            Seed::Grain => self.resources[Resource::Grain] -= 1,
            Seed::Vegetable => self.resources[Resource::Vegetable] -= 1,
        }
    }

    pub fn can_sow(&self) -> bool {
        (self.resources[Resource::Grain] > 0 || self.resources[Resource::Vegetable] > 0)
            && self.farm.can_sow()
    }

    pub fn fence(&mut self) {
        assert!(self.can_fence());
        // Follow convention as mentioned in can_fence()
        let mut recurse: bool = false;
        for (p, w) in &FENCING_CHOICES[self.fences_left] {
            if self.resources[Resource::Wood] >= *w
                && *p <= (self.empty_farmyard_spaces() + self.unfenced_stables)
            {
                let no_empty_farmyard_spaces_left: bool = self.empty_farmyard_spaces() == 0;
                self.pastures.push(Pasture::create_new(
                    *p,
                    &mut self.unfenced_stables,
                    no_empty_farmyard_spaces_left,
                ));

                // let best_spaces = self.farm.best_spaces(*p, &FarmyardSpace::Pasture);
                // assert!(best_spaces.len() == *p);
                // for space_idx in best_spaces {
                //     self.farm.farmyard_spaces[space_idx] = FarmyardSpace::Pasture;
                // }

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

        for (_p, w) in &FENCING_CHOICES[self.fences_left] {
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
        self.family_members() < MAX_FAMILY_MEMBERS
            && self.family_members() < self.farm.room_indices().len()
    }

    pub fn can_grow_family_without_room(&self) -> bool {
        self.family_members() < MAX_FAMILY_MEMBERS
    }

    pub fn renovate(&mut self) {
        assert!(self.can_renovate());
        // TODO for cards like Conservator this must be implemented in a more general way
        pay_for_resource(&self.renovation_cost, &mut self.resources);
        let current_type = &self.house;
        let rooms = self.farm.room_indices().len();

        match current_type {
            House::Wood => {
                self.house = House::Clay;
                self.build_room_cost[Resource::Wood] = 0;
                self.build_room_cost[Resource::Clay] = 5;
                self.renovation_cost[Resource::Stone] = rooms;
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
        let positions = self.farm.best_room_positions();
        let idx = positions.choose(&mut rand::thread_rng()).unwrap();
        self.farm.build_room(*idx);

        match self.house {
            House::Wood => self.renovation_cost[Resource::Clay] += 1,
            House::Clay => self.renovation_cost[Resource::Stone] += 1,
            House::Stone => (),
        }
    }

    pub fn can_build_room(&self) -> bool {
        if self.farm.best_room_positions().is_empty() {
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
        let idxs = self.farm.best_field_positions();
        assert!(!idxs.is_empty());
        let idx = idxs.choose(&mut rand::thread_rng()).unwrap();
        self.farm.add_field(*idx);
    }

    pub fn can_add_new_field(&self) -> bool {
        !self.farm.best_field_positions().is_empty()
    }

    pub fn reset_for_next_round(&mut self) {
        self.adults += self.children;
        self.children = 0;
        self.people_placed = 0;
        self.majors_used_for_harvest.clear();
        self.harvest_paid = false;
        self.before_round_start = true;
    }

    pub fn workers(&self) -> usize {
        self.adults
    }

    pub fn family_members(&self) -> usize {
        self.adults + self.children
    }

    pub fn increment_people_placed(&mut self) {
        self.people_placed += 1;
        self.before_round_start = false;
    }

    pub fn all_people_placed(&self) -> bool {
        self.people_placed == self.adults
    }

    pub fn can_use_exchange(&self, res_ex: &ResourceExchange) -> bool {
        self.resources[res_ex.from.clone()] >= res_ex.num_from
    }

    pub fn use_exchange(&mut self, res_ex: &ResourceExchange) {
        assert!(self.can_use_exchange(res_ex));
        self.resources[res_ex.from.clone()] -= res_ex.num_from;
        self.resources[res_ex.to.clone()] += res_ex.num_to;
    }

    pub fn has_cooking_improvement(&self) -> bool {
        self.major_cards
            .contains(&MajorImprovement::Fireplace(Cheaper(true)))
            | self
                .major_cards
                .contains(&MajorImprovement::Fireplace(Cheaper(false)))
            | self
                .major_cards
                .contains(&MajorImprovement::CookingHearth(Cheaper(true)))
            | self
                .major_cards
                .contains(&MajorImprovement::CookingHearth(Cheaper(false)))
    }

    pub fn has_resources_to_cook(&self) -> bool {
        self.resources[Resource::Sheep]
            + self.resources[Resource::Pigs]
            + self.resources[Resource::Cattle]
            + self.resources[Resource::Vegetable]
            > 0
    }

    pub fn can_play_occupation(&self, cheaper: bool) -> bool {
        let mut required_food = if cheaper { 1 } else { 2 };
        if self.occupations.is_empty() && cheaper {
            required_food = 0;
        }
        if self.occupations.len() < 2 && !cheaper {
            required_food = 1;
        }

        // If can pay directly
        if required_food <= self.resources[Resource::Food] {
            return true;
        }

        // If cannot pay directly, but can convert some resources
        required_food -= self.resources[Resource::Food];

        let raw_grain_and_veg =
            self.resources[Resource::Grain] + self.resources[Resource::Vegetable];
        if required_food <= raw_grain_and_veg {
            return true;
        }

        // Required food must be less than 3, and minimum food gained by cooking is 2
        if self.has_cooking_improvement() && self.has_resources_to_cook() {
            return true;
        }

        false
    }

    pub fn display(&self) {
        println!("Score {}", scoring::score(self, false));
        print!(
            "House and Family [{}/{}]",
            "\u{1f464}".repeat(self.people_placed),
            "\u{1f464}".repeat(self.adults - self.people_placed)
        );
        if self.children > 0 {
            print!("[{}]", "\u{1f476}".repeat(self.children));
        }

        print!("Resources ");
        print_resources(&self.resources);
        if self.unfenced_stables > 0 {
            print!("[{}]", "\u{26fa}".repeat(self.unfenced_stables));
        }

        if self.begging_tokens > 0 {
            print!("[{}]", "\u{1f37d}".repeat(self.begging_tokens));
        }

        println!();

        if !self.pastures.is_empty() {
            print!("Pastures ");
            for p in &self.pastures {
                p.display();
            }
            println!();
        }

        MajorImprovement::display(&self.major_cards);
        Occupation::display(&self.occupations);

        println!();
    }
}
