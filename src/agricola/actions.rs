use super::card::AssistantTiller;
use super::card::*;
use super::farm::Seed;
use super::fencing::PastureConfig;
use super::flag::*;
use super::major_improvements::MajorImprovement;
use super::quantity::*;
use super::state::{
    State, BAKING_MAJOR_IMPROVEMENT_INDICES, FIREPLACE_AND_COOKING_HEARTH_INDICES,
    FIREPLACE_INDICES,
};
pub const NUM_RESOURCE_SPACES: usize = 18;

// Tuple <called from grain utilization, baked bread already>
#[derive(Debug, Clone, Hash)]
pub struct CalledFromGrainUtilization(bool, bool);

#[derive(Debug, Clone, Hash)]
pub struct CalledFromHouseRedevelopment(bool);

#[derive(Debug, Clone, Hash)]
pub struct CalledFromFarmRedevelopment(bool);

#[derive(Debug, Clone, Hash)]
pub struct NumGrainToBake(usize);

#[derive(Debug, Clone, Hash)]
pub struct ReturnFireplace(bool);

#[derive(Debug, Clone, Hash)]
pub struct WithRoom(bool);

#[derive(Debug, Clone, Hash)]
pub struct CalledFromCultivation(bool);

#[derive(Debug, Clone, Hash)]
pub struct UsedOven(bool);

#[derive(Debug, Clone, Hash)]
pub enum ConversionStage {
    Harvest,
    BeforePlayOccupation(bool),
}

#[derive(Debug, Clone, Hash)]
pub enum Action {
    UseCopse,
    UseGrove,
    UseForest,
    UseResourceMarket,
    UseHollow,
    UseClayPit,
    UseReedBank,
    UseTravelingPlayers,
    UseFishing,
    UseDayLaborer,
    UseGrainSeeds,
    UseMeetingPlace,
    UseSheepMarket,
    UseWesternQuarry,
    UsePigMarket,
    UseVegetableSeeds,
    UseEasternQuarry,
    UseCattleMarket,
    UseFarmland,
    UseFarmExpansion,
    UseLessons(bool),
    UseGrainUtilization,
    UseFencing,
    UseImprovements,
    UseWishForChildren,
    UseHouseRedevelopment,
    UseCultivation,
    UseUrgentWishForChildren,
    UseFarmRedevelopment, // TODO
    StartRound,
    PlaceWorker,
    BuildRoom(usize),
    BuildStable(usize),
    BuildCard(MajorImprovement, ReturnFireplace),
    Harvest,
    EndTurn,
    EndGame,
    BuildMajor,
    BakeBread(CalledFromGrainUtilization, NumGrainToBake),
    Sow(CalledFromGrainUtilization, Seed),
    Renovate(CalledFromHouseRedevelopment, CalledFromFarmRedevelopment),
    GrowFamily(WithRoom),
    Fence(PastureConfig),
    Plow(CalledFromCultivation, usize),
    Convert(ResourceExchange, Option<MajorImprovement>, ConversionStage),
    PreHarvest,
    PayFoodOrBeg,
    StartGame,
    PlayOccupation(usize, usize),    // Occ index and food required
    GetResourceFromChildless(usize), // index of Grain or Vegetable
}

impl Action {
    #[allow(clippy::too_many_lines)]
    pub fn next_choices(state: &State) -> Vec<Self> {
        let mut ret: Vec<Self> = Vec::new();
        match &state.last_action {
            Self::GetResourceFromChildless(_res) => vec![Self::PlaceWorker],
            Self::UseLessons(cheaper) => Self::occupation_choices(state, *cheaper),
            Self::EndGame => vec![],
            Self::StartGame => vec![Self::StartRound],
            Self::StartRound => vec![Self::PlaceWorker],
            Self::PlaceWorker => Self::place_worker_choices(state),
            Self::UseFarmland => {
                let field_opt = state.field_options();
                if !field_opt.is_empty() {
                    for opt in field_opt {
                        ret.push(Self::Plow(CalledFromCultivation(false), opt));
                    }
                }
                ret
            }
            Self::UseFarmExpansion => Self::farm_expansion_choices(state),
            Self::UseFencing => Self::fencing_choices(state),
            Self::UseGrainUtilization => Self::grain_utilization_choices(state, false),
            Self::BuildRoom(_) | Self::BuildStable(_) => {
                ret.extend(Self::farm_expansion_choices(state));
                ret.push(Self::EndTurn);
                ret
            }
            Self::Sow(called_from_grain_util, _seed) => {
                if called_from_grain_util.0 {
                    ret.extend(Self::grain_utilization_choices(
                        state,
                        called_from_grain_util.1,
                    ));
                } else {
                    ret.extend(Self::sow_choices(state, called_from_grain_util));
                }
                ret.push(Self::EndTurn);
                ret
            }
            Self::UseImprovements => vec![Self::BuildMajor], // TODO : Add BuildMinor here
            Self::BuildMajor => Self::build_major_choices(state),
            Self::BuildCard(MajorImprovement::ClayOven | MajorImprovement::StoneOven, _) => {
                ret.extend(Self::baking_choices(state, false));
                ret.push(Self::EndTurn);
                ret
            }
            Self::BakeBread(called_from_grain_util, _num_grain_to_bake) => {
                if called_from_grain_util.0 {
                    ret.extend(Self::grain_utilization_choices(state, true));
                }
                ret.push(Self::EndTurn);
                ret
            }
            Self::UseHouseRedevelopment => {
                ret.push(Self::Renovate(
                    CalledFromHouseRedevelopment(true),
                    CalledFromFarmRedevelopment(false),
                ));
                ret
            }
            Self::Renovate(from_house_redev, from_farm_redev) => {
                Self::renovate_choices(state, from_house_redev, from_farm_redev)
            }
            // TODO add minor build
            Self::UseWishForChildren => vec![Self::GrowFamily(WithRoom(true))],
            Self::UseUrgentWishForChildren => vec![Self::GrowFamily(WithRoom(false))],
            Self::UseCultivation => {
                // using baked_bread = true, but this is irrelevant
                ret.extend(Self::sow_choices(
                    state,
                    &CalledFromGrainUtilization(false, true),
                ));
                let field_opt = state.field_options();
                if !field_opt.is_empty() {
                    for opt in field_opt {
                        ret.push(Self::Plow(CalledFromCultivation(true), opt));
                    }
                } else {
                    ret.push(Self::EndTurn);
                }
                ret
            }
            Self::Plow(from_cultivation, _) => {
                if from_cultivation.0 {
                    // using baked_bread = true, but this is irrelevant
                    ret.extend(Self::sow_choices(
                        state,
                        &CalledFromGrainUtilization(false, true),
                    ));
                }
                ret.push(Self::EndTurn);
                ret
            }
            Self::UseFarmRedevelopment => {
                ret.push(Self::Renovate(
                    CalledFromHouseRedevelopment(false),
                    CalledFromFarmRedevelopment(true),
                ));
                ret
            }
            Self::EndTurn => Self::end_turn_choices(state),
            Self::Harvest => {
                if !state.harvest_paid() {
                    ret.push(Self::PreHarvest);
                } else if state.can_init_new_round() {
                    ret.push(Self::StartRound);
                } else {
                    ret.push(Self::EndGame);
                }
                ret
            }
            Self::PreHarvest => Self::harvest_choices(state),
            Self::Convert(_, _, conversion_stage) => {
                match conversion_stage {
                    ConversionStage::Harvest => {
                        ret.extend(Self::harvest_choices(state));
                    }
                    ConversionStage::BeforePlayOccupation(cheaper) => {
                        ret.extend(Self::occupation_choices(state, *cheaper));
                    }
                }
                ret
            }
            Self::PayFoodOrBeg => vec![Self::Harvest],
            Self::UseDayLaborer => {
                ret.extend(Self::day_laborer_choices(state));
                ret
            }
            _ => vec![Self::EndTurn],
        }
    }

    fn occupation_choices(state: &State, cheaper: bool) -> Vec<Self> {
        let mut required_food = if cheaper { 1 } else { 2 };
        // First Occ on L1 = 0 else 1. So 0, 1, 1, 1, ..
        // First two Occs on L2 = 1 else 2. So 1, 1, 2, 2, 2, ..
        if state.num_occupations_played() == 0 && cheaper {
            required_food = 0;
        }
        if state.num_occupations_played() < 2 && !cheaper {
            required_food = 1;
        }

        let mut ret: Vec<Self> = Vec::new();

        if state.current_player_quantities()[Food.index()] < required_food {
            ret.extend(Self::anytime_conversions(
                state,
                &ConversionStage::BeforePlayOccupation(cheaper),
            ));
        } else {
            for occ in &state.occupations_available() {
                ret.push(Self::PlayOccupation(*occ, required_food));
            }
        }
        ret
    }

    fn day_laborer_choices(state: &State) -> Vec<Self> {
        let mut ret: Vec<Self> = Vec::new();
        let field_opt = state.field_options();
        if !field_opt.is_empty() && state.current_player_cards()[AssistantTiller.index()] {
            for opt in field_opt {
                ret.push(Self::Plow(CalledFromCultivation(false), opt));
            }
        }
        ret.push(Self::EndTurn);
        ret
    }

    fn end_turn_choices(state: &State) -> Vec<Self> {
        let mut ret: Vec<Self> = Vec::new();
        if state.people_placed_this_round < state.total_workers() {
            ret.push(Self::PlaceWorker);
        } else if state.is_harvest() {
            ret.push(Self::Harvest);
        } else if state.can_init_new_round() {
            ret.push(Self::StartRound);
        } else {
            panic!("EndTurn should not result in EndGame directly");
        }
        ret
    }

    fn renovate_choices(
        state: &State,
        from_house_redev: &CalledFromHouseRedevelopment,
        from_farm_redev: &CalledFromFarmRedevelopment,
    ) -> Vec<Self> {
        let mut ret: Vec<Self> = Vec::new();
        if from_house_redev.0 && state.available_majors_to_build().iter().any(|x| *x) {
            // TODO also add minor check
            ret.push(Self::BuildMajor);
        }
        if from_farm_redev.0 && state.can_fence() {
            ret.extend(Self::fencing_choices(state));
        }
        ret.push(Self::EndTurn);
        ret
    }

    fn anytime_conversions(state: &State, stage: &ConversionStage) -> Vec<Self> {
        let mut ret: Vec<Self> = Vec::new();
        if state.current_player_quantities()[Grain.index()] > 0 {
            ret.push(Self::Convert(
                ResourceExchange {
                    from: Grain.index(),
                    to: Food.index(),
                    num_from: 1,
                    num_to: 1,
                },
                None,
                stage.clone(),
            ));
        }

        if state.current_player_quantities()[Vegetable.index()] > 0 {
            ret.push(Self::Convert(
                ResourceExchange {
                    from: Vegetable.index(),
                    to: Food.index(),
                    num_from: 1,
                    num_to: 1,
                },
                None,
                stage.clone(),
            ));
        }

        for idx in FIREPLACE_AND_COOKING_HEARTH_INDICES {
            if Some(state.current_player_idx) == state.major_improvements[idx].1 {
                for exchange in state.major_improvements[idx].0.exchanges() {
                    if state.can_use_exchange(&exchange) {
                        ret.push(Self::Convert(exchange, None, stage.clone()));
                    }
                }
            }
        }

        ret
    }

    fn harvest_choices(state: &State) -> Vec<Self> {
        let mut ret: Vec<Self> = Vec::new();

        ret.extend(Self::anytime_conversions(state, &ConversionStage::Harvest));

        for (major, opt_idx, _) in &state.major_improvements {
            if Some(state.current_player_idx) == *opt_idx {
                for exchange in major.exchanges() {
                    if state.can_use_exchange(&exchange) {
                        ret.push(Self::Convert(
                            exchange,
                            Some(major.clone()),
                            ConversionStage::Harvest,
                        ));
                    }
                }
            }
        }

        // Option to beg is only present when there are really no conversions possible
        // Otherwise this leads to a bad average fitness from random sampling early on
        if ret.is_empty() || state.got_enough_food() {
            ret.push(Self::PayFoodOrBeg);
        }

        ret
    }

    fn farm_expansion_choices(state: &State) -> Vec<Self> {
        let mut ret: Vec<Self> = Vec::new();
        let room_options = state.room_options();
        for idx in room_options {
            ret.push(Self::BuildRoom(idx));
        }
        let stable_options = state.stable_options();
        for idx in stable_options {
            ret.push(Self::BuildStable(idx));
        }
        ret
    }

    fn fencing_choices(state: &State) -> Vec<Self> {
        let mut ret: Vec<Self> = Vec::new();
        let pasture_configs = state.fencing_choices();
        for ps_conf in pasture_configs {
            ret.push(Self::Fence(ps_conf));
        }
        ret.push(Self::EndTurn);
        ret
    }

    fn sow_choices(state: &State, from_grain_util: &CalledFromGrainUtilization) -> Vec<Self> {
        let mut ret: Vec<Self> = Vec::new();
        if state.can_sow() && state.current_player_quantities()[Grain.index()] > 0 {
            ret.push(Self::Sow(from_grain_util.clone(), Seed::Grain));
        }
        if state.can_sow() && state.current_player_quantities()[Vegetable.index()] > 0 {
            ret.push(Self::Sow(from_grain_util.clone(), Seed::Vegetable));
        }
        ret
    }

    fn baking_choices(state: &State, from_grain_util: bool) -> Vec<Self> {
        let mut ret: Vec<Self> = Vec::new();
        for idx in BAKING_MAJOR_IMPROVEMENT_INDICES {
            if Some(state.current_player_idx) == state.major_improvements[idx].1 {
                for grain in 1..=state.current_player_quantities()[Grain.index()] {
                    ret.push(Self::BakeBread(
                        CalledFromGrainUtilization(from_grain_util, false),
                        NumGrainToBake(grain),
                    ));
                }
                break;
            }
        }
        ret
    }

    fn grain_utilization_choices(state: &State, baked_already: bool) -> Vec<Self> {
        let mut ret: Vec<Self> = Vec::new();
        ret.extend(Self::sow_choices(
            state,
            &CalledFromGrainUtilization(true, baked_already),
        ));
        if !baked_already {
            ret.extend(Self::baking_choices(state, true));
        }
        ret
    }

    fn place_worker_choices(state: &State) -> Vec<Self> {
        let mut ret: Vec<Self> = Vec::new();

        // At the start of each round, if you have at least 3 rooms but only 2 people, you get 1 food and 1 crop of your choice (grain or vegetable).
        if state.before_round_start()
            && state.current_player_cards()[Childless.index()]
            && state.family_members(state.current_player_idx) == 2
            && state.can_grow_family_with_room()
        {
            ret.push(Self::GetResourceFromChildless(Grain.index()));
            ret.push(Self::GetResourceFromChildless(Vegetable.index()));
        }

        if !ret.is_empty() {
            return ret;
        }

        for action in &state.open_spaces {
            if state.occupied_spaces.contains(&action.action_idx()) {
                continue;
            }

            match action {
                Self::UseFarmland => {
                    if state.field_options().is_empty() {
                        continue;
                    }
                }
                Self::UseFarmExpansion => {
                    if state.room_options().is_empty() && state.stable_options().is_empty() {
                        continue;
                    }
                }
                Self::UseImprovements => {
                    // TODO : Also check minor builds here
                    if !state.available_majors_to_build().iter().any(|x| *x) {
                        continue;
                    }
                }
                Self::UseFencing => {
                    if !state.can_fence() {
                        continue;
                    }
                }
                Self::UseGrainUtilization => {
                    if !state.can_sow() && !state.can_bake_bread(state.current_player_idx) {
                        continue;
                    }
                }
                Self::UseHouseRedevelopment | Self::UseFarmRedevelopment => {
                    if !state.can_renovate() {
                        continue;
                    }
                }
                Self::UseWishForChildren => {
                    if !state.can_grow_family_with_room() {
                        continue;
                    }
                }
                Self::UseUrgentWishForChildren => {
                    if !state.can_grow_family_without_room() {
                        continue;
                    }
                }
                Self::UseCultivation => {
                    if !state.can_sow() && state.field_options().is_empty() {
                        continue;
                    }
                }

                Self::UseLessons(cheaper) => {
                    if state.occupations_available().is_empty()
                        || !state.can_play_occupation(*cheaper)
                    {
                        continue;
                    }
                }
                _ => (),
            }

            ret.push(action.clone());
        }
        ret
    }

    fn cooking_hearth_choices(state: &State, cheaper: bool) -> Vec<Self> {
        let mut ret: Vec<Self> = Vec::new();

        for fp_idx in FIREPLACE_INDICES {
            if Some(state.current_player_idx) == state.major_improvements[fp_idx].1 {
                ret.push(Self::BuildCard(
                    MajorImprovement::CookingHearth { cheaper },
                    ReturnFireplace(true),
                ));
            }
        }

        if can_pay_for_resource(
            &MajorImprovement::CookingHearth { cheaper }.cost(),
            state.current_player_quantities(),
        ) {
            ret.push(Self::BuildCard(
                MajorImprovement::CookingHearth { cheaper },
                ReturnFireplace(false),
            ));
        }
        ret
    }

    fn build_major_choices(state: &State) -> Vec<Self> {
        let mut ret: Vec<Self> = Vec::new();

        for (major_idx, available) in state.available_majors_to_build().iter().enumerate() {
            if !available {
                continue;
            }
            match state.major_improvements[major_idx].0 {
                MajorImprovement::CookingHearth { cheaper } => {
                    ret.extend(Self::cooking_hearth_choices(state, cheaper));
                }
                _ => ret.push(Self::BuildCard(
                    state.major_improvements[major_idx].0.clone(),
                    ReturnFireplace(false),
                )),
            }
        }
        ret
    }

    pub fn update_resources_on_accumulation_spaces(&self, res: &mut Resources) {
        match self {
            Self::UseCopse => res[Wood.index()] += 1,
            Self::UseGrove => res[Wood.index()] += 2,
            Self::UseForest => res[Wood.index()] += 3,
            Self::UseHollow => res[Clay.index()] += 2,
            Self::UseClayPit => res[Clay.index()] += 1,
            Self::UseReedBank => res[Reed.index()] += 1,
            Self::UseTravelingPlayers | Self::UseFishing => res[Food.index()] += 1,
            Self::UseWesternQuarry | Self::UseEasternQuarry => res[Stone.index()] += 1,
            Self::UseSheepMarket => res[Sheep.index()] += 1,
            Self::UsePigMarket => res[Boar.index()] += 1,
            Self::UseCattleMarket => res[Cattle.index()] += 1,
            _ => (),
        }
    }

    pub fn collect_resources(&self, state: &mut State, resource_idx: usize) {
        let mut res = state.resource_map[resource_idx];
        match self {
            Self::UseCopse
            | Self::UseGrove
            | Self::UseForest
            | Self::UseHollow
            | Self::UseClayPit
            | Self::UseReedBank
            | Self::UseTravelingPlayers
            | Self::UseFishing
            | Self::UseWesternQuarry
            | Self::UseEasternQuarry => {
                take_resource(&res, state.current_player_quantities_mut());
                res = new_res();
            }
            Self::UseSheepMarket | Self::UsePigMarket | Self::UseCattleMarket => {
                take_resource(&res, state.current_player_quantities_mut());
                res = new_res();
                state.accommodate_animals(false);
            }
            Self::UseResourceMarket
            | Self::UseDayLaborer
            | Self::UseGrainSeeds
            | Self::UseVegetableSeeds
            | Self::UseMeetingPlace => {
                take_resource(&res, state.current_player_quantities_mut());
            }
            _ => (),
        }
        state.resource_map[resource_idx] = res;
    }

    pub fn resource_map_idx(&self) -> Option<usize> {
        match self {
            Self::UseCopse => Some(0),
            Self::UseGrove => Some(1),
            Self::UseForest => Some(2),
            Self::UseResourceMarket => Some(3),
            Self::UseHollow => Some(4),
            Self::UseClayPit => Some(5),
            Self::UseReedBank => Some(6),
            Self::UseTravelingPlayers => Some(7),
            Self::UseFishing => Some(8),
            Self::UseDayLaborer => Some(9),
            Self::UseGrainSeeds => Some(10),
            Self::UseMeetingPlace => Some(11),
            Self::UseSheepMarket => Some(12),
            Self::UseWesternQuarry => Some(13),
            Self::UsePigMarket => Some(14),
            Self::UseVegetableSeeds => Some(15),
            Self::UseEasternQuarry => Some(16),
            Self::UseCattleMarket => Some(17),
            _ => None,
        }
    }

    pub fn init_resource_map() -> [Resources; NUM_RESOURCE_SPACES] {
        let mut resource_map = [new_res(); NUM_RESOURCE_SPACES];
        resource_map[Self::UseResourceMarket.resource_map_idx().unwrap()] = {
            let mut res = new_res();
            res[Food.index()] = 1;
            res[Stone.index()] = 1;
            res[Reed.index()] = 1;
            res
        };
        resource_map[Self::UseDayLaborer.resource_map_idx().unwrap()] = {
            let mut res = new_res();
            res[Food.index()] = 2;
            res
        };
        resource_map[Self::UseGrainSeeds.resource_map_idx().unwrap()] = {
            let mut res = new_res();
            res[Grain.index()] = 1;
            res
        };
        resource_map[Self::UseVegetableSeeds.resource_map_idx().unwrap()] = {
            let mut res = new_res();
            res[Vegetable.index()] = 1;
            res
        };
        resource_map[Self::UseMeetingPlace.resource_map_idx().unwrap()] = {
            let mut res = new_res();
            res[Food.index()] = 1;
            res
        };
        resource_map
    }

    pub fn display(&self) {
        println!("\nChosen Action : {self:?}");
    }

    pub fn initial_open_spaces() -> Vec<Self> {
        vec![
            Self::UseCopse,
            Self::UseGrove,
            Self::UseForest,
            Self::UseResourceMarket,
            Self::UseHollow,
            Self::UseClayPit,
            Self::UseReedBank,
            Self::UseTravelingPlayers,
            Self::UseFishing,
            Self::UseDayLaborer,
            Self::UseGrainSeeds,
            Self::UseMeetingPlace,
            Self::UseFarmland,
            Self::UseFarmExpansion,
            Self::UseLessons(true),
            Self::UseLessons(false),
        ]
    }

    // Since a Vector acts like a stack, it's easier to pop from the back.
    // Hence, the stages are in reverse order.
    pub fn initial_hidden_spaces() -> Vec<Vec<Self>> {
        vec![
            vec![Self::UseFarmRedevelopment],
            vec![Self::UseCultivation, Self::UseUrgentWishForChildren],
            vec![Self::UseEasternQuarry, Self::UseCattleMarket],
            vec![Self::UsePigMarket, Self::UseVegetableSeeds],
            vec![
                Self::UseWesternQuarry,
                Self::UseWishForChildren,
                Self::UseHouseRedevelopment,
            ],
            vec![
                Self::UseGrainUtilization,
                Self::UseFencing,
                Self::UseSheepMarket,
                Self::UseImprovements,
            ],
        ]
    }

    pub fn action_idx(&self) -> usize {
        match self {
            Self::UseCopse => 0,
            Self::UseGrove => 1,
            Self::UseForest => 2,
            Self::UseResourceMarket => 3,
            Self::UseHollow => 4,
            Self::UseClayPit => 5,
            Self::UseReedBank => 6,
            Self::UseTravelingPlayers => 7,
            Self::UseFishing => 8,
            Self::UseDayLaborer => 9,
            Self::UseGrainSeeds => 10,
            Self::UseMeetingPlace => 11,
            Self::UseFarmland => 12,
            Self::UseFarmExpansion => 13,
            Self::UseLessons(true) => 14,
            Self::UseLessons(false) => 15,
            Self::UseGrainUtilization => 16,
            Self::UseFencing => 17,
            Self::UseSheepMarket => 18,
            Self::UseImprovements => 19,
            Self::UseWesternQuarry => 20,
            Self::UseWishForChildren => 21,
            Self::UseHouseRedevelopment => 22,
            Self::UsePigMarket => 23,
            Self::UseVegetableSeeds => 24,
            Self::UseEasternQuarry => 25,
            Self::UseCattleMarket => 26,
            Self::UseCultivation => 27,
            Self::UseUrgentWishForChildren => 28,
            Self::UseFarmRedevelopment => 29,
            Self::StartRound => 30,
            Self::PlaceWorker => 31,
            Self::BuildRoom(_) => 32,
            Self::BuildStable(_) => 33,
            Self::BuildCard(_, _) => 34,
            Self::Harvest => 35,
            Self::EndTurn => 36,
            Self::EndGame => 37,
            Self::BuildMajor => 38,
            Self::BakeBread(_, _) => 39,
            Self::Sow(_, _) => 40,
            Self::Renovate(_, _) => 41,
            Self::GrowFamily(_) => 42,
            Self::Fence(_) => 43,
            Self::Plow(_, _) => 44,
            Self::Convert(_, _, _) => 45,
            Self::PreHarvest => 46,
            Self::PayFoodOrBeg => 47,
            Self::StartGame => 48,
            Self::PlayOccupation(_, _) => 49,
            Self::GetResourceFromChildless(_) => 50,
        }
    }

    pub fn apply_choice(&self, state: &mut State) {
        state.add_action(self);
        match self {
            Self::GetResourceFromChildless(res) => {
                // At the start of each round, if you have at least 3 rooms but only 2 people, you get 1 food and 1 crop of your choice (grain or vegetable).
                state.current_player_quantities_mut()[*res] += 1;
                state.current_player_quantities_mut()[Food.index()] += 1;
                state.current_player_flags_mut()[BeforeRoundStart.index()] = false;
            }
            Self::StartRound => {
                state.init_new_round();
            }
            Self::UseMeetingPlace => {
                state.starting_player_idx = state.current_player_idx;
            }
            Self::PlayOccupation(occ, food_cost) => {
                //state.player_mut().occupations.push(occ.clone());
                state.current_player_cards_mut()[*occ] = true;
                state.current_player_quantities_mut()[Food.index()] -= food_cost;
            }
            Self::Plow(_, pasture_idx) => {
                state.add_new_field(pasture_idx);
            }
            Self::Fence(pasture_config) => {
                state.fence(pasture_config);
            }
            Self::BuildRoom(pasture_idx) => {
                state.build_room(pasture_idx);
            }
            Self::BuildStable(pasture_idx) => {
                state.build_stable(pasture_idx);
            }
            Self::Sow(_called_from_grain_util, seed) => {
                state.sow_field(seed);
            }
            Self::BuildCard(major, return_fireplace) => {
                state.build_major(major, return_fireplace.0);
            }
            Self::BakeBread(_called_from_grain_util, num_grain_to_bake) => {
                state.bake_bread(num_grain_to_bake.0);
            }
            Self::Renovate(_from_house_redev, _from_farm_redev) => {
                state.renovate();
            }
            Self::GrowFamily(with_room) => state.grow_family(with_room.0),
            Self::EndTurn => state.end_turn(),
            Self::PreHarvest => state.harvest_fields(),
            Self::Convert(res_ex, opt_major, _) => {
                state.use_exchange(res_ex);
                if let Some(major) = opt_major {
                    state.major_improvements[major.index()].2 += 1;
                }
            }
            Self::PayFoodOrBeg => state.pay_food_or_beg(),
            _ => (),
        }

        // Collect resources if possible
        if let Some(resource_idx) = self.resource_map_idx() {
            self.collect_resources(state, resource_idx);
            state.set_can_renovate();
        }
    }
}
