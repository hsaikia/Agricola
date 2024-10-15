use agricola_game::agricola::{farm::Farm, fencing::get_all_pasture_configs};

fn main() {
    let mut farm = Farm::new();
    farm.build_room(6);
    farm.add_field(14);

    let possible_room_positions = farm.possible_room_positions();
    println!("Room positions {possible_room_positions:?}");

    for room_position in possible_room_positions {
        let mut tmp_farm = farm.clone();
        tmp_farm.build_room(room_position);

        let flexibility = tmp_farm.flexibility();
        println!("R {room_position} => {flexibility}");
    }

    let field_positions = farm.possible_field_positions();
    println!("Field positions {field_positions:?}");

    for field_position in field_positions {
        let mut tmp_farm = farm.clone();
        tmp_farm.add_field(field_position);

        let flexibility = tmp_farm.flexibility();
        println!("F {field_position} => {flexibility}");
    }

    let stable_positions = farm.possible_stable_positions();
    println!("Stable positions {stable_positions:?}");

    for stable_position in stable_positions {
        let mut tmp_farm = farm.clone();
        tmp_farm.build_stable(stable_position);

        let flexibility = tmp_farm.flexibility();
        println!("S {stable_position} => {flexibility}");
    }

    let pasture_configs = get_all_pasture_configs(&farm.farmyard_spaces);
    //println!("Pasture configs {:?}", pasture_configs);
    for pasture_config in pasture_configs {
        let mut tmp_farm = farm.clone();
        let mut wood = 15;
        tmp_farm.fence_spaces(&pasture_config, &mut wood);

        let flexibility = tmp_farm.flexibility();

        println!("P {:?} => {}", pasture_config.pastures, flexibility);
    }
}
