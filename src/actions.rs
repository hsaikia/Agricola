use crate::farm::PlantedSeed;
use crate::major_improvements::{Cheaper, MajorImprovement};
use crate::player::Player;
use crate::primitives::{can_pay_for_resource, ResourceExchange};
use crate::state::{Event, State};

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
pub struct CalledFromCultivation(bool);

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
    BuildFireplace(Cheaper),
    BuildCookingHearth(Cheaper, ReturnFireplace),
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
    BakeBread(CalledFromGrainUtilization, NumGrainToBake),
    Sow(CalledFromGrainUtilization, PlantedSeed),
    Renovate(CalledFromHouseRedevelopment, CalledFromFarmRedevelopment),
    GrowFamily(WithRoom),
    Fence, // TODO make generic
    Plow(CalledFromCultivation),
    Convert(ResourceExchange, Option<MajorImprovement>, Harvest),
    PreHarvest,
    PayFoodOrBeg,
    StartGame,
}

impl Action {
    pub fn next_choices(state: &State) -> Vec<Self> {
        let player = &state.players[state.current_player_idx];
        let mut ret: Vec<Self> = Vec::new();
        match &state.last_action {
            Self::EndGame => return vec![],
            Self::StartGame => return vec![Self::StartRound],
            Self::StartRound => return vec![Self::PlaceWorker],
            Self::PlaceWorker => return Self::place_worker_choices(state),
            Self::UseFarmland => return vec![Self::Plow(CalledFromCultivation(false))],
            Self::UseFarmExpansion => return Self::farm_expansion_choices(player),
            Self::UseFencing => return vec![Self::Fence],
            Self::UseGrainUtilization => return Self::grain_utilization_choices(player),
            Self::BuildRoom | Self::BuildStable => {
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
            Self::BuildMajor => return Self::build_major_choices(state),
            Self::BuildClayOven | Self::BuildStoneOven => {
                ret.extend(Self::baking_choices(player, false));
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
                return Self::renovate_choices(state, from_house_redev, from_farm_redev);
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
                    ret.push(Self::Plow(CalledFromCultivation(true)));
                }
            }
            Self::Plow(from_cultivation) => {
                if from_cultivation.0 {
                    ret.extend(Self::sow_choices(player, false));
                }
                ret.push(Self::EndTurn);
            }
            Self::UseFarmRedevelopment => {
                ret.push(Self::Renovate(
                    CalledFromHouseRedevelopment(false),
                    CalledFromFarmRedevelopment(true),
                ));
            }
            Self::EndTurn => {
                Self::end_turn_choices(state);
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
                if harvest.0 && !player.harvest_paid {
                    ret.extend(Self::harvest_choices(player));
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
                state.init_new_round();
            }
            Self::UseMeetingPlace => {
                state.starting_player_idx = state.current_player_idx;
            }
            Self::Plow(_) => {
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
            Self::BuildFireplace(cheaper) => {
                state.build_major(MajorImprovement::Fireplace(cheaper.clone()));
            }
            Self::BuildCookingHearth(cheaper, return_fireplace) => {
                if return_fireplace.0 {
                    state.replace_fireplace_with_cooking_hearth(MajorImprovement::CookingHearth(
                        cheaper.clone(),
                    ));
                } else {
                    state.build_major(MajorImprovement::CookingHearth(cheaper.clone()));
                }
            }
            Self::BuildClayOven => state.build_major(MajorImprovement::ClayOven),
            Self::BuildStoneOven => state.build_major(MajorImprovement::StoneOven),
            Self::BuildJoinery => state.build_major(MajorImprovement::Joinery),
            Self::BuildPottery => state.build_major(MajorImprovement::Pottery),
            Self::BuildBasketmakersWorkshop => {
                state.build_major(MajorImprovement::BasketmakersWorkshop);
            }
            Self::BakeBread(_called_from_grain_util, num_grain_to_bake) => {
                state.bake_bread(num_grain_to_bake.0);
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
                state.renovate();
            }
            Self::GrowFamily(with_room) => state.grow_family(with_room.0),
            Self::EndTurn => state.end_turn(),
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
            ret.push(Self::Fence);
        }
        ret.push(Self::EndTurn);
        ret
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

    fn cooking_hearth_choices(player: &Player, cheaper: &Cheaper) -> Vec<Self> {
        let mut ret: Vec<Self> = Vec::new();
        if player
            .major_cards
            .contains(&MajorImprovement::Fireplace(cheaper.clone()))
            || player
                .major_cards
                .contains(&MajorImprovement::Fireplace(cheaper.other()))
        {
            ret.push(Self::BuildCookingHearth(
                cheaper.clone(),
                ReturnFireplace(true),
            ));
        }
        if can_pay_for_resource(
            &MajorImprovement::CookingHearth(cheaper.clone()).cost(),
            &player.resources,
        ) {
            ret.push(Self::BuildCookingHearth(
                cheaper.clone(),
                ReturnFireplace(false),
            ));
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
        ret.push(Self::EndTurn);
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
                    if !player.can_sow() && !player.can_add_new_field() {
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
                MajorImprovement::Fireplace(cheaper) => {
                    ret.push(Self::BuildFireplace(cheaper.clone()));
                }
                MajorImprovement::CookingHearth(cheaper) => {
                    ret.extend(Self::cooking_hearth_choices(player, cheaper));
                }
                MajorImprovement::Well => ret.push(Self::BuildWell),
                MajorImprovement::ClayOven => ret.push(Self::BuildClayOven),
                MajorImprovement::StoneOven => ret.push(Self::BuildStoneOven),
                MajorImprovement::Joinery => ret.push(Self::BuildJoinery),
                MajorImprovement::Pottery => ret.push(Self::BuildPottery),
                MajorImprovement::BasketmakersWorkshop => {
                    ret.push(Self::BuildBasketmakersWorkshop);
                }
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

    pub fn display_all(actions: &[Self]) {
        print!("\n\nActions : ");
        for (i, a) in actions.iter().enumerate() {
            print!("[#{i}. {a:?}]");
        }
        println!();
    }

    pub fn initial_open_spaces() -> &'static [Self] {
        &[
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
            Self::BuildFireplace(_) => 34,
            Self::BuildCookingHearth(_, _) => 35,
            Self::BuildWell => 36,
            Self::BuildClayOven => 37,
            Self::BuildStoneOven => 38,
            Self::BuildJoinery => 39,
            Self::BuildPottery => 40,
            Self::BuildBasketmakersWorkshop => 41,
            Self::Harvest => 42,
            Self::EndTurn => 43,
            Self::EndGame => 44,
            Self::BuildMajor => 45,
            Self::BakeBread(_, _) => 46,
            Self::Sow(_, _) => 47,
            Self::Renovate(_, _) => 48,
            Self::GrowFamily(_) => 49,
            Self::Fence => 50,
            Self::Plow(_) => 51,
            Self::Convert(_, _, _) => 52,
            Self::PreHarvest => 53,
            Self::PayFoodOrBeg => 54,
            Self::StartGame => 55,
        }
    }
}
