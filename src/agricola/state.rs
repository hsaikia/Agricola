use super::actions::{Action, NUM_RESOURCE_SPACES};
use super::algorithms::PlayerType;
use super::card::{NUM_CARDS, OCCUPATIONS_INDICES};
use super::display::format_resources;
use super::farm::{Farm, Seed};
use super::fencing::PastureConfig;
use super::flag::*;
use super::major_improvements::{MajorImprovement, TOTAL_MAJOR_IMPROVEMENTS};
use super::quantity::*;
use super::scoring::score_farm;
use core::panic;
use rand::Rng;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const INITIAL_OPEN_SPACES: usize = 16;
pub const NUM_ACTION_SPACES: usize = 30;
const MAX_NUM_PLAYERS: usize = 4;
const MAX_FAMILY_MEMBERS: usize = 5;

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
    pub func: fn(res: &mut [usize; NUM_QUANTITIES]),
}

#[derive(Clone, Hash)]
pub struct State {
    pub num_players: usize,
    pub resource_map: [Resources; NUM_RESOURCE_SPACES],
    pub open_spaces: Vec<Action>,
    pub occupied_spaces: Vec<usize>,
    pub hidden_spaces: Vec<Vec<Action>>,
    player_types: [PlayerType; MAX_NUM_PLAYERS],
    player_quantities: [[usize; NUM_QUANTITIES]; MAX_NUM_PLAYERS],
    player_flags: [[bool; NUM_FLAGS]; MAX_NUM_PLAYERS],
    player_cards: [[bool; NUM_CARDS]; MAX_NUM_PLAYERS],
    farms: [Farm; MAX_NUM_PLAYERS],
    pub major_improvements: [(MajorImprovement, Option<usize>, usize); 10], // (Major, PlayerIdx, Number_of_times_used_in_harvest)
    pub current_player_idx: usize,
    pub starting_player_idx: usize,
    pub people_placed_this_round: usize,
    pub last_action: Action,
    pub start_round_events: Vec<Event>,
}

impl State {
    pub fn new(players: &[PlayerType]) -> Option<Self> {
        if players.is_empty() {
            return None;
        }

        let first_player_idx = rand::thread_rng().gen_range(0..players.len());
        let mut player_quantities = [[0; NUM_QUANTITIES]; MAX_NUM_PLAYERS];
        for (i, player_quantities) in player_quantities.iter_mut().enumerate().take(players.len()) {
            if i == first_player_idx {
                player_quantities[Food.index()] = 2;
            } else {
                player_quantities[Food.index()] = 3;
            }
            player_quantities[Rooms.index()] = 2;
            player_quantities[AdultMembers.index()] = 2;
        }

        let mut player_flags = [[false; NUM_FLAGS]; MAX_NUM_PLAYERS];
        for player_flags in player_flags.iter_mut().take(players.len()) {
            player_flags[WoodHouse.index()] = true;
        }

        let mut player_types = [PlayerType::MCTSMachine; MAX_NUM_PLAYERS];
        for (i, player_type) in player_types.iter_mut().enumerate().take(players.len()) {
            *player_type = players[i];
        }

        let state = State {
            num_players: players.len(),
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
            player_types,
            player_quantities,
            player_flags,
            player_cards: [[false; NUM_CARDS]; MAX_NUM_PLAYERS],
            farms: core::array::from_fn(|_| Farm::new()),
            current_player_idx: first_player_idx,
            starting_player_idx: first_player_idx,
            people_placed_this_round: 0,
            last_action: Action::StartGame,
            start_round_events: vec![],
        };
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

    pub fn harvest_paid(&self) -> bool {
        self.current_player_flags()[HarvestPaid.index()]
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

    pub fn harvest_fields(&mut self) {
        let crops = self.current_farm_mut().harvest_fields();
        for crop in crops {
            match crop {
                Seed::Grain => self.current_player_quantities_mut()[Grain.index()] += 1,
                Seed::Vegetable => self.current_player_quantities_mut()[Vegetable.index()] += 1,
            }
        }
    }

    pub fn family_members(&self, player_idx: usize) -> usize {
        self.player_quantities(player_idx)[AdultMembers.index()]
            + self.player_quantities(player_idx)[Children.index()]
    }

    fn food_required(&self) -> usize {
        2 * self.current_player_quantities()[AdultMembers.index()]
            + self.current_player_quantities()[Children.index()]
    }

    pub fn got_enough_food(&self) -> bool {
        2 * self.current_player_quantities()[AdultMembers.index()]
            + self.current_player_quantities()[Children.index()]
            <= self.current_player_quantities()[Food.index()]
    }

    // After paying for harvest - this function needs to be called to clear the empty hidden space
    pub fn remove_empty_stage(&mut self) {
        if self.hidden_spaces.last().is_some() && self.hidden_spaces.last().unwrap().is_empty() {
            self.hidden_spaces.pop();
        }
    }

    pub fn can_grow_family_with_room(&self) -> bool {
        self.family_members(self.current_player_idx) < MAX_FAMILY_MEMBERS
            && self.current_player_flags()[HasRoomToGrow.index()]
    }

    pub fn grow_family_with_room(&mut self) {
        assert!(self.can_grow_family_with_room());
        self.current_player_quantities_mut()[Children.index()] += 1;
        if self.family_members(self.current_player_idx)
            >= self.current_player_quantities()[Rooms.index()]
        {
            self.current_player_flags_mut()[HasRoomToGrow.index()] = false;
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

    pub fn player_type(&self, player_idx: usize) -> PlayerType {
        self.player_types[player_idx]
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
                match major {
                    MajorImprovement::Joinery => {
                        if self.player_quantities(*idx)[Wood.index()] >= 7 {
                            ret[*idx] += 3;
                        } else if self.player_quantities(*idx)[Wood.index()] >= 5 {
                            ret[*idx] += 2;
                        } else if self.player_quantities(*idx)[Wood.index()] >= 3 {
                            ret[*idx] += 1;
                        }
                    }
                    MajorImprovement::Pottery => {
                        if self.player_quantities(*idx)[Clay.index()] >= 7 {
                            ret[*idx] += 3;
                        } else if self.player_quantities(*idx)[Clay.index()] >= 5 {
                            ret[*idx] += 2;
                        } else if self.player_quantities(*idx)[Clay.index()] >= 3 {
                            ret[*idx] += 1;
                        }
                    }
                    MajorImprovement::BasketmakersWorkshop => {
                        if self.player_quantities(*idx)[Reed.index()] >= 5 {
                            ret[*idx] += 3;
                        } else if self.player_quantities(*idx)[Reed.index()] >= 4 {
                            ret[*idx] += 2;
                        } else if self.player_quantities(*idx)[Reed.index()] >= 2 {
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
        for (idx, card_score) in card_scores.iter().enumerate().take(self.num_players) {
            scores.push(self.score(idx) + *card_score as f32);
        }
        scores
    }

    pub fn reset_for_next_round(&mut self) {
        self.player_quantities.iter_mut().for_each(|p| {
            p[MembersPlacedThisRound.index()] = 0;
            p[AdultMembers.index()] += p[Children.index()];
            p[Children.index()] = 0;
        });

        self.player_flags.iter_mut().for_each(|p| {
            p[HarvestPaid.index()] = false;
            p[BeforeRoundStart.index()] = true;
        });
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
        self.reset_for_next_round();
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
        let start_round_events = self.start_round_events.clone();
        for event in start_round_events {
            for i in 0..self.num_players {
                if event.round == current_round && event.player_idx == i {
                    (event.func)(self.player_quantities_mut(i));
                }
            }
        }
    }

    pub fn add_new_field(&mut self, idx: &usize) {
        self.current_farm_mut().add_field(*idx);
    }

    pub fn field_options(&self) -> Vec<usize> {
        self.current_farm().possible_field_positions()
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
                    self.player_quantities(self.current_player_idx),
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

    fn room_material_idx(&self) -> usize {
        if self.current_player_flags()[WoodHouse.index()] {
            Wood.index()
        } else if self.current_player_flags()[ClayHouse.index()] {
            Clay.index()
        } else if self.current_player_flags()[StoneHouse.index()] {
            Stone.index()
        } else {
            panic!("No house type set");
        }
    }

    pub fn set_can_build_room(&mut self) {
        self.current_player_flags_mut()[CanBuildRoom.index()] =
            !self.current_farm().possible_room_positions().is_empty()
                && self.current_player_quantities()[self.room_material_idx()] >= 5
                && self.current_player_quantities()[Reed.index()] >= 2;
    }

    pub fn can_build_room(&self) -> bool {
        self.current_player_flags()[CanBuildRoom.index()]
    }

    // Builds a single room
    pub fn build_room(&mut self, idx: &usize) {
        assert!(self.can_build_room());
        // By default Rooms cost 5 of the corresponding building resource (as the material of the house and 2 Reed)
        let room_material_idx = self.room_material_idx();
        self.current_player_quantities_mut()[room_material_idx] -= 5;
        self.current_player_quantities_mut()[Reed.index()] -= 2;
        self.current_farm_mut().build_room(*idx);

        // Increment player quantities
        self.current_player_quantities_mut()[Rooms.index()] += 1;
        let rooms = self.current_player_quantities()[Rooms.index()];

        // Set the Can Grow flag
        if rooms > self.family_members(self.current_player_idx) {
            self.current_player_flags_mut()[HasRoomToGrow.index()] = true;
        }

        // Can Renovate flag is perhaps dirty, set it
        self.set_can_renovate();

        // Can Build Room flag is perhaps dirty, set it
        self.set_can_build_room();
    }

    pub fn room_options(&self) -> Vec<usize> {
        if self.can_build_room() {
            return self.current_farm().possible_room_positions();
        }
        Vec::new()
    }

    pub fn set_can_build_stable(&mut self) {
        self.current_player_flags_mut()[CanBuildStable.index()] =
            self.current_player_quantities()[Wood.index()] >= 2;
    }

    pub fn can_build_stable(&self) -> bool {
        self.current_player_flags()[CanBuildStable.index()]
    }

    // Builds a single stable
    pub fn build_stable(&mut self, idx: &usize) {
        assert!(self.can_build_stable());
        self.current_player_quantities_mut()[Wood.index()] -= 2;
        self.current_farm_mut().build_stable(*idx);
    }

    pub fn stable_options(&self) -> Vec<usize> {
        if self.can_build_stable() {
            return self.current_farm().possible_stable_positions();
        }
        Vec::new()
    }

    pub fn set_can_renovate(&mut self) {
        self.current_player_flags_mut()[CanRenovate.index()] =
            self.current_player_quantities()[Reed.index()] >= 1
                && ((self.current_player_flags()[WoodHouse.index()]
                    && self.current_player_quantities()[Clay.index()]
                        >= self.current_player_quantities()[Rooms.index()])
                    || (self.current_player_flags()[ClayHouse.index()]
                        && self.current_player_quantities()[Stone.index()]
                            >= self.current_player_quantities()[Rooms.index()]));
    }

    pub fn can_renovate(&self) -> bool {
        self.current_player_flags()[CanRenovate.index()]
    }

    fn renovation_material_idx(&self) -> Option<usize> {
        if self.current_player_flags()[WoodHouse.index()] {
            Some(Clay.index())
        } else if self.current_player_flags()[ClayHouse.index()] {
            Some(Stone.index())
        } else {
            None
        }
    }

    pub fn renovate(&mut self) {
        assert!(self.current_player_flags()[CanRenovate.index()]);
        // TODO for cards like Conservator this must be implemented in a more general way
        let renovation_material_idx = self.renovation_material_idx();

        if let Some(renovation_material_idx) = renovation_material_idx {
            self.current_player_quantities_mut()[renovation_material_idx] -=
                self.current_player_quantities()[Rooms.index()];
            self.current_player_quantities_mut()[Reed.index()] -= 1;

            if renovation_material_idx == Clay.index() {
                self.current_player_flags_mut()[WoodHouse.index()] = false;
                self.current_player_flags_mut()[ClayHouse.index()] = true;
                self.current_player_flags_mut()[StoneHouse.index()] = false;
            } else if renovation_material_idx == Stone.index() {
                self.current_player_flags_mut()[WoodHouse.index()] = false;
                self.current_player_flags_mut()[ClayHouse.index()] = false;
                self.current_player_flags_mut()[StoneHouse.index()] = true;
            }
        }
    }

    pub fn build_major(&mut self, major: &MajorImprovement, return_fireplace: bool) {
        match major {
            &MajorImprovement::Fireplace { cheaper: _ }
            | &MajorImprovement::CookingHearth { cheaper: _ } => {
                self.current_player_flags_mut()[HasCookingImprovement.index()] = true;
            }
            _ => {}
        }

        if return_fireplace && matches!(major, MajorImprovement::CookingHearth { cheaper: _ }) {
            self.replace_fireplace_with_cooking_hearth();
        } else {
            pay_for_resource(&major.cost(), self.current_player_quantities_mut());
            self.major_improvements[major.index()].1 = Some(self.current_player_idx);

            if matches!(major, MajorImprovement::Well) {
                let current_round = self.current_round();
                let func = |res: &mut Quantities| {
                    res[Food.index()] += 1;
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

    pub fn fencing_choices(&self) -> Vec<PastureConfig> {
        self.current_farm()
            .fencing_options(self.current_player_quantities()[Wood.index()])
    }

    pub fn fence(&mut self, pasture_config: &PastureConfig) {
        assert!(self.can_fence());
        let mut wood = self.current_player_quantities()[Wood.index()];
        self.current_farm_mut()
            .fence_spaces(pasture_config, &mut wood);
        self.current_player_quantities_mut()[Wood.index()] = wood;
    }

    pub fn can_fence(&self) -> bool {
        !self
            .current_farm()
            .fencing_options(self.current_player_quantities()[Wood.index()])
            .is_empty()
    }

    pub fn can_use_exchange(&self, res_ex: &ResourceExchange) -> bool {
        self.current_player_quantities()[res_ex.from] >= res_ex.num_from
    }

    pub fn use_exchange(&mut self, res_ex: &ResourceExchange) {
        assert!(self.can_use_exchange(res_ex));
        self.current_player_quantities_mut()[res_ex.from] -= res_ex.num_from;
        self.current_player_quantities_mut()[res_ex.to] += res_ex.num_to;
    }

    pub fn has_resources_to_cook(&self) -> bool {
        self.current_player_quantities()[Sheep.index()]
            + self.current_player_quantities()[Boar.index()]
            + self.current_player_quantities()[Cattle.index()]
            + self.current_player_quantities()[Vegetable.index()]
            > 0
    }

    pub fn can_bake_bread(&self, player_idx: usize) -> bool {
        // Check if any of the baking improvements are present
        // And at least one grain in supply

        (Some(player_idx) == self.major_improvements[MajorImprovement::ClayOven.index()].1
            || Some(player_idx) == self.major_improvements[MajorImprovement::StoneOven.index()].1
            || Some(player_idx)
                == self.major_improvements[MajorImprovement::Fireplace { cheaper: true }.index()].1
            || Some(player_idx)
                == self.major_improvements[MajorImprovement::Fireplace { cheaper: false }.index()]
                    .1
            || Some(player_idx)
                == self.major_improvements
                    [MajorImprovement::CookingHearth { cheaper: true }.index()]
                .1
            || Some(player_idx)
                == self.major_improvements
                    [MajorImprovement::CookingHearth { cheaper: false }.index()]
                .1)
            && self.current_player_quantities()[Grain.index()] > 0
    }

    pub fn sow_field(&mut self, seed: &Seed) {
        self.current_farm_mut().sow_field(seed);
        match seed {
            Seed::Grain => self.current_player_quantities_mut()[Grain.index()] -= 1,
            Seed::Vegetable => self.current_player_quantities_mut()[Vegetable.index()] -= 1,
        }
    }

    pub fn can_sow(&self) -> bool {
        (self.current_player_quantities()[Grain.index()] > 0
            || self.current_player_quantities()[Vegetable.index()] > 0)
            && self.current_farm().can_sow()
    }

    pub fn bake_bread(&mut self, num_grain_to_bake: usize) {
        assert!(self.can_bake_bread(self.current_player_idx));
        let mut num_grain_to_bake = num_grain_to_bake;
        while num_grain_to_bake > 0 {
            if self.current_player_quantities()[Grain.index()] == 0 {
                break;
            }
            if Some(self.current_player_idx)
                == self.major_improvements[MajorImprovement::ClayOven.index()].1
                && self.major_improvements[MajorImprovement::ClayOven.index()].2 == 0
            {
                self.current_player_quantities_mut()[Food.index()] += 5;
                self.major_improvements[MajorImprovement::ClayOven.index()].2 = 1;
            } else if Some(self.current_player_idx)
                == self.major_improvements[MajorImprovement::StoneOven.index()].1
                && self.major_improvements[MajorImprovement::StoneOven.index()].2 < 2
            {
                self.current_player_quantities_mut()[Food.index()] += 4;
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
                self.current_player_quantities_mut()[Food.index()] += 3;
            } else if Some(self.current_player_idx)
                == self.major_improvements[MajorImprovement::Fireplace { cheaper: true }.index()].1
                || Some(self.current_player_idx)
                    == self.major_improvements
                        [MajorImprovement::Fireplace { cheaper: false }.index()]
                    .1
            {
                self.current_player_quantities_mut()[Food.index()] += 2;
            }

            self.current_player_quantities_mut()[Grain.index()] -= 1;
            num_grain_to_bake -= 1;
        }
    }

    pub fn all_people_placed(&self) -> bool {
        self.current_player_quantities()[MembersPlacedThisRound.index()]
            == self.current_player_quantities()[AdultMembers.index()]
    }

    pub fn increment_people_placed(&mut self) {
        self.current_player_quantities_mut()[MembersPlacedThisRound.index()] += 1;
        self.current_player_flags_mut()[BeforeRoundStart.index()] = false;
    }

    pub fn pay_food_or_beg(&mut self) {
        let food_required = self.food_required();
        self.current_player_flags_mut()[HarvestPaid.index()] = true;
        if food_required > self.current_player_quantities()[Food.index()] {
            self.current_player_quantities_mut()[BeggingTokens.index()] +=
                food_required - self.current_player_quantities()[Food.index()];
            self.current_player_quantities_mut()[Food.index()] = 0;
        } else {
            self.current_player_quantities_mut()[Food.index()] -= food_required;
        }

        self.accommodate_animals(true);
        self.current_player_idx = (self.current_player_idx + 1) % self.num_players;
        self.remove_empty_stage();
    }

    pub fn accommodate_animals(&mut self, breed: bool) {
        let mut quantites = *self.current_player_quantities();

        if breed {
            if quantites[Sheep.index()] > 1 {
                quantites[Sheep.index()] += 1;
            }
            if quantites[Boar.index()] > 1 {
                quantites[Boar.index()] += 1;
            }
            if quantites[Cattle.index()] > 1 {
                quantites[Cattle.index()] += 1;
            }
        }

        let animals = [
            quantites[Sheep.index()],
            quantites[Boar.index()],
            quantites[Cattle.index()],
        ];
        let leftover = self.current_farm_mut().accommodate_animals(&animals);

        quantites[Sheep.index()] -= leftover[0];
        quantites[Boar.index()] -= leftover[1];
        quantites[Cattle.index()] -= leftover[2];

        // TODO: Don't just toss animals this way - give a choice to the player and re-org using that choice
        let mut owns_ch = false;
        for ch_idx in COOKING_HEARTH_INDICES {
            if Some(self.current_player_idx) == self.major_improvements[ch_idx].1 {
                quantites[Food.index()] += 2 * leftover[0] + 3 * leftover[1] + 4 * leftover[2];
                owns_ch = true;
                break;
            }
        }

        if !owns_ch {
            for fp_idx in FIREPLACE_INDICES {
                if Some(self.current_player_idx) == self.major_improvements[fp_idx].1 {
                    quantites[Food.index()] += 2 * leftover[0] + 2 * leftover[1] + 3 * leftover[2];
                    return;
                }
            }
        }

        *self.current_player_quantities_mut() = quantites;
    }

    pub fn total_workers(&self) -> usize {
        self.player_quantities
            .iter()
            .map(|q| q[AdultMembers.index()])
            .sum()
    }

    pub fn before_round_start(&self) -> bool {
        self.current_player_flags()[BeforeRoundStart.index()]
    }

    pub fn score(&self, player_idx: usize) -> f32 {
        let mut ret: f32 = 0.0;
        // House, Family and Empty Spaces
        ret += 3.0 * self.family_members(player_idx) as f32;
        // Begging Tokens
        ret -= 3.0 * self.player_quantities(player_idx)[BeggingTokens.index()] as f32;
        // Score farm
        ret += score_farm(self, player_idx) as f32;

        ret
    }

    pub fn end_turn(&mut self) {
        // Increment people placed by player
        self.increment_people_placed();

        // Increment workers placed
        self.people_placed_this_round += 1;

        // Advance to next player
        self.current_player_idx = (self.current_player_idx + 1) % self.num_players;

        // Skip over players that have all their workers placed
        if self.people_placed_this_round < self.total_workers() {
            while self.all_people_placed() {
                self.current_player_idx = (self.current_player_idx + 1) % self.num_players;
            }
        }
    }

    pub fn grow_family_without_room(&mut self) {
        assert!(self.can_grow_family_without_room());
        self.current_player_quantities_mut()[Children.index()] += 1;
    }

    pub fn can_grow_family_without_room(&self) -> bool {
        self.family_members(self.current_player_idx) < MAX_FAMILY_MEMBERS
    }

    pub fn grow_family(&mut self, with_room: bool) {
        if with_room {
            self.grow_family_with_room();
        } else {
            self.grow_family_without_room();
        }
    }

    pub fn num_occupations_played(&self) -> usize {
        OCCUPATIONS_INDICES.iter().fold(0, |acc, idx| {
            if self.player_cards[self.current_player_idx][*idx] {
                acc + 1
            } else {
                acc
            }
        })
    }

    pub fn can_play_occupation(&self, cheaper: bool) -> bool {
        let mut required_food = if cheaper { 1 } else { 2 };
        if self.num_occupations_played() == 0 && cheaper {
            required_food = 0;
        }
        if self.num_occupations_played() < 2 && !cheaper {
            required_food = 1;
        }

        // If can pay directly
        if required_food <= self.current_player_quantities()[Food.index()] {
            return true;
        }

        // If cannot pay directly, but can convert some resources
        required_food -= self.current_player_quantities()[Food.index()];

        let raw_grain_and_veg = self.current_player_quantities()[Grain.index()]
            + self.current_player_quantities()[Vegetable.index()];
        if required_food <= raw_grain_and_veg {
            return true;
        }

        // Required food must be less than 3, and minimum food gained by cooking is 2
        if self.current_player_flags()[HasCookingImprovement.index()]
            && self.has_resources_to_cook()
        {
            return true;
        }

        false
    }

    pub fn format(&self) -> String {
        let mut ret: String = String::new();
        for action in &self.open_spaces {
            let idx = action.action_idx();
            if self.occupied_spaces.contains(&idx) {
                ret.push_str(&format!("\n[X] {:?} is occupied", action));
            } else {
                ret.push_str(&format!("\n[-] {:?}", action));
                if action.resource_map_idx().is_some() {
                    ret.push_str(&format_resources(
                        &self.resource_map[action.resource_map_idx().unwrap()],
                    ));
                }
            }
        }

        ret.push_str("\n\n=== Available majors ===\n");
        for (major, owner, _) in &self.major_improvements {
            ret.push_str(&format!("\n{major:?}"));
            if owner.is_some() {
                ret.push_str(&format!(". Owned by Player {}", owner.unwrap() + 1));
            }
        }
        ret
    }

    fn card_available(&self, card_idx: usize) -> bool {
        (0..self.num_players).all(|i| !self.player_cards[i][card_idx])
    }

    pub fn occupations_available(&self) -> Vec<usize> {
        OCCUPATIONS_INDICES
            .iter()
            .filter(|idx| self.card_available(**idx))
            .copied()
            .collect()
    }

    pub fn player_quantities(&self, player_idx: usize) -> &[usize; NUM_QUANTITIES] {
        &self.player_quantities[player_idx]
    }

    pub fn player_quantities_mut(&mut self, player_idx: usize) -> &mut [usize; NUM_QUANTITIES] {
        &mut self.player_quantities[player_idx]
    }

    pub fn player_flags(&self, player_idx: usize) -> &[bool; NUM_FLAGS] {
        &self.player_flags[player_idx]
    }

    pub fn player_flags_mut(&mut self, player_idx: usize) -> &mut [bool; NUM_FLAGS] {
        &mut self.player_flags[player_idx]
    }

    pub fn current_player_quantities(&self) -> &[usize; NUM_QUANTITIES] {
        &self.player_quantities[self.current_player_idx]
    }

    pub fn current_player_quantities_mut(&mut self) -> &mut [usize; NUM_QUANTITIES] {
        &mut self.player_quantities[self.current_player_idx]
    }

    pub fn current_player_flags(&self) -> &[bool; NUM_FLAGS] {
        &self.player_flags[self.current_player_idx]
    }

    pub fn current_player_flags_mut(&mut self) -> &mut [bool; NUM_FLAGS] {
        &mut self.player_flags[self.current_player_idx]
    }

    pub fn player_cards(&self, player_idx: usize) -> &[bool; NUM_CARDS] {
        &self.player_cards[player_idx]
    }

    pub fn player_cards_mut(&mut self, player_idx: usize) -> &mut [bool; NUM_CARDS] {
        &mut self.player_cards[player_idx]
    }

    pub fn current_player_cards(&self) -> &[bool; NUM_CARDS] {
        &self.player_cards[self.current_player_idx]
    }

    pub fn current_player_cards_mut(&mut self) -> &mut [bool; NUM_CARDS] {
        &mut self.player_cards[self.current_player_idx]
    }

    pub fn current_farm(&self) -> &Farm {
        self.player_farm(self.current_player_idx)
    }

    pub fn current_farm_mut(&mut self) -> &mut Farm {
        &mut self.farms[self.current_player_idx]
    }

    pub fn player_farm(&self, player_idx: usize) -> &Farm {
        &self.farms[player_idx]
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_can_use_farm_expansion() {
        env::set_var("RUN_BACKTRACE", "1");
        let mut state = State::new(&[PlayerType::Human]).unwrap();

        assert!(state.current_player_flags()[WoodHouse.index()]);

        let room_positions = state.current_farm().possible_room_positions();
        assert_eq!(room_positions.len(), 3);

        state.current_player_quantities_mut()[Wood.index()] = 5;
        state.current_player_quantities_mut()[Reed.index()] = 2;

        assert!(!state.can_build_room());
        state.set_can_build_room();
        assert!(state.can_build_room());
        assert!(!state.can_build_stable());
        state.set_can_build_stable();
        assert!(state.can_build_stable());
    }
}
