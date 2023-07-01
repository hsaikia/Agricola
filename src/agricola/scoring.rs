use crate::agricola::farm::{FarmyardSpace, House, Seed};
use crate::agricola::major_improvements::MajorImprovement;
use crate::agricola::player::Player;
use crate::agricola::primitives::Resource;

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
fn score_fields(player: &Player, debug: bool) -> i32 {
    let mut num_grain: usize = 0;
    let mut num_veg: usize = 0;
    let num_fields: usize = player.farm.field_indices().len();

    for space in &player.farm.farmyard_spaces {
        if let FarmyardSpace::PlantedField(crop, amount) = space {
            match *crop {
                Seed::Grain => num_grain += amount,
                Seed::Vegetable => num_veg += amount,
            }
        }
    }

    num_grain += player.resources[Resource::Grain];
    num_veg += player.resources[Resource::Vegetable];
    let gr_score = calc_score(num_grain, &GRAIN_SCORE);
    let veg_score = calc_score(num_veg, &VEGETABLE_SCORE);

    if debug {
        print!("\nScoring {num_grain} Grain {gr_score} and {num_veg} Veggies {veg_score}.");
    }

    gr_score + veg_score + calc_score(num_fields, &FIELD_SCORE)
}

fn score_animals(player: &Player, debug: bool) -> i32 {
    // All animals kept as pets and in unfenced stables
    let res = player.animals_as_resources();
    let sh_score = calc_score(res[Resource::Sheep], &SHEEP_SCORE);
    let pig_score = calc_score(res[Resource::Pigs], &PIGS_SCORE);
    let cow_score = calc_score(res[Resource::Cattle], &CATTLE_SCORE);

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

#[allow(clippy::cast_possible_wrap)]
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

#[allow(clippy::cast_possible_wrap)]
fn score_begging_tokens(player: &Player) -> i32 {
    -3 * player.begging_tokens as i32
}

#[allow(clippy::cast_possible_wrap)]
fn score_house_family_empty_spaces(player: &Player) -> i32 {
    let mut ret: i32 = 0;
    let rooms = player.farm.room_indices().len();

    // House
    match player.house {
        House::Clay => ret += rooms as i32,
        House::Stone => ret += 2 * rooms as i32,
        House::Wood => (),
    }

    // Family members
    ret += 3 * player.family_members() as i32;

    // Empty spaces
    ret -= player.empty_farmyard_spaces() as i32;

    ret
}

#[allow(clippy::cast_possible_wrap)]
fn score_cards(player: &Player) -> i32 {
    let mut ret: i32 = 0;
    // Score Majors
    for major in &player.major_cards {
        ret += major.points() as i32;
        match major {
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
    // Animals
    ret += score_animals(player, debug);
    // Score cards
    ret += score_cards(player);

    ret
}

pub fn score(player: &Player, debug: bool) -> i32 {
    let mut ret: i32 = 0;

    // Fields (Grains and Veggies)
    ret += score_fields(player, debug);
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
