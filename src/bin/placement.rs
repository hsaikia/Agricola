use agricola_game::agricola::{farm::Farm, fencing::get_all_pasture_configs};

fn main() {
    let mut farm = Farm::new();
    farm.add_field(14);

    // Find best room placement
    let room_positions = farm.possible_room_positions();
    println!("Room positions {:?}", room_positions);
    for room_position in room_positions {
        let mut tmp_farm = farm.clone();
        tmp_farm.build_room(room_position);

        let num_stable_positions = tmp_farm.possible_stable_positions().len();
        let num_field_positions = tmp_farm.possible_field_positions().len();
        let num_pasture_configs = get_all_pasture_configs(&tmp_farm.farmyard_spaces).len();

        println!(
            "R {} => S {} F {} P {}",
            room_position, num_stable_positions, num_field_positions, num_pasture_configs
        );
    }

    let stable_positions = farm.possible_stable_positions();
    println!("Stable positions {:?}", stable_positions);
    for stable_position in stable_positions {
        let mut tmp_farm = farm.clone();
        tmp_farm.build_stable(stable_position);

        let num_room_positions = tmp_farm.possible_room_positions().len();
        let num_field_positions = tmp_farm.possible_field_positions().len();
        let num_pasture_configs = get_all_pasture_configs(&tmp_farm.farmyard_spaces).len();

        println!(
            "S {} => R {} F {} P {}",
            stable_position, num_room_positions, num_field_positions, num_pasture_configs
        );
    }

    let field_positions = farm.possible_field_positions();
    println!("Field positions {:?}", field_positions);
    for field_position in field_positions {
        let mut tmp_farm = farm.clone();
        tmp_farm.add_field(field_position);

        let num_room_positions = tmp_farm.possible_room_positions().len();
        let num_stable_positions = tmp_farm.possible_stable_positions().len();
        let num_pasture_configs = get_all_pasture_configs(&tmp_farm.farmyard_spaces).len();

        println!(
            "F {} => R {} S {} P {}",
            field_position, num_room_positions, num_stable_positions, num_pasture_configs
        );
    }

    let pasture_configs = get_all_pasture_configs(&farm.farmyard_spaces);
    //println!("Pasture configs {:?}", pasture_configs);
    for pasture_config in pasture_configs {
        let mut tmp_farm = farm.clone();
        let mut wood = 15;
        tmp_farm.fence_spaces(&pasture_config, &mut wood);

        let num_room_positions = tmp_farm.possible_room_positions().len();
        let num_stable_positions = tmp_farm.possible_stable_positions().len();
        let num_field_positions = tmp_farm.possible_field_positions().len();

        println!(
            "P {:?} => R {} S {} F {}",
            pasture_config.pastures, num_room_positions, num_stable_positions, num_field_positions
        );
    }
}
