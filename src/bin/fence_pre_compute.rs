use std::collections::HashMap;

use agricola_game::agricola::{farm::FarmyardSpace, fencing::*};

fn test1() -> Vec<PastureConfig> {
    let mut farmyard_spaces = [FarmyardSpace::Empty; 15];
    farmyard_spaces[5] = FarmyardSpace::Room;
    farmyard_spaces[10] = FarmyardSpace::Room;

    let fences_available = 15;
    let fences_used = 0;

    get_all_pasture_configs(&farmyard_spaces, fences_available, fences_used)
}

fn test2() -> Vec<PastureConfig> {
    let mut farmyard_spaces = [FarmyardSpace::Empty; 15];
    farmyard_spaces[5] = FarmyardSpace::Room;
    farmyard_spaces[10] = FarmyardSpace::Room;
    farmyard_spaces[13] = FarmyardSpace::FencedPasture(None, false, 0);
    farmyard_spaces[14] = FarmyardSpace::FencedPasture(None, false, 1);

    let fences_available = 8;
    let fences_used = 7;

    get_all_pasture_configs(&farmyard_spaces, fences_available, fences_used)
}

fn main() {
    let pasture_configs = test1();
    let mut size_config_map : HashMap<u64, Vec<PastureConfig>> = HashMap::new();
    for pasture_config in pasture_configs.iter() {
        // println!(
        //     "Pastures {:?} Wood {} Sizes {:?}",
        //     pasture_config.pastures,
        //     pasture_config.wood,
        //     pasture_sizes_from_hash(pasture_config.hash)
        // );
        size_config_map.entry(pasture_config.hash).or_default().push(pasture_config.clone());
    }

    for (k, v) in size_config_map.iter() {
        println!("Pasture size {:?} configs {}", pasture_sizes_from_hash(*k), v.len());
    }
}
