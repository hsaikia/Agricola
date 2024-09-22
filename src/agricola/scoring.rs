use super::farm::{FarmyardSpace, House, Seed};
use super::fencing::get_existing_pastures;
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
    let mut num_fields: usize = 0;

    let num_pastures = get_existing_pastures(&player.farm.farmyard_spaces)
        .iter()
        .filter(|p| !p.is_empty())
        .count();

    for space in &player.farm.farmyard_spaces {
        match *space {
            FarmyardSpace::Empty => score -= 1,
            FarmyardSpace::Room => match player.house {
                House::Clay => score += 1,
                House::Stone => score += 2,
                House::Wood => (),
            },
            FarmyardSpace::FencedPasture(has_stable, _) => {
                if has_stable {
                    score += 1;
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
    score += PIGS_SCORE[res[Boar.index()].min(PIGS_SCORE.len() - 1)];
    score += CATTLE_SCORE[res[Cattle.index()].min(CATTLE_SCORE.len() - 1)];

    score
}

pub fn score(player: &Player) -> f32 {
    let mut ret: f32 = 0.0;

    // House, Family and Empty Spaces
    ret += 3.0 * player.family_members() as f32;
    // Begging Tokens
    ret -= 3.0 * player.begging_tokens as f32;
    // Score farm
    ret += score_farm(player) as f32;

    ret
}
