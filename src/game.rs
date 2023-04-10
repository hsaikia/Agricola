use crate::actions::{Action, NUM_RESOURCE_SPACES};
use crate::algorithms::Kind;
use crate::major_improvements::{Cheaper, MajorImprovement};
use crate::player::Player;
use crate::primitives::{pay_for_resource, print_resources, Resources};
use crate::scoring;
use rand::Rng;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const INITIAL_OPEN_SPACES: usize = 16;

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
}

impl State {
    pub fn create_new(num_players: usize, human_player: bool, default_ai_id: usize) -> State {
        let first_player_idx = rand::thread_rng().gen_range(0..num_players);
        let mut state = State {
            resource_map: Action::init_resource_map(),
            open_spaces: Action::initial_open_spaces().to_vec(),
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
        };
        state.init_players(first_player_idx, num_players, human_player, default_ai_id);
        state
    }

    pub fn current_round(&self) -> usize {
        self.open_spaces.len() - INITIAL_OPEN_SPACES
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

    pub fn replace_fireplace_with_cooking_hearth(&mut self, major: MajorImprovement) {
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

        self.major_improvements.retain(|x| x != &major);
        player.major_cards.retain(|x| x != &returned_fireplace);
        player.major_cards.push(major);
        self.major_improvements.push(returned_fireplace);
    }

    pub fn build_major(&mut self, major: MajorImprovement) {
        let player = &mut self.players[self.current_player_idx];
        pay_for_resource(&major.cost(), &mut player.resources);
        self.major_improvements.retain(|x| x != &major);
        player.major_cards.push(major);
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
