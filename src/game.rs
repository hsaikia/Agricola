use crate::major_improvements::{MajorImprovement, ALL_MAJORS};
use crate::player::{Kind, Player};
use crate::primitives::{
    new_res, pay_for_resource, print_resources, take_resource, Resource, Resources,
};
use crate::scoring;
use rand::Rng;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io;

const NUM_GAMES_TO_SIMULATE: usize = 10000;
const MCTS_EXPLORATION_PARAM: f32 = 2.0;

#[derive(Eq, PartialEq, Clone, Hash)]
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

lazy_static! {
    static ref ACTION_SPACE_NAMES: HashMap<ActionSpace, &'static str> = {
        let mut ret = HashMap::new();
        ret.insert(ActionSpace::Copse, "Copse");
        ret.insert(ActionSpace::Grove, "Grove");
        ret.insert(ActionSpace::Forest, "Forest");
        ret.insert(ActionSpace::ResourceMarket, "Resource Market");
        ret.insert(ActionSpace::Hollow, "Hollow");
        ret.insert(ActionSpace::ClayPit, "Clay Pit");
        ret.insert(ActionSpace::ReedBank, "Reed Bank");
        ret.insert(ActionSpace::TravelingPlayers, "Traveling Players");
        ret.insert(ActionSpace::Fishing, "Fishing");
        ret.insert(ActionSpace::DayLaborer, "Day Laborer");
        ret.insert(ActionSpace::GrainSeeds, "Grain Seeds");
        ret.insert(ActionSpace::MeetingPlace, "Meeting Place");
        ret.insert(ActionSpace::Farmland, "Farmland");
        ret.insert(ActionSpace::FarmExpansion, "Farm Expansion");
        ret.insert(ActionSpace::Lessons1, "Lessons1");
        ret.insert(ActionSpace::Lessons2, "Lessons2");
        ret.insert(ActionSpace::GrainUtilization, "Grain Utilization");
        ret.insert(ActionSpace::Fencing, "Fencing");
        ret.insert(ActionSpace::SheepMarket, "Sheep Market");
        ret.insert(ActionSpace::Improvements, "Improvements");
        ret.insert(ActionSpace::WesternQuarry, "Western Quarry");
        ret.insert(ActionSpace::WishForChildren, "Wish For Children");
        ret.insert(ActionSpace::HouseRedevelopment, "House Redevelopment");
        ret.insert(ActionSpace::PigMarket, "Pig Market");
        ret.insert(ActionSpace::VegetableSeeds, "Vegetable Seeds");
        ret.insert(ActionSpace::EasternQuarry, "Eastern Quarry");
        ret.insert(ActionSpace::CattleMarket, "Cattle Market");
        ret.insert(ActionSpace::Cultivation, "Cultivation");
        ret.insert(
            ActionSpace::UrgentWishForChildren,
            "Urgent Wish For Children",
        );
        ret.insert(ActionSpace::FarmRedevelopment, "Farm Redevelopment");
        ret
    };
}

#[derive(Clone, Hash)]
pub struct Space {
    action_space: ActionSpace,
    occupied: bool,
    accumulation_space: bool,
    resource_space: bool, // all accumulation spaces are also resource spaces
    resource: Resources,
}

impl Space {
    pub fn create_new(p_action_space: ActionSpace, p_resource: Option<Resources>) -> Space {
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
            action_space: p_action_space,
            occupied: false,
            accumulation_space: p_accumulation_space,
            resource_space: p_resource_space,
            resource: def_resource,
        }
    }
}

#[derive(Clone)]
struct GameState {
    average_fitness: Vec<f32>, // Wins for each player
    total_games: u32,
}

impl GameState {
    fn new(num_players: usize) -> Self {
        GameState {
            average_fitness: vec![0.0; num_players],
            total_games: 0,
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn add_fitness(&mut self, fitness: &Vec<i32>) {
        assert!(fitness.len() == self.average_fitness.len());

        for it in fitness.iter().zip(self.average_fitness.iter_mut()) {
            let (a, b) = it;
            *b = (*b * self.total_games as f32 + *a as f32) / (self.total_games + 1) as f32;
        }
        self.total_games += 1;
    }

    // Returns index of the node to traverse
    #[allow(clippy::cast_precision_loss)]
    fn choose_uct(
        player_to_play: usize,
        nodes: &Vec<u64>,
        mcts_cache: &HashMap<u64, GameState>,
    ) -> usize {
        assert!(!nodes.is_empty());
        // Use UCT formula to sample a child node
        let mut total: u32 = 0;
        let mut idx: usize = 0;
        let mut highest_uct: f32 = 0.0;
        let mut min_fitness: f32 = f32::INFINITY;
        let mut max_fitness: f32 = f32::NEG_INFINITY;

        for (i, node) in nodes.iter().enumerate() {
            // If child key isn't in the cache, it hasn't been explored - explore it immediately
            if !mcts_cache.contains_key(node) {
                return i;
            }
            let fitness = mcts_cache[node].average_fitness[player_to_play];
            min_fitness = fitness.min(min_fitness);
            max_fitness = fitness.max(max_fitness);
            total += mcts_cache[node].total_games;
        }

        for (i, node) in nodes.iter().enumerate() {
            let n: u32 = mcts_cache[node].total_games;
            // If any node has never been seen before - explore it
            if n == 0 {
                return i;
            }
            let fitness = mcts_cache[node].average_fitness[player_to_play];
            // UCT constant is 2
            assert!(total > 0);
            let p = (fitness - min_fitness) / (max_fitness - min_fitness)
                + f32::sqrt(MCTS_EXPLORATION_PARAM * total as f32 / n as f32);

            if p > highest_uct {
                highest_uct = p;
                idx = i;
            }
        }
        idx
    }
}

#[derive(Clone, Hash)]
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
    pub fn create_new(
        p_spaces: Vec<Space>,
        first_player_idx: usize,
        num_players: usize,
        human_player: bool,
        default_ai_id: usize,
    ) -> Game {
        let mut state = Game {
            spaces: p_spaces,
            major_improvements: [true; 10],
            players: Vec::<Player>::new(),
            current_player_idx: first_player_idx,
            starting_player_idx: first_player_idx,
            next_visible_idx: 16,
            people_placed_this_round: 0,
        };
        state.init_players(first_player_idx, num_players, human_player, default_ai_id);
        state
    }

    fn get_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        s.finish()
    }

    fn play_random(&mut self, debug: bool) -> bool {
        let maybe_actions = self.get_all_available_actions(debug);
        if debug {
            self.display();
        }
        if let Some(actions) = maybe_actions {
            // Chose a random action
            let action_idx = rand::thread_rng().gen_range(0..actions.len());
            self.apply_action(&actions[action_idx], debug);
            return true;
        }
        false
    }

    fn play_human(&mut self, debug: bool) -> bool {
        let maybe_actions = self.get_all_available_actions(debug);
        if debug {
            self.display();
        }
        if let Some(actions) = maybe_actions {
            print!("\n\nAvailable actions : ");
            for (i, e) in actions.iter().enumerate() {
                print!(
                    ", {}:{}",
                    i, &ACTION_SPACE_NAMES[&self.spaces[e[0]].action_space]
                );
                if e.len() > 1 {
                    print!("{e:?}");
                }
            }

            print!(
                "\nEnter an action index between 0 and {} inclusive : ",
                actions.len() - 1
            );

            let stdin = io::stdin();
            let mut user_input = String::new();
            let _res = stdin.read_line(&mut user_input);

            match user_input.trim().parse::<usize>() {
                Ok(input_int) => {
                    println!("Your input [{input_int}]");

                    if input_int >= actions.len() {
                        println!("Invalid action .. quitting");
                        return false;
                    }

                    self.apply_action(&actions[input_int], debug);
                }
                Err(_e) => return self.play_human(debug), // parsing failed - try again
            }

            return true;
        }
        false
    }

    #[allow(clippy::cast_precision_loss)]
    fn play_machine_uniform(&mut self, debug: bool) -> bool {
        let maybe_actions = self.get_all_available_actions(debug);
        if debug {
            self.display();
        }
        if let Some(actions) = maybe_actions {
            // 1. Simulate n games from each action with each player replaced by a random move AI.
            // 2. Compute the average score for each action from the n playouts.
            // 3. Select the move that gives rise to the maximum average score.
            let mut best_action_idx: usize = 0;
            let mut best_average: f32 = f32::NEG_INFINITY;

            let num_games_per_action: usize = NUM_GAMES_TO_SIMULATE / actions.len();

            println!();
            // Play n simulated games for each action
            for (i, action) in actions.iter().enumerate() {
                let mut sum = 0;
                for _ in 0..num_games_per_action {
                    // Clone the current game state
                    let mut tmp_game = self.clone();
                    // Play the current action
                    tmp_game.apply_action(action, false);
                    // Replace all players with a random move AI
                    tmp_game.replace_all_players_with_random_bots();
                    // Play the game out until the end
                    tmp_game.play(false);
                    // Sum resultant values
                    sum += tmp_game.fitness()[self.current_player_idx];
                }
                let avg = sum as f32 / num_games_per_action as f32;

                print!(
                    "\nAvg score from {} simulated playouts for Action {}{:?} is {}.",
                    num_games_per_action,
                    &ACTION_SPACE_NAMES[&self.spaces[action[0]].action_space],
                    action,
                    avg
                );

                if avg > best_average {
                    best_average = avg;
                    best_action_idx = i;
                }
            }
            self.apply_action(&actions[best_action_idx], debug);
            return true;
        }
        false
    }

    fn play_machine_mcts(&mut self, debug: bool) -> bool {
        let maybe_actions = self.get_all_available_actions(debug);
        if debug {
            self.display();
        }
        if let Some(actions) = maybe_actions {
            let mut action_hashes: Vec<u64> = vec![];
            // Add the child node hashes
            for action in &actions {
                let mut tmp_game = self.clone();
                tmp_game.apply_action(action, false);
                action_hashes.push(tmp_game.get_hash());
            }

            // Initialize cache
            let mut mcts_cache: HashMap<u64, GameState> = HashMap::new();

            for _g in 0..NUM_GAMES_TO_SIMULATE {
                let sample_idx =
                    GameState::choose_uct(self.current_player_idx, &action_hashes, &mcts_cache);
                let mut tmp_game = self.clone();
                tmp_game.apply_action(&actions[sample_idx], false);

                //print!("\nGame {}", g);
                let mut expand_node_hash = tmp_game.get_hash();
                let mut path: Vec<u64> = vec![expand_node_hash];

                // Expand along the tree
                while mcts_cache.contains_key(&expand_node_hash) {
                    // In Agricola - since the future board state changes randomly
                    // get_all_available_actions can return different actions for the same current game state.
                    let maybe_sub_actions = tmp_game.get_all_available_actions(false);
                    if let Some(sub_actions) = maybe_sub_actions {
                        // Generate all child hashes
                        let mut sub_action_hashes: Vec<u64> = vec![];
                        for sub_action in &sub_actions {
                            let mut tmp_game2 = tmp_game.clone();
                            tmp_game2.apply_action(sub_action, false);
                            sub_action_hashes.push(tmp_game2.get_hash());
                        }
                        let chosen_idx = GameState::choose_uct(
                            tmp_game.current_player_idx,
                            &sub_action_hashes,
                            &mcts_cache,
                        );
                        tmp_game.apply_action(&sub_actions[chosen_idx], false);
                        expand_node_hash = tmp_game.get_hash();
                        path.push(tmp_game.get_hash());
                    } else {
                        break;
                    }
                }
                // Add node to cache

                if let std::collections::hash_map::Entry::Vacant(e) =
                    mcts_cache.entry(expand_node_hash)
                {
                    e.insert(GameState::new(tmp_game.players.len()));
                }

                // Perform playout
                // Replace all players with a random move AI
                tmp_game.replace_all_players_with_random_bots();
                // Play the game out until the end
                tmp_game.play(false);
                // Calculate result and backpropagate to the root
                let res = tmp_game.fitness();
                for node in &path {
                    let mut state = mcts_cache[node].clone();
                    state.add_fitness(&res);
                    mcts_cache.insert(*node, state);
                }
            }

            print!("\nCache has {} entries", mcts_cache.len());

            // Choose the best move again according to UCT
            let mut best_action_idx: usize = 0;
            let mut best_fitness: f32 = f32::NEG_INFINITY;

            let mut total_games = 0;
            for (i, action) in actions.iter().enumerate() {
                let mut tmp_game = self.clone();
                tmp_game.apply_action(action, false);
                let action_hash = tmp_game.get_hash();

                let games: u32 = mcts_cache[&action_hash].total_games;
                total_games += games;
                let fitness: f32 =
                    mcts_cache[&action_hash].average_fitness[self.current_player_idx];
                if fitness > best_fitness {
                    best_fitness = fitness;
                    best_action_idx = i;
                }
                print!(
                    "\nFitness from {} simulated playouts for Action {}{:?} is {:.2}.",
                    games,
                    &ACTION_SPACE_NAMES[&self.spaces[action[0]].action_space],
                    action,
                    fitness
                );
            }
            print!("\nTotal Games simulated {total_games}.");
            self.apply_action(&actions[best_action_idx], debug);
            return true;
        }
        false
    }

    fn play_move(&mut self, debug: bool) -> bool {
        match self.player_kind() {
            Kind::RandomMachine => self.play_random(debug),
            Kind::Human => self.play_human(debug),
            Kind::UniformMachine => self.play_machine_uniform(debug),
            Kind::MCTSMachine => self.play_machine_mcts(debug),
        }
    }

    pub fn play(&mut self, debug: bool) {
        loop {
            let status = self.play_move(debug);
            if !status {
                break;
            }
        }
    }

    fn get_all_available_actions(&mut self, debug: bool) -> Option<Vec<Vec<usize>>> {
        if self.people_placed_this_round == 0 {
            if self.next_visible_idx == 20
                || self.next_visible_idx == 23
                || self.next_visible_idx == 25
                || self.next_visible_idx == 27
                || self.next_visible_idx == 29
                || self.next_visible_idx == 30
            {
                self.harvest(debug);
            }

            if self.next_visible_idx == 30 {
                if debug {
                    println!("\nGAME OVER!");
                }
                return None;
            }

            // Init a new round
            self.init_new_round(debug);
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

            return Some(self.available_place_worker_actions(debug));
        }

        self.people_placed_this_round = 0;
        self.get_all_available_actions(debug)
    }

    fn apply_action(&mut self, action_vec: &Vec<usize>, debug: bool) {
        // This function assumes that the action is available to the current player

        let action_idx = action_vec[0];

        let space = &mut self.spaces[action_idx];
        if debug {
            print!(
                "\nCurrent Player {} chooses action {} {action_vec:?}.",
                self.current_player_idx, &ACTION_SPACE_NAMES[&space.action_space]
            );
        }

        let player = &mut self.players[self.current_player_idx];

        if space.resource_space {
            // Assumes that the space is accessible and the current player can use this action.
            let res = &space.resource;
            take_resource(res, &mut player.resources);
            // Zero resources if space is an accumulation space
            if space.accumulation_space {
                space.resource = new_res();
            }
        }
        match space.action_space {
            // If animal accumulation space, re-org animals in the farm
            ActionSpace::SheepMarket | ActionSpace::PigMarket | ActionSpace::CattleMarket => {
                player.reorg_animals(false);
            }
            // TODO also implement playing minor improvement here
            ActionSpace::MeetingPlace => self.starting_player_idx = self.current_player_idx,
            ActionSpace::Farmland => player.add_new_field(),
            ActionSpace::FarmExpansion => {
                assert!(action_vec.len() == 3);
                // Build action_vec[1] rooms
                for _ in 0..action_vec[1] {
                    player.build_room();
                }
                // Build action_vec[2] rooms
                for _ in 0..action_vec[2] {
                    player.build_stable();
                }
            }
            // Since we assume that the actions are all accessible
            // no further checks are necessary
            ActionSpace::GrainUtilization => {
                player.sow();
                player.bake_bread();
            }
            ActionSpace::Improvements => {
                player.build_major(action_vec[1], &mut self.major_improvements, debug);
                // TODO add minors
            }
            ActionSpace::Fencing => {
                player.fence();
            }
            ActionSpace::HouseRedevelopment => {
                player.renovate();
                // If major improvement choice is present, build that major
                if action_vec.len() > 1 {
                    player.build_major(action_vec[1], &mut self.major_improvements, debug);
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
        // Set the space to occupied
        space.occupied = true;
        // Increment people placed by player
        player.increment_people_placed();
        self.people_placed_this_round += 1;
        // Move to the next player
        self.advance_turn();
    }

    fn available_place_worker_actions(&self, _debug: bool) -> Vec<Vec<usize>> {
        let player = &self.players[self.current_player_idx];
        let mut actions: Vec<Vec<usize>> = Vec::new();
        for i in 0..self.next_visible_idx {
            let mut additional_subsequent_choices: Vec<Vec<usize>> = Vec::new();
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
                ActionSpace::Lessons1 | ActionSpace::Lessons2 => continue, // TODO Not implemented - action not available
                ActionSpace::FarmExpansion => {
                    if !player.can_build_room() && !player.can_build_stable() {
                        continue;
                    }
                    additional_subsequent_choices = player.all_possible_room_stable_builds();
                    // if debug {
                    //     print!("FarmExpansion Choices {:?}", choices);
                    // }
                }
                ActionSpace::Improvements => {
                    additional_subsequent_choices = MajorImprovement::available_majors_to_build(
                        &player.major_cards,
                        &self.major_improvements,
                        &player.resources,
                    );
                    if additional_subsequent_choices.is_empty() {
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
                    // Check if after renovating there are enough resources left to play a minor
                    let mut res = player.resources; // Copy
                    pay_for_resource(&player.renovation_cost, &mut res);
                    additional_subsequent_choices = MajorImprovement::available_majors_to_build(
                        &player.major_cards,
                        &self.major_improvements,
                        &res,
                    );
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

            if additional_subsequent_choices.is_empty() {
                actions.push(vec![i]);
            } else {
                for choices in additional_subsequent_choices {
                    let mut all_choices = vec![i];
                    all_choices.extend(choices);
                    actions.push(all_choices);
                }
            }
        }
        actions
    }

    fn player_kind(&self) -> Kind {
        self.players[self.current_player_idx].kind()
    }

    fn replace_all_players_with_random_bots(&mut self) {
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

    fn advance_turn(&mut self) {
        self.current_player_idx = (self.current_player_idx + 1) % self.players.len();
    }

    fn harvest(&mut self, debug: bool) {
        for (pi, p) in self.players.iter_mut().enumerate() {
            if debug {
                print!("\nPlayer {pi} paying for harvest..");
            }
            p.harvest(debug);
        }
    }

    fn init_new_round(&mut self, debug: bool) {
        assert!(self.next_visible_idx < 30);

        // Shuffle
        // 16, 17, 18 .. 19
        // 20, 21 .. 22
        // 23 .. 24
        // 25 .. 26
        // 27 .. 28
        // 29

        if [16, 17, 18].contains(&self.next_visible_idx) {
            let random_idx = rand::thread_rng().gen_range(self.next_visible_idx..20);
            self.spaces.swap(random_idx, self.next_visible_idx);
        } else if [20, 21].contains(&self.next_visible_idx) {
            let random_idx = rand::thread_rng().gen_range(self.next_visible_idx..23);
            self.spaces.swap(random_idx, self.next_visible_idx);
        } else if self.next_visible_idx == 23 {
            let random_idx = rand::thread_rng().gen_range(self.next_visible_idx..25);
            self.spaces.swap(random_idx, self.next_visible_idx);
        } else if self.next_visible_idx == 25 {
            let random_idx = rand::thread_rng().gen_range(self.next_visible_idx..27);
            self.spaces.swap(random_idx, self.next_visible_idx);
        } else if self.next_visible_idx == 27 {
            let random_idx = rand::thread_rng().gen_range(self.next_visible_idx..29);
            self.spaces.swap(random_idx, self.next_visible_idx);
        }

        if debug {
            // Reveal the next action space
            println!(
                "\n\nRound {}. Action {} is revealed!",
                self.next_visible_idx - 15,
                &ACTION_SPACE_NAMES[&self.spaces[self.next_visible_idx].action_space]
            );
        }

        // Increment the next action space idx
        self.next_visible_idx += 1;

        // Set start player
        self.current_player_idx = self.starting_player_idx;

        // Reset workers
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
                ActionSpace::TravelingPlayers | ActionSpace::Fishing => {
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
                ActionSpace::WesternQuarry | ActionSpace::EasternQuarry => {
                    res[Resource::Stone] += 1;
                }
                _ => (),
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

    pub fn display(&self) {
        println!("\n\n-- Board --");
        for i in 0..self.next_visible_idx {
            let space = &self.spaces[i];
            if space.occupied {
                print!(
                    "\n[X] {} is occupied",
                    &ACTION_SPACE_NAMES[&space.action_space]
                );
            } else {
                print!("\n[-] {} ", &ACTION_SPACE_NAMES[&space.action_space]);
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
