use super::{actions::Action, state::State};

const MAX_DEPTH: u8 = 4;

pub fn search(
    state: &State,
    player_idx: usize,
    depth: u8,
    alpha: &mut f64,
    beta: &mut f64,
    num_seen: &mut usize,
) -> f64 {
    if depth == MAX_DEPTH {
        return state.scores()[player_idx];
    }

    let actions = Action::next_choices(state);
    if actions.is_empty() {
        return state.scores()[player_idx];
    }

    if actions.len() == 1 {
        let mut state_tmp = state.clone();
        actions[0].0.apply_choice(&mut state_tmp);
        *num_seen += 1;
        return search(&state_tmp, player_idx, depth, alpha, beta, num_seen);
    }

    if state.current_player_idx == player_idx {
        let mut best: f64 = -100_000.0;
        for (action, _) in &actions {
            let mut state_tmp = state.clone();
            action.apply_choice(&mut state_tmp);
            *num_seen += 1;
            let v = search(&state_tmp, player_idx, depth + 1, alpha, beta, num_seen);
            if best < v {
                best = v;
            }
            if best >= *beta {
                break;
            }

            *alpha = best.max(*alpha);
        }
        best
    } else {
        let mut best: f64 = 100_000.0;
        for (action, _) in &actions {
            let mut state_tmp = state.clone();
            action.apply_choice(&mut state_tmp);
            *num_seen += 1;
            let v = search(&state_tmp, player_idx, depth + 1, alpha, beta, num_seen);
            if best > v {
                best = v;
            }
            if best <= *alpha {
                break;
            }

            *beta = best.min(*beta);
        }
        best
    }
}

#[must_use]
pub fn best_move(state: &State) -> Option<Action> {
    let actions = Action::next_choices(state);
    if actions.is_empty() {
        println!("GAME OVER");
        return None;
    }

    if actions.len() == 1 {
        return Some(actions[0].0.clone());
    }

    let player_idx = state.current_player_idx;

    let mut best_action: Option<Action> = None;
    let mut best: f64 = -100_000.0;
    let mut num_seen: usize = 0;

    for (action, _) in &actions {
        let mut state_tmp = state.clone();
        action.apply_choice(&mut state_tmp);
        let v = search(
            &state_tmp,
            player_idx,
            0,
            &mut -100_000.0,
            &mut 100_000.0,
            &mut num_seen,
        );
        if best < v {
            best = v;
            best_action = Some(action.clone());
        }
    }

    println!("Player {player_idx} chooses Action {best_action:?}. Position searched {num_seen}");

    best_action
}
