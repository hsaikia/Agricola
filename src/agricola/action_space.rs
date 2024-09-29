use rand::Rng;

use super::quantity::{
    Boar, Cattle, Clay, Food, Grain, Quantities, Quantity, Reed, Resources, Sheep, Stone,
    Vegetable, Wood,
};

pub fn take_resources(player_quantities: &mut Quantities, resources: &Resources) {
    for (a, b) in resources.iter().zip(player_quantities.iter_mut()) {
        *b += *a;
    }
}

pub trait ActionSpace {
    fn index(&self) -> usize;
}

pub struct Copse;
pub struct Grove;
pub struct Forest;
pub struct ResourceMarket;
pub struct Hollow;
pub struct ClayPit;
pub struct ReedBank;
pub struct TravelingPlayers;
pub struct Fishing;
pub struct DayLaborer;
pub struct GrainSeeds;
pub struct MeetingPlace;
pub struct Farmland;
pub struct FarmExpansion;
pub struct Lessons1;
pub struct Lessons2;
pub struct SheepMarket;
pub struct GrainUtilization;
pub struct Fencing;
pub struct Improvements;
pub struct WishForChildren;
pub struct WesternQuarry;
pub struct HouseRedevelopment;
pub struct PigMarket;
pub struct VegetableSeeds;
pub struct EasternQuarry;
pub struct CattleMarket;
pub struct Cultivation;
pub struct UrgentWishForChildren;
pub struct FarmRedevelopment;

impl ActionSpace for Copse {
    fn index(&self) -> usize {
        0
    }
}

impl ActionSpace for Grove {
    fn index(&self) -> usize {
        1
    }
}

impl ActionSpace for Forest {
    fn index(&self) -> usize {
        2
    }
}

impl ActionSpace for ResourceMarket {
    fn index(&self) -> usize {
        3
    }
}

impl ActionSpace for Hollow {
    fn index(&self) -> usize {
        4
    }
}

impl ActionSpace for ClayPit {
    fn index(&self) -> usize {
        5
    }
}

impl ActionSpace for ReedBank {
    fn index(&self) -> usize {
        6
    }
}

impl ActionSpace for TravelingPlayers {
    fn index(&self) -> usize {
        7
    }
}

impl ActionSpace for Fishing {
    fn index(&self) -> usize {
        8
    }
}

impl ActionSpace for DayLaborer {
    fn index(&self) -> usize {
        9
    }
}

impl ActionSpace for GrainSeeds {
    fn index(&self) -> usize {
        10
    }
}

impl ActionSpace for MeetingPlace {
    fn index(&self) -> usize {
        11
    }
}

impl ActionSpace for Farmland {
    fn index(&self) -> usize {
        12
    }
}

impl ActionSpace for FarmExpansion {
    fn index(&self) -> usize {
        13
    }
}

impl ActionSpace for Lessons1 {
    fn index(&self) -> usize {
        14
    }
}

impl ActionSpace for Lessons2 {
    fn index(&self) -> usize {
        15
    }
}

impl ActionSpace for SheepMarket {
    fn index(&self) -> usize {
        16
    }
}

impl ActionSpace for GrainUtilization {
    fn index(&self) -> usize {
        17
    }
}

impl ActionSpace for Fencing {
    fn index(&self) -> usize {
        18
    }
}

impl ActionSpace for Improvements {
    fn index(&self) -> usize {
        19
    }
}

impl ActionSpace for WishForChildren {
    fn index(&self) -> usize {
        20
    }
}

impl ActionSpace for WesternQuarry {
    fn index(&self) -> usize {
        21
    }
}

impl ActionSpace for HouseRedevelopment {
    fn index(&self) -> usize {
        22
    }
}

impl ActionSpace for PigMarket {
    fn index(&self) -> usize {
        23
    }
}

impl ActionSpace for VegetableSeeds {
    fn index(&self) -> usize {
        24
    }
}

impl ActionSpace for EasternQuarry {
    fn index(&self) -> usize {
        25
    }
}

impl ActionSpace for CattleMarket {
    fn index(&self) -> usize {
        26
    }
}

impl ActionSpace for Cultivation {
    fn index(&self) -> usize {
        27
    }
}

impl ActionSpace for UrgentWishForChildren {
    fn index(&self) -> usize {
        28
    }
}

impl ActionSpace for FarmRedevelopment {
    fn index(&self) -> usize {
        29
    }
}

pub const OPEN_SPACES: usize = 16;
pub const NUM_ACTION_SPACES: usize = 30;
pub const ACCUMULATION_SPACE_INDICES: [usize; 13] = [0, 1, 2, 4, 5, 6, 7, 8, 16, 21, 23, 25, 26];
pub const RESOURCE_SPACE_INDICES: [usize; 5] = [3, 9, 10, 11, 24];

pub const ACTION_SPACE_NAMES: [&str; NUM_ACTION_SPACES] = [
    "Copse",
    "Grove",
    "Forest",
    "Resource Market",
    "Hollow",
    "Clay Pit",
    "Reed Bank",
    "Traveling Players",
    "Fishing",
    "Day Laborer",
    "Grain Seeds",
    "Meeting Place",
    "Farmland",
    "Farm Expansion",
    "Lessons(1)",
    "Lessons(2)",
    "Sheep Market",
    "Grain Utilization",
    "Fencing",
    "Improvements",
    "Wish For Children",
    "Western Quarry",
    "House Redevelopment",
    "Pig Market",
    "Vegetable Seeds",
    "Eastern Quarry",
    "Cattle Market",
    "Cultivation",
    "Urgent Wish For Children",
    "Farm Redevelopment",
];

pub fn accumulate(idx: usize, res: &mut Resources) {
    match idx {
        0 => {
            res[Wood.index()] += 1;
        }
        1 => {
            res[Wood.index()] += 2;
        }
        2 => {
            res[Wood.index()] += 3;
        }
        4 => {
            res[Clay.index()] += 2;
        }
        5 => {
            res[Clay.index()] += 1;
        }
        6 => {
            res[Reed.index()] += 1;
        }
        7 | 8 => {
            res[Food.index()] += 1;
        }
        16 => {
            res[Sheep.index()] += 1;
        }
        21 | 25 => {
            res[Stone.index()] += 1;
        }
        23 => {
            res[Boar.index()] += 1;
        }
        26 => {
            res[Cattle.index()] += 1;
        }
        _ => {}
    }
}

pub fn get_resource(idx: usize, res: &mut Quantities) {
    match idx {
        3 => {
            res[Food.index()] += 1;
            res[Stone.index()] += 1;
            res[Reed.index()] += 1;
        }
        9 => {
            res[Food.index()] += 2;
        }
        10 => {
            res[Grain.index()] += 1;
        }
        11 => {
            res[Food.index()] += 1;
        }
        24 => {
            res[Vegetable.index()] += 1;
        }
        _ => {}
    }
}

pub fn randomize_action_spaces(sequence: &mut [usize], round: usize) {
    let mut rng = rand::thread_rng();
    if round == 1 || round == 2 || round == 3 {
        let next_idx = rng.gen_range(round - 1..4);
        sequence.swap(OPEN_SPACES + round - 1, OPEN_SPACES + next_idx);
    } else if round == 5 || round == 6 {
        let next_idx = rng.gen_range(round - 1..7);
        sequence.swap(OPEN_SPACES + round - 1, OPEN_SPACES + next_idx);
    } else if round == 8 {
        let next_idx = rng.gen_range(7..9);
        sequence.swap(OPEN_SPACES + round - 1, OPEN_SPACES + next_idx);
    } else if round == 10 {
        let next_idx = rng.gen_range(9..11);
        sequence.swap(OPEN_SPACES + round - 1, OPEN_SPACES + next_idx);
    } else if round == 12 {
        let next_idx = rng.gen_range(11..13);
        sequence.swap(OPEN_SPACES + round - 1, OPEN_SPACES + next_idx);
    }
}
