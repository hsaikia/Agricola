use super::farm::Seed;
use super::major_improvements::MajorImprovement;
use super::occupations::Occupation;
use super::player::Player;
use super::primitives::{
    can_pay_for_resource, new_res, take_resource, Resource, ResourceExchange, Resources,
};
use super::state::State;
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
    Fence(Vec<usize>),
    Plow(CalledFromCultivation, usize),
    Convert(ResourceExchange, Option<MajorImprovement>, ConversionStage),
    PreHarvest,
    PayFoodOrBeg,
    StartGame,
    PlayOccupation(Occupation, usize),
    GetResourceFromChildless(Resource),
}

impl Action {
    #[allow(clippy::too_many_lines)]
    pub fn next_choices(state: &State) -> Vec<Self> {
        let player = &state.players[state.current_player_idx];
        let mut ret: Vec<Self> = Vec::new();
        match &state.last_action {
            Self::GetResourceFromChildless(_res) => vec![Self::PlaceWorker],
            Self::UseLessons(cheaper) => Self::occupation_choices(state, *cheaper),
            Self::EndGame => vec![],
            Self::StartGame => vec![Self::StartRound],
            Self::StartRound => vec![Self::PlaceWorker],
            Self::PlaceWorker => Self::place_worker_choices(state),
            Self::UseFarmland => {
                let field_opt = player.field_options();
                if !field_opt.is_empty() {
                    for opt in field_opt {
                        ret.push(Self::Plow(CalledFromCultivation(true), opt));
                    }
                } 
                ret
            } 
            Self::UseFarmExpansion => Self::farm_expansion_choices(player),
            Self::UseFencing => Self::fencing_choices(player),
            Self::UseGrainUtilization => Self::grain_utilization_choices(player, false),
            Self::BuildRoom(_) | Self::BuildStable(_) => {
                ret.extend(Self::farm_expansion_choices(player));
                ret.push(Self::EndTurn);
                ret
            }
            Self::Sow(called_from_grain_util, _seed) => {
                if called_from_grain_util.0 {
                    ret.extend(Self::grain_utilization_choices(
                        player,
                        called_from_grain_util.1,
                    ));
                } else {
                    ret.extend(Self::sow_choices(player, called_from_grain_util));
                }
                ret.push(Self::EndTurn);
                ret
            }
            Self::UseImprovements => vec![Self::BuildMajor], // TODO : Add BuildMinor here
            Self::BuildMajor => Self::build_major_choices(state),
            Self::BuildCard(major, _) => match major {
                MajorImprovement::ClayOven | MajorImprovement::StoneOven => {
                    ret.extend(Self::baking_choices(player, false));
                    ret.push(Self::EndTurn);
                    ret
                }
                _ => vec![Self::EndTurn],
            },
            Self::BakeBread(called_from_grain_util, _num_grain_to_bake) => {
                if called_from_grain_util.0 {
                    ret.extend(Self::grain_utilization_choices(player, true));
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
                    player,
                    &CalledFromGrainUtilization(false, true),
                ));
                let field_opt = player.field_options();
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
                        player,
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
                if !player.harvest_paid {
                    ret.push(Self::PreHarvest);
                } else if state.can_init_new_round() {
                    ret.push(Self::StartRound);
                } else {
                    ret.push(Self::EndGame);
                }
                ret
            }
            Self::PreHarvest => Self::harvest_choices(player),
            Self::Convert(_, _, conversion_stage) => {
                match conversion_stage {
                    ConversionStage::Harvest => {
                        ret.extend(Self::harvest_choices(player));
                    }
                    ConversionStage::BeforePlayOccupation(cheaper) => {
                        ret.extend(Self::occupation_choices(state, *cheaper));
                    }
                }
                ret
            }
            Self::PayFoodOrBeg => vec![Self::Harvest],
            Self::UseDayLaborer => {
                ret.extend(Self::day_laborer_choices(player));
                ret
            }
            _ => vec![Self::EndTurn],
        }
    }

    fn occupation_choices(state: &State, cheaper: bool) -> Vec<Self> {
        let player = &state.players[state.current_player_idx];
        let mut required_food = if cheaper { 1 } else { 2 };
        // First Occ on L1 = 0 else 1. So 0, 1, 1, 1, ..
        // First two Occs on L2 = 1 else 2. So 1, 1, 2, 2, 2, ..
        if player.occupations.is_empty() && cheaper {
            required_food = 0;
        }
        if player.occupations.len() < 2 && !cheaper {
            required_food = 1;
        }

        let mut ret: Vec<Self> = Vec::new();

        if player.resources[Resource::Food] < required_food {
            ret.extend(Self::anytime_conversions(
                player,
                &ConversionStage::BeforePlayOccupation(cheaper),
            ));
        } else {
            for occ in &state.available_occupations {
                ret.push(Self::PlayOccupation(occ.clone(), required_food));
            }
        }
        ret
    }

    fn day_laborer_choices(player: &Player) -> Vec<Self> {
        let mut ret: Vec<Self> = Vec::new();
        let field_opt = player.field_options();
        if !field_opt.is_empty() && player.occupations.contains(&Occupation::AssistantTiller) {
            for opt in field_opt {
                ret.push(Self::Plow(CalledFromCultivation(false), opt));
            }
            
        }
        ret.push(Self::EndTurn);
        ret
    }

    fn end_turn_choices(state: &State) -> Vec<Self> {
        let mut ret: Vec<Self> = Vec::new();
        let total_workers = state.players.iter().map(Player::workers).sum();
        if state.people_placed_this_round < total_workers {
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
        let player = &state.players[state.current_player_idx];
        let mut ret: Vec<Self> = Vec::new();
        if from_house_redev.0
            && MajorImprovement::can_build_major(
                &player.major_cards,
                &state.major_improvements,
                &player.resources,
            )
        {
            // TODO also add minor check
            ret.push(Self::BuildMajor);
        }
        if from_farm_redev.0 && player.can_fence() {
            ret.extend(Self::fencing_choices(player));
        }
        ret.push(Self::EndTurn);
        ret
    }

    fn anytime_conversions(player: &Player, stage: &ConversionStage) -> Vec<Self> {
        let mut ret: Vec<Self> = Vec::new();
        if player.resources[Resource::Grain] > 0 {
            ret.push(Self::Convert(
                ResourceExchange {
                    from: Resource::Grain,
                    to: Resource::Food,
                    num_from: 1,
                    num_to: 1,
                },
                None,
                stage.clone(),
            ));
        }

        if player.resources[Resource::Vegetable] > 0 {
            ret.push(Self::Convert(
                ResourceExchange {
                    from: Resource::Vegetable,
                    to: Resource::Food,
                    num_from: 1,
                    num_to: 1,
                },
                None,
                stage.clone(),
            ));
        }

        for major in &player.major_cards {
            match major {
                MajorImprovement::Fireplace(true | false)
                | MajorImprovement::CookingHearth(true | false) => {
                    for exchange in major.exchanges() {
                        if player.can_use_exchange(&exchange) {
                            ret.push(Self::Convert(exchange, None, stage.clone()));
                        }
                    }
                }
                _ => (),
            }
        }

        ret
    }

    fn harvest_choices(player: &Player) -> Vec<Self> {
        let mut ret: Vec<Self> = Vec::new();

        ret.extend(Self::anytime_conversions(player, &ConversionStage::Harvest));

        for major in &player.major_cards {
            match major {
                MajorImprovement::Joinery
                | MajorImprovement::Pottery
                | MajorImprovement::BasketmakersWorkshop => {
                    if player.majors_used_for_harvest.contains(major) {
                        continue;
                    }

                    for exchange in major.exchanges() {
                        if player.can_use_exchange(&exchange) {
                            ret.push(Self::Convert(
                                exchange,
                                Some(major.clone()),
                                ConversionStage::Harvest,
                            ));
                        }
                    }
                }
                _ => (),
            }
        }

        // Option to beg is only present when there are really no conversions possible
        // Otherwise this leads to a bad average fitness from random sampling early on
        if ret.is_empty() || player.got_enough_food() {
            ret.push(Self::PayFoodOrBeg);
        }

        ret
    }

    fn farm_expansion_choices(player: &Player) -> Vec<Self> {
        let mut ret: Vec<Self> = Vec::new();
        let room_options = player.room_options();
        for idx in room_options {
            ret.push(Self::BuildRoom(idx));
        }
        let stable_options = player.stable_options();
        for idx in stable_options {
            ret.push(Self::BuildStable(idx));
        }
        ret
    }

    fn fencing_choices(player: &Player) -> Vec<Self> {
        let mut ret: Vec<Self> = Vec::new();
        let pasture_sizes = player.fencing_choices();
        for ps in pasture_sizes {
            ret.push(Self::Fence(ps));
        }
        ret
    }

    fn sow_choices(player: &Player, from_grain_util: &CalledFromGrainUtilization) -> Vec<Self> {
        let mut ret: Vec<Self> = Vec::new();
        if player.can_sow() && player.resources[Resource::Grain] > 0 {
            ret.push(Self::Sow(from_grain_util.clone(), Seed::Grain));
        }
        if player.can_sow() && player.resources[Resource::Vegetable] > 0 {
            ret.push(Self::Sow(from_grain_util.clone(), Seed::Vegetable));
        }
        ret
    }

    fn baking_choices(player: &Player, from_grain_util: bool) -> Vec<Self> {
        let mut ret: Vec<Self> = Vec::new();
        if player.resources[Resource::Grain] > 1
            && player.major_cards.contains(&MajorImprovement::StoneOven)
        {
            ret.push(Self::BakeBread(
                CalledFromGrainUtilization(from_grain_util, false),
                NumGrainToBake(2),
            ));
        }
        if player.can_bake_bread() {
            ret.push(Self::BakeBread(
                CalledFromGrainUtilization(from_grain_util, false),
                NumGrainToBake(1),
            ));
        }
        ret
    }

    fn grain_utilization_choices(player: &Player, baked_already: bool) -> Vec<Self> {
        let mut ret: Vec<Self> = Vec::new();
        ret.extend(Self::sow_choices(
            player,
            &CalledFromGrainUtilization(true, baked_already),
        ));
        if !baked_already {
            ret.extend(Self::baking_choices(player, true));
        }
        ret
    }

    fn place_worker_choices(state: &State) -> Vec<Self> {
        let player = &state.players[state.current_player_idx];
        let mut ret: Vec<Self> = Vec::new();

        // At the start of each round, if you have at least 3 rooms but only 2 people, you get 1 food and 1 crop of your choice (grain or vegetable).
        if player.before_round_start && player.occupations.contains(&Occupation::Childless) {
            let people = player.adults + player.children; // children should always be zero (grown into adults) at this point
            if people == 2 && people < player.farm.room_indices().len() {
                ret.push(Self::GetResourceFromChildless(Resource::Grain));
                ret.push(Self::GetResourceFromChildless(Resource::Vegetable));
            }
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
                    if player.field_options().is_empty(){
                        continue;
                    }
                }
                Self::UseFarmExpansion => {
                    if player.room_options().is_empty() && player.stable_options().is_empty() {
                        continue;
                    }
                }
                Self::UseImprovements => {
                    // TODO : Also check minor builds here
                    if !MajorImprovement::can_build_major(
                        &player.major_cards,
                        &state.major_improvements,
                        &player.resources,
                    ) {
                        continue;
                    }
                }
                Self::UseFencing => {
                    if !player.can_fence() {
                        continue;
                    }
                }
                Self::UseGrainUtilization => {
                    if !player.can_sow() && !player.can_bake_bread() {
                        continue;
                    }
                }
                Self::UseHouseRedevelopment | Self::UseFarmRedevelopment => {
                    if !player.can_renovate() {
                        continue;
                    }
                }
                Self::UseWishForChildren => {
                    if !player.can_grow_family_with_room() {
                        continue;
                    }
                }
                Self::UseUrgentWishForChildren => {
                    if !player.can_grow_family_without_room() {
                        continue;
                    }
                }
                Self::UseCultivation => {
                    if !player.can_sow() && player.field_options().is_empty() {
                        continue;
                    }
                }

                Self::UseLessons(cheaper) => {
                    if state.available_occupations.is_empty()
                        || !player.can_play_occupation(*cheaper)
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

    fn cooking_hearth_choices(player: &Player, cheaper: bool) -> Vec<Self> {
        let mut ret: Vec<Self> = Vec::new();
        if player
            .major_cards
            .contains(&MajorImprovement::Fireplace(cheaper))
            || player
                .major_cards
                .contains(&MajorImprovement::Fireplace(!cheaper))
        {
            ret.push(Self::BuildCard(
                MajorImprovement::CookingHearth(cheaper),
                ReturnFireplace(true),
            ));
        }
        if can_pay_for_resource(
            &MajorImprovement::CookingHearth(cheaper).cost(),
            &player.resources,
        ) {
            ret.push(Self::BuildCard(
                MajorImprovement::CookingHearth(cheaper),
                ReturnFireplace(false),
            ));
        }
        ret
    }

    fn build_major_choices(state: &State) -> Vec<Self> {
        let player = &state.players[state.current_player_idx];
        let mut ret: Vec<Self> = Vec::new();

        let majors_available = MajorImprovement::available_majors_to_build(
            &player.major_cards,
            &state.major_improvements,
            &player.resources,
        );
        for major in &majors_available {
            match major {
                MajorImprovement::CookingHearth(cheaper) => {
                    ret.extend(Self::cooking_hearth_choices(player, *cheaper));
                }
                _ => ret.push(Self::BuildCard(major.clone(), ReturnFireplace(false))),
            }
        }
        ret
    }

    pub fn update_resources_on_accumulation_spaces(&self, res: &mut Resources) {
        match self {
            Self::UseCopse => res[Resource::Wood] += 1,
            Self::UseGrove => res[Resource::Wood] += 2,
            Self::UseForest => res[Resource::Wood] += 3,
            Self::UseHollow => res[Resource::Clay] += 2,
            Self::UseClayPit => res[Resource::Clay] += 1,
            Self::UseReedBank => res[Resource::Reed] += 1,
            Self::UseTravelingPlayers | Self::UseFishing => res[Resource::Food] += 1,
            Self::UseWesternQuarry | Self::UseEasternQuarry => res[Resource::Stone] += 1,
            Self::UseSheepMarket => res[Resource::Sheep] += 1,
            Self::UsePigMarket => res[Resource::Pigs] += 1,
            Self::UseCattleMarket => res[Resource::Cattle] += 1,
            _ => (),
        }
    }

    pub fn collect_resources(&self, player: &mut Player, res: &mut Resources) {
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
                take_resource(res, &mut player.resources);
                *res = new_res();
            }
            Self::UseSheepMarket | Self::UsePigMarket | Self::UseCattleMarket => {
                take_resource(res, &mut player.resources);
                *res = new_res();
                player.reorg_animals(false);
            }
            Self::UseResourceMarket
            | Self::UseDayLaborer
            | Self::UseGrainSeeds
            | Self::UseVegetableSeeds
            | Self::UseMeetingPlace => {
                take_resource(res, &mut player.resources);
            }
            _ => (),
        }
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
            res[Resource::Food] = 1;
            res[Resource::Stone] = 1;
            res[Resource::Reed] = 1;
            res
        };
        resource_map[Self::UseDayLaborer.resource_map_idx().unwrap()] = {
            let mut res = new_res();
            res[Resource::Food] = 2;
            res
        };
        resource_map[Self::UseGrainSeeds.resource_map_idx().unwrap()] = {
            let mut res = new_res();
            res[Resource::Grain] = 1;
            res
        };
        resource_map[Self::UseVegetableSeeds.resource_map_idx().unwrap()] = {
            let mut res = new_res();
            res[Resource::Vegetable] = 1;
            res
        };
        resource_map[Self::UseMeetingPlace.resource_map_idx().unwrap()] = {
            let mut res = new_res();
            res[Resource::Food] = 1;
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
                let player = &mut state.players[state.current_player_idx];
                player.resources[res.clone()] += 1;
                player.resources[Resource::Food] += 1;
                player.before_round_start = false;
            }
            Self::StartRound => {
                state.init_new_round();
            }
            Self::UseMeetingPlace => {
                state.starting_player_idx = state.current_player_idx;
            }
            Self::PlayOccupation(occ, food_cost) => {
                let player = &mut state.players[state.current_player_idx];
                player.occupations.push(occ.clone());
                state.available_occupations.retain(|x| x != occ);
                player.resources[Resource::Food] -= food_cost;
            }
            Self::Plow(_, pasture_idx) => {
                let player = &mut state.players[state.current_player_idx];
                player.add_new_field(pasture_idx);
            }
            Self::Fence(pasture_indices) => {
                let player = &mut state.players[state.current_player_idx];
                player.fence(pasture_indices);
            }
            Self::BuildRoom(pasture_idx) => {
                let player = &mut state.players[state.current_player_idx];
                player.build_room(pasture_idx);
            }
            Self::BuildStable(pasture_idx) => {
                let player = &mut state.players[state.current_player_idx];
                player.build_stable(pasture_idx);
            }
            Self::Sow(_called_from_grain_util, seed) => {
                let player = &mut state.players[state.current_player_idx];
                player.sow_field(seed);
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
            Self::PreHarvest => state.pre_harvest(),
            Self::Convert(res_ex, opt_major, _) => {
                let player = &mut state.players[state.current_player_idx];
                player.use_exchange(res_ex);
                if let Some(major) = opt_major {
                    player.majors_used_for_harvest.push(major.clone());
                }
            }
            Self::PayFoodOrBeg => state.pay_food_or_beg(),
            _ => (),
        }

        let player = &mut state.players[state.current_player_idx];
        // Collect resources if possible
        if let Some(resource_idx) = self.resource_map_idx() {
            let res = &mut state.resource_map[resource_idx];
            self.collect_resources(player, res);
        }
    }
}
