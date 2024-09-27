use super::algorithms::PlayerType;
use super::farm::{Farm, House, Seed};
use super::fencing::PastureConfig;
use super::major_improvements::MajorImprovement;
use super::occupations::Occupation;
use super::quantity::*;

#[derive(Clone, Hash)]
pub struct Player {
    // Animals in this resources array are the ones that are pets in the house and the ones that are kept in unfenced stables
    pub player_type: PlayerType,
    pub resources: Resources,
    pub house: House,
    pub occupations: Vec<Occupation>,
    pub farm: Farm,
    pub has_cooking_improvement: bool,
}

impl Player {
    pub fn create_new(food: usize, player_type: PlayerType) -> Self {
        let mut res = new_res();
        res[Food.index()] = food;

        Player {
            player_type,
            resources: res,
            house: House::Wood,
            occupations: vec![],
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

    pub fn player_type(&self) -> PlayerType {
        self.player_type.clone()
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

    pub fn add_new_field(&mut self, idx: &usize) {
        self.farm.add_field(*idx);
    }

    pub fn field_options(&self) -> Vec<usize> {
        self.farm.possible_field_positions()
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
}
