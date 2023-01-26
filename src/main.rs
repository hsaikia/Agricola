mod farm;
mod state;

fn main() {
    let num_players: usize = 4;
    let mut state = state::get_init_state(num_players);

    state.play_round();
    println!("-- NEXT ROUND ---");
    state.play_round();
}
