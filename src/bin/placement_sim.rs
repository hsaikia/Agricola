use agricola_game::agricola::{
    farm::{Farm, FarmyardSpace, L, MAX_FENCES, W},
    fencing::{get_all_pasture_configs, get_existing_pastures, MAX_PASTURES},
    scoring::{FIELD_SCORE, PASTURE_SCORE},
};
use rand::Rng;

const PASTURE_EMOJIS: [[&str; MAX_PASTURES]; 2] = [
    ["[p1]", "[p2]", "[p3]", "[p4]"],
    ["[P1]", "[P2]", "[P3]", "[P4]"],
];

fn print(farm: &Farm) -> String {
    let mut ret = String::from("\n\n\n");
    for i in 0..W {
        for j in 0..L {
            let idx = i * L + j;
            match farm.farmyard_spaces[idx] {
                FarmyardSpace::Empty => {
                    ret.push_str("[--]");
                }
                FarmyardSpace::Room => {
                    ret.push_str("[CR]");
                }
                FarmyardSpace::Field(_) => {
                    ret.push_str("[FF]");
                }
                FarmyardSpace::FencedPasture(stable, pasture_idx) => {
                    if stable {
                        ret.push_str(PASTURE_EMOJIS[1][pasture_idx]);
                    } else {
                        ret.push_str(PASTURE_EMOJIS[0][pasture_idx]);
                    }
                }
                FarmyardSpace::UnfencedStable => {
                    ret.push_str("[us]");
                }
            }
        }
        ret.push('\n');
    }

    ret
}

fn score(farm: &Farm) -> i32 {
    let mut score = 0;
    let mut fields = 0;

    let num_pastures = get_existing_pastures(&farm.farmyard_spaces)
        .iter()
        .filter(|p| !p.is_empty())
        .count();

    score += PASTURE_SCORE[num_pastures.min(PASTURE_SCORE.len() - 1)];

    for space in &farm.farmyard_spaces {
        match *space {
            FarmyardSpace::Field(_) => fields += 1,
            FarmyardSpace::FencedPasture(stable, _) => {
                if stable {
                    score += 1;
                }
            }
            FarmyardSpace::Room => score += 1, // Assuming a clay house
            FarmyardSpace::Empty => score -= 1,
            FarmyardSpace::UnfencedStable => (),
        }
    }

    score += FIELD_SCORE[fields.min(FIELD_SCORE.len() - 1)];

    score
}

fn sim(farm: &Farm) -> i32 {
    let mut farm = farm.clone();
    let mut rng = rand::thread_rng();
    let mut can_build_room = true;
    let mut can_build_stable = true;
    let mut can_build_field = true;
    let mut can_fence = true;
    while !farm.empty_indices().is_empty()
        && (can_build_room || can_build_stable || can_build_field || can_fence)
    {
        let choice = rng.gen_range(0..4);
        //println!("Choice {} Can build room {} Can build field {} Can build stable {} Can fence {}", choice, can_build_room, can_build_field, can_build_stable, can_fence);
        if choice == 0 {
            let room_positions = farm.possible_room_positions();
            if room_positions.is_empty() {
                can_build_room = false;
            } else {
                let room_position = room_positions[rng.gen_range(0..room_positions.len())];
                farm.build_room(room_position);
            }
        } else if choice == 1 {
            let field_positions = farm.possible_field_positions();
            if field_positions.is_empty() {
                can_build_field = false;
            } else {
                let field_position = field_positions[rng.gen_range(0..field_positions.len())];
                farm.add_field(field_position);
            }
        } else if choice == 2 {
            let stable_positions = farm.possible_stable_positions();
            if !stable_positions.is_empty() && farm.can_build_stable() {
                let stable_position = stable_positions[rng.gen_range(0..stable_positions.len())];
                farm.build_stable(stable_position);
            } else {
                can_build_stable = false;
            }
        } else {
            let pasture_configs = get_all_pasture_configs(&farm.farmyard_spaces);
            if !pasture_configs.is_empty() && farm.fences_used < MAX_FENCES {
                //println!("Pasture configs {:?}", pasture_configs);
                let pasture_config = &pasture_configs[rng.gen_range(0..pasture_configs.len())];
                let mut wood = 15;
                farm.fence_spaces(pasture_config, &mut wood);
            } else {
                can_fence = false;
            }
        }
        //println!("{}\nScore {}", print(&farm), score(&farm));
    }
    println!("{}\nScore {}", print(&farm), score(&farm));
    score(&farm)
}

#[allow(clippy::cast_precision_loss)]
fn main() {
    let mut farm = Farm::new();
    farm.build_room(0);
    farm.add_field(14);
    let mut average_score = 0;
    let num_games = 1000;
    for i in 0..num_games {
        let score = sim(&farm);
        println!("Sim {i} => {score}");
        average_score += score;
    }

    println!("{}", average_score as f32 / num_games as f32);
}
