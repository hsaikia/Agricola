//use std::collections::HashMap;
use crate::farm;
use rand::prelude::SliceRandom;
use rand::Rng;
use std::ops::{Index, IndexMut};

enum ActionSpace {
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
    HouseDevelopment,
    PigMarket,
    VegetableSeeds,
    EasternQuarry,
    CattleMarket,
    Cultivation,
    UrgentWishForChildren,
    FarmRedevelopment,
}

const NUM_HIDDEN_SPACES: usize = 14;
const HIDDEN_SPACES: [ActionSpace; NUM_HIDDEN_SPACES] = [
    ActionSpace::GrainUtilization,
    ActionSpace::Fencing,
    ActionSpace::SheepMarket,
    ActionSpace::Improvements,
    ActionSpace::WesternQuarry,
    ActionSpace::WishForChildren,
    ActionSpace::HouseDevelopment,
    ActionSpace::PigMarket,
    ActionSpace::VegetableSeeds,
    ActionSpace::EasternQuarry,
    ActionSpace::CattleMarket,
    ActionSpace::Cultivation,
    ActionSpace::UrgentWishForChildren,
    ActionSpace::FarmRedevelopment,
];

enum Resource {
    Food,
    Wood,
    Clay,
    Stone,
    Reed,
    Grain,
}

const NUM_RESOURCES: usize = 6;
const RESOURCES: [&str; NUM_RESOURCES] = ["Food", "Wood", "Clay", "Stone", "Reed", "Grain"];
type Resources = [u32; NUM_RESOURCES];

impl Index<Resource> for Resources {
    type Output = u32;
    fn index(&self, res: Resource) -> &Self::Output {
        &self[res as usize]
    }
}

impl IndexMut<Resource> for Resources {
    fn index_mut(&mut self, res: Resource) -> &mut u32 {
        &mut self[res as usize]
    }
}

fn print_resources(res: &Resources) {
    for i in 0..NUM_RESOURCES {
        if res[i] > 0 {
            print!("[{} {}]", res[i], RESOURCES[i]);
        }
    }
}

struct Space {
    name: String,
    action_space: ActionSpace,
    visible: bool,
    occupied: bool,
    accumulation_space: bool,
    resource_space: bool, // all acc spaces are also resource spaces
    resource: Resources,
}

struct Player {
    resource: Resources,
    people_placed: u32,
    family: u32,
    farm: farm::Farm,
}

impl Player {
    fn take_res(&mut self, acc_res: &Resources) {
        for (i, elem) in acc_res.iter().enumerate().take(NUM_RESOURCES) {
            self.resource[i] += elem;
        }
    }
}

pub struct State {
    spaces: Vec<Space>,
    players: Vec<Player>,
    current_player_idx: usize,
    starting_player_idx: usize,
}

impl State {
    fn display(&self) {
        println!("\n\n-- Board --");
        for space in &self.spaces {
            if space.occupied {
                print!("\n[X] {} is occupied", &space.name);
            } else {
                print!("\n[-] {} ", &space.name);
                print_resources(&space.resource);
            }
        }

        println!("\n\n-- Players --");
        for i in 0..self.players.len() {
            let p = &self.players[i];

            println!();
            print!("Player {} ({}/{}) has ", i, p.people_placed, p.family);
            print_resources(&p.resource);
            p.farm.display();
            if i == self.current_player_idx {
                print!("[X]");
            }
            if i == self.starting_player_idx {
                print!("[S]");
            }
        }
    }

    fn init_new_round(&mut self) {
        // TODO reveal new card

        // Set start player
        self.current_player_idx = self.starting_player_idx;

        for player in &mut self.players {
            player.people_placed = 0;
        }

        // Update accumulation spaces
        for space in &mut self.spaces {
            space.occupied = false;

            if !space.accumulation_space {
                continue;
            }

            let res = &mut space.resource;
            match space.action_space {
                ActionSpace::Copse => {
                    res[Resource::Wood] += 1;
                }
                ActionSpace::Grove => {
                    res[Resource::Wood] += 2;
                }
                ActionSpace::Forest => {
                    res[Resource::Wood] += 3;
                }
                ActionSpace::Hollow => {
                    res[Resource::Clay] += 2;
                }
                ActionSpace::ClayPit => {
                    res[Resource::Clay] += 1;
                }
                ActionSpace::ReedBank => {
                    res[Resource::Reed] += 1;
                }
                ActionSpace::TravelingPlayers => {
                    res[Resource::Food] += 1;
                }
                ActionSpace::Fishing => {
                    res[Resource::Food] += 1;
                }
                _ => (),
            }
        }
    }

    pub fn play_round(&mut self) {
        let mut total_people = 0;
        for p in &self.players {
            total_people += p.family;
        }

        // Init round
        self.init_new_round();

        for _ in 0..total_people {
            // If current player has run out of people, then move to the next player
            while self.players[self.current_player_idx].people_placed
                == self.players[self.current_player_idx].family
            {
                self.advance_turn();
            }

            // Display board
            self.display();

            // Print available actions
            let remaining_actions = self.available_actions();
            println!();
            for a in &remaining_actions {
                print!("\n{}.{} available.", a, &self.spaces[*a].name);
            }

            // Chose a random action
            let action_idx = rand::thread_rng().gen_range(0..remaining_actions.len());
            self.apply_action(remaining_actions[action_idx]);
        }

        // Display final board
        self.display();
    }

    fn apply_action(&mut self, action_idx: usize) {
        // This function assumes that the action is available to the current player

        let space = &self.spaces[action_idx];
        print!(
            "\nCurrent Player {} uses action {}.",
            self.current_player_idx, &space.name
        );

        let player = &mut self.players[self.current_player_idx];
        if space.resource_space {
            // Assumes that the space is visible and the current player can use this action so not occupied, or player has some special card.
            let res = &space.resource;
            player.take_res(res);

            // Zero resources if space is an accumulation space
            if space.accumulation_space {
                self.spaces[action_idx].resource = [0; NUM_RESOURCES];
            }
        } else {
            match space.action_space {
                // TODO also implement playing minor improvement here
                ActionSpace::MeetingPlace => self.starting_player_idx = self.current_player_idx,
                ActionSpace::Farmland => player.farm.add_new_field(),
                _ => (),
            }
        }
        // Increment people placed by player
        player.people_placed += 1;
        // Set the space to occupied
        self.spaces[action_idx].occupied = true;
        // Move to the next player
        self.advance_turn();
    }

    fn advance_turn(&mut self) {
        self.current_player_idx = (self.current_player_idx + 1) % self.players.len();
    }

    fn available_actions(&self) -> Vec<usize> {
        let mut v: Vec<usize> = (0..self.spaces.len()).collect();
        v.retain(|&i| !self.spaces[i].occupied);
        v
    }

    fn init_players(&mut self, first_idx: usize, num: usize) {
        for i in 0..num {
            let mut res = [0; NUM_RESOURCES];
            res[Resource::Food] = if i == first_idx { 2 } else { 3 };
            let player = Player {
                resource: res,
                people_placed: 0,
                family: 2,
                farm: farm::Farm::new(),
            };
            self.players.push(player);
        }
    }
}

pub fn get_init_state(num_players: usize) -> State {
    assert!(num_players > 0);
    assert!(num_players < 5);

    let first_player_idx = rand::thread_rng().gen_range(0..num_players);

    println!("First player is {}", first_player_idx);

    let spaces = vec![
        Space {
            name: String::from("Copse"),
            action_space: ActionSpace::Copse,
            visible: true,
            occupied: false,
            accumulation_space: true,
            resource_space: true,
            resource: [0; NUM_RESOURCES],
        },
        Space {
            name: String::from("Grove"),
            action_space: ActionSpace::Grove,
            visible: true,
            occupied: false,
            accumulation_space: true,
            resource_space: true,
            resource: [0; NUM_RESOURCES],
        },
        Space {
            name: String::from("Forest"),
            action_space: ActionSpace::Forest,
            visible: true,
            occupied: false,
            accumulation_space: true,
            resource_space: true,
            resource: [0; NUM_RESOURCES],
        },
        Space {
            name: String::from("Resource Market"),
            action_space: ActionSpace::ResourceMarket,
            visible: true,
            occupied: false,
            accumulation_space: false,
            resource_space: true,
            resource: {
                let mut res = [0; NUM_RESOURCES];
                res[Resource::Food] = 1;
                res[Resource::Stone] = 1;
                res[Resource::Reed] = 1;
                res
            },
        },
        Space {
            name: String::from("Hollow"),
            action_space: ActionSpace::Hollow,
            visible: true,
            occupied: false,
            accumulation_space: true,
            resource_space: true,
            resource: [0; NUM_RESOURCES],
        },
        Space {
            name: String::from("Clay Pit"),
            action_space: ActionSpace::ClayPit,
            visible: true,
            occupied: false,
            accumulation_space: true,
            resource_space: true,
            resource: [0; NUM_RESOURCES],
        },
        Space {
            name: String::from("Reed Bank"),
            action_space: ActionSpace::ReedBank,
            visible: true,
            occupied: false,
            accumulation_space: true,
            resource_space: true,
            resource: [0; NUM_RESOURCES],
        },
        Space {
            name: String::from("Traveling Players"),
            action_space: ActionSpace::TravelingPlayers,
            visible: true,
            occupied: false,
            accumulation_space: true,
            resource_space: true,
            resource: [0; NUM_RESOURCES],
        },
        Space {
            name: String::from("Fishing"),
            action_space: ActionSpace::Fishing,
            visible: true,
            occupied: false,
            accumulation_space: true,
            resource_space: true,
            resource: [0; NUM_RESOURCES],
        },
        Space {
            name: String::from("Day Laborer"),
            action_space: ActionSpace::DayLaborer,
            visible: true,
            occupied: false,
            accumulation_space: false,
            resource_space: true,
            resource: {
                let mut res = [0; NUM_RESOURCES];
                res[Resource::Food] = 2;
                res
            },
        },
        Space {
            name: String::from("Grain Seeds"),
            action_space: ActionSpace::GrainSeeds,
            visible: true,
            occupied: false,
            accumulation_space: false,
            resource_space: true,
            resource: {
                let mut res = [0; NUM_RESOURCES];
                res[Resource::Grain] = 1;
                res
            },
        },
        Space {
            name: String::from("Meeting Place"),
            action_space: ActionSpace::MeetingPlace,
            visible: true,
            occupied: false,
            accumulation_space: false,
            resource_space: false,
            resource: [0; NUM_RESOURCES],
        },
        Space {
            name: String::from("Farmland"),
            action_space: ActionSpace::Farmland,
            visible: true,
            occupied: false,
            accumulation_space: false,
            resource_space: false,
            resource: [0; NUM_RESOURCES],
        },
        Space {
            name: String::from("Farm Expansion"),
            action_space: ActionSpace::FarmExpansion,
            visible: true,
            occupied: false,
            accumulation_space: false,
            resource_space: false,
            resource: [0; NUM_RESOURCES],
        },
        Space {
            name: String::from("Lessons 1"),
            action_space: ActionSpace::Lessons1,
            visible: true,
            occupied: false,
            accumulation_space: false,
            resource_space: false,
            resource: [0; NUM_RESOURCES],
        },
        Space {
            name: String::from("Lessons 2"),
            action_space: ActionSpace::Lessons2,
            visible: true,
            occupied: false,
            accumulation_space: false,
            resource_space: false,
            resource: [0; NUM_RESOURCES],
        },
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

    let mut stages: Vec<&usize> = stages.iter().flat_map(|s| s.iter()).collect();
    stages.retain(|&i| i < &100);

    println!("Shuffled: {:?}", stages);

    let mut state = State {
        spaces,
        players: Vec::<Player>::new(),
        current_player_idx: first_player_idx,
        starting_player_idx: first_player_idx,
    };

    state.init_players(first_player_idx, num_players);
    state
}
