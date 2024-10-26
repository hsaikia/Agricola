use crate::agricola::fencing::{is_future_extension, remove_farmyard_idx};

use super::action_space::{
    accumulate, randomize_action_spaces, ACCUMULATION_SPACE_INDICES, ACTION_SPACE_NAMES,
    NUM_ACTION_SPACES, OPEN_SPACES,
};
use super::actions::Action;
use super::algorithms::PlayerType;
use super::card::{
    cost, points, Card, ClayOven, CookingHearth1, CookingHearth2, Fireplace1, Fireplace2,
    StoneOven, Well, BAKING_IMPROVEMENTS_INDICES, CARD_NAMES, COOKING_HEARTH_INDICES,
    COOKING_IMPROVEMENTS_INDICES, FIREPLACE_INDICES, MAJOR_IMPROVEMENTS_INDICES, NUM_CARDS,
    OCCUPATIONS_INDICES,
};
use super::display::format_resources;
use super::farm::{Farm, Seed};
use super::fencing::{get_all_pasture_configs, PastureConfig};
use super::flag::{
    BakedOnceWithClayOven, BakedOnceWithStoneOven, BakedTwiceWithStoneOven, BeforeRoundStart,
    CanBuildRoom, CanBuildStable, CanRenovate, ClayHouse, Flag, HarvestPaid, HasCookingImprovement,
    HasRoomToGrow, StoneHouse, UsedBasketmakersWorkshop, UsedJoinery, UsedPottery, WoodHouse,
    NUM_FLAGS,
};
use super::quantity::{
    can_pay_for_resource, pay_for_resource, AdultMembers, BeggingTokens, Boar, Cattle, Children,
    Clay, Food, Grain, MembersPlacedThisRound, Quantities, Quantity, Reed, ResourceExchange,
    Resources, Rooms, Sheep, Stone, Vegetable, Wood, NUM_QUANTITIES, NUM_RESOURCES,
};
use super::scoring::score_farm;
use core::panic;
use derivative::Derivative;
use rand::Rng;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub const MAX_NUM_PLAYERS: usize = 4;
const MAX_FAMILY_MEMBERS: usize = 5;
const EPSILON: f64 = 1e-6;

#[derive(Clone, Hash, Debug)]
pub struct Event {
    pub round: usize,
    pub player_idx: usize,
    pub func: fn(res: &mut [usize; NUM_QUANTITIES]),
}

#[derive(Clone, Derivative, Hash)]
pub struct State {
    pub num_players: usize,
    pub current_round: usize,
    pub accumulated_resources: [Resources; NUM_ACTION_SPACES], // Only accumulation spaces are used
    pub action_spaces: [usize; NUM_ACTION_SPACES],
    pub occupied: [bool; NUM_ACTION_SPACES],
    pub harvest_done: bool,
    player_types: [PlayerType; MAX_NUM_PLAYERS],
    player_quantities: [[usize; NUM_QUANTITIES]; MAX_NUM_PLAYERS],
    player_flags: [[bool; NUM_FLAGS]; MAX_NUM_PLAYERS],
    player_cards: [[bool; NUM_CARDS]; MAX_NUM_PLAYERS],
    farms: [Farm; MAX_NUM_PLAYERS],
    pub current_player_idx: usize,
    pub starting_player_idx: usize,
    pub people_placed_this_round: usize,
    pub last_action: Action,
    pub start_round_events: Vec<Event>,
    #[derivative(Hash = "ignore")]
    pub fence_options_cache: [Vec<PastureConfig>; MAX_NUM_PLAYERS],
}

impl State {
    /// # Panics
    /// Will panic if initialization fails
    #[must_use]
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

        let mut player_types = [PlayerType::MctsAI; MAX_NUM_PLAYERS];
        for (i, player_type) in player_types.iter_mut().enumerate().take(players.len()) {
            *player_type = players[i];
        }

        let farm = Farm::new();
        let all_fence_options = get_all_pasture_configs(&farm.farmyard_spaces);

        let state = State {
            num_players: players.len(),
            current_round: 0,
            accumulated_resources: [[0; NUM_RESOURCES]; NUM_ACTION_SPACES],
            action_spaces: (0..NUM_ACTION_SPACES)
                .collect::<Vec<usize>>()
                .try_into()
                .unwrap(),
            occupied: [false; NUM_ACTION_SPACES],
            harvest_done: false,
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
            fence_options_cache: core::array::from_fn(|_| all_fence_options.clone()),
        };
        Some(state)
    }

    pub fn add_action(&mut self, action: &Action) {
        // Set space to occupied of action corresponds to an action space
        if action.action_idx() < NUM_ACTION_SPACES {
            // This only works because the indices of actions and action spaces are the same.
            // TODO : Fix this duplication
            self.occupied[action.action_idx()] = true;
        }

        // Add action to the sequence of actions taken by the current player
        self.last_action = action.clone();
    }

    #[must_use]
    pub fn harvest_paid(&self) -> bool {
        self.current_player_flags()[HarvestPaid.index()]
    }

    #[must_use]
    pub fn can_init_new_round(&self) -> bool {
        // If all stages are done
        if OPEN_SPACES + self.current_round == NUM_ACTION_SPACES {
            return false;
        }

        // If stages left or last stage, but harvest is yet to be completed
        if (self.current_round == 4
            || self.current_round == 7
            || self.current_round == 9
            || self.current_round == 11
            || self.current_round == 13
            || self.current_round == 14)
            && !self.harvest_done
        {
            return false;
        }
        true
    }

    // When all rounds in the previous stage are played - it is time for harvest
    #[must_use]
    pub fn is_harvest(&self) -> bool {
        (self.current_round == 4
            || self.current_round == 7
            || self.current_round == 9
            || self.current_round == 11
            || self.current_round == 13
            || self.current_round == 14)
            && !self.harvest_done
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

    #[must_use]
    pub fn family_members(&self, player_idx: usize) -> usize {
        self.player_quantities(player_idx)[AdultMembers.index()]
            + self.player_quantities(player_idx)[Children.index()]
    }

    fn food_required(&self) -> usize {
        2 * self.current_player_quantities()[AdultMembers.index()]
            + self.current_player_quantities()[Children.index()]
    }

    #[must_use]
    pub fn got_enough_food(&self) -> bool {
        2 * self.current_player_quantities()[AdultMembers.index()]
            + self.current_player_quantities()[Children.index()]
            <= self.current_player_quantities()[Food.index()]
    }

    #[must_use]
    pub fn can_grow_family_with_room(&self) -> bool {
        self.family_members(self.current_player_idx) < MAX_FAMILY_MEMBERS
            && self.current_player_flags()[HasRoomToGrow.index()]
    }

    /// # Panics
    /// Will panic if the family cannot grow without room
    pub fn grow_family_with_room(&mut self) {
        assert!(self.can_grow_family_with_room());
        self.current_player_quantities_mut()[Children.index()] += 1;
        if self.family_members(self.current_player_idx)
            >= self.current_player_quantities()[Rooms.index()]
        {
            self.current_player_flags_mut()[HasRoomToGrow.index()] = false;
        }
    }

    #[must_use]
    pub fn get_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        s.finish()
    }

    #[must_use]
    pub fn player_type(&self, player_idx: usize) -> PlayerType {
        self.player_types[player_idx]
    }

    /// # Panics
    /// Will panic if partial comparison fails
    #[must_use]
    pub fn fitness(&self) -> [f64; MAX_NUM_PLAYERS] {
        let scores = self.scores();

        if scores.len() == 1 {
            return scores;
        }

        let mut fitness = scores;
        let mut sorted_scores = scores;

        // Sort in decreasing order
        sorted_scores.sort_by(|a, b| b.partial_cmp(a).unwrap());

        // Fitness of winner is defined by the margin of victory = so difference from own score and second best score
        // Fitness of losers are defined by the margin of defeat = so difference from own score and best score
        for f in &mut fitness {
            if (*f - sorted_scores[0]).abs() < EPSILON {
                // winner
                *f -= sorted_scores[1];
            } else {
                *f -= sorted_scores[0];
            }
        }
        fitness
    }

    #[allow(clippy::cast_possible_wrap)]
    fn score_majors(&self) -> [i32; MAX_NUM_PLAYERS] {
        let mut ret: [i32; MAX_NUM_PLAYERS] = [0; MAX_NUM_PLAYERS];
        for (i, val) in ret.iter_mut().enumerate().take(self.num_players) {
            for major_idx in MAJOR_IMPROVEMENTS_INDICES {
                if self.player_cards[i][major_idx] {
                    *val += points(major_idx, self.player_quantities(i)) as i32;
                }
            }
        }
        ret
    }

    #[must_use]
    pub fn scores(&self) -> [f64; MAX_NUM_PLAYERS] {
        let mut scores: [f64; MAX_NUM_PLAYERS] = [0.0; MAX_NUM_PLAYERS];
        let card_scores = self.score_majors();
        for (idx, card_score) in card_scores.iter().enumerate().take(self.num_players) {
            scores[idx] = self.score(idx) + f64::from(*card_score);
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

    /// # Panics
    ///
    /// Will panic if a new round cannot be initialized
    pub fn init_new_round(&mut self) {
        assert!(self.can_init_new_round());
        self.current_round += 1;
        randomize_action_spaces(&mut self.action_spaces, self.current_round);

        // Reset workers
        self.reset_for_next_round();
        self.people_placed_this_round = 0;

        self.occupied = [false; NUM_ACTION_SPACES];

        // Update accumulation spaces
        for i in 0..OPEN_SPACES + self.current_round {
            let idx = self.action_spaces[i];
            accumulate(idx, &mut self.accumulated_resources[idx]);
        }

        // Delete old events
        self.start_round_events
            .retain(|e| e.round >= self.current_round);

        // Reset start player
        self.current_player_idx = self.starting_player_idx;

        // Look for start round events
        let start_round_events = self.start_round_events.clone();
        for event in start_round_events {
            for i in 0..self.num_players {
                if event.round == self.current_round && event.player_idx == i {
                    (event.func)(self.player_quantities_mut(i));
                }
            }
        }

        // Reset harvest flag
        self.harvest_done = false;
    }

    pub fn add_new_field(&mut self, idx: &usize) {
        self.current_farm_mut().add_field(*idx);
        remove_farmyard_idx(&mut self.fence_options_cache[self.current_player_idx], *idx);
    }

    #[must_use]
    pub fn field_options(&self) -> Vec<usize> {
        self.current_farm().possible_field_positions()
    }

    fn is_major_played(&self, major_idx: usize) -> bool {
        for i in 0..self.num_players {
            if self.player_cards[i][major_idx] {
                return true;
            }
        }
        false
    }

    #[must_use]
    pub fn available_majors_to_build(&self) -> [bool; NUM_CARDS] {
        let mut available: [bool; NUM_CARDS] = [false; NUM_CARDS];

        for major_idx in MAJOR_IMPROVEMENTS_INDICES {
            if self.is_major_played(major_idx) {
                continue;
            }
            if (major_idx == CookingHearth1.index() || major_idx == CookingHearth2.index())
                && (self.current_player_cards()[Fireplace1.index()]
                    || self.current_player_cards()[Fireplace2.index()])
            {
                available[major_idx] = true;
                continue;
            }
            if can_pay_for_resource(&cost(major_idx), self.current_player_quantities()) {
                available[major_idx] = true;
            }
        }

        available
    }

    pub fn replace_fireplace_with_cooking_hearth(&mut self) {
        if self.current_player_cards()[Fireplace1.index()] {
            self.current_player_cards_mut()[Fireplace1.index()] = false;
            if !self.is_major_played(CookingHearth1.index()) {
                self.current_player_cards_mut()[CookingHearth1.index()] = true;
            } else if !self.is_major_played(CookingHearth2.index()) {
                self.current_player_cards_mut()[CookingHearth2.index()] = true;
            }
        } else if self.current_player_cards()[Fireplace2.index()] {
            self.current_player_cards_mut()[Fireplace2.index()] = false;
            if !self.is_major_played(CookingHearth1.index()) {
                self.current_player_cards_mut()[CookingHearth1.index()] = true;
            } else if !self.is_major_played(CookingHearth2.index()) {
                self.current_player_cards_mut()[CookingHearth2.index()] = true;
            }
        }
    }

    pub fn room_material_idx(&self, player_idx: usize) -> usize {
        if self.player_flags(player_idx)[WoodHouse.index()] {
            Wood.index()
        } else if self.player_flags(player_idx)[ClayHouse.index()] {
            Clay.index()
        } else if self.player_flags(player_idx)[StoneHouse.index()] {
            Stone.index()
        } else {
            panic!("No house type set");
        }
    }

    pub fn set_can_build_room(&mut self) {
        self.current_player_flags_mut()[CanBuildRoom.index()] =
            !self.current_farm().possible_room_positions().is_empty()
                && self.current_player_quantities()
                    [self.room_material_idx(self.current_player_idx)]
                    >= 5
                && self.current_player_quantities()[Reed.index()] >= 2;
    }

    #[must_use]
    pub fn can_build_room(&self) -> bool {
        self.current_player_flags()[CanBuildRoom.index()]
    }

    /// Builds a single room
    /// # Panics
    /// Will panic if the player cannot build a room
    pub fn build_room(&mut self, idx: &usize) {
        assert!(self.can_build_room());
        // By default Rooms cost 5 of the corresponding building resource (as the material of the house and 2 Reed)
        let room_material_idx = self.room_material_idx(self.current_player_idx);
        self.current_player_quantities_mut()[room_material_idx] -= 5;
        self.current_player_quantities_mut()[Reed.index()] -= 2;
        self.current_farm_mut().build_room(*idx);
        remove_farmyard_idx(&mut self.fence_options_cache[self.current_player_idx], *idx);

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

        // Can Build Stable flag is perhaps dirty, set it
        self.set_can_build_stable();
    }

    #[must_use]
    pub fn room_options(&self) -> Vec<usize> {
        if self.can_build_room() {
            return self.current_farm().possible_room_positions();
        }
        Vec::new()
    }

    pub fn set_can_build_stable(&mut self) {
        self.current_player_flags_mut()[CanBuildStable.index()] =
            self.current_player_quantities()[Wood.index()] >= 2
                && self.current_farm().can_build_stable();
    }

    #[must_use]
    pub fn can_build_stable(&self) -> bool {
        self.current_player_flags()[CanBuildStable.index()]
    }

    /// Builds a single stable
    /// # Panics
    /// Will panic if the player cannot build a stable
    pub fn build_stable(&mut self, idx: &usize) {
        assert!(self.can_build_stable());
        self.current_player_quantities_mut()[Wood.index()] -= 2;
        self.current_farm_mut().build_stable(*idx);
        // Set flags
        self.set_can_build_stable();
        self.set_can_build_room();
    }

    #[must_use]
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

    #[must_use]
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

    /// # Panics
    /// Will panic if the player cannot renovate
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

    pub fn build_major(&mut self, major_idx: usize, return_fireplace: bool) {
        if COOKING_IMPROVEMENTS_INDICES.contains(&major_idx) {
            self.current_player_flags_mut()[HasCookingImprovement.index()] = true;
        }

        if return_fireplace
            && (major_idx == CookingHearth1.index() || major_idx == CookingHearth2.index())
        {
            self.replace_fireplace_with_cooking_hearth();
        } else {
            pay_for_resource(&cost(major_idx), self.current_player_quantities_mut());
            self.current_player_cards_mut()[major_idx] = true;

            if major_idx == Well.index() {
                let func = |res: &mut Quantities| {
                    res[Food.index()] += 1;
                };
                for i in 1..=5 {
                    self.start_round_events.push(Event {
                        round: self.current_round + i,
                        player_idx: self.current_player_idx,
                        func,
                    });
                }
            }
        }
    }

    #[must_use]
    pub fn fencing_choices(&self) -> Vec<PastureConfig> {
        self.current_farm().fencing_options(
            &self.fence_options_cache[self.current_player_idx],
            self.current_player_quantities()[Wood.index()],
        )
    }

    /// # Panics
    /// Will panic if the player cannot fence
    pub fn fence(&mut self, pasture_config: &PastureConfig) {
        assert!(self.can_fence());
        let mut wood = self.current_player_quantities()[Wood.index()];
        self.current_farm_mut()
            .fence_spaces(pasture_config, &mut wood);
        self.current_player_quantities_mut()[Wood.index()] = wood;
        self.fence_options_cache[self.current_player_idx]
            .retain(|x| is_future_extension(&x.pastures, &pasture_config.pastures));
        // Set flags
        self.set_can_build_stable();
        self.set_can_build_room();
    }

    #[must_use]
    pub fn can_fence(&self) -> bool {
        !self
            .current_farm()
            .fencing_options(
                &self.fence_options_cache[self.current_player_idx],
                self.current_player_quantities()[Wood.index()],
            )
            .is_empty()
    }

    #[must_use]
    pub fn can_use_exchange(&self, res_ex: &ResourceExchange) -> bool {
        self.current_player_quantities()[res_ex.from] >= res_ex.num_from
    }

    /// # Panics
    /// Will panic if the player cannot use the exchange
    pub fn use_exchange(&mut self, res_ex: &ResourceExchange) {
        assert!(self.can_use_exchange(res_ex));
        self.current_player_quantities_mut()[res_ex.from] -= res_ex.num_from;
        self.current_player_quantities_mut()[res_ex.to] += res_ex.num_to;
    }

    #[must_use]
    pub fn has_resources_to_cook(&self) -> bool {
        self.current_player_quantities()[Sheep.index()]
            + self.current_player_quantities()[Boar.index()]
            + self.current_player_quantities()[Cattle.index()]
            + self.current_player_quantities()[Vegetable.index()]
            > 0
    }

    #[must_use]
    pub fn can_bake_bread(&self, player_idx: usize) -> bool {
        // Check if any of the baking improvements are present
        // And at least one grain in supply
        BAKING_IMPROVEMENTS_INDICES
            .iter()
            .any(|&i| self.player_cards[player_idx][i])
            && self.current_player_quantities()[Grain.index()] > 0
    }

    pub fn sow_field(&mut self, seed: &Seed) {
        self.current_farm_mut().sow_field(seed);
        match seed {
            Seed::Grain => self.current_player_quantities_mut()[Grain.index()] -= 1,
            Seed::Vegetable => self.current_player_quantities_mut()[Vegetable.index()] -= 1,
        }
    }

    #[must_use]
    pub fn can_sow(&self) -> bool {
        (self.current_player_quantities()[Grain.index()] > 0
            || self.current_player_quantities()[Vegetable.index()] > 0)
            && self.current_farm().can_sow()
    }

    /// # Panics
    /// Will panic if the player cannot bake
    pub fn bake_bread(&mut self, num_grain_to_bake: usize) {
        assert!(self.can_bake_bread(self.current_player_idx));
        let mut num_grain_to_bake = num_grain_to_bake;
        while num_grain_to_bake > 0 {
            if self.current_player_quantities()[Grain.index()] == 0 {
                break;
            }

            if self.current_player_cards()[ClayOven.index()]
                && !self.current_player_flags()[BakedOnceWithClayOven.index()]
            {
                self.current_player_quantities_mut()[Food.index()] += 5;
                self.current_player_flags_mut()[BakedOnceWithClayOven.index()] = true;
            } else if self.current_player_cards()[StoneOven.index()]
                && !self.current_player_flags()[BakedOnceWithStoneOven.index()]
            {
                self.current_player_quantities_mut()[Food.index()] += 4;
                self.current_player_flags_mut()[BakedOnceWithStoneOven.index()] = true;
            } else if self.current_player_cards()[StoneOven.index()]
                && !self.current_player_flags()[BakedTwiceWithStoneOven.index()]
            {
                self.current_player_quantities_mut()[Food.index()] += 4;
                self.current_player_flags_mut()[BakedTwiceWithStoneOven.index()] = true;
            } else if self.current_player_cards()[CookingHearth1.index()]
                || self.current_player_cards()[CookingHearth2.index()]
            {
                self.current_player_quantities_mut()[Food.index()] += 3;
            } else if self.current_player_cards()[Fireplace1.index()]
                || self.current_player_cards()[Fireplace2.index()]
            {
                self.current_player_quantities_mut()[Food.index()] += 2;
            }

            self.current_player_quantities_mut()[Grain.index()] -= 1;
            num_grain_to_bake -= 1;
        }

        self.current_player_flags_mut()[BakedOnceWithClayOven.index()] = false;
        self.current_player_flags_mut()[BakedOnceWithStoneOven.index()] = false;
        self.current_player_flags_mut()[BakedTwiceWithStoneOven.index()] = false;
    }

    #[must_use]
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
        self.current_player_flags_mut()[UsedJoinery.index()] = false;
        self.current_player_flags_mut()[UsedPottery.index()] = false;
        self.current_player_flags_mut()[UsedBasketmakersWorkshop.index()] = false;

        if food_required > self.current_player_quantities()[Food.index()] {
            self.current_player_quantities_mut()[BeggingTokens.index()] +=
                food_required - self.current_player_quantities()[Food.index()];
            self.current_player_quantities_mut()[Food.index()] = 0;
        } else {
            self.current_player_quantities_mut()[Food.index()] -= food_required;
        }

        self.accommodate_animals(true);
        self.current_player_idx = (self.current_player_idx + 1) % self.num_players;

        // When all players have paid for harvest, set the global flag to true
        if (0..self.num_players).all(|i| self.player_flags(i)[HarvestPaid.index()]) {
            self.harvest_done = true;
        }
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
            if self.player_cards[self.current_player_idx][ch_idx] {
                quantites[Food.index()] += 2 * leftover[0] + 3 * leftover[1] + 4 * leftover[2];
                owns_ch = true;
                break;
            }
        }

        if !owns_ch {
            for fp_idx in FIREPLACE_INDICES {
                if self.player_cards[self.current_player_idx][fp_idx] {
                    quantites[Food.index()] += 2 * leftover[0] + 2 * leftover[1] + 3 * leftover[2];
                    break;
                }
            }
        }

        *self.current_player_quantities_mut() = quantites;
    }

    #[must_use]
    pub fn total_workers(&self) -> usize {
        self.player_quantities
            .iter()
            .map(|q| q[AdultMembers.index()])
            .sum()
    }

    #[must_use]
    pub fn before_round_start(&self) -> bool {
        self.current_player_flags()[BeforeRoundStart.index()]
    }

    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn score(&self, player_idx: usize) -> f64 {
        let mut ret: f64 = 0.0;
        // House, Family and Empty Spaces
        ret += 3.0 * self.family_members(player_idx) as f64;
        // Begging Tokens
        ret -= 3.0 * self.player_quantities(player_idx)[BeggingTokens.index()] as f64;
        // Score farm
        ret += f64::from(score_farm(self, player_idx));

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

    /// # Panics
    /// Will panic if the assertion fails
    pub fn grow_family_without_room(&mut self) {
        assert!(self.can_grow_family_without_room());
        self.current_player_quantities_mut()[Children.index()] += 1;
    }

    #[must_use]
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

    #[must_use]
    pub fn num_occupations_played(&self) -> usize {
        OCCUPATIONS_INDICES.iter().fold(0, |acc, idx| {
            if self.player_cards[self.current_player_idx][*idx] {
                acc + 1
            } else {
                acc
            }
        })
    }

    #[must_use]
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

    /// # Panics
    /// Will panic if a card has a wrong player idx
    #[must_use]
    pub fn format(&self) -> String {
        let mut ret: String = String::new();

        for i in (0..NUM_ACTION_SPACES).take(OPEN_SPACES + self.current_round) {
            let idx = self.action_spaces[i];
            if self.occupied[idx] {
                ret.push_str(&format!("\n[X] {}", ACTION_SPACE_NAMES[idx]));
            } else {
                ret.push_str(&format!("\n[-] {}", ACTION_SPACE_NAMES[idx]));
                if ACCUMULATION_SPACE_INDICES.contains(&idx) {
                    ret.push_str(&format_resources(&self.accumulated_resources[idx]));
                }
            }
        }

        ret.push_str("\n\n=== Cards ===\n");

        for (card_idx, card_name) in CARD_NAMES.iter().enumerate() {
            if self.card_available(card_idx) {
                ret.push_str(&format!("\n[-] {card_name}"));
            } else {
                let owner_idx = (0..self.num_players)
                    .find(|i| self.player_cards[*i][card_idx])
                    .unwrap();
                ret.push_str(&format!(
                    "\n[X] {} is owned by Player {}",
                    card_name,
                    owner_idx + 1
                ));
            }
        }

        ret
    }

    #[must_use]
    pub fn card_available(&self, card_idx: usize) -> bool {
        (0..self.num_players).all(|i| !self.player_cards[i][card_idx])
    }

    #[must_use]
    pub fn occupations_available(&self) -> Vec<usize> {
        OCCUPATIONS_INDICES
            .iter()
            .filter(|idx| self.card_available(**idx))
            .copied()
            .collect()
    }

    #[must_use]
    pub fn player_quantities(&self, player_idx: usize) -> &[usize; NUM_QUANTITIES] {
        &self.player_quantities[player_idx]
    }

    pub fn player_quantities_mut(&mut self, player_idx: usize) -> &mut [usize; NUM_QUANTITIES] {
        &mut self.player_quantities[player_idx]
    }

    #[must_use]
    pub fn player_flags(&self, player_idx: usize) -> &[bool; NUM_FLAGS] {
        &self.player_flags[player_idx]
    }

    pub fn player_flags_mut(&mut self, player_idx: usize) -> &mut [bool; NUM_FLAGS] {
        &mut self.player_flags[player_idx]
    }

    #[must_use]
    pub fn current_player_quantities(&self) -> &[usize; NUM_QUANTITIES] {
        &self.player_quantities[self.current_player_idx]
    }

    pub fn current_player_quantities_mut(&mut self) -> &mut [usize; NUM_QUANTITIES] {
        &mut self.player_quantities[self.current_player_idx]
    }

    #[must_use]
    pub fn current_player_flags(&self) -> &[bool; NUM_FLAGS] {
        &self.player_flags[self.current_player_idx]
    }

    pub fn current_player_flags_mut(&mut self) -> &mut [bool; NUM_FLAGS] {
        &mut self.player_flags[self.current_player_idx]
    }

    #[must_use]
    pub fn player_cards(&self, player_idx: usize) -> &[bool; NUM_CARDS] {
        &self.player_cards[player_idx]
    }

    pub fn player_cards_mut(&mut self, player_idx: usize) -> &mut [bool; NUM_CARDS] {
        &mut self.player_cards[player_idx]
    }

    #[must_use]
    pub fn current_player_cards(&self) -> &[bool; NUM_CARDS] {
        &self.player_cards[self.current_player_idx]
    }

    pub fn current_player_cards_mut(&mut self) -> &mut [bool; NUM_CARDS] {
        &mut self.player_cards[self.current_player_idx]
    }

    #[must_use]
    pub fn current_farm(&self) -> &Farm {
        self.player_farm(self.current_player_idx)
    }

    pub fn current_farm_mut(&mut self) -> &mut Farm {
        &mut self.farms[self.current_player_idx]
    }

    #[must_use]
    pub fn player_farm(&self, player_idx: usize) -> &Farm {
        &self.farms[player_idx]
    }

    pub fn play_weighted_random(&mut self, opt_depth: Option<usize>) {
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
                choices[0].0.apply_choice(self);
                continue;
            }

            d += 1;

            // Chose a random action
            let mut total_weight = 0.0;
            for (_, weight) in &choices {
                total_weight += weight;
            }

            let mut action_idx_weight = rand::thread_rng().gen_range(0.0..total_weight);
            for (action, weight) in &choices {
                if action_idx_weight < *weight {
                    action.apply_choice(self);
                    break;
                }
                action_idx_weight -= weight;
            }
        }
    }

    pub fn play_random(&mut self, opt_depth: Option<usize>) {
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
                choices[0].0.apply_choice(self);
                continue;
            }

            d += 1;

            // Chose a random action
            let action_idx = rand::thread_rng().gen_range(0..choices.len());
            choices[action_idx].0.apply_choice(self);
        }
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

        // There shouldbe 3 neighboring spots
        assert_eq!(room_positions.len(), 3);

        // Add resources to enable building one room
        state.current_player_quantities_mut()[Wood.index()] = 5;
        state.current_player_quantities_mut()[Reed.index()] = 2;

        // Flag isn't set, so room cannot be built yet
        assert!(!state.can_build_room());

        // Set the flag
        state.set_can_build_room();

        // Now room can be built
        assert!(state.can_build_room());

        // Flag isn't set, so stable cannot be built yet
        assert!(!state.can_build_stable());

        // Set the flag
        state.set_can_build_stable();

        // Now stable can be built
        assert!(state.can_build_stable());

        // Build two stable
        state.build_stable(&0);
        state.build_stable(&1);

        // Now there isn't enough wood to build another stable
        assert!(!state.can_build_stable());

        // Add more wood
        state.current_player_quantities_mut()[Wood.index()] = 10;
        // Set flag again
        state.set_can_build_stable();

        assert!(state.can_build_stable());

        // Build 2 more stables
        state.build_stable(&2);
        state.build_stable(&3);

        // Now there still is enough wood to build another stable but MAX_STABLES is reached
        assert!(!state.can_build_stable());
    }
}
