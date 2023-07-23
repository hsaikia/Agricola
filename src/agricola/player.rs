use super::algorithms::PlayerType;
use super::farm::{Animal, Farm, FarmyardSpace, House, Seed};
use super::major_improvements::{Cheaper, MajorImprovement};
use super::occupations::Occupation;
use super::primitives::{
    can_pay_for_resource, new_res, pay_for_resource, Resource, ResourceExchange, Resources,
    RESOURCE_EMOJIS,
};
use rand::seq::SliceRandom;

const MAX_FAMILY_MEMBERS: usize = 5;
const STARTING_PEOPLE: usize = 2;

#[derive(Clone, Hash)]
pub struct Player {
    // Animals in this resources array are the ones that are pets in the house and the ones that are kept in unfenced stables
    pub player_type: PlayerType,
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
    pub majors_used_for_harvest: Vec<MajorImprovement>,
    pub occupations: Vec<Occupation>,
    pub harvest_paid: bool,
    pub before_round_start: bool,
    pub farm: Farm,
}

impl Player {
    pub fn create_new(food: usize, player_type: PlayerType) -> Self {
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
            player_type,
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
            majors_used_for_harvest: vec![],
            occupations: vec![],
            harvest_paid: false,
            before_round_start: true,
            farm: Farm::new(),
        }
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

    pub fn player_type(&self) -> PlayerType {
        self.player_type.clone()
    }

    pub fn reorg_animals(&mut self, breed: bool) {
        self.farm.reorg_animals(&mut self.resources, breed);

        let sheep = self.resources[Resource::Sheep];
        self.resources[Resource::Sheep] = 0;
        let pigs = self.resources[Resource::Pigs];
        self.resources[Resource::Pigs] = 0;
        let cattle = self.resources[Resource::Cattle];
        self.resources[Resource::Cattle] = 0;

        if self
            .major_cards
            .contains(&MajorImprovement::CookingHearth(Cheaper(true)))
            || self
                .major_cards
                .contains(&MajorImprovement::CookingHearth(Cheaper(false)))
        {
            self.resources[Resource::Food] += 2 * sheep + 3 * pigs + 4 * cattle;
        } else if self
            .major_cards
            .contains(&MajorImprovement::Fireplace(Cheaper(true)))
            || self
                .major_cards
                .contains(&MajorImprovement::Fireplace(Cheaper(false)))
        {
            self.resources[Resource::Food] += 2 * sheep + 2 * pigs + 3 * cattle;
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

    pub fn fencing_choices(&self) -> Vec<Vec<usize>> {
        let mut ret: Vec<Vec<usize>> = Vec::new();
        let fencing_arrangements = self.farm.fencing_options(self.resources[Resource::Wood]);
        for idxs in fencing_arrangements {
            ret.push(idxs);
        }
        ret
    }

    pub fn fence(&mut self, fence_indices: &Vec<usize>) {
        assert!(self.can_fence());
        let fencing_options = self.farm.fencing_options(self.resources[Resource::Wood]);
        let mut wood = 0;
        for (_i, fo) in fencing_options.iter().enumerate() {
            if fo == fence_indices {
                wood = fo.len();
            }
        }

        self.farm.fence_spaces(fence_indices);
        self.resources[Resource::Wood] -= wood;
    }

    pub fn can_fence(&self) -> bool {
        !self
            .farm
            .fencing_options(self.resources[Resource::Wood])
            .is_empty()
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
        let positions = self.farm.best_stable_positions();
        let idx = positions.choose(&mut rand::thread_rng()).unwrap();
        self.farm.build_stable(*idx);
    }

    pub fn can_build_stable(&self) -> bool {
        if !self.farm.can_build_stable() {
            return false;
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

    pub fn display_resources(&self) -> String {
        let res = &self.resources;
        let mut ret = format!(
            "\n\n\n\n{:2} {}   ",
            res[Resource::Wood],
            RESOURCE_EMOJIS[Resource::Wood as usize]
        );
        ret = format!(
            "{ret}{:2} {}   ",
            res[Resource::Clay],
            RESOURCE_EMOJIS[Resource::Clay as usize]
        );
        ret = format!(
            "{ret}{:2} {}   ",
            res[Resource::Stone],
            RESOURCE_EMOJIS[Resource::Stone as usize]
        );
        ret = format!(
            "{ret}{:2} {}",
            res[Resource::Reed],
            RESOURCE_EMOJIS[Resource::Reed as usize]
        );
        ret = format!(
            "{ret}\n{:2} {}   ",
            res[Resource::Grain],
            RESOURCE_EMOJIS[Resource::Grain as usize]
        );
        ret = format!(
            "{ret}{:2} {}   ",
            res[Resource::Vegetable],
            RESOURCE_EMOJIS[Resource::Vegetable as usize]
        );
        ret = format!(
            "{ret}{:2} {}   ",
            res[Resource::Food],
            RESOURCE_EMOJIS[Resource::Food as usize]
        );
        ret = format!("{ret}{:2} \u{1f37d}", self.begging_tokens);
        ret = format!(
            "{ret}\n{:2} {}   ",
            res[Resource::Sheep],
            RESOURCE_EMOJIS[Resource::Sheep as usize]
        );
        ret = format!(
            "{ret}{:2} {}   ",
            res[Resource::Pigs],
            RESOURCE_EMOJIS[Resource::Pigs as usize]
        );
        ret = format!(
            "{ret}{:2} {}",
            res[Resource::Cattle],
            RESOURCE_EMOJIS[Resource::Cattle as usize]
        );
        ret = format!("{ret}\n\n{:2} 👤   ", self.adults);
        ret = format!("{ret}{:2} 👶", self.children);

        ret = format!("{ret}\n\n{}", MajorImprovement::display(&self.major_cards));
        ret = format!("{ret}\n\n{}", Occupation::display(&self.occupations));
        ret
    }

    pub fn display_farm(&self) -> (String, String) {
        const SX: usize = 5;
        const SY: usize = 3;
        let mut ret = String::from("\n\n\n");
        let mut stuff = String::from("\n\n\n");
        for ii in 0..2 * SY + 1 {
            for jj in 0..2 * SX + 1 {
                let fidx = ii * (2 * SX + 1) + jj;

                if (ii + jj) % 2 == 1 {
                    // Fence
                    if self.farm.fences[fidx] {
                        if ii % 2 == 0 {
                            ret = format!("{ret} ━━ ");
                            stuff = format!("{stuff} ━━ ");
                        } else {
                            ret = format!("{ret}┃");
                            stuff = format!("{stuff}┃");
                        }
                    } else if ii % 2 == 0 {
                        ret = format!("{ret}    ");
                        stuff = format!("{stuff}    ");
                    } else {
                        ret = format!("{ret} ");
                        stuff = format!("{stuff} ");
                    }
                } else if ii % 2 == 1 && jj % 2 == 1 {
                    // Farmyard space
                    let idx = (ii / 2) * 5 + jj / 2;
                    match self.farm.farmyard_spaces[idx] {
                        FarmyardSpace::Empty => {
                            ret = format!("{ret} 🔲 ");
                            stuff = format!("{stuff} 🔲 ");
                        }
                        FarmyardSpace::Room => {
                            match self.house {
                                House::Wood => ret = format!("{ret} 🛖 "),
                                House::Clay => ret = format!("{ret} 🏠 "),
                                House::Stone => ret = format!("{ret} 🏰 "),
                            }
                            stuff = format!("{stuff} 🔲 ");
                        }
                        FarmyardSpace::Field(opt_seed) => {
                            ret = format!("{ret} 🟩 ");
                            if let Some((seed, amt)) = opt_seed {
                                match seed {
                                    Seed::Grain => stuff = format!("{stuff}{amt} 🌾"),
                                    Seed::Vegetable => stuff = format!("{stuff}{amt} 🎃"),
                                }
                            } else {
                                stuff = format!("{stuff} 🔲 ");
                            }
                        }
                        FarmyardSpace::UnfencedStable(opt_animal) => {
                            ret = format!("{ret} 🔶 ");
                            if let Some((animal, amt)) = opt_animal {
                                match animal {
                                    Animal::Sheep => stuff = format!("{stuff}{amt} 🐑"),
                                    Animal::Pigs => stuff = format!("{stuff}{amt} 🐖"),
                                    Animal::Cattle => stuff = format!("{stuff}{amt} 🐄"),
                                }
                            } else {
                                stuff = format!("{stuff} 🔲 ");
                            }
                        }
                        FarmyardSpace::FencedPasture(opt_animal, has_stable) => {
                            if has_stable {
                                ret = format!("{ret} 🔶 ");
                            } else {
                                ret = format!("{ret} 🔲 ");
                            }
                            if let Some((animal, amt)) = opt_animal {
                                match animal {
                                    Animal::Sheep => stuff = format!("{stuff}{amt} 🐑"),
                                    Animal::Pigs => stuff = format!("{stuff}{amt} 🐖"),
                                    Animal::Cattle => stuff = format!("{stuff}{amt} 🐄"),
                                }
                            } else {
                                stuff = format!("{stuff} 🔲 ");
                            }
                        }
                    }
                } else {
                    ret = format!("{ret}+");
                    stuff = format!("{stuff}+");
                }
            }
            ret = format!("{}\n", ret);
            stuff = format!("{}\n", stuff);
        }
        (ret, stuff)
    }
}
