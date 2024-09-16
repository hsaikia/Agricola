use std::collections::HashMap;

use agricola_game::agricola::{farm::FarmyardSpace, fencing::*};

fn test1() -> Vec<PastureConfig> {
    let mut farmyard_spaces = [FarmyardSpace::Empty; 15];
    farmyard_spaces[5] = FarmyardSpace::Room;
    farmyard_spaces[10] = FarmyardSpace::Room;

    get_all_pasture_configs(&farmyard_spaces)
}

fn test2() -> Vec<PastureConfig> {
    let mut farmyard_spaces = [FarmyardSpace::Empty; 15];
    farmyard_spaces[2] = FarmyardSpace::Field(None);
    farmyard_spaces[5] = FarmyardSpace::Room;
    farmyard_spaces[10] = FarmyardSpace::Room;

    farmyard_spaces[13] = FarmyardSpace::FencedPasture(None, false, 0);
    farmyard_spaces[14] = FarmyardSpace::FencedPasture(None, false, 1);

    get_all_pasture_configs(&farmyard_spaces)
}

fn main() {
    let pasture_configs = test2();
    let mut size_config_map: HashMap<u64, Vec<PastureConfig>> = HashMap::new();
    for pasture_config in pasture_configs.iter() {
        println!(
            "Pastures {:?} Wood {} Sizes {:?}",
            pasture_config.pastures,
            pasture_config.wood,
            pasture_sizes_from_hash(pasture_config.hash)
        );
        size_config_map
            .entry(pasture_config.hash)
            .or_default()
            .push(pasture_config.clone());
    }

    for (k, v) in size_config_map.iter() {
        println!(
            "Pasture size {:?} configs {}",
            pasture_sizes_from_hash(*k),
            v.len()
        );
    }
}
