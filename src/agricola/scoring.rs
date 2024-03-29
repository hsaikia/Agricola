use super::farm::{Animal, FarmyardSpace, House, Seed};
use super::major_improvements::MajorImprovement;
use super::player::Player;
use super::primitives::*;

const FIELD_SCORE: [i32; 6] = [-1, -1, 1, 2, 3, 4];
const PASTURE_SCORE: [i32; 5] = [-1, 1, 2, 3, 4];
const GRAIN_SCORE: [i32; 9] = [-1, 1, 1, 1, 2, 2, 3, 3, 4];
const VEGETABLE_SCORE: [i32; 5] = [-1, 1, 2, 3, 4];
const SHEEP_SCORE: [i32; 9] = [-1, 1, 1, 1, 2, 2, 3, 3, 4];
const PIGS_SCORE: [i32; 8] = [-1, 1, 1, 2, 2, 3, 3, 4];
const CATTLE_SCORE: [i32; 7] = [-1, 1, 2, 2, 3, 3, 4];

fn score_farm(player: &Player) -> i32 {
    let mut score = 0;
    let mut res: Resources = player.resources;
    let mut num_pastures: usize = 0;
    let mut num_fields: usize = 0;

    let pastures = player.farm.pastures_and_capacities();
    for (p, c) in pastures {
        if c > p.len() {
            num_pastures += 1; // score all pastures except for unfenced stables
        }
    }

    for space in &player.farm.farmyard_spaces {
        match *space {
            FarmyardSpace::Empty => score -= 1,
            FarmyardSpace::Room => match player.house {
                House::Clay => score += 1,
                House::Stone => score += 2,
                House::Wood => (),
            },
            FarmyardSpace::FencedPasture(opt_animal, has_stable) => {
                if has_stable {
                    score += 1;
                }
                if let Some(animal) = opt_animal {
                    match animal.0 {
                        Animal::Sheep => res[Sheep.index()] += 1,
                        Animal::Boar => res[Boar.index()] += 1,
                        Animal::Cow => res[Cow.index()] += 1,
                    }
                }
            }
            FarmyardSpace::Field(Some((seed, amt))) => {
                num_fields += 1;
                match seed {
                    Seed::Grain => res[Grain.index()] += amt,
                    Seed::Vegetable => res[Vegetable.index()] += amt,
                }
            }
            _ => (),
        }
    }

    score += PASTURE_SCORE[num_pastures.min(PASTURE_SCORE.len() - 1)];
    score += FIELD_SCORE[num_fields.min(FIELD_SCORE.len() - 1)];
    score += GRAIN_SCORE[res[Grain.index()].min(GRAIN_SCORE.len() - 1)];
    score += VEGETABLE_SCORE[res[Vegetable.index()].min(VEGETABLE_SCORE.len() - 1)];
    score += SHEEP_SCORE[res[Sheep.index()].min(SHEEP_SCORE.len() - 1)];
    score += PIGS_SCORE[res[Sheep.index()].min(PIGS_SCORE.len() - 1)];
    score += CATTLE_SCORE[res[Sheep.index()].min(CATTLE_SCORE.len() - 1)];

    score
}

#[allow(clippy::cast_possible_wrap)]
fn score_cards(player: &Player) -> i32 {
    let mut ret: i32 = 0;
    // Score Majors
    for major in &player.major_cards {
        ret += major.points() as i32;
        match major {
            MajorImprovement::Joinery => {
                if player.resources[Wood.index()] >= 7 {
                    ret += 3;
                } else if player.resources[Wood.index()] >= 5 {
                    ret += 2;
                } else if player.resources[Wood.index()] >= 3 {
                    ret += 1;
                }
            }
            MajorImprovement::Pottery => {
                if player.resources[Clay.index()] >= 7 {
                    ret += 3;
                } else if player.resources[Clay.index()] >= 5 {
                    ret += 2;
                } else if player.resources[Clay.index()] >= 3 {
                    ret += 1;
                }
            }
            MajorImprovement::BasketmakersWorkshop => {
                if player.resources[Reed.index()] >= 5 {
                    ret += 3;
                } else if player.resources[Reed.index()] >= 4 {
                    ret += 2;
                } else if player.resources[Reed.index()] >= 2 {
                    ret += 1;
                }
            }
            _ => (),
        }
    }
    ret
}

pub fn score(player: &Player) -> i32 {
    let mut ret: i32 = 0;

    // House, Family and Empty Spaces
    ret += 3 * player.family_members() as i32;
    // All resources
    ret += score_cards(player);
    // Begging Tokens
    ret -= 3 * player.begging_tokens as i32;
    // Score farm
    ret += score_farm(player);

    ret
}
