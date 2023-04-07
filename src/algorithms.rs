use crate::actions::Action;
use crate::game::State;
use rand::Rng;
use std::io;

const NUM_GAMES_TO_SIMULATE: usize = 10000;

#[derive(Clone, Hash)]
pub enum Kind {
    Human,
    RandomMachine,
    UniformMachine,
    //MCTSMachine,
}

impl Kind {
    fn play_human(state: &mut State, debug: bool) -> bool {
        let choices = state.last_action.next_choices(&state);

        if debug {
            state.display();
            Action::display_all(&choices);
        }

        if choices.len() == 1 {
            choices[0].display();
            choices[0].apply_choice(state);
            return true;
        }

        print!(
            "\nEnter an action index between 0 and {} inclusive : ",
            choices.len() - 1
        );

        let stdin = io::stdin();
        let mut user_input = String::new();
        let _res = stdin.read_line(&mut user_input);

        match user_input.trim().parse::<usize>() {
            Ok(input_int) => {
                println!("Your input [{input_int}]");

                if input_int >= choices.len() {
                    println!("Invalid action .. quitting");
                    return false;
                }

                if debug {
                    choices[input_int].display();
                }
                choices[input_int].apply_choice(state);
            }
            Err(_e) => return Self::play_human(state, debug), // parsing failed - try again
        }

        true
    }

    fn play_random(state: &mut State, debug: bool) -> bool {
        match state.last_action {
            Action::EndGame => return false,
            _ => (),
        }

        let choices = state.last_action.next_choices(&state);
        if debug {
            state.display();
            Action::display_all(&choices);
        }

        // Chose a random action
        let action_idx = rand::thread_rng().gen_range(0..choices.len());
        if debug {
            choices[action_idx].display();
        }
        choices[action_idx].apply_choice(state);
        true
    }

    #[allow(clippy::cast_precision_loss)]
    fn play_machine_uniform(state: &mut State, debug: bool) -> bool {
        match state.last_action {
            Action::EndGame => return false,
            _ => (),
        }

        let choices = state.last_action.next_choices(&state);

        if debug {
            state.display();
            Action::display_all(&choices);
        }

        if choices.len() == 1 {
            if debug {
                choices[0].display();
            }
            choices[0].apply_choice(state);
            return true;
        }

        // 1. Simulate n games from each action with each player replaced by a random move AI.
        // 2. Compute the average score for each action from the n playouts.
        // 3. Select the move that gives rise to the maximum average score.
        let mut best_action_idx: usize = 0;
        let mut best_average: f32 = f32::NEG_INFINITY;

        let num_games_per_action: usize = NUM_GAMES_TO_SIMULATE / choices.len();

        println!();
        // Play n simulated games for each action
        for (i, action) in choices.iter().enumerate() {
            let mut sum = 0;
            for _ in 0..num_games_per_action {
                // Clone the current game state
                let mut tmp_game = state.clone();
                // Play the current action
                action.apply_choice(&mut tmp_game);
                // Replace all players with a random move AI
                tmp_game.replace_all_players_with_random_bots();
                // Play the game out until the end
                tmp_game.play(false);
                // Sum resultant values
                sum += tmp_game.fitness()[state.current_player_idx];
            }
            let avg = sum as f32 / num_games_per_action as f32;

            print!(
                "\nAvg score from {} simulated playouts for Action {:?} is {}.",
                num_games_per_action, action, avg
            );

            if avg > best_average {
                best_average = avg;
                best_action_idx = i;
            }
        }
        if debug {
            choices[best_action_idx].display();
        }
        choices[best_action_idx].apply_choice(state);
        true
    }

    pub fn play(&self, state: &mut State, debug: bool) -> bool {
        match self {
            Kind::Human => Kind::play_human(state, debug),
            Kind::RandomMachine => Kind::play_random(state, debug),
            Kind::UniformMachine => Kind::play_machine_uniform(state, debug),
        }
    }
}
