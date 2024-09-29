use super::farm::{FarmyardSpace, Seed};
use super::fencing::get_existing_pastures;
use super::flag::{ClayHouse, Flag, WoodHouse};
use super::quantity::{Boar, Cattle, Grain, Quantity, Sheep, Vegetable};
use super::state::State;

const FIELD_SCORE: [i32; 6] = [-1, -1, 1, 2, 3, 4];
const PASTURE_SCORE: [i32; 5] = [-1, 1, 2, 3, 4];
const GRAIN_SCORE: [i32; 9] = [-1, 1, 1, 1, 2, 2, 3, 3, 4];
const VEGETABLE_SCORE: [i32; 5] = [-1, 1, 2, 3, 4];
const SHEEP_SCORE: [i32; 9] = [-1, 1, 1, 1, 2, 2, 3, 3, 4];
const PIGS_SCORE: [i32; 8] = [-1, 1, 1, 2, 2, 3, 3, 4];
const CATTLE_SCORE: [i32; 7] = [-1, 1, 2, 2, 3, 3, 4];
const HOUSE_SCORE: [i32; 3] = [0, 1, 2];

#[must_use]
pub fn score_farm(state: &State, player_idx: usize) -> i32 {
    let mut score = 0;
    let mut num_fields: usize = 0;
    let mut player_quantities = *state.player_quantities(player_idx);

    let house_type_idx = if state.player_flags(player_idx)[WoodHouse.index()] {
        0
    } else if state.player_flags(player_idx)[ClayHouse.index()] {
        1
    } else {
        2
    };

    let num_pastures = get_existing_pastures(&state.player_farm(player_idx).farmyard_spaces)
        .iter()
        .filter(|p| !p.is_empty())
        .count();

    for space in &state.player_farm(player_idx).farmyard_spaces {
        match *space {
            FarmyardSpace::Empty => score -= 1,
            FarmyardSpace::Room => score += HOUSE_SCORE[house_type_idx],
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
