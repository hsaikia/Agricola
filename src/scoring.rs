use crate::farm::{House, PlantedSeed};
use crate::major_improvements::{MajorImprovement, ALL_MAJORS};
use crate::player::Player;
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

fn score_plants(player: &Player, debug: bool) -> i32 {
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
    let gr_score = calc_score(num_grain, &GRAIN_SCORE);
    let veg_score = calc_score(num_veg, &VEGETABLE_SCORE);

    if debug {
        print!(
            "\nScoring {} Grain {} and {} Veggies {}.",
            num_grain, gr_score, num_veg, veg_score
        );
    }

    gr_score + veg_score
}

fn score_animals(player: &Player, debug: bool) -> i32 {
    // All animals kept as pets and in unfenced stables
    let res = player.animals_as_resources();
    let sh_score = calc_score(res[Resource::Sheep] as usize, &SHEEP_SCORE);
    let pig_score = calc_score(res[Resource::Pigs] as usize, &PIGS_SCORE);
    let cow_score = calc_score(res[Resource::Cattle] as usize, &CATTLE_SCORE);

    if debug {
        print!(
            "\nScoring {} Sheep {}. {} Pigs {}. {} Cows {}.",
            res[Resource::Sheep],
            sh_score,
            res[Resource::Pigs],
            pig_score,
            res[Resource::Cattle],
            cow_score
        );
    }

    sh_score + pig_score + cow_score
}

fn score_fields(player: &Player) -> i32 {
    calc_score(player.fields.len(), &FIELD_SCORE)
}

fn score_pastures(player: &Player) -> i32 {
    let mut ret: i32 = 0;
    // Number of Pastures
    ret += calc_score(player.pastures.len(), &PASTURE_SCORE);
    // Number of fenced stables
    for pasture in &player.pastures {
        ret += pasture.stables as i32;
    }
    ret
}

fn score_begging_tokens(player: &Player) -> i32 {
    -3 * player.begging_tokens as i32
}

fn score_house_family_empty_spaces(player: &Player) -> i32 {
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

    ret
}

fn score_cards(player: &Player) -> i32 {
    let mut ret: i32 = 0;
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
    ret
}

pub fn score_resources(player: &Player, debug: bool) -> i32 {
    let mut ret: i32 = 0;
    // Grain and Veggies
    ret += score_plants(player, debug);
    // Animals
    ret += score_animals(player, debug);
    // Score cards
    ret += score_cards(player);

    ret
}

pub fn score(player: &Player, debug: bool) -> i32 {
    let mut ret: i32 = 0;

    // Fields
    ret += score_fields(player);
    // Pastures
    ret += score_pastures(player);
    // House, Family and Empty Spaces
    ret += score_house_family_empty_spaces(player);
    // All resources
    ret += score_resources(player, debug);
    // Begging Tokens
    ret += score_begging_tokens(player);
    // TODO Score minors/occs
    ret
}
