use crate::game::{ActionSpace, Game, Space};
use crate::primitives::{new_res, Resource};
use rand::Rng;

pub fn get_init_state(num_players: usize, debug: bool) -> Game {
    assert!(num_players > 0);
    assert!(num_players < 5);

    let first_player_idx = rand::thread_rng().gen_range(0..num_players);

    if debug {
        println!("First player is {}", first_player_idx);
    }

    let spaces = vec![
        Space::create_new("Copse", ActionSpace::Copse, Some(new_res())),
        Space::create_new("Grove", ActionSpace::Grove, Some(new_res())),
        Space::create_new("Forest", ActionSpace::Forest, Some(new_res())),
        Space::create_new(
            "Resource Market",
            ActionSpace::ResourceMarket,
            Some({
                let mut res = new_res();
                res[Resource::Food] = 1;
                res[Resource::Stone] = 1;
                res[Resource::Reed] = 1;
                res
            }),
        ),
        Space::create_new("Hollow", ActionSpace::Hollow, Some(new_res())),
        Space::create_new("Clay Pit", ActionSpace::ClayPit, Some(new_res())),
        Space::create_new("Reed Bank", ActionSpace::ReedBank, Some(new_res())),
        Space::create_new(
            "Traveling Players",
            ActionSpace::TravelingPlayers,
            Some(new_res()),
        ),
        Space::create_new("Fishing", ActionSpace::Fishing, Some(new_res())),
        Space::create_new(
            "Day Laborer",
            ActionSpace::DayLaborer,
            Some({
                let mut res = new_res();
                res[Resource::Food] = 2;
                res
            }),
        ),
        Space::create_new(
            "Grain Seeds",
            ActionSpace::GrainSeeds,
            Some({
                let mut res = new_res();
                res[Resource::Grain] = 1;
                res
            }),
        ),
        Space::create_new("Meeting Place", ActionSpace::MeetingPlace, None),
        Space::create_new("Farmland", ActionSpace::Farmland, None),
        Space::create_new("Farm Expansion", ActionSpace::FarmExpansion, None),
        Space::create_new("Lessons 1", ActionSpace::Lessons1, None),
        Space::create_new("Lessons 2", ActionSpace::Lessons2, None),
        Space::create_new("Grain Utilization", ActionSpace::GrainUtilization, None),
        Space::create_new("Fencing", ActionSpace::Fencing, None),
        Space::create_new("Sheep Market", ActionSpace::SheepMarket, Some(new_res())),
        Space::create_new("Improvements", ActionSpace::Improvements, None),
        Space::create_new(
            "Western Quarry",
            ActionSpace::WesternQuarry,
            Some(new_res()),
        ),
        Space::create_new("Wish For Children", ActionSpace::WishForChildren, None),
        Space::create_new("House Redevelopment", ActionSpace::HouseRedevelopment, None),
        Space::create_new("Pig Market", ActionSpace::PigMarket, Some(new_res())),
        Space::create_new(
            "Vegetable Seeds",
            ActionSpace::VegetableSeeds,
            Some({
                let mut res = new_res();
                res[Resource::Vegetable] = 1;
                res
            }),
        ),
        Space::create_new(
            "Eastern Quarry",
            ActionSpace::EasternQuarry,
            Some(new_res()),
        ),
        Space::create_new("Cattle Market", ActionSpace::CattleMarket, Some(new_res())),
        Space::create_new("Cultivation", ActionSpace::Cultivation, None),
        Space::create_new(
            "Urgent Wish For Children",
            ActionSpace::UrgentWishForChildren,
            None,
        ),
        Space::create_new("Farm Redevelopment", ActionSpace::FarmRedevelopment, None),
    ];

    Game::create_new(spaces, first_player_idx, num_players)
}
