use crate::farm::PlantedSeed;
use crate::game::{Event, State};
use crate::major_improvements::MajorImprovement;
use crate::player::Player;
use crate::primitives::{can_pay_for_resource, ResourceExchange};
use rand::Rng;

use crate::primitives::{new_res, take_resource, Resource, Resources};
pub const NUM_RESOURCE_SPACES: usize = 18;
pub const NUM_ACTION_SPACES: usize = 30;

#[derive(Debug, Clone, Hash)]
pub struct CalledFromGrainUtilization(bool);

#[derive(Debug, Clone, Hash)]
pub struct CalledFromHouseRedevelopment(bool);

#[derive(Debug, Clone, Hash)]
pub struct CalledFromFarmRedevelopment(bool);

#[derive(Debug, Clone, Hash)]
pub struct NumGrainToBake(u32);

#[derive(Debug, Clone, Hash)]
pub struct ReturnFireplace(bool);

#[derive(Debug, Clone, Hash)]
pub struct WithRoom(bool);

#[derive(Debug, Clone, Hash)]
pub struct Harvest(bool);

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
    UseLessons1,
    UseLessons2,
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
    BuildRoom,
    BuildStable,
    BuildFireplace2,
    BuildFireplace3,
    BuildCookingHearth4(ReturnFireplace),
    BuildCookingHearth5(ReturnFireplace),
    BuildWell,
    BuildClayOven,
    BuildStoneOven,
    BuildJoinery,
    BuildPottery,
    BuildBasketmakersWorkshop,
    Harvest,
    EndTurn,
    EndGame,
    BuildMajor,
    BuildMinor,
    BakeBread(CalledFromGrainUtilization, NumGrainToBake),
    Sow(CalledFromGrainUtilization, PlantedSeed),
    Renovate(CalledFromHouseRedevelopment, CalledFromFarmRedevelopment),
    GrowFamily(WithRoom),
    Fence, // TODO make generic
    Plow,
    Convert(ResourceExchange, Option<MajorImprovement>, Harvest),
    PreHarvest,
    PayFoodOrBeg,
}

impl Action {
    pub fn next_choices(&self, state: &State) -> Vec<Self> {
        let player = &state.players[state.current_player_idx];
        let mut ret: Vec<Self> = Vec::new();
        match self {
            Self::StartRound => return vec![Self::PlaceWorker],
            Self::PlaceWorker => return Self::place_worker_choices(state),
            Self::UseFarmland => return vec![Self::Plow],
            Self::UseFarmExpansion => return Self::farm_expansion_choices(player),
            Self::UseFencing => return vec![Self::Fence],
            Self::UseGrainUtilization => return Self::grain_utilization_choices(player),
            Self::BuildRoom => {
                ret.extend(Self::farm_expansion_choices(player));
                ret.push(Self::EndTurn);
            }
            Self::BuildStable => {
                ret.extend(Self::farm_expansion_choices(player));
                ret.push(Self::EndTurn);
            }
            Self::Sow(called_from_grain_util, _seed) => {
                if called_from_grain_util.0 {
                    ret.extend(Self::grain_utilization_choices(player));
                } else {
                    ret.extend(Self::sow_choices(player, false));
                }
                ret.push(Self::EndTurn);
            }
            Self::UseImprovements => {
                // TODO : Add BuildMinor here
                return vec![Self::BuildMajor];
            }
            Self::BuildMajor => {
                let majors_available = MajorImprovement::available_majors_to_build(
                    &player.major_cards,
                    &state.major_improvements,
                    &player.resources,
                );
                for major in &majors_available {
                    match major {
                        MajorImprovement::Fireplace2 => ret.push(Self::BuildFireplace2),
                        MajorImprovement::Fireplace3 => ret.push(Self::BuildFireplace3),
                        MajorImprovement::CookingHearth4 | MajorImprovement::CookingHearth5 => {
                            ret.extend(Self::cooking_hearth_choices(player, major.clone()))
                        }
                        MajorImprovement::Well => ret.push(Self::BuildWell),
                        MajorImprovement::ClayOven => ret.push(Self::BuildClayOven),
                        MajorImprovement::StoneOven => ret.push(Self::BuildStoneOven),
                        MajorImprovement::Joinery => ret.push(Self::BuildJoinery),
                        MajorImprovement::Pottery => ret.push(Self::BuildPottery),
                        MajorImprovement::BasketmakersWorkshop => {
                            ret.push(Self::BuildBasketmakersWorkshop)
                        }
                    }
                }
            }
            Self::BuildClayOven | Self::BuildStoneOven => {
                ret.extend(Self::baking_choices(player, false));
                ret.push(Self::EndTurn);
            }
            Self::BakeBread(called_from_grain_util, _num_grain_to_bake) => {
                if called_from_grain_util.0 {
                    ret.extend(Self::grain_utilization_choices(player));
                }
                ret.push(Self::EndTurn);
            }
            Self::UseHouseRedevelopment => {
                ret.push(Self::Renovate(
                    CalledFromHouseRedevelopment(true),
                    CalledFromFarmRedevelopment(false),
                ));
            }
            Self::Renovate(from_house_redev, from_farm_redev) => {
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
                    ret.push(Self::Fence);
                }
                ret.push(Self::EndTurn);
            }
            Self::UseWishForChildren => {
                ret.push(Self::GrowFamily(WithRoom(true)));
                // TODO add minor build
            }
            Self::UseUrgentWishForChildren => {
                ret.push(Self::GrowFamily(WithRoom(false)));
                // TODO add minor build
            }
            Self::UseCultivation => {
                if player.can_add_new_field() {
                    ret.push(Self::Plow);
                }
                ret.extend(Self::sow_choices(player, false));
            }
            Self::UseFarmRedevelopment => {
                ret.push(Self::Renovate(
                    CalledFromHouseRedevelopment(false),
                    CalledFromFarmRedevelopment(true),
                ));
            }
            Self::EndTurn => {
                let total_workers = state.players.iter().map(|p| p.workers()).sum();
                if state.people_placed_this_round < total_workers {
                    ret.push(Self::PlaceWorker);
                } else {
                    if state.is_harvest() {
                        ret.push(Self::Harvest);
                    } else if state.can_init_new_round() {
                        ret.push(Self::StartRound);
                    } else {
                        panic!("EndTurn should not result in EndGame directly");
                    }
                }
            }
            Self::Harvest => {
                if !player.harvest_paid {
                    ret.push(Self::PreHarvest);
                } else if state.can_init_new_round() {
                    ret.push(Self::StartRound);
                } else {
                    ret.push(Self::EndGame);
                }
            }

            Self::PreHarvest => {
                if !player.harvest_paid {
                    ret.extend(Self::harvest_choices(player));
                }
            }

            Self::Convert(_, _, harvest) => {
                if harvest.0 {
                    if !player.harvest_paid {
                        ret.extend(Self::harvest_choices(player));
                    }
                }
            }

            Self::PayFoodOrBeg => {
                return vec![Self::Harvest];
            }

            _ => return vec![Self::EndTurn],
        }
        ret
    }

    pub fn apply_choice(&self, state: &mut State) {
        // Set space to occupied of action corresponds to an action space
        if self.action_idx() < NUM_ACTION_SPACES {
            state.occupied_spaces.push(self.action_idx());
        }

        // Add action to the sequence of actions taken by the current player
        state.last_action = self.clone();

        match self {
            Self::StartRound => {
                Self::init_new_round(state);

                // Current Round
                let current_round = state.current_round();

                // Delete old events
                state
                    .start_round_events
                    .retain(|e| e.round >= current_round);

                // Current player
                let player = &mut state.players[state.current_player_idx];

                // Look for start round events
                for event in &state.start_round_events {
                    if event.round == current_round && event.player_idx == state.current_player_idx
                    {
                        player.resources = (event.func)(player.resources);
                    }
                }
            }
            Self::UseMeetingPlace => {
                state.starting_player_idx = state.current_player_idx;
            }
            Self::Plow => {
                let player = &mut state.players[state.current_player_idx];
                player.add_new_field();
            }
            Self::Fence => {
                let player = &mut state.players[state.current_player_idx];
                player.fence();
            }
            Self::BuildRoom => {
                let player = &mut state.players[state.current_player_idx];
                player.build_room();
            }
            Self::BuildStable => {
                let player = &mut state.players[state.current_player_idx];
                player.build_stable();
            }
            Self::Sow(_called_from_grain_util, seed) => {
                let player = &mut state.players[state.current_player_idx];
                player.sow_field(seed);
            }
            Self::BuildFireplace2
            | Self::BuildFireplace3
            | Self::BuildClayOven
            | Self::BuildStoneOven
            | Self::BuildJoinery
            | Self::BuildPottery
            | Self::BuildBasketmakersWorkshop => {
                match self {
                    Self::BuildFireplace2 => state.build_major(MajorImprovement::Fireplace2),
                    Self::BuildFireplace3 => state.build_major(MajorImprovement::Fireplace3),
                    Self::BuildClayOven => state.build_major(MajorImprovement::ClayOven),
                    Self::BuildStoneOven => state.build_major(MajorImprovement::StoneOven),
                    Self::BuildJoinery => state.build_major(MajorImprovement::Joinery),
                    Self::BuildPottery => state.build_major(MajorImprovement::Pottery),
                    Self::BuildBasketmakersWorkshop => {
                        state.build_major(MajorImprovement::BasketmakersWorkshop)
                    }
                    _ => (),
                };
            }
            Self::BuildCookingHearth4(return_fireplace) => {
                if return_fireplace.0 {
                    state.replace_fireplace_with_cooking_hearth(MajorImprovement::CookingHearth4);
                } else {
                    state.build_major(MajorImprovement::CookingHearth4);
                }
            }
            Self::BuildCookingHearth5(return_fireplace) => {
                if return_fireplace.0 {
                    state.replace_fireplace_with_cooking_hearth(MajorImprovement::CookingHearth5);
                } else {
                    state.build_major(MajorImprovement::CookingHearth5);
                }
            }
            Self::BakeBread(_called_from_grain_util, num_grain_to_bake) => {
                let player = &mut state.players[state.current_player_idx];
                if num_grain_to_bake.0 == 1 {
                    assert!(player.can_bake_bread());
                    player.resources[Resource::Grain] -= 1;
                    if player.major_cards.contains(&MajorImprovement::ClayOven) {
                        // Clay Oven converts one grain to 5 food.
                        player.resources[Resource::Food] += 5;
                    } else if player.major_cards.contains(&MajorImprovement::StoneOven) {
                        // Stone Oven conversion is upto two grain for 4 food each.
                        player.resources[Resource::Food] += 4;
                    } else if player
                        .major_cards
                        .contains(&MajorImprovement::CookingHearth4)
                        || player
                            .major_cards
                            .contains(&MajorImprovement::CookingHearth5)
                    {
                        // Hearth converts one grain to 3 food.
                        player.resources[Resource::Food] += 3;
                    } else if player.major_cards.contains(&MajorImprovement::Fireplace2)
                        || player.major_cards.contains(&MajorImprovement::Fireplace3)
                    {
                        // Fireplace converts one grain to 2 food.
                        player.resources[Resource::Food] += 2;
                    }
                } else if num_grain_to_bake.0 == 2 {
                    let player = &mut state.players[state.current_player_idx];
                    assert!(
                        player.resources[Resource::Grain] > 1
                            && player.major_cards.contains(&MajorImprovement::StoneOven)
                    );
                    // Stone Oven conversion is upto two grain for 4 food each.
                    player.resources[Resource::Grain] -= 2;
                    player.resources[Resource::Food] += 8;
                }
            }
            Self::BuildWell => {
                let current_round = state.current_round();
                state.build_major(MajorImprovement::Well);
                let func = |mut res: Resources| {
                    res[Resource::Food] += 1;
                    res
                };
                for i in 1..=5 {
                    state.start_round_events.push(Event {
                        round: current_round + i,
                        player_idx: state.current_player_idx,
                        func,
                    });
                }
            }
            Self::Renovate(_from_house_redev, _from_farm_redev) => {
                let player = &mut state.players[state.current_player_idx];
                player.renovate();
            }
            Self::GrowFamily(with_room) => {
                let player = &mut state.players[state.current_player_idx];
                if with_room.0 {
                    player.grow_family_with_room();
                } else {
                    player.grow_family_without_room();
                }
            }
            Self::EndTurn => {
                let player = &mut state.players[state.current_player_idx];
                // Increment people placed by player
                player.increment_people_placed();

                // Increment workers placed
                state.people_placed_this_round += 1;

                // Advance to next player
                state.current_player_idx = (state.current_player_idx + 1) % state.players.len();

                // Skip over players that have all their workers placed
                let total_workers = state.players.iter().map(|p| p.workers()).sum();
                if state.people_placed_this_round < total_workers {
                    while state.players[state.current_player_idx].all_people_placed() {
                        state.current_player_idx =
                            (state.current_player_idx + 1) % state.players.len();
                    }
                }
            }
            Self::PreHarvest => {
                let player = &mut state.players[state.current_player_idx];
                // Harvest grain and veggies
                player.harvest_fields();
                // Move all animals to the resources array
                player.add_animals_in_pastures_to_resources();
            }
            Self::Convert(res_ex, opt_major, harvest) => {
                if harvest.0 {
                    let player = &mut state.players[state.current_player_idx];
                    player.use_exchange(res_ex);
                    if let Some(major) = opt_major {
                        player.major_used_for_harvest.push(major.clone());
                    }
                }
            }
            Self::PayFoodOrBeg => {
                let player = &mut state.players[state.current_player_idx];
                let food_required = 2 * player.adults + player.children;
                if food_required > player.resources[Resource::Food] {
                    player.begging_tokens += food_required - player.resources[Resource::Food];
                } else {
                    player.resources[Resource::Food] -= food_required;
                }

                player.harvest_paid = true;
                player.breed_and_reorg_animals(false);
                state.current_player_idx = (state.current_player_idx + 1) % state.players.len();
                state.remove_empty_stage();
            }
            _ => (),
        }

        let player = &mut state.players[state.current_player_idx];
        // Collect resources if possible
        if let Some(resource_idx) = self.resource_map_idx() {
            let res = &mut state.resource_map[resource_idx];
            self.collect_resources(player, res);
        }
    }

    fn harvest_choices(player: &Player) -> Vec<Self> {
        let mut ret: Vec<Self> = Vec::new();
        for major in &player.major_cards {
            if let Some(exchanges) = major.exchanges(&player.major_used_for_harvest) {
                for exchange in &exchanges {
                    if player.can_use_exchange(exchange) {
                        ret.push(Self::Convert(
                            exchange.clone(),
                            Some(major.clone()),
                            Harvest(true),
                        ));
                    }
                }
            }
        }

        if player.resources[Resource::Grain] > 0 {
            ret.push(Self::Convert(
                ResourceExchange {
                    from: Resource::Grain,
                    to: Resource::Food,
                    num_from: 1,
                    num_to: 1,
                },
                None,
                Harvest(true),
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
                Harvest(true),
            ));
        }

        ret.push(Self::PayFoodOrBeg);
        ret
    }

    fn cooking_hearth_choices(player: &Player, hearth: MajorImprovement) -> Vec<Self> {
        let mut ret: Vec<Self> = Vec::new();
        match hearth {
            MajorImprovement::CookingHearth4 => {
                if player.major_cards.contains(&MajorImprovement::Fireplace2)
                    || player.major_cards.contains(&MajorImprovement::Fireplace3)
                {
                    ret.push(Self::BuildCookingHearth4(ReturnFireplace(true)));
                }
                if can_pay_for_resource(&MajorImprovement::CookingHearth4.cost(), &player.resources)
                {
                    ret.push(Self::BuildCookingHearth4(ReturnFireplace(false)));
                }
            }
            MajorImprovement::CookingHearth5 => {
                if player.major_cards.contains(&MajorImprovement::Fireplace2)
                    || player.major_cards.contains(&MajorImprovement::Fireplace3)
                {
                    ret.push(Self::BuildCookingHearth5(ReturnFireplace(true)));
                }
                if can_pay_for_resource(&MajorImprovement::CookingHearth5.cost(), &player.resources)
                {
                    ret.push(Self::BuildCookingHearth5(ReturnFireplace(false)));
                }
            }
            _ => (),
        }
        ret
    }

    fn farm_expansion_choices(player: &Player) -> Vec<Self> {
        let mut ret: Vec<Self> = Vec::new();
        if player.can_build_room() {
            ret.push(Self::BuildRoom);
        }
        if player.can_build_stable() {
            ret.push(Self::BuildStable);
        }
        ret
    }

    fn sow_choices(player: &Player, from_grain_util: bool) -> Vec<Self> {
        let mut ret: Vec<Self> = Vec::new();
        if player.can_sow() && player.resources[Resource::Grain] > 0 {
            ret.push(Self::Sow(
                CalledFromGrainUtilization(from_grain_util),
                PlantedSeed::Grain,
            ));
        }
        if player.can_sow() && player.resources[Resource::Vegetable] > 0 {
            ret.push(Self::Sow(
                CalledFromGrainUtilization(from_grain_util),
                PlantedSeed::Vegetable,
            ));
        }
        ret
    }

    fn baking_choices(player: &Player, from_grain_util: bool) -> Vec<Self> {
        let mut ret: Vec<Self> = Vec::new();
        if player.resources[Resource::Grain] > 1
            && player.major_cards.contains(&MajorImprovement::StoneOven)
        {
            ret.push(Self::BakeBread(
                CalledFromGrainUtilization(from_grain_util),
                NumGrainToBake(2),
            ));
        }
        if player.can_bake_bread() {
            ret.push(Self::BakeBread(
                CalledFromGrainUtilization(from_grain_util),
                NumGrainToBake(1),
            ));
        }
        ret
    }

    fn grain_utilization_choices(player: &Player) -> Vec<Self> {
        let mut ret: Vec<Self> = Vec::new();
        ret.extend(Self::sow_choices(player, true));
        ret.extend(Self::baking_choices(player, true));
        ret
    }

    fn place_worker_choices(state: &State) -> Vec<Self> {
        let player = &state.players[state.current_player_idx];
        let mut ret: Vec<Self> = Vec::new();
        for action in &state.open_spaces {
            if state.occupied_spaces.contains(&action.action_idx()) {
                continue;
            }

            match action {
                Self::UseFarmland => {
                    if !player.can_add_new_field() {
                        continue;
                    }
                }
                Self::UseFarmExpansion => {
                    if !player.can_build_room() && !player.can_build_stable() {
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
                Self::UseHouseRedevelopment => {
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
                    if !player.can_sow() && !player.can_add_new_field() {
                        continue;
                    }
                }
                Self::UseFarmRedevelopment => {
                    if !player.can_renovate() {
                        continue;
                    }
                }
                // TODO : Implement Occupations
                Self::UseLessons1 | Self::UseLessons2 => continue,
                _ => (),
            }

            ret.push(action.clone());
        }
        ret
    }

    fn init_new_round(state: &mut State) {
        assert!(state.can_init_new_round());

        let maybe_curr_stage = state.hidden_spaces.pop();
        if let Some(mut curr_stage) = maybe_curr_stage {
            assert!(!curr_stage.is_empty());
            let random_idx = rand::thread_rng().gen_range(0..curr_stage.len());
            let last_idx = curr_stage.len() - 1;
            curr_stage.swap(random_idx, last_idx);
            let next_action_space = curr_stage.pop().unwrap();

            // Reveal the new action space
            state.open_spaces.push(next_action_space);
            // Put the rest of the hidden spaces in the current stage back
            state.hidden_spaces.push(curr_stage);
        }

        // Set start player
        state.current_player_idx = state.starting_player_idx;

        // Reset workers
        state
            .players
            .iter_mut()
            .for_each(|player| player.reset_for_next_round());
        state.people_placed_this_round = 0;

        // Update accumulation spaces
        state.occupied_spaces.clear();
        for action in &state.open_spaces {
            if action.resource_map_idx().is_none() {
                continue;
            }
            let res = &mut state.resource_map[action.resource_map_idx().unwrap()];
            action.update_resources_on_accumulation_spaces(res);
        }
    }

    fn update_resources_on_accumulation_spaces(&self, res: &mut Resources) {
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
            | Self::UseSheepMarket
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
        println!("\nChosen Action : {:?}", self);
    }

    pub fn display_all(actions: &Vec<Self>) {
        print!("\n\nActions : ");
        for (i, a) in actions.iter().enumerate() {
            print!("[#{i}. {:?}]", a);
        }
        println!();
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
            Self::UseLessons1,
            Self::UseLessons2,
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
            Self::UseLessons1 => 14,
            Self::UseLessons2 => 15,
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
            Self::BuildRoom => 32,
            Self::BuildStable => 33,
            Self::BuildFireplace2 => 34,
            Self::BuildFireplace3 => 35,
            Self::BuildCookingHearth4(_) => 36,
            Self::BuildCookingHearth5(_) => 37,
            Self::BuildWell => 38,
            Self::BuildClayOven => 39,
            Self::BuildStoneOven => 40,
            Self::BuildJoinery => 41,
            Self::BuildPottery => 42,
            Self::BuildBasketmakersWorkshop => 43,
            Self::Harvest => 44,
            Self::EndTurn => 45,
            Self::EndGame => 46,
            Self::BuildMajor => 47,
            Self::BuildMinor => 48,
            Self::BakeBread(_, _) => 49,
            Self::Sow(_, _) => 50,
            Self::Renovate(_, _) => 51,
            Self::GrowFamily(_) => 52,
            Self::Fence => 53,
            Self::Plow => 54,
            Self::Convert(_, _, _) => 55,
            Self::PreHarvest => 56,
            Self::PayFoodOrBeg => 57,
        }
    }
}
