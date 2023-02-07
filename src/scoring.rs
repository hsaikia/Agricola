use crate::player::{Player};
use crate::farm::{Animal, House, PlantedSeed};
use crate::major_improvements::{MajorImprovement, ALL_MAJORS};
use crate::primitives::Resource;

const FIELD_SCORE: [i32; 6] = [-1, -1, 1, 2, 3, 4];
const PASTURE_SCORE: [i32; 5] = [-1, 1, 2, 3, 4];
const GRAIN_SCORE: [i32; 9] = [-1, 1, 1, 1, 2, 2, 3, 3, 4];
const VEGETABLE_SCORE: [i32; 5] = [-1, 1, 2, 3, 4];
const SHEEP_SCORE: [i32; 9] = [-1, 1, 1, 1, 2, 2, 3, 3, 4];
const PIGS_SCORE: [i32; 8] = [-1, 1, 1, 2, 2, 3, 3, 4];
const CATTLE_SCORE: [i32; 7] = [-1, 1, 2, 2, 3, 3, 4];

fn calc_score(num: usize, scores: &[i32]) -> i32 {
    if num >= scores.len() {
        scores[scores.len() - 1]
    } else {
        scores[num]
    }
}

fn score_plants(player : &Player) -> i32 {
    let mut num_grain: usize = 0;
    let mut num_veg: usize = 0;
    for field in &player.fields {
        match field.seed {
            PlantedSeed::Grain => num_grain += field.amount as usize,
            PlantedSeed::Vegetable => num_veg += field.amount as usize,
            _ => (),
        }
    }
    num_grain += player.resources[Resource::Grain] as usize;
    num_veg += player.resources[Resource::Vegetable] as usize;

    calc_score(num_grain, &GRAIN_SCORE) + calc_score(num_veg, &VEGETABLE_SCORE)
}

fn score_animals(player : &Player) -> i32 {
    // All animals kept as pets and in unfenced stables
    let mut num_sheep = player.resources[Resource::Sheep];
    let mut num_pigs = player.resources[Resource::Pigs];
    let mut num_cattle = player.resources[Resource::Cattle];

    for pasture in &player.pastures {
        match pasture.animal {
            Animal::Sheep => num_sheep += pasture.amount,
            Animal::Pigs => num_pigs += pasture.amount,
            Animal::Cattle => num_cattle += pasture.amount,
            _ => (),
        }
    }

    calc_score(num_sheep as usize, &SHEEP_SCORE)
        + calc_score(num_pigs as usize, &PIGS_SCORE)
        + calc_score(num_cattle as usize, &CATTLE_SCORE)
}

fn score_fields(player : &Player) -> i32 {
    calc_score(player.fields.len(), &FIELD_SCORE)
}

fn score_pastures(player : &Player) -> i32 {
    let mut ret: i32 = 0;
    // Number of Pastures
    ret += calc_score(player.pastures.len(), &PASTURE_SCORE);
    // Number of fenced stables
    for pasture in &player.pastures {
        ret += pasture.stables as i32;
    }
    ret
}

fn score_house_family_empty_spaces_begging(player : &Player) -> i32 {
    let mut ret: i32 = 0;

    // House
    match player.house {
        House::Clay => ret += player.rooms as i32,
        House::Stone => ret += 2 * player.rooms as i32,
        _ => (),
    }

    // Family members
    ret += 3 * player.family_members() as i32;

    // Empty spaces
    ret -= player.empty_farmyard_spaces() as i32;

    // Begging Tokens
    ret -= 3 * player.begging_tokens as i32;
    ret
}

pub fn score(player : &Player) -> i32 {
    let mut ret: i32 = 0;

    // Fields
    ret += score_fields(&player);
    // Pastures
    ret += score_pastures(&player);
    // Grain and Veggies
    ret += score_plants(&player);
    // Animals
    ret += score_animals(&player);
    // House, Family and Empty Spaces
    ret += score_house_family_empty_spaces_begging(&player);

    // Score Majors
    for (i, e) in player.major_cards.iter().enumerate() {
        if !e {
            continue;
        }
        ret += ALL_MAJORS[i].points() as i32;
        match ALL_MAJORS[i] {
            MajorImprovement::Joinery => {
                if player.resources[Resource::Wood] >= 7 {
                    ret += 3;
                } else if player.resources[Resource::Wood] >= 5 {
                    ret += 2;
                } else if player.resources[Resource::Wood] >= 3 {
                    ret += 1;
                }
            }
            MajorImprovement::Pottery => {
                if player.resources[Resource::Clay] >= 7 {
                    ret += 3;
                } else if player.resources[Resource::Clay] >= 5 {
                    ret += 2;
                } else if player.resources[Resource::Clay] >= 3 {
                    ret += 1;
                }
            }
            MajorImprovement::BasketmakersWorkshop => {
                if player.resources[Resource::Reed] >= 5 {
                    ret += 3;
                } else if player.resources[Resource::Reed] >= 4 {
                    ret += 2;
                } else if player.resources[Resource::Reed] >= 2 {
                    ret += 1;
                }
            }
            _ => (),
        }
    }

    // TODO Score minors/occs

    ret
}
