use super::actions::{Action, NUM_RESOURCE_SPACES};
use super::algorithms::PlayerType;
use super::major_improvements::{MajorImprovement, TOTAL_MAJOR_IMPROVEMENTS};
use super::occupations::Occupation;
use super::player::Player;
use super::primitives::*;
use super::scoring;
use rand::Rng;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const INITIAL_OPEN_SPACES: usize = 16;
pub const NUM_ACTION_SPACES: usize = 30;
const MAX_NUM_PLAYERS: usize = 4;

pub const FIREPLACE_INDICES: [usize; 2] = [
    MajorImprovement::Fireplace { cheaper: true }.index(),
    MajorImprovement::Fireplace { cheaper: false }.index(),
];
pub const COOKING_HEARTH_INDICES: [usize; 2] = [
    MajorImprovement::CookingHearth { cheaper: true }.index(),
    MajorImprovement::CookingHearth { cheaper: false }.index(),
];

pub const FIREPLACE_AND_COOKING_HEARTH_INDICES: [usize; 4] = [
    MajorImprovement::Fireplace { cheaper: true }.index(),
    MajorImprovement::Fireplace { cheaper: false }.index(),
    MajorImprovement::CookingHearth { cheaper: true }.index(),
    MajorImprovement::CookingHearth { cheaper: false }.index(),
];

pub const BAKING_MAJOR_IMPROVEMENT_INDICES: [usize; 6] = [
    MajorImprovement::Fireplace { cheaper: true }.index(),
    MajorImprovement::Fireplace { cheaper: false }.index(),
    MajorImprovement::CookingHearth { cheaper: true }.index(),
    MajorImprovement::CookingHearth { cheaper: false }.index(),
    MajorImprovement::ClayOven.index(),
    MajorImprovement::StoneOven.index(),
];

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
    pub major_improvements: [(MajorImprovement, Option<usize>, usize); 10], // (Major, PlayerIdx, Number_of_times_used_in_harvest)
    pub current_player_idx: usize,
    pub starting_player_idx: usize,
    pub people_placed_this_round: usize,
    pub last_action: Action,
    pub start_round_events: Vec<Event>,
    pub available_occupations: Vec<Occupation>,
}

impl State {
    pub fn new(players: &[PlayerType]) -> Option<Self> {
        if players.is_empty() {
            return None;
        }

        let first_player_idx = rand::thread_rng().gen_range(0..players.len());
        let mut state = State {
            resource_map: Action::init_resource_map(),
            open_spaces: Action::initial_open_spaces(),
            occupied_spaces: Vec::new(),
            hidden_spaces: Action::initial_hidden_spaces(),
            major_improvements: [
                (MajorImprovement::Fireplace { cheaper: true }, None, 0),
                (MajorImprovement::Fireplace { cheaper: false }, None, 0),
                (MajorImprovement::CookingHearth { cheaper: true }, None, 0),
                (MajorImprovement::CookingHearth { cheaper: false }, None, 0),
                (MajorImprovement::Well, None, 0),
                (MajorImprovement::ClayOven, None, 0),
                (MajorImprovement::StoneOven, None, 0),
                (MajorImprovement::Joinery, None, 0),
                (MajorImprovement::Pottery, None, 0),
                (MajorImprovement::BasketmakersWorkshop, None, 0),
            ],
            players: Vec::<Player>::new(),
            current_player_idx: first_player_idx,
            starting_player_idx: first_player_idx,
            people_placed_this_round: 0,
            last_action: Action::StartGame,
            start_round_events: vec![],
            available_occupations: Occupation::all(),
        };
        state.init_players(players, first_player_idx);
        //println!("New Game Started");
        Some(state)
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
        // player.farm.farm_animals_to_resources(&mut player.resources);
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

    pub fn play_random(&mut self, path: &mut Vec<u64>, opt_depth: Option<usize>) {
        let mut d: usize = 0;
        loop {
            if let Some(depth) = opt_depth {
                if d == depth {
                    break;
                }
            }
            let choices = Action::next_choices(self);
            if choices.is_empty() {
                break;
            }

            // Only one choice, play it
            if choices.len() == 1 {
                choices[0].apply_choice(self);
                continue;
            }

            d += 1;

            // Chose a random action
            let action_idx = rand::thread_rng().gen_range(0..choices.len());
            choices[action_idx].apply_choice(self);
            path.push(self.get_hash());
        }
    }

    pub fn player_type(&self) -> PlayerType {
        self.players[self.current_player_idx].player_type()
    }

    pub fn fitness(&self) -> Vec<f32> {
        let scores = self.scores();

        if scores.len() == 1 {
            return scores;
        }

        let mut fitness = scores.clone();
        let mut sorted_scores = scores;

        // Sort in decreasing order
        sorted_scores.sort_by(|a, b| b.partial_cmp(a).unwrap());

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

    #[allow(clippy::cast_possible_wrap)]
    fn score_cards(&self) -> [i32; MAX_NUM_PLAYERS] {
        let mut ret: [i32; MAX_NUM_PLAYERS] = [0; MAX_NUM_PLAYERS];
        // Score Majors
        for (major, opt_idx, _) in &self.major_improvements {
            if let Some(idx) = opt_idx {
                ret[*idx] += major.points() as i32;
                let player = &self.players[*idx];
                match major {
                    MajorImprovement::Joinery => {
                        if player.resources[Wood.index()] >= 7 {
                            ret[*idx] += 3;
                        } else if player.resources[Wood.index()] >= 5 {
                            ret[*idx] += 2;
                        } else if player.resources[Wood.index()] >= 3 {
                            ret[*idx] += 1;
                        }
                    }
                    MajorImprovement::Pottery => {
                        if player.resources[Clay.index()] >= 7 {
                            ret[*idx] += 3;
                        } else if player.resources[Clay.index()] >= 5 {
                            ret[*idx] += 2;
                        } else if player.resources[Clay.index()] >= 3 {
                            ret[*idx] += 1;
                        }
                    }
                    MajorImprovement::BasketmakersWorkshop => {
                        if player.resources[Reed.index()] >= 5 {
                            ret[*idx] += 3;
                        } else if player.resources[Reed.index()] >= 4 {
                            ret[*idx] += 2;
                        } else if player.resources[Reed.index()] >= 2 {
                            ret[*idx] += 1;
                        }
                    }
                    _ => (),
                }
            }
        }
        ret
    }

    pub fn scores(&self) -> Vec<f32> {
        let mut scores: Vec<f32> = Vec::new();
        let card_scores = self.score_cards();
        for (idx, p) in self.players.iter().enumerate() {
            scores.push(scoring::score(p) + card_scores[idx] as f32);
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

        // Reset majors if used in harvest
        for (_, _, n_used) in &mut self.major_improvements {
            *n_used = 0;
        }

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

    fn init_players(&mut self, player_types: &[PlayerType], first_idx: usize) {
        for (i, player_type) in player_types.iter().enumerate() {
            let food = if i == first_idx { 2 } else { 3 };
            let player = Player::create_new(food, player_type.clone());
            self.players.push(player);
        }
    }

    pub fn available_majors_to_build(&self) -> [bool; TOTAL_MAJOR_IMPROVEMENTS] {
        let mut available: [bool; TOTAL_MAJOR_IMPROVEMENTS] = [false; TOTAL_MAJOR_IMPROVEMENTS];

        for f_idx in FIREPLACE_INDICES {
            if Some(self.current_player_idx) == self.major_improvements[f_idx].1 {
                for ch_idx in COOKING_HEARTH_INDICES {
                    if self.major_improvements[ch_idx].1.is_none() {
                        available[ch_idx] = true;
                        break;
                    }
                }
                break;
            }
        }

        for (idx, (major, player_idx, _)) in self.major_improvements.iter().enumerate() {
            if player_idx.is_none()
                && can_pay_for_resource(
                    &major.cost(),
                    &self.players[self.current_player_idx].resources,
                )
            {
                available[idx] = true;
            }
        }

        available
    }

    pub fn replace_fireplace_with_cooking_hearth(&mut self) {
        for f_idx in FIREPLACE_INDICES {
            if Some(self.current_player_idx) == self.major_improvements[f_idx].1 {
                for ch_idx in COOKING_HEARTH_INDICES {
                    if self.major_improvements[ch_idx].1.is_none() {
                        self.major_improvements[ch_idx].1 = Some(self.current_player_idx);
                        self.major_improvements[f_idx].1 = None;
                        break;
                    }
                }
                break;
            }
        }
    }

    pub fn build_major(&mut self, major: &MajorImprovement, return_fireplace: bool) {
        let player = &mut self.players[self.current_player_idx];
        match major {
            &MajorImprovement::Fireplace { cheaper: _ }
            | &MajorImprovement::CookingHearth { cheaper: _ } => {
                player.has_cooking_improvement = true;
            }
            _ => {}
        }

        if return_fireplace && matches!(major, MajorImprovement::CookingHearth { cheaper: _ }) {
            self.replace_fireplace_with_cooking_hearth();
        } else {
            pay_for_resource(&major.cost(), &mut player.resources);
            self.major_improvements[major.index()].1 = Some(self.current_player_idx);

            if matches!(major, MajorImprovement::Well) {
                let current_round = self.current_round();
                let func = |mut res: Resources| {
                    res[Food.index()] += 1;
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

    pub fn bake_bread(&mut self, num_grain_to_bake: usize) {
        let player = &mut self.players[self.current_player_idx];
        assert!(player.can_bake_bread(self.current_player_idx, &self.major_improvements));
        let mut num_grain_to_bake = num_grain_to_bake;
        while num_grain_to_bake > 0 {
            if player.resources[Grain.index()] == 0 {
                break;
            }
            if Some(self.current_player_idx)
                == self.major_improvements[MajorImprovement::ClayOven.index()].1
                && self.major_improvements[MajorImprovement::ClayOven.index()].2 == 0
            {
                player.resources[Food.index()] += 5;
                self.major_improvements[MajorImprovement::ClayOven.index()].2 = 1;
            } else if Some(self.current_player_idx)
                == self.major_improvements[MajorImprovement::StoneOven.index()].1
                && self.major_improvements[MajorImprovement::StoneOven.index()].2 < 2
            {
                player.resources[Food.index()] += 4;
                self.major_improvements[MajorImprovement::StoneOven.index()].2 += 1;
            } else if Some(self.current_player_idx)
                == self.major_improvements
                    [MajorImprovement::CookingHearth { cheaper: true }.index()]
                .1
                || Some(self.current_player_idx)
                    == self.major_improvements
                        [MajorImprovement::CookingHearth { cheaper: false }.index()]
                    .1
            {
                player.resources[Food.index()] += 3;
            } else if Some(self.current_player_idx)
                == self.major_improvements[MajorImprovement::Fireplace { cheaper: true }.index()].1
                || Some(self.current_player_idx)
                    == self.major_improvements
                        [MajorImprovement::Fireplace { cheaper: false }.index()]
                    .1
            {
                player.resources[Food.index()] += 2;
            }

            player.resources[Grain.index()] -= 1;
            num_grain_to_bake -= 1;
        }
    }

    pub fn pay_food_or_beg(&mut self) {
        let player = &mut self.players[self.current_player_idx];
        let food_required = 2 * player.adults + player.children;
        if food_required > player.resources[Food.index()] {
            player.begging_tokens += food_required - player.resources[Food.index()];
            player.resources[Food.index()] = 0;
        } else {
            player.resources[Food.index()] -= food_required;
        }

        player.harvest_paid = true;
        player.reorg_animals(self.current_player_idx, &self.major_improvements, true);
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

    pub fn format(&self) -> String {
        let mut ret: String = String::new();
        for action in &self.open_spaces {
            let idx = action.action_idx();
            if self.occupied_spaces.contains(&idx) {
                ret = format!("{}\n[X] {:?} is occupied", ret, action);
            } else {
                ret = format!("{}\n[-] {:?}", ret, action);
                if action.resource_map_idx().is_some() {
                    ret = format!(
                        "{}{}",
                        ret,
                        format_resources(&self.resource_map[action.resource_map_idx().unwrap()])
                    );
                }
            }
        }

        ret = format!("{}\n\n=== Available majors ===\n", ret);
        for major in &self.major_improvements {
            ret = format!("{}\n{major:?}", ret);
        }
        ret
    }

    pub fn player(&self) -> &Player {
        &self.players[self.current_player_idx]
    }

    pub fn player_mut(&mut self) -> &mut Player {
        &mut self.players[self.current_player_idx]
    }
}
