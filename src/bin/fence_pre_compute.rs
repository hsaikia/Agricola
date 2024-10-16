use std::collections::HashMap;

use agricola_game::agricola::{
    farm::FarmyardSpace,
    fencing::{get_all_pasture_configs, pasture_sizes_from_hash, PastureConfig},
};

fn example_config() -> Vec<PastureConfig> {
    let mut farmyard_spaces = [FarmyardSpace::Empty; 15];

    farmyard_spaces[5] = FarmyardSpace::Room;
    farmyard_spaces[10] = FarmyardSpace::Room;

    get_all_pasture_configs(&farmyard_spaces)
}

fn main() {
    let pasture_configs = example_config();
    let mut size_config_map: HashMap<u64, Vec<PastureConfig>> = HashMap::new();
    for pasture_config in &pasture_configs {
        println!(
            "Pastures {:?} Wood {} Sizes {:?} Extensions {}",
            pasture_config.pastures,
            pasture_config.wood,
            pasture_sizes_from_hash(pasture_config.hash),
            pasture_config.extensions
        );
        size_config_map
            .entry(pasture_config.hash)
            .or_default()
            .push(pasture_config.clone());
    }

    for (k, v) in &size_config_map {
        println!(
            "Pasture size {:?} configs {}",
            pasture_sizes_from_hash(*k),
            v.len()
        );
    }
}
