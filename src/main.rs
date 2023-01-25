use rand::Rng;

mod farm;
mod state;

fn main() {
    let num_players: usize = 4;
    let first_player_idx = rand::thread_rng().gen_range(0..num_players);

    println!("First player is {}", first_player_idx);

    let mut state = state::get_init_state(first_player_idx, num_players);

    state.play_round();

    println!("-- NEXT ROUND ---");

    state.play_round();
}
