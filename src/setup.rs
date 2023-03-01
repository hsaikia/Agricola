use crate::game::{ActionSpace, Game, Space};
use crate::primitives::{new_res, Resource};
use rand::Rng;

pub fn get_init_state(num_players: usize, human_player: bool, debug: bool) -> Game {
    assert!(num_players > 0);
    assert!(num_players < 5);

    let first_player_idx = rand::thread_rng().gen_range(0..num_players);

    if debug {
        println!("First player is {}", first_player_idx);
    }

    let spaces = vec![
        Space::create_new(ActionSpace::Copse, Some(new_res())),
        Space::create_new(ActionSpace::Grove, Some(new_res())),
        Space::create_new(ActionSpace::Forest, Some(new_res())),
        Space::create_new(
            ActionSpace::ResourceMarket,
            Some({
                let mut res = new_res();
                res[Resource::Food] = 1;
                res[Resource::Stone] = 1;
                res[Resource::Reed] = 1;
                res
            }),
        ),
        Space::create_new(ActionSpace::Hollow, Some(new_res())),
        Space::create_new(ActionSpace::ClayPit, Some(new_res())),
        Space::create_new(ActionSpace::ReedBank, Some(new_res())),
        Space::create_new(ActionSpace::TravelingPlayers, Some(new_res())),
        Space::create_new(ActionSpace::Fishing, Some(new_res())),
        Space::create_new(
            ActionSpace::DayLaborer,
            Some({
                let mut res = new_res();
                res[Resource::Food] = 2;
                res
            }),
        ),
        Space::create_new(
            ActionSpace::GrainSeeds,
            Some({
                let mut res = new_res();
                res[Resource::Grain] = 1;
                res
            }),
        ),
        Space::create_new(
            ActionSpace::MeetingPlace,
            Some({
                let mut res = new_res();
                res[Resource::Food] = 1; // In the base game, a player also gets a food when they use the MeetingPlace
                res
            }),
        ),
        Space::create_new(ActionSpace::Farmland, None),
        Space::create_new(ActionSpace::FarmExpansion, None),
        Space::create_new(ActionSpace::Lessons1, None),
        Space::create_new(ActionSpace::Lessons2, None),
        Space::create_new(ActionSpace::GrainUtilization, None),
        Space::create_new(ActionSpace::Fencing, None),
        Space::create_new(ActionSpace::SheepMarket, Some(new_res())),
        Space::create_new(ActionSpace::Improvements, None),
        Space::create_new(ActionSpace::WesternQuarry, Some(new_res())),
        Space::create_new(ActionSpace::WishForChildren, None),
        Space::create_new(ActionSpace::HouseRedevelopment, None),
        Space::create_new(ActionSpace::PigMarket, Some(new_res())),
        Space::create_new(
            ActionSpace::VegetableSeeds,
            Some({
                let mut res = new_res();
                res[Resource::Vegetable] = 1;
                res
            }),
        ),
        Space::create_new(ActionSpace::EasternQuarry, Some(new_res())),
        Space::create_new(ActionSpace::CattleMarket, Some(new_res())),
        Space::create_new(ActionSpace::Cultivation, None),
        Space::create_new(ActionSpace::UrgentWishForChildren, None),
        Space::create_new(ActionSpace::FarmRedevelopment, None),
    ];

    Game::create_new(spaces, first_player_idx, num_players, human_player)
}
