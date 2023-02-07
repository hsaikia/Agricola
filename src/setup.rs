use crate::game::{ActionSpace, Game, Space, HIDDEN_SPACES};
use crate::primitives::{new_res, Resource};
use rand::prelude::SliceRandom;
use rand::Rng;

pub fn get_init_state(num_players: usize, debug: bool) -> Game {
    assert!(num_players > 0);
    assert!(num_players < 5);

    let first_player_idx = rand::thread_rng().gen_range(0..num_players);

    if debug {
        println!("First player is {}", first_player_idx);
    }

    let mut spaces = vec![
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
    ];

    let mut rng = &mut rand::thread_rng();
    let mut stages: [[usize; 4]; 5] = [
        [0, 1, 2, 3],
        [4, 5, 6, 100],
        [7, 8, 100, 100],
        [9, 10, 100, 100],
        [11, 12, 100, 100],
    ];

    for stage in &mut stages {
        stage.shuffle(&mut rng);
    }

    let mut stages: Vec<usize> = stages.iter().flat_map(|s| s.iter()).cloned().collect();
    stages.retain(|&i| i < 100);

    stages.push(13);

    for idx in stages {
        match HIDDEN_SPACES[idx] {
            ActionSpace::GrainUtilization => spaces.push(Space::create_new(
                "Grain Utilization",
                ActionSpace::GrainUtilization,
                None,
            )),
            ActionSpace::Fencing => {
                spaces.push(Space::create_new("Fencing", ActionSpace::Fencing, None))
            }
            ActionSpace::SheepMarket => spaces.push(Space::create_new(
                "Sheep Market",
                ActionSpace::SheepMarket,
                Some(new_res()),
            )),
            ActionSpace::Improvements => spaces.push(Space::create_new(
                "Improvements",
                ActionSpace::Improvements,
                None,
            )),
            ActionSpace::WesternQuarry => spaces.push(Space::create_new(
                "Western Quarry",
                ActionSpace::WesternQuarry,
                Some(new_res()),
            )),
            ActionSpace::WishForChildren => spaces.push(Space::create_new(
                "Wish For Children",
                ActionSpace::WishForChildren,
                None,
            )),
            ActionSpace::HouseRedevelopment => spaces.push(Space::create_new(
                "House Redevelopment",
                ActionSpace::HouseRedevelopment,
                None,
            )),
            ActionSpace::PigMarket => spaces.push(Space::create_new(
                "Pig Market",
                ActionSpace::PigMarket,
                Some(new_res()),
            )),
            ActionSpace::VegetableSeeds => spaces.push(Space::create_new(
                "Vegetable Seeds",
                ActionSpace::VegetableSeeds,
                Some({
                    let mut res = new_res();
                    res[Resource::Vegetable] = 1;
                    res
                }),
            )),
            ActionSpace::EasternQuarry => spaces.push(Space::create_new(
                "Eastern Quarry",
                ActionSpace::EasternQuarry,
                Some(new_res()),
            )),
            ActionSpace::CattleMarket => spaces.push(Space::create_new(
                "Cattle Market",
                ActionSpace::CattleMarket,
                Some(new_res()),
            )),
            ActionSpace::Cultivation => spaces.push(Space::create_new(
                "Cultivation",
                ActionSpace::Cultivation,
                None,
            )),
            ActionSpace::UrgentWishForChildren => spaces.push(Space::create_new(
                "Urgent Wish For Children",
                ActionSpace::UrgentWishForChildren,
                None,
            )),
            ActionSpace::FarmRedevelopment => spaces.push(Space::create_new(
                "Farm Redevelopment",
                ActionSpace::FarmRedevelopment,
                None,
            )),
            _ => (),
        }
    }

    Game::create_new(spaces, first_player_idx, num_players)
}
