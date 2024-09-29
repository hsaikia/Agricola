use super::action_space::*;
use super::card::*;
use super::display::RESOURCE_EMOJIS;
use super::farm::Seed;
use super::fencing::PastureConfig;
use super::flag::*;
use super::quantity::*;
use super::state::State;
use std::fmt::Debug;
use std::fmt::Formatter;
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

#[derive(Clone, Hash)]
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
    UseFarmRedevelopment,
    StartRound,
    PlaceWorker,
    BuildRoom(usize),
    BuildStable(usize),
    BuildCard(usize, ReturnFireplace),
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
    Convert(ResourceExchange, Option<usize>, ConversionStage), // Exchange, harvest flag index, stage
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
            Self::BuildCard(idx, _) => {
                if *idx == ClayOven.index() || *idx == StoneOven.index() {
                    ret.extend(Self::baking_choices(state, false));
                }
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

        for (idx, owned) in state.current_player_cards().iter().enumerate() {
            if *owned {
                for exchange in anytime_exchanges(idx) {
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

        for (idx, owned) in state.current_player_cards().iter().enumerate() {
            if *owned {
                for exchange in harvest_exchanges(idx) {
                    if state.can_use_exchange(&exchange) {
                        if idx == Joinery.index()
                            && !state.current_player_flags()[UsedJoinery.index()]
                        {
                            ret.push(Self::Convert(
                                exchange,
                                Some(UsedJoinery.index()),
                                ConversionStage::Harvest,
                            ));
                        } else if idx == Pottery.index()
                            && !state.current_player_flags()[UsedPottery.index()]
                        {
                            ret.push(Self::Convert(
                                exchange,
                                Some(UsedPottery.index()),
                                ConversionStage::Harvest,
                            ));
                        } else if idx == BasketmakersWorkshop.index()
                            && !state.current_player_flags()[UsedBasketmakersWorkshop.index()]
                        {
                            ret.push(Self::Convert(
                                exchange,
                                Some(UsedBasketmakersWorkshop.index()),
                                ConversionStage::Harvest,
                            ));
                        }
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
        if BAKING_IMPROVEMENTS_INDICES
            .iter()
            .any(|&x| state.current_player_cards()[x])
        {
            for grain in 1..=state.current_player_quantities()[Grain.index()] {
                ret.push(Self::BakeBread(
                    CalledFromGrainUtilization(from_grain_util, false),
                    NumGrainToBake(grain),
                ));
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

        for i in 0..OPEN_SPACES + state.current_round {
            let idx = state.action_spaces[i];
            if state.occupied[idx] {
                continue;
            }

            if idx == Farmland.index() && state.field_options().is_empty() {
                continue;
            }
            if idx == FarmExpansion.index()
                && state.room_options().is_empty()
                && state.stable_options().is_empty()
            {
                continue;
            }

            if idx == Improvements.index() && !state.available_majors_to_build().iter().any(|x| *x)
            {
                continue;
            }

            if idx == Fencing.index() && !state.can_fence() {
                continue;
            }

            if idx == GrainUtilization.index()
                && !state.can_sow()
                && !state.can_bake_bread(state.current_player_idx)
            {
                continue;
            }

            if idx == HouseRedevelopment.index() && !state.can_renovate() {
                continue;
            }

            if idx == FarmRedevelopment.index() && !state.can_renovate() {
                continue;
            }

            if idx == WishForChildren.index() && !state.can_grow_family_with_room() {
                continue;
            }

            if idx == UrgentWishForChildren.index() && !state.can_grow_family_without_room() {
                continue;
            }

            if idx == Cultivation.index() && !state.can_sow() && state.field_options().is_empty() {
                continue;
            }

            if idx == Lessons1.index() && state.occupations_available().is_empty() {
                continue;
            }

            if idx == Lessons2.index() && state.occupations_available().is_empty() {
                continue;
            }
            ret.push(Self::action_space_idx_to_action(idx));
        }
        ret
    }

    fn build_major_choices(state: &State) -> Vec<Self> {
        let mut ret: Vec<Self> = Vec::new();

        for major_idx in MAJOR_IMPROVEMENTS_INDICES {
            if !state.card_available(major_idx) {
                continue;
            }

            if (major_idx == CookingHearth1.index() || major_idx == CookingHearth2.index())
                && (state.current_player_cards()[Fireplace1.index()]
                    || state.current_player_cards()[Fireplace2.index()])
            {
                ret.push(Self::BuildCard(major_idx, ReturnFireplace(true)));
            }

            if can_pay_for_resource(&cost(major_idx), state.current_player_quantities()) {
                ret.push(Self::BuildCard(major_idx, ReturnFireplace(false)));
            }
        }

        ret
    }

    pub fn display(&self) {
        println!("\nChosen Action : {self:?}");
    }

    fn action_space_idx_to_action(idx: usize) -> Action {
        match idx {
            0 => Self::UseCopse,
            1 => Self::UseGrove,
            2 => Self::UseForest,
            3 => Self::UseResourceMarket,
            4 => Self::UseHollow,
            5 => Self::UseClayPit,
            6 => Self::UseReedBank,
            7 => Self::UseTravelingPlayers,
            8 => Self::UseFishing,
            9 => Self::UseDayLaborer,
            10 => Self::UseGrainSeeds,
            11 => Self::UseMeetingPlace,
            12 => Self::UseFarmland,
            13 => Self::UseFarmExpansion,
            14 => Self::UseLessons(true),
            15 => Self::UseLessons(false),
            16 => Self::UseSheepMarket,
            17 => Self::UseGrainUtilization,
            18 => Self::UseFencing,
            19 => Self::UseImprovements,
            20 => Self::UseWishForChildren,
            21 => Self::UseWesternQuarry,
            22 => Self::UseHouseRedevelopment,
            23 => Self::UsePigMarket,
            24 => Self::UseVegetableSeeds,
            25 => Self::UseEasternQuarry,
            26 => Self::UseCattleMarket,
            27 => Self::UseCultivation,
            28 => Self::UseUrgentWishForChildren,
            29 => Self::UseFarmRedevelopment,
            _ => panic!("Invalid action space index"),
        }
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
            Self::UseSheepMarket => 16,
            Self::UseGrainUtilization => 17,
            Self::UseFencing => 18,
            Self::UseImprovements => 19,
            Self::UseWishForChildren => 20,
            Self::UseWesternQuarry => 21,
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
            Self::BuildCard(major_idx, return_fireplace) => {
                state.build_major(*major_idx, return_fireplace.0);
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
            Self::Convert(res_ex, opt_major_used, _) => {
                state.use_exchange(res_ex);
                if let Some(used_major_idx) = opt_major_used {
                    state.current_player_flags_mut()[*used_major_idx] = true;
                }
            }
            Self::PayFoodOrBeg => state.pay_food_or_beg(),
            _ => (),
        }

        // Collect resources if possible
        if ACCUMULATION_SPACE_INDICES.contains(&self.action_idx()) {
            let res = state.accumulated_resources[self.action_idx()];
            take_resources(state.current_player_quantities_mut(), &res);

            if self.action_idx() == SheepMarket.index()
                || self.action_idx() == PigMarket.index()
                || self.action_idx() == CattleMarket.index()
            {
                state.accommodate_animals(false);
            }

            state.accumulated_resources[self.action_idx()] = new_res();
            state.set_can_renovate();
            state.set_can_build_room();
            state.set_can_build_stable();
        }

        if RESOURCE_SPACE_INDICES.contains(&self.action_idx()) {
            get_resource(self.action_idx(), state.current_player_quantities_mut());
            state.set_can_renovate();
            state.set_can_build_room();
            state.set_can_build_stable();
        }
    }
}

impl Debug for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UseCopse => write!(f, "Copse ({})", RESOURCE_EMOJIS[Wood.index()]),
            Self::UseGrove => write!(f, "Grove ({})", RESOURCE_EMOJIS[Wood.index()].repeat(2)),
            Self::UseForest => write!(f, "Forest ({})", RESOURCE_EMOJIS[Wood.index()].repeat(3)),
            Self::UseResourceMarket => write!(
                f,
                "Resource Market ({} {} {})",
                RESOURCE_EMOJIS[Food.index()],
                RESOURCE_EMOJIS[Stone.index()],
                RESOURCE_EMOJIS[Reed.index()]
            ),
            Self::UseHollow => write!(f, "Hollow ({})", RESOURCE_EMOJIS[Clay.index()].repeat(2)),
            Self::UseClayPit => write!(f, "Clay Pit ({})", RESOURCE_EMOJIS[Clay.index()]),
            Self::UseReedBank => write!(f, "Reed Bank ({})", RESOURCE_EMOJIS[Reed.index()]),
            Self::UseTravelingPlayers => {
                write!(f, "Traveling Players ({})", RESOURCE_EMOJIS[Food.index()])
            }
            Self::UseFishing => write!(f, "Fishing {}", RESOURCE_EMOJIS[Food.index()]),
            Self::UseDayLaborer => write!(
                f,
                "Day Laborer ({})",
                RESOURCE_EMOJIS[Food.index()].repeat(2)
            ),
            Self::UseGrainSeeds => write!(f, "Grain Seeds ({})", RESOURCE_EMOJIS[Grain.index()]),
            Self::UseMeetingPlace => write!(f, "Meeting Place"),
            Self::UseFarmland => write!(f, "Farmland"),
            Self::UseFarmExpansion => write!(f, "Farm Expansion"),
            Self::UseLessons(cheaper) => write!(f, "Lessons ({})", if *cheaper { 1 } else { 2 }),
            Self::UseGrainUtilization => write!(f, "Grain Utilization"),
            Self::UseFencing => write!(f, "Fencing"),
            Self::UseImprovements => write!(f, "Improvements"),
            Self::UseWishForChildren => write!(f, "Wish For Children"),
            Self::UseHouseRedevelopment => write!(f, "House Redevelopment"),
            Self::UseCultivation => write!(f, "Cultivation"),
            Self::UseUrgentWishForChildren => write!(f, "Urgent Wish For Children"),
            Self::UseFarmRedevelopment => write!(f, "Farm Redevelopment"),
            Self::StartRound => write!(f, "Start Round"),
            Self::PlaceWorker => write!(f, "Place Worker"),
            Self::BuildRoom(idx) => write!(f, "Build Room ({})", idx),
            Self::BuildStable(idx) => write!(f, "Build Stable ({})", idx),
            Self::BuildCard(idx, _) => write!(f, "Build Card ({})", CARD_NAMES[*idx]),
            Self::Harvest => write!(f, "Harvest"),
            Self::EndTurn => write!(f, "End Turn"),
            Self::EndGame => write!(f, "End Game"),
            Self::BuildMajor => write!(f, "Build Major"),
            Self::BakeBread(_, num) => write!(f, "Bake Bread from {} Grain", num.0),
            Self::Sow(_, seed) => write!(f, "Sow ({:?})", seed),
            Self::Renovate(_, _) => write!(f, "Renovate"),
            Self::GrowFamily(_) => write!(f, "Grow Family"),
            Self::Fence(pasture_config) => write!(f, "Fence {:?}", pasture_config),
            Self::Plow(_, pasture_idx) => write!(f, "Plow ({})", pasture_idx),
            Self::Convert(res_ex, _, _) => write!(
                f,
                "Convert ({}{} to {}{})",
                res_ex.num_from,
                RESOURCE_EMOJIS[res_ex.from],
                res_ex.num_to,
                RESOURCE_EMOJIS[res_ex.to]
            ),
            Self::PreHarvest => write!(f, "Pre Harvest"),
            Self::PayFoodOrBeg => write!(f, "Pay Food (Or Beg)"),
            Self::StartGame => write!(f, "Start Game"),
            Self::PlayOccupation(occ, _) => write!(f, "Play Occupation ({})", CARD_NAMES[*occ]),
            Self::GetResourceFromChildless(res) => write!(
                f,
                "Childless ({} + {})",
                RESOURCE_EMOJIS[Food.index()],
                RESOURCE_EMOJIS[*res]
            ),
            Self::UseSheepMarket => write!(f, "Sheep Market ({})", RESOURCE_EMOJIS[Sheep.index()]),
            Self::UseWesternQuarry => {
                write!(f, "Western Quarry ({})", RESOURCE_EMOJIS[Stone.index()])
            }
            Self::UsePigMarket => write!(f, "Pig Market ({})", RESOURCE_EMOJIS[Boar.index()]),
            Self::UseVegetableSeeds => write!(
                f,
                "Vegetable Seeds ({})",
                RESOURCE_EMOJIS[Vegetable.index()]
            ),
            Self::UseEasternQuarry => {
                write!(f, "Eastern Quarry ({})", RESOURCE_EMOJIS[Stone.index()])
            }
            Self::UseCattleMarket => {
                write!(f, "Cattle Market ({})", RESOURCE_EMOJIS[Cattle.index()])
            }
        }
    }
}
