use super::algorithms::PlayerType;
use super::farm::{Farm, House, Seed};
use super::fencing::PastureConfig;
use super::major_improvements::MajorImprovement;
use super::occupations::Occupation;
use super::primitives::*;
use super::state::{COOKING_HEARTH_INDICES, FIREPLACE_INDICES};

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
    pub house: House,
    pub occupations: Vec<Occupation>,
    pub harvest_paid: bool,
    pub before_round_start: bool,
    pub farm: Farm,
    pub has_cooking_improvement: bool,
}

impl Player {
    pub fn create_new(food: usize, player_type: PlayerType) -> Self {
        let mut res = new_res();
        res[Food.index()] = food;

        let mut room_cost = new_res();
        room_cost[Wood.index()] = 5;
        room_cost[Reed.index()] = 2;

        let mut stable_cost = new_res();
        stable_cost[Wood.index()] = 2;

        let mut reno_cost = new_res();
        reno_cost[Clay.index()] = 2;
        reno_cost[Reed.index()] = 1;

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
            house: House::Wood,
            occupations: vec![],
            harvest_paid: false,
            before_round_start: true,
            farm: Farm::new(),
            has_cooking_improvement: false,
        }
    }

    pub fn harvest_fields(&mut self) {
        let crops = self.farm.harvest_fields();
        for crop in crops {
            match crop {
                Seed::Grain => self.resources[Grain.index()] += 1,
                Seed::Vegetable => self.resources[Vegetable.index()] += 1,
            }
        }
    }

    pub fn got_enough_food(&self) -> bool {
        2 * self.adults + self.children <= self.resources[Food.index()]
    }

    pub fn player_type(&self) -> PlayerType {
        self.player_type.clone()
    }

    pub fn reorg_animals(
        &mut self,
        player_idx: usize,
        majors: &[(MajorImprovement, Option<usize>, usize)],
        breed: bool,
    ) {
        //self.farm.reorg_animals(&mut self.resources, breed);

        let sheep = self.resources[Sheep.index()];
        self.resources[Sheep.index()] = 0;
        let pigs = self.resources[Boar.index()];
        self.resources[Boar.index()] = 0;
        let cattle = self.resources[Cattle.index()];
        self.resources[Cattle.index()] = 0;

        let mut owns_ch = false;
        for ch_idx in COOKING_HEARTH_INDICES {
            if Some(player_idx) == majors[ch_idx].1 {
                self.resources[Food.index()] += 2 * sheep + 3 * pigs + 4 * cattle;
                owns_ch = true;
                break;
            }
        }

        if !owns_ch {
            for fp_idx in FIREPLACE_INDICES {
                if Some(player_idx) == majors[fp_idx].1 {
                    self.resources[Food.index()] += 2 * sheep + 2 * pigs + 3 * cattle;
                    return;
                }
            }
        }
    }

    pub fn can_bake_bread(
        &self,
        player_idx: usize,
        majors: &[(MajorImprovement, Option<usize>, usize)],
    ) -> bool {
        // Check if any of the baking improvements are present
        // And at least one grain in supply

        (Some(player_idx) == majors[MajorImprovement::ClayOven.index()].1
            || Some(player_idx) == majors[MajorImprovement::StoneOven.index()].1
            || Some(player_idx) == majors[MajorImprovement::Fireplace { cheaper: true }.index()].1
            || Some(player_idx) == majors[MajorImprovement::Fireplace { cheaper: false }.index()].1
            || Some(player_idx)
                == majors[MajorImprovement::CookingHearth { cheaper: true }.index()].1
            || Some(player_idx)
                == majors[MajorImprovement::CookingHearth { cheaper: false }.index()].1)
            && self.resources[Grain.index()] > 0
    }

    pub fn sow_field(&mut self, seed: &Seed) {
        self.farm.sow_field(seed);
        match seed {
            Seed::Grain => self.resources[Grain.index()] -= 1,
            Seed::Vegetable => self.resources[Vegetable.index()] -= 1,
        }
    }

    pub fn can_sow(&self) -> bool {
        (self.resources[Grain.index()] > 0 || self.resources[Vegetable.index()] > 0)
            && self.farm.can_sow()
    }

    pub fn fencing_choices(&self) -> Vec<PastureConfig> {
        self.farm.fencing_options(self.resources[Wood.index()])
    }

    pub fn fence(&mut self, pasture_config: &PastureConfig) {
        assert!(self.can_fence());
        self.farm
            .fence_spaces(pasture_config, &mut self.resources[Wood.index()]);
    }

    pub fn can_fence(&self) -> bool {
        !self
            .farm
            .fencing_options(self.resources[Wood.index()])
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
                self.build_room_cost[Wood.index()] = 0;
                self.build_room_cost[Clay.index()] = 5;
                self.renovation_cost[Stone.index()] = rooms;
                self.renovation_cost[Clay.index()] = 0;
            }
            House::Clay => {
                self.house = House::Stone;
                self.build_room_cost[Clay.index()] = 0;
                self.build_room_cost[Stone.index()] = 5;
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
    pub fn build_room(&mut self, idx: &usize) {
        pay_for_resource(&self.build_room_cost, &mut self.resources);
        self.farm.build_room(*idx);

        match self.house {
            House::Wood => self.renovation_cost[Clay.index()] += 1,
            House::Clay => self.renovation_cost[Stone.index()] += 1,
            House::Stone => (),
        }
    }

    // Builds a single stable
    pub fn build_stable(&mut self, idx: &usize) {
        pay_for_resource(&self.build_stable_cost, &mut self.resources);
        self.farm.build_stable(*idx);
    }

    pub fn add_new_field(&mut self, idx: &usize) {
        self.farm.add_field(*idx);
    }

    pub fn can_build_room(&self) -> bool {
        !self.room_options().is_empty()
    }

    pub fn room_options(&self) -> Vec<usize> {
        if can_pay_for_resource(&self.build_room_cost, &self.resources) {
            return self.farm.possible_room_positions();
        }
        Vec::new()
    }

    pub fn stable_options(&self) -> Vec<usize> {
        if can_pay_for_resource(&self.build_stable_cost, &self.resources)
            && self.farm.can_build_stable()
        {
            return self.farm.possible_stable_positions();
        }
        Vec::new()
    }

    pub fn field_options(&self) -> Vec<usize> {
        self.farm.possible_field_positions()
    }

    pub fn reset_for_next_round(&mut self) {
        self.adults += self.children;
        self.children = 0;
        self.people_placed = 0;
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
        self.resources[res_ex.from] >= res_ex.num_from
    }

    pub fn use_exchange(&mut self, res_ex: &ResourceExchange) {
        assert!(self.can_use_exchange(res_ex));
        self.resources[res_ex.from] -= res_ex.num_from;
        self.resources[res_ex.to] += res_ex.num_to;
    }

    pub fn has_resources_to_cook(&self) -> bool {
        self.resources[Sheep.index()]
            + self.resources[Boar.index()]
            + self.resources[Cattle.index()]
            + self.resources[Vegetable.index()]
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
        if required_food <= self.resources[Food.index()] {
            return true;
        }

        // If cannot pay directly, but can convert some resources
        required_food -= self.resources[Food.index()];

        let raw_grain_and_veg = self.resources[Grain.index()] + self.resources[Vegetable.index()];
        if required_food <= raw_grain_and_veg {
            return true;
        }

        // Required food must be less than 3, and minimum food gained by cooking is 2
        if self.has_cooking_improvement && self.has_resources_to_cook() {
            return true;
        }

        false
    }

    pub fn display_resources(&self) -> String {
        let res = &self.resources;
        let mut ret = format!(
            "\n\n\n\n{:2} {}   ",
            res[Wood.index()],
            RESOURCE_EMOJIS[Wood.index()]
        );
        ret = format!(
            "{ret}{:2} {}   ",
            res[Clay.index()],
            RESOURCE_EMOJIS[Clay.index()]
        );
        ret = format!(
            "{ret}{:2} {}   ",
            res[Stone.index()],
            RESOURCE_EMOJIS[Stone.index()]
        );
        ret = format!(
            "{ret}{:2} {}",
            res[Reed.index()],
            RESOURCE_EMOJIS[Reed.index()]
        );
        ret = format!(
            "{ret}\n{:2} {}   ",
            res[Grain.index()],
            RESOURCE_EMOJIS[Grain.index()]
        );
        ret = format!(
            "{ret}{:2} {}   ",
            res[Vegetable.index()],
            RESOURCE_EMOJIS[Vegetable.index()]
        );
        ret = format!(
            "{ret}{:2} {}   ",
            res[Food.index()],
            RESOURCE_EMOJIS[Food.index()]
        );
        ret = format!("{ret}{:2} \u{1f37d}", self.begging_tokens);
        ret = format!(
            "{ret}\n{:2} {}   ",
            res[Sheep.index()],
            RESOURCE_EMOJIS[Sheep.index()]
        );
        ret = format!(
            "{ret}{:2} {}   ",
            res[Boar.index()],
            RESOURCE_EMOJIS[Boar.index()]
        );
        ret = format!(
            "{ret}{:2} {}",
            res[Cattle.index()],
            RESOURCE_EMOJIS[Cattle.index()]
        );
        ret = format!("{ret}\n\n{:2} 👤   ", self.adults);
        ret = format!("{ret}{:2} 👶", self.children);

        ret = format!("{ret}\n\n{}", Occupation::display(&self.occupations));
        ret
    }

    pub fn display_farm(&self) -> (String, String) {
        // TODO : Fix these!
        let ret = String::from("\n\n\nTODO");
        let stuff = String::from("\n\n\nTODO");

        (ret, stuff)
    }
}
