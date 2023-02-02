use crate::game::{ActionSpace, Game, Space, HIDDEN_SPACES};
use crate::major_improvements::{MajorImprovement, MajorImprovementType};
use crate::primitives::{Resource, NUM_RESOURCES};
use rand::prelude::SliceRandom;
use rand::Rng;

pub fn get_init_state(num_players: usize) -> Game {
    assert!(num_players > 0);
    assert!(num_players < 5);

    let first_player_idx = rand::thread_rng().gen_range(0..num_players);

    println!("First player is {}", first_player_idx);

    let mut spaces = vec![
        Space::create_new("Copse", ActionSpace::Copse, Some([0; NUM_RESOURCES])),
        Space::create_new("Grove", ActionSpace::Grove, Some([0; NUM_RESOURCES])),
        Space::create_new("Forest", ActionSpace::Forest, Some([0; NUM_RESOURCES])),
        Space::create_new(
            "Resource Market",
            ActionSpace::ResourceMarket,
            Some({
                let mut res = [0; NUM_RESOURCES];
                res[Resource::Food] = 1;
                res[Resource::Stone] = 1;
                res[Resource::Reed] = 1;
                res
            }),
        ),
        Space::create_new("Hollow", ActionSpace::Hollow, Some([0; NUM_RESOURCES])),
        Space::create_new("Clay Pit", ActionSpace::ClayPit, Some([0; NUM_RESOURCES])),
        Space::create_new("Reed Bank", ActionSpace::ReedBank, Some([0; NUM_RESOURCES])),
        Space::create_new(
            "Traveling Players",
            ActionSpace::TravelingPlayers,
            Some([0; NUM_RESOURCES]),
        ),
        Space::create_new("Fishing", ActionSpace::Fishing, Some([0; NUM_RESOURCES])),
        Space::create_new(
            "Day Laborer",
            ActionSpace::DayLaborer,
            Some({
                let mut res = [0; NUM_RESOURCES];
                res[Resource::Food] = 2;
                res
            }),
        ),
        Space::create_new(
            "Grain Seeds",
            ActionSpace::GrainSeeds,
            Some({
                let mut res = [0; NUM_RESOURCES];
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
                Some([0; NUM_RESOURCES]),
            )),
            ActionSpace::Improvements => spaces.push(Space::create_new(
                "Improvements",
                ActionSpace::Improvements,
                None,
            )),
            ActionSpace::WesternQuarry => spaces.push(Space::create_new(
                "Western Quarry",
                ActionSpace::WesternQuarry,
                Some([0; NUM_RESOURCES]),
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
                Some([0; NUM_RESOURCES]),
            )),
            ActionSpace::VegetableSeeds => spaces.push(Space::create_new(
                "Vegetable Seeds",
                ActionSpace::VegetableSeeds,
                Some({
                    let mut res = [0; NUM_RESOURCES];
                    res[Resource::Vegetable] = 1;
                    res
                }),
            )),
            ActionSpace::EasternQuarry => spaces.push(Space::create_new(
                "Eastern Quarry",
                ActionSpace::EasternQuarry,
                Some([0; NUM_RESOURCES]),
            )),
            ActionSpace::CattleMarket => spaces.push(Space::create_new(
                "Cattle Market",
                ActionSpace::CattleMarket,
                Some([0; NUM_RESOURCES]),
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

    let mut majors: Vec<MajorImprovement> = Vec::new();

    majors.push(MajorImprovement::create_new(
        "Fireplace (2)",
        MajorImprovementType::Fireplace2,
        {
            let mut res = [0; NUM_RESOURCES];
            res[Resource::Clay] = 2;
            res
        },
        1,
    ));

    majors.push(MajorImprovement::create_new(
        "Fireplace (3)",
        MajorImprovementType::Fireplace3,
        {
            let mut res = [0; NUM_RESOURCES];
            res[Resource::Clay] = 3;
            res
        },
        1,
    ));

    majors.push(MajorImprovement::create_new(
        "Cooking Hearth (4)",
        MajorImprovementType::CookingHearth4,
        {
            let mut res = [0; NUM_RESOURCES];
            res[Resource::Clay] = 4;
            res
        },
        1,
    ));

    majors.push(MajorImprovement::create_new(
        "Cooking Hearth (5)",
        MajorImprovementType::CookingHearth5,
        {
            let mut res = [0; NUM_RESOURCES];
            res[Resource::Clay] = 5;
            res
        },
        1,
    ));

    majors.push(MajorImprovement::create_new(
        "Well",
        MajorImprovementType::Well,
        {
            let mut res = [0; NUM_RESOURCES];
            res[Resource::Wood] = 1;
            res[Resource::Stone] = 3;
            res
        },
        4,
    ));

    majors.push(MajorImprovement::create_new(
        "Clay Oven",
        MajorImprovementType::ClayOven,
        {
            let mut res = [0; NUM_RESOURCES];
            res[Resource::Clay] = 3;
            res[Resource::Stone] = 1;
            res
        },
        2,
    ));

    majors.push(MajorImprovement::create_new(
        "Stone Oven",
        MajorImprovementType::StoneOven,
        {
            let mut res = [0; NUM_RESOURCES];
            res[Resource::Clay] = 1;
            res[Resource::Stone] = 3;
            res
        },
        3,
    ));

    majors.push(MajorImprovement::create_new(
        "Joinery",
        MajorImprovementType::Joinery,
        {
            let mut res = [0; NUM_RESOURCES];
            res[Resource::Wood] = 2;
            res[Resource::Stone] = 2;
            res
        },
        2,
    ));

    majors.push(MajorImprovement::create_new(
        "Pottery",
        MajorImprovementType::Pottery,
        {
            let mut res = [0; NUM_RESOURCES];
            res[Resource::Clay] = 2;
            res[Resource::Stone] = 2;
            res
        },
        2,
    ));

    majors.push(MajorImprovement::create_new(
        "Basketmaker's Workshop",
        MajorImprovementType::BasketmakersWorkshop,
        {
            let mut res = [0; NUM_RESOURCES];
            res[Resource::Reed] = 2;
            res[Resource::Stone] = 2;
            res
        },
        2,
    ));

    Game::create_new(spaces, majors, first_player_idx, num_players)
}
