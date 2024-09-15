use agricola_game::agricola::{farm::FarmyardSpace, fencing::*};

fn main() {
    let mut farmyard_spaces = [FarmyardSpace::Empty; 15];
    farmyard_spaces[5] = FarmyardSpace::Room;
    farmyard_spaces[10] = FarmyardSpace::Room;

    let pasture_configs = get_all_pasture_configs(&farmyard_spaces);

    for pasture_config in pasture_configs.iter() {
        println!(
            "Pastures {:?} Wood {} Extensions {} Sizes {:?}",
            pasture_config.pastures,
            pasture_config.wood,
            pasture_config.extensions,
            pasture_sizes_from_hash(pasture_config.hash)
        );
    }
}
