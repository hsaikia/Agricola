use super::farm::{FarmyardSpace, House, Seed};
use super::fencing::get_existing_pastures;
use super::player::Player;
use super::quantity::*;

const FIELD_SCORE: [i32; 6] = [-1, -1, 1, 2, 3, 4];
const PASTURE_SCORE: [i32; 5] = [-1, 1, 2, 3, 4];
const GRAIN_SCORE: [i32; 9] = [-1, 1, 1, 1, 2, 2, 3, 3, 4];
const VEGETABLE_SCORE: [i32; 5] = [-1, 1, 2, 3, 4];
const SHEEP_SCORE: [i32; 9] = [-1, 1, 1, 1, 2, 2, 3, 3, 4];
const PIGS_SCORE: [i32; 8] = [-1, 1, 1, 2, 2, 3, 3, 4];
const CATTLE_SCORE: [i32; 7] = [-1, 1, 2, 2, 3, 3, 4];

pub fn score_farm(player: &Player, player_quantities: &[usize; NUM_QUANTITIES]) -> i32 {
    let mut score = 0;
    let mut num_fields: usize = 0;
    let mut player_quantities = *player_quantities;

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
                    Seed::Grain => player_quantities[Grain.index()] += amt,
                    Seed::Vegetable => player_quantities[Vegetable.index()] += amt,
                }
            }
            _ => (),
        }
    }

    score += PASTURE_SCORE[num_pastures.min(PASTURE_SCORE.len() - 1)];
    score += FIELD_SCORE[num_fields.min(FIELD_SCORE.len() - 1)];
    score += GRAIN_SCORE[player_quantities[Grain.index()].min(GRAIN_SCORE.len() - 1)];
    score += VEGETABLE_SCORE[player_quantities[Vegetable.index()].min(VEGETABLE_SCORE.len() - 1)];
    score += SHEEP_SCORE[player_quantities[Sheep.index()].min(SHEEP_SCORE.len() - 1)];
    score += PIGS_SCORE[player_quantities[Boar.index()].min(PIGS_SCORE.len() - 1)];
    score += CATTLE_SCORE[player_quantities[Cattle.index()].min(CATTLE_SCORE.len() - 1)];

    score
}
