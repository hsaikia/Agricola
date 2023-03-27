use crate::player::Player;

use crate::primitives::{new_res, take_resource, Resource, Resources};
pub const NUM_RESOURCE_SPACES: usize = 18;
pub const NUM_INITIAL_OPEN_SPACES: usize = 16;

#[derive(Eq, PartialEq, Clone, Hash, Debug)]
pub enum ActionSpace {
    Copse,
    Grove,
    Forest,
    ResourceMarket,
    Hollow,
    ClayPit,
    ReedBank,
    TravelingPlayers,
    Fishing,
    DayLaborer,
    GrainSeeds,
    MeetingPlace,
    Farmland,
    FarmExpansion,
    Lessons1,
    Lessons2,
    GrainUtilization,
    Fencing,
    SheepMarket,
    Improvements,
    WesternQuarry,
    WishForChildren,
    HouseRedevelopment,
    PigMarket,
    VegetableSeeds,
    EasternQuarry,
    CattleMarket,
    Cultivation,
    UrgentWishForChildren,
    FarmRedevelopment,
}

impl ActionSpace {
    pub fn initial_open_spaces() -> Vec<ActionSpace> {
        vec![
            Self::Copse,
            Self::Grove,
            Self::Forest,
            Self::ResourceMarket,
            Self::Hollow,
            Self::ClayPit,
            Self::ReedBank,
            Self::TravelingPlayers,
            Self::Fishing,
            Self::DayLaborer,
            Self::GrainSeeds,
            Self::MeetingPlace,
            Self::Farmland,
            Self::FarmExpansion,
            Self::Lessons1,
            Self::Lessons2,
        ]
    }

    // Since a Vector acts like a stack, it's easier to pop from the back.
    // Hence, the stages are in reverse order.
    pub fn initial_hidden_spaces() -> Vec<Vec<ActionSpace>> {
        vec![
            vec![Self::FarmRedevelopment],
            vec![Self::Cultivation, Self::UrgentWishForChildren],
            vec![Self::EasternQuarry, Self::CattleMarket],
            vec![Self::PigMarket, Self::VegetableSeeds],
            vec![
                Self::WesternQuarry,
                Self::WishForChildren,
                Self::HouseRedevelopment,
            ],
            vec![
                Self::GrainUtilization,
                Self::Fencing,
                Self::SheepMarket,
                Self::Improvements,
            ],
        ]
    }

    pub fn action_space_from_idx(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Self::Copse),
            1 => Some(Self::Grove),
            2 => Some(Self::Forest),
            3 => Some(Self::ResourceMarket),
            4 => Some(Self::Hollow),
            5 => Some(Self::ClayPit),
            6 => Some(Self::ReedBank),
            7 => Some(Self::TravelingPlayers),
            8 => Some(Self::Fishing),
            9 => Some(Self::DayLaborer),
            10 => Some(Self::GrainSeeds),
            11 => Some(Self::MeetingPlace),
            12 => Some(Self::Farmland),
            13 => Some(Self::FarmExpansion),
            14 => Some(Self::Lessons1),
            15 => Some(Self::Lessons2),
            16 => Some(Self::GrainUtilization),
            17 => Some(Self::Fencing),
            18 => Some(Self::SheepMarket),
            19 => Some(Self::Improvements),
            20 => Some(Self::WesternQuarry),
            21 => Some(Self::WishForChildren),
            22 => Some(Self::HouseRedevelopment),
            23 => Some(Self::PigMarket),
            24 => Some(Self::VegetableSeeds),
            25 => Some(Self::EasternQuarry),
            26 => Some(Self::CattleMarket),
            27 => Some(Self::Cultivation),
            28 => Some(Self::UrgentWishForChildren),
            29 => Some(Self::FarmRedevelopment),
            _ => None,
        }
    }

    pub fn action_space_idx(&self) -> usize {
        match self {
            Self::Copse => 0,
            Self::Grove => 1,
            Self::Forest => 2,
            Self::ResourceMarket => 3,
            Self::Hollow => 4,
            Self::ClayPit => 5,
            Self::ReedBank => 6,
            Self::TravelingPlayers => 7,
            Self::Fishing => 8,
            Self::DayLaborer => 9,
            Self::GrainSeeds => 10,
            Self::MeetingPlace => 11,
            Self::Farmland => 12,
            Self::FarmExpansion => 13,
            Self::Lessons1 => 14,
            Self::Lessons2 => 15,
            Self::GrainUtilization => 16,
            Self::Fencing => 17,
            Self::SheepMarket => 18,
            Self::Improvements => 19,
            Self::WesternQuarry => 20,
            Self::WishForChildren => 21,
            Self::HouseRedevelopment => 22,
            Self::PigMarket => 23,
            Self::VegetableSeeds => 24,
            Self::EasternQuarry => 25,
            Self::CattleMarket => 26,
            Self::Cultivation => 27,
            Self::UrgentWishForChildren => 28,
            Self::FarmRedevelopment => 29,
        }
    }

    pub fn resource_map_idx(&self) -> Option<usize> {
        match self {
            Self::Copse => Some(0),
            Self::Grove => Some(1),
            Self::Forest => Some(2),
            Self::Hollow => Some(3),
            Self::ClayPit => Some(4),
            Self::ReedBank => Some(5),
            Self::TravelingPlayers => Some(6),
            Self::Fishing => Some(7),
            Self::SheepMarket => Some(8),
            Self::WesternQuarry => Some(9),
            Self::PigMarket => Some(10),
            Self::EasternQuarry => Some(11),
            Self::CattleMarket => Some(12),
            Self::ResourceMarket => Some(13),
            Self::DayLaborer => Some(14),
            Self::GrainSeeds => Some(15),
            Self::VegetableSeeds => Some(16),
            Self::MeetingPlace => Some(17),
            _ => None,
        }
    }

    pub fn init_resource_map() -> [Resources; NUM_RESOURCE_SPACES] {
        let mut resource_map = [new_res(); NUM_RESOURCE_SPACES];
        resource_map[Self::ResourceMarket.resource_map_idx().unwrap()] = {
            let mut res = new_res();
            res[Resource::Food] = 1;
            res[Resource::Stone] = 1;
            res[Resource::Reed] = 1;
            res
        };
        resource_map[Self::DayLaborer.resource_map_idx().unwrap()] = {
            let mut res = new_res();
            res[Resource::Food] = 2;
            res
        };
        resource_map[Self::GrainSeeds.resource_map_idx().unwrap()] = {
            let mut res = new_res();
            res[Resource::Grain] = 1;
            res
        };
        resource_map[Self::VegetableSeeds.resource_map_idx().unwrap()] = {
            let mut res = new_res();
            res[Resource::Vegetable] = 1;
            res
        };
        resource_map[Self::MeetingPlace.resource_map_idx().unwrap()] = {
            let mut res = new_res();
            res[Resource::Food] = 1;
            res
        };
        resource_map
    }

    pub fn update_resources_on_accumulation_spaces(&self, res: &mut Resources) {
        match self {
            Self::Copse => res[Resource::Wood] += 1,
            Self::Grove => res[Resource::Wood] += 2,
            Self::Forest => res[Resource::Wood] += 3,
            Self::Hollow => res[Resource::Clay] += 2,
            Self::ClayPit => res[Resource::Clay] += 1,
            Self::ReedBank => res[Resource::Reed] += 1,
            Self::TravelingPlayers | Self::Fishing => res[Resource::Food] += 1,
            Self::WesternQuarry | Self::EasternQuarry => res[Resource::Stone] += 1,
            Self::SheepMarket => res[Resource::Sheep] += 1,
            Self::PigMarket => res[Resource::Pigs] += 1,
            Self::CattleMarket => res[Resource::Cattle] += 1,
            _ => (),
        }
    }

    pub fn collect_resources(&self, player: &mut Player, res: &mut Resources) {
        match self {
            Self::Copse
            | Self::Grove
            | Self::Forest
            | Self::Hollow
            | Self::ClayPit
            | Self::ReedBank
            | Self::TravelingPlayers
            | Self::Fishing
            | Self::SheepMarket
            | Self::WesternQuarry
            | Self::PigMarket
            | Self::EasternQuarry
            | Self::CattleMarket => {
                take_resource(res, &mut player.resources);
                *res = new_res();
            }
            Self::ResourceMarket
            | Self::DayLaborer
            | Self::GrainSeeds
            | Self::VegetableSeeds
            | Self::MeetingPlace => {
                take_resource(res, &mut player.resources);
            }
            _ => (),
        }
    }
}

// trait Choice {
//     fn apply(&self, game_state : &mut GameState) -> Vec<Box<dyn Choice>>;
// }

// impl Choice for ActionSpace {
//     fn apply(&self, game_state : &mut GameState) -> Vec<Box<dyn Choice>> {
//         match self {
//             // TODO : This assumes that the states
//             Copse | Grove | Forest => game_state.collect_resources(*self as usize),
//             _ => (),
//         }
//     }
// }
