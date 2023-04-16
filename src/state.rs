use crate::actions::{Action, NUM_RESOURCE_SPACES};
use crate::algorithms::Kind;
use crate::major_improvements::{Cheaper, MajorImprovement};
use crate::occupations::Occupation;
use crate::player::Player;
use crate::primitives::{pay_for_resource, print_resources, Resource, Resources};
use crate::scoring;
use rand::Rng;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const INITIAL_OPEN_SPACES: usize = 16;
pub const NUM_ACTION_SPACES: usize = 30;

#[derive(Clone, Hash, Debug)]
pub struct Event {
    pub round: usize,
    pub player_idx: usize,
    pub func: fn(res: Resources) -> Resources,
}

#[derive(Clone, Hash)]
pub struct State {
    pub resource_map: [Resources; NUM_RESOURCE_SPACES],
    pub open_spaces: Vec<Action>,
    pub occupied_spaces: Vec<usize>,
    pub hidden_spaces: Vec<Vec<Action>>,
    pub players: Vec<Player>,
    pub major_improvements: Vec<MajorImprovement>,
    pub current_player_idx: usize,
    pub starting_player_idx: usize,
    pub people_placed_this_round: u32,
    pub last_action: Action,
    pub start_round_events: Vec<Event>,
    pub available_occupations: Vec<Occupation>,
}

impl State {
    pub fn create_new(num_players: usize, human_player: bool, default_ai_id: usize) -> State {
        let first_player_idx = rand::thread_rng().gen_range(0..num_players);
        let mut state = State {
            resource_map: Action::init_resource_map(),
            open_spaces: Action::initial_open_spaces(),
            occupied_spaces: Vec::new(),
            hidden_spaces: Action::initial_hidden_spaces(),
            major_improvements: vec![
                MajorImprovement::Fireplace(Cheaper(true)),
                MajorImprovement::Fireplace(Cheaper(false)),
                MajorImprovement::CookingHearth(Cheaper(true)),
                MajorImprovement::CookingHearth(Cheaper(false)),
                MajorImprovement::Well,
                MajorImprovement::ClayOven,
                MajorImprovement::StoneOven,
                MajorImprovement::Joinery,
                MajorImprovement::Pottery,
                MajorImprovement::BasketmakersWorkshop,
            ],
            players: Vec::<Player>::new(),
            current_player_idx: first_player_idx,
            starting_player_idx: first_player_idx,
            people_placed_this_round: 0,
            last_action: Action::StartGame,
            start_round_events: vec![],
            available_occupations: Occupation::all(),
        };
        state.init_players(first_player_idx, num_players, human_player, default_ai_id);
        state
    }

    pub fn current_round(&self) -> usize {
        self.open_spaces.len() - INITIAL_OPEN_SPACES
    }

    pub fn add_action(&mut self, action: &Action) {
        // Set space to occupied of action corresponds to an action space
        if action.action_idx() < NUM_ACTION_SPACES {
            self.occupied_spaces.push(action.action_idx());
        }

        // Add action to the sequence of actions taken by the current player
        self.last_action = action.clone();
    }

    pub fn can_init_new_round(&self) -> bool {
        // If all stages are done
        if self.hidden_spaces.is_empty() {
            return false;
        }

        // If stages left or last stage, but harvest is yet to be completed
        if self.hidden_spaces.last().unwrap().is_empty() {
            return false;
        }

        true
    }

    // When all rounds in the previous stage are played - it is time for harvest
    pub fn is_harvest(&self) -> bool {
        if self.hidden_spaces.last().is_some() && self.hidden_spaces.last().unwrap().is_empty() {
            return true;
        }
        false
    }

    pub fn pre_harvest(&mut self) {
        let player = &mut self.players[self.current_player_idx];
        // Harvest grain and veggies
        player.harvest_fields();
        // Move all animals to the resources array
        player.add_animals_in_pastures_to_resources();
    }

    // After paying for harvest - this function needs to be called to clear the empty hidden space
    pub fn remove_empty_stage(&mut self) {
        if self.hidden_spaces.last().is_some() && self.hidden_spaces.last().unwrap().is_empty() {
            self.hidden_spaces.pop();
        }
    }

    pub fn get_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        s.finish()
    }

    pub fn play(&mut self, debug: bool) {
        loop {
            let algorithm = self.player_kind();
            let status = algorithm.play(self, debug);
            if !status {
                break;
            }
        }
    }

    fn player_kind(&self) -> Kind {
        self.players[self.current_player_idx].kind()
    }

    pub fn replace_all_players_with_random_bots(&mut self) {
        for p in &mut self.players {
            p.kind = Kind::RandomMachine;
        }
    }

    pub fn fitness(&self) -> Vec<i32> {
        let scores = self.scores();

        if scores.len() == 1 {
            return scores;
        }

        let mut fitness = scores.clone();
        let mut sorted_scores = scores;

        // Sort in decreasing order
        sorted_scores.sort_by_key(|k| -k);

        // Fitness of winner is defined by the margin of victory = so difference from own score and second best score
        // Fitness of losers are defined by the margin of defeat = so difference from own score and best score
        for f in &mut fitness {
            if *f == sorted_scores[0] {
                // winner
                *f -= sorted_scores[1];
            } else {
                *f -= sorted_scores[0];
            }
        }
        fitness
    }

    pub fn scores(&self) -> Vec<i32> {
        let mut scores: Vec<i32> = Vec::new();
        for p in &self.players {
            scores.push(scoring::score(p, false));
        }
        scores
    }

    pub fn init_new_round(&mut self) {
        assert!(self.can_init_new_round());

        let maybe_curr_stage = self.hidden_spaces.pop();
        if let Some(mut curr_stage) = maybe_curr_stage {
            assert!(!curr_stage.is_empty());
            let random_idx = rand::thread_rng().gen_range(0..curr_stage.len());
            let last_idx = curr_stage.len() - 1;
            curr_stage.swap(random_idx, last_idx);
            let next_action_space = curr_stage.pop().unwrap();

            // Reveal the new action space
            self.open_spaces.push(next_action_space);
            // Put the rest of the hidden spaces in the current stage back
            self.hidden_spaces.push(curr_stage);
        }

        // Reset workers
        self.players
            .iter_mut()
            .for_each(Player::reset_for_next_round);
        self.people_placed_this_round = 0;

        // Update accumulation spaces
        self.occupied_spaces.clear();
        for action in &self.open_spaces {
            if action.resource_map_idx().is_none() {
                continue;
            }
            let res = &mut self.resource_map[action.resource_map_idx().unwrap()];
            action.update_resources_on_accumulation_spaces(res);
        }

        // Current Round
        let current_round = self.current_round();

        // Delete old events
        self.start_round_events.retain(|e| e.round >= current_round);

        // Reset start player
        self.current_player_idx = self.starting_player_idx;

        // Look for start round events
        for event in &self.start_round_events {
            for (i, player) in self.players.iter_mut().enumerate() {
                if event.round == current_round && event.player_idx == i {
                    player.resources = (event.func)(player.resources);
                }
            }
        }
    }

    fn init_players(
        &mut self,
        first_idx: usize,
        num: usize,
        human_player: bool,
        default_ai_id: usize,
    ) {
        for i in 0..num {
            let food = if i == first_idx { 2 } else { 3 };
            let player_kind = if human_player && i == 0 {
                Kind::Human
            } else {
                match default_ai_id {
                    0 => Kind::RandomMachine,
                    1 => Kind::UniformMachine,
                    2 => Kind::MCTSMachine,
                    _ => return,
                }
            };
            let player = Player::create_new(food, player_kind);
            self.players.push(player);
        }
    }

    pub fn replace_fireplace_with_cooking_hearth(&mut self, major: &MajorImprovement) {
        let player = &mut self.players[self.current_player_idx];
        assert!(
            player
                .major_cards
                .contains(&MajorImprovement::Fireplace(Cheaper(true)))
                || player
                    .major_cards
                    .contains(&MajorImprovement::Fireplace(Cheaper(false)))
        );

        let mut returned_fireplace = MajorImprovement::Fireplace(Cheaper(true));
        if player
            .major_cards
            .contains(&MajorImprovement::Fireplace(Cheaper(false)))
        {
            returned_fireplace = MajorImprovement::Fireplace(Cheaper(false));
        }

        self.major_improvements.retain(|x| x != major);
        player.major_cards.retain(|x| x != &returned_fireplace);
        player.major_cards.push(major.clone());
        self.major_improvements.push(returned_fireplace);
    }

    pub fn build_major(&mut self, major: &MajorImprovement, return_fireplace: bool) {
        if return_fireplace {
            self.replace_fireplace_with_cooking_hearth(major);
        } else {
            let player = &mut self.players[self.current_player_idx];
            pay_for_resource(&major.cost(), &mut player.resources);
            self.major_improvements.retain(|x| x != major);
            player.major_cards.push(major.clone());

            if matches!(major, MajorImprovement::Well) {
                let current_round = self.current_round();
                let func = |mut res: Resources| {
                    res[Resource::Food] += 1;
                    res
                };
                for i in 1..=5 {
                    self.start_round_events.push(Event {
                        round: current_round + i,
                        player_idx: self.current_player_idx,
                        func,
                    });
                }
            }
        }
    }

    pub fn bake_bread(&mut self, num_grain_to_bake: u32) {
        let player = &mut self.players[self.current_player_idx];
        if num_grain_to_bake == 1 {
            assert!(player.can_bake_bread());
            player.resources[Resource::Grain] -= 1;
            if player.major_cards.contains(&MajorImprovement::ClayOven) {
                // Clay Oven converts one grain to 5 food.
                player.resources[Resource::Food] += 5;
            } else if player.major_cards.contains(&MajorImprovement::StoneOven) {
                // Stone Oven converts upto two grain for 4 food each.
                player.resources[Resource::Food] += 4;
            } else if player
                .major_cards
                .contains(&MajorImprovement::CookingHearth(Cheaper(true)))
                || player
                    .major_cards
                    .contains(&MajorImprovement::CookingHearth(Cheaper(false)))
            {
                // Hearth converts one grain to 3 food.
                player.resources[Resource::Food] += 3;
            } else if player
                .major_cards
                .contains(&MajorImprovement::Fireplace(Cheaper(true)))
                || player
                    .major_cards
                    .contains(&MajorImprovement::Fireplace(Cheaper(false)))
            {
                // Fireplace converts one grain to 2 food.
                player.resources[Resource::Food] += 2;
            }
        } else if num_grain_to_bake == 2 {
            assert!(
                player.resources[Resource::Grain] > 1
                    && player.major_cards.contains(&MajorImprovement::StoneOven)
            );
            // Stone Oven converts upto two grain for 4 food each.
            player.resources[Resource::Grain] -= 2;
            player.resources[Resource::Food] += 8;
        }
    }

    pub fn pay_food_or_beg(&mut self) {
        let player = &mut self.players[self.current_player_idx];
        let food_required = 2 * player.adults + player.children;
        if food_required > player.resources[Resource::Food] {
            player.begging_tokens += food_required - player.resources[Resource::Food];
            player.resources[Resource::Food] = 0;
        } else {
            player.resources[Resource::Food] -= food_required;
        }

        player.harvest_paid = true;
        player.breed_and_reorg_animals();
        self.current_player_idx = (self.current_player_idx + 1) % self.players.len();
        self.remove_empty_stage();
    }

    pub fn end_turn(&mut self) {
        let player = &mut self.players[self.current_player_idx];
        // Increment people placed by player
        player.increment_people_placed();

        // Increment workers placed
        self.people_placed_this_round += 1;

        // Advance to next player
        self.current_player_idx = (self.current_player_idx + 1) % self.players.len();

        // Skip over players that have all their workers placed
        let total_workers = self.players.iter().map(Player::workers).sum();
        if self.people_placed_this_round < total_workers {
            while self.players[self.current_player_idx].all_people_placed() {
                self.current_player_idx = (self.current_player_idx + 1) % self.players.len();
            }
        }
    }

    pub fn renovate(&mut self) {
        let player = &mut self.players[self.current_player_idx];
        player.renovate();
    }

    pub fn grow_family(&mut self, with_room: bool) {
        let player = &mut self.players[self.current_player_idx];
        if with_room {
            player.grow_family_with_room();
        } else {
            player.grow_family_without_room();
        }
    }

    pub fn display(&self) {
        println!("\n\n-- Board --");

        for action in &self.open_spaces {
            let idx = action.action_idx();
            if self.occupied_spaces.contains(&idx) {
                print!("\n[X] {:?} is occupied", &action);
            } else {
                print!("\n[-] {:?} ", &action);
                if action.resource_map_idx().is_some() {
                    print_resources(&self.resource_map[action.resource_map_idx().unwrap()]);
                }
            }
        }

        print!("\nMajors Available [");
        MajorImprovement::display(&self.major_improvements);
        println!("]");

        println!("\n\n-- Players --");
        for i in 0..self.players.len() {
            let p = &self.players[i];
            print!("\nFarm {i} ");
            if i == self.current_player_idx {
                print!("[Turn]");
            }
            if i == self.starting_player_idx {
                print!("[Start Player]");
            }
            println!();
            p.display();
        }
    }
}
