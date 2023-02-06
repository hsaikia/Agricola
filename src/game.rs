use crate::major_improvements::ALL_MAJORS;
use crate::player::{Player, PlayerKind};
use crate::primitives::{new_res, print_resources, Resource, Resources};
use rand::Rng;
use std::io;

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

const NUM_HIDDEN_SPACES: usize = 14;
pub const HIDDEN_SPACES: [ActionSpace; NUM_HIDDEN_SPACES] = [
    ActionSpace::GrainUtilization,
    ActionSpace::Fencing,
    ActionSpace::SheepMarket,
    ActionSpace::Improvements,
    ActionSpace::WesternQuarry,
    ActionSpace::WishForChildren,
    ActionSpace::HouseRedevelopment,
    ActionSpace::PigMarket,
    ActionSpace::VegetableSeeds,
    ActionSpace::EasternQuarry,
    ActionSpace::CattleMarket,
    ActionSpace::Cultivation,
    ActionSpace::UrgentWishForChildren,
    ActionSpace::FarmRedevelopment,
];

pub struct Space {
    name: String,
    action_space: ActionSpace,
    occupied: bool,
    accumulation_space: bool,
    resource_space: bool, // all acc spaces are also resource spaces
    resource: Resources,
}

impl Space {
    pub fn create_new(
        p_name: &str,
        p_action_space: ActionSpace,
        p_resource: Option<Resources>,
    ) -> Space {
        let mut p_resource_space = false;
        let mut p_accumulation_space = false;
        let mut def_resource = new_res();

        if let Some(res) = p_resource {
            def_resource = res;
            p_resource_space = true;
            let res_sum: u32 = def_resource.iter().sum();
            if res_sum == 0 {
                p_accumulation_space = true;
            }
        };

        Space {
            name: String::from(p_name),
            action_space: p_action_space,
            occupied: false,
            accumulation_space: p_accumulation_space,
            resource_space: p_resource_space,
            resource: def_resource,
        }
    }
}

pub struct Game {
    spaces: Vec<Space>,
    players: Vec<Player>,
    major_improvements: [bool; 10],
    current_player_idx: usize,
    starting_player_idx: usize,
    next_visible_idx: usize,
    people_placed_this_round: u32,
}

impl Game {
    pub fn create_new(p_spaces: Vec<Space>, first_player_idx: usize, num_players: usize) -> Game {
        let mut state = Game {
            spaces: p_spaces,
            major_improvements: [true; 10],
            players: Vec::<Player>::new(),
            current_player_idx: first_player_idx,
            starting_player_idx: first_player_idx,
            next_visible_idx: 16,
            people_placed_this_round: 0,
        };
        state.init_players(first_player_idx, num_players);
        state
    }

    pub fn play(&mut self) {
        loop {
            let maybe_actions = self.get_all_available_actions();
            self.display();
            match maybe_actions {
                Some(actions) => {
                    println!("\n\nAvailable actions {:?}", actions);

                    match self.player_kind() {
                        PlayerKind::DumbMachine => {
                            // Chose a random action
                            let action_idx = rand::thread_rng().gen_range(0..actions.len());
                            self.apply_action(actions[action_idx]);
                        }
                        PlayerKind::Human => {
                            let stdin = io::stdin();
                            print!("Enter an action index : ");
                            let mut user_input = String::new();
                            let _res = stdin.read_line(&mut user_input);

                            match user_input.trim().parse::<usize>() {
                                Ok(input_int) => {
                                    println!("Your input [{}]", input_int);

                                    if !actions.contains(&input_int) {
                                        println!("Invalid action.. quitting");
                                        break;
                                    }

                                    self.apply_action(input_int);
                                }
                                Err(_e) => continue,
                            }
                        }
                        PlayerKind::Machine => todo!(),
                    }
                }
                None => break,
            }
        }
    }

    pub fn get_all_available_actions(&mut self) -> Option<Vec<usize>> {
        if self.people_placed_this_round == 0 {
            if self.next_visible_idx == 20
                || self.next_visible_idx == 23
                || self.next_visible_idx == 25
                || self.next_visible_idx == 27
                || self.next_visible_idx == 29
                || self.next_visible_idx == 30
            {
                self.harvest();
            }

            if self.next_visible_idx == 30 {
                println!("\nGAME OVER!");
                return None;
            }

            // Init a new round
            self.init_new_round();
        }

        let mut total_people = 0;
        for p in &self.players {
            total_people += p.workers();
        }

        if self.people_placed_this_round < total_people {
            // If current player has run out of people, then move to the next player
            while self.players[self.current_player_idx].all_people_placed() {
                self.advance_turn();
            }

            return Some(self.available_actions());
        }

        self.people_placed_this_round = 0;
        self.get_all_available_actions()
    }

    fn harvest(&mut self) {
        for (pi, p) in self.players.iter_mut().enumerate() {
            print!("\nPlayer {} paying for harvest..", pi);
            p.harvest();
        }
    }

    fn init_new_round(&mut self) {
        assert!(self.next_visible_idx < 30);

        // Reveal the next action space
        println!(
            "\n\nRound {}. Action {} is revealed!",
            self.next_visible_idx - 15,
            &self.spaces[self.next_visible_idx].name
        );
        self.next_visible_idx += 1;

        // Set start player
        self.current_player_idx = self.starting_player_idx;

        for player in &mut self.players {
            player.reset_for_next_round();
        }

        // Update accumulation spaces
        for i in 0..self.next_visible_idx {
            let mut space = &mut self.spaces[i];
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
                ActionSpace::SheepMarket => {
                    res[Resource::Sheep] += 1;
                }
                ActionSpace::PigMarket => {
                    res[Resource::Pigs] += 1;
                }
                ActionSpace::CattleMarket => {
                    res[Resource::Cattle] += 1;
                }
                ActionSpace::WesternQuarry => {
                    res[Resource::Stone] += 1;
                }
                ActionSpace::EasternQuarry => {
                    res[Resource::Stone] += 1;
                }
                _ => (),
            }
        }
    }

    pub fn apply_action(&mut self, action_idx: usize) {
        // This function assumes that the action is available to the current player

        let space = &self.spaces[action_idx];
        print!(
            "Current Player {} uses action {}.",
            self.current_player_idx, &space.name
        );

        let player = &mut self.players[self.current_player_idx];
        if space.resource_space {
            // Assumes that the space is accessible and the current player can use this action.
            let res = &space.resource;
            player.take_res(res);

            // If animal accumulation space, re-org animals in the farm
            match space.action_space {
                ActionSpace::SheepMarket | ActionSpace::PigMarket | ActionSpace::CattleMarket => {
                    player.reorg_animals(false)
                }
                _ => (),
            }

            // Zero resources if space is an accumulation space
            if space.accumulation_space {
                self.spaces[action_idx].resource = new_res();
            }
        } else {
            match space.action_space {
                // TODO also implement playing minor improvement here
                ActionSpace::MeetingPlace => self.starting_player_idx = self.current_player_idx,
                ActionSpace::Farmland => player.add_new_field(),
                ActionSpace::FarmExpansion => {
                    // TODO this should be a choice
                    if player.can_build_rooms() {
                        player.build_rooms();
                    }
                    if player.can_build_stables() {
                        player.build_stables();
                    }
                }
                // Since we assume that the actions are all accessible
                // no further checks are necessary
                ActionSpace::GrainUtilization => {
                    player.sow();
                    player.bake_bread();
                }
                ActionSpace::Improvements => {
                    player.build_major(&mut self.major_improvements);
                    // TODO add minors
                }
                ActionSpace::Fencing => {
                    player.fence();
                }
                ActionSpace::HouseRedevelopment => {
                    player.renovate();
                    if player.can_build_major(&self.major_improvements) {
                        player.build_major(&mut self.major_improvements);
                    }
                    // TODO minors
                }
                ActionSpace::WishForChildren => {
                    player.grow_family_with_room();
                }
                ActionSpace::UrgentWishForChildren => {
                    player.grow_family_without_room();
                }
                ActionSpace::FarmRedevelopment => {
                    player.renovate();
                    if player.can_fence() {
                        player.fence();
                    }
                }
                ActionSpace::Cultivation => {
                    if player.can_add_new_field() {
                        player.add_new_field();
                    }
                    if player.can_sow() {
                        player.sow();
                    }
                }
                _ => (),
            }
        }
        // Increment people placed by player
        player.increment_people_placed();
        self.people_placed_this_round += 1;
        // Set the space to occupied
        self.spaces[action_idx].occupied = true;
        // Move to the next player
        self.advance_turn();
    }

    fn advance_turn(&mut self) {
        self.current_player_idx = (self.current_player_idx + 1) % self.players.len();
    }

    fn available_actions(&self) -> Vec<usize> {
        let player = &self.players[self.current_player_idx];
        let mut ret: Vec<usize> = Vec::new();
        for i in 0..self.next_visible_idx {
            let space = &self.spaces[i];
            if space.occupied {
                continue;
            }
            match space.action_space {
                ActionSpace::Farmland => {
                    if !player.can_add_new_field() {
                        continue;
                    }
                }
                ActionSpace::Lessons1 => continue, // TODO Not implemented - action not available
                ActionSpace::Lessons2 => continue, // TODO Not implemented - action not available
                ActionSpace::FarmExpansion => {
                    if !player.can_build_rooms() && !player.can_build_stables() {
                        continue;
                    }
                }
                ActionSpace::Improvements => {
                    if !player.can_build_major(&self.major_improvements) {
                        continue;
                    }
                    // TODO - Add minors
                }
                ActionSpace::WishForChildren => {
                    if !player.can_grow_family_with_room() {
                        continue;
                    }
                }
                ActionSpace::Fencing => {
                    if !player.can_fence() {
                        continue;
                    }
                }
                ActionSpace::GrainUtilization => {
                    if !player.can_sow() {
                        continue;
                    }
                }
                ActionSpace::HouseRedevelopment => {
                    if !player.can_renovate() {
                        continue;
                    }
                }
                ActionSpace::Cultivation => {
                    if !player.can_add_new_field() && !player.can_sow() {
                        continue;
                    }
                }
                ActionSpace::FarmRedevelopment => {
                    if !player.can_renovate() {
                        continue;
                    }
                }
                ActionSpace::UrgentWishForChildren => {
                    if !player.can_grow_family_without_room() {
                        continue;
                    }
                }
                _ => (),
            }
            ret.push(i);
        }
        ret
    }

    fn player_kind(&self) -> PlayerKind {
        self.players[self.current_player_idx].kind()
    }

    fn init_players(&mut self, first_idx: usize, num: usize) {
        for i in 0..num {
            let food = if i == first_idx { 2 } else { 3 };
            if i == 0 {
                let player = Player::create_new(food, PlayerKind::Human);
                self.players.push(player);
            } else {
                let player = Player::create_new(food, PlayerKind::DumbMachine);
                self.players.push(player);
            }
        }
    }

    pub fn display(&self) {
        println!("\n\n-- Board --");
        for i in 0..self.next_visible_idx {
            let space = &self.spaces[i];
            if space.occupied {
                print!("\n[X] {}.{} is occupied", i, &space.name);
            } else {
                print!("\n[-] {}.{} ", i, &space.name);
                print_resources(&space.resource);
            }
        }

        print!("\nMajors Available [");
        for (i, e) in self.major_improvements.iter().enumerate() {
            if *e {
                print!("{}, ", &ALL_MAJORS[i].name());
            }
        }
        println!("]");

        println!("\n\n-- Players --");
        for i in 0..self.players.len() {
            let p = &self.players[i];
            print!("\n{}.", i);
            p.display();
            if i == self.current_player_idx {
                print!("[X]");
            }
            if i == self.starting_player_idx {
                print!("[S]");
            }
        }
    }
}
