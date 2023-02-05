#[macro_use]
extern crate lazy_static;

mod farm;
mod game;
mod major_improvements;
mod player;
mod primitives;
mod setup;
// mod fencing;

fn main() {
    let num_players: usize = 4;
    let mut game = setup::get_init_state(num_players);
    game.play();
    // fencing::test_fencing(15);
}
