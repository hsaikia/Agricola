use super::quantity::*;

pub trait Card {
    fn index(&self) -> usize;
}

pub const NUM_CARDS: usize = 12;
pub const MAJOR_IMPROVEMENTS_INDICES: [usize; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
pub const COOKING_IMPROVEMENTS_INDICES: [usize; 4] = [0, 1, 2, 3];
pub const FIREPLACE_INDICES: [usize; 2] = [0, 1];
pub const COOKING_HEARTH_INDICES: [usize; 2] = [2, 3];
pub const BAKING_IMPROVEMENTS_INDICES: [usize; 6] = [0, 1, 2, 3, 5, 6];
pub const OCCUPATIONS_INDICES: [usize; 2] = [10, 11];

pub const CARD_NAMES: [&str; NUM_CARDS] = [
    "Major : Fireplace (2 🧱)",
    "Major : Fireplace (3 🧱)",
    "Major : Cooking Hearth (4 🧱)",
    "Major : Cooking Hearth (5 🧱)",
    "Major : Well",
    "Major : Clay Oven",
    "Major : Stone Oven",
    "Major : Joinery",
    "Major : Pottery",
    "Major : Basketmaker's Workshop",
    "Occupation : Assistant Tiller",
    "Occupation : Childless",
];

/// Major improvements
pub struct Fireplace1;
pub struct Fireplace2;
pub struct CookingHearth1;
pub struct CookingHearth2;
pub struct Well;
pub struct ClayOven;
pub struct StoneOven;
pub struct Joinery;
pub struct Pottery;
pub struct BasketmakersWorkshop;

/// Occupations
pub struct AssistantTiller;
pub struct Childless;

impl Card for Fireplace1 {
    fn index(&self) -> usize {
        0
    }
}

impl Card for Fireplace2 {
    fn index(&self) -> usize {
        1
    }
}

impl Card for CookingHearth1 {
    fn index(&self) -> usize {
        2
    }
}

impl Card for CookingHearth2 {
    fn index(&self) -> usize {
        3
    }
}

impl Card for Well {
    fn index(&self) -> usize {
        4
    }
}

impl Card for ClayOven {
    fn index(&self) -> usize {
        5
    }
}

impl Card for StoneOven {
    fn index(&self) -> usize {
        6
    }
}

impl Card for Joinery {
    fn index(&self) -> usize {
        7
    }
}

impl Card for Pottery {
    fn index(&self) -> usize {
        8
    }
}

impl Card for BasketmakersWorkshop {
    fn index(&self) -> usize {
        9
    }
}

impl Card for AssistantTiller {
    fn index(&self) -> usize {
        10
    }
}

impl Card for Childless {
    fn index(&self) -> usize {
        11
    }
}

pub trait MajorImprovement {
    fn anytime_exchanges(&self) -> Vec<ResourceExchange> {
        vec![]
    }
    fn baking_exchanges(&self) -> Vec<ResourceExchange> {
        vec![]
    }
    fn harvest_exchanges(&self) -> Vec<ResourceExchange> {
        vec![]
    }
    fn points(&self, quantities: &Quantities) -> u32;
    fn cost(&self) -> Resources;
}

impl MajorImprovement for Fireplace1 {
    fn anytime_exchanges(&self) -> Vec<ResourceExchange> {
        vec![
            ResourceExchange {
                from: Sheep.index(),
                to: Food.index(),
                num_from: 1,
                num_to: 2,
            },
            ResourceExchange {
                from: Boar.index(),
                to: Food.index(),
                num_from: 1,
                num_to: 2,
            },
            ResourceExchange {
                from: Vegetable.index(),
                to: Food.index(),
                num_from: 1,
                num_to: 2,
            },
            ResourceExchange {
                from: Cattle.index(),
                to: Food.index(),
                num_from: 1,
                num_to: 3,
            },
        ]
    }

    fn baking_exchanges(&self) -> Vec<ResourceExchange> {
        vec![ResourceExchange {
            from: Grain.index(),
            to: Food.index(),
            num_from: 1,
            num_to: 2,
        }]
    }

    fn points(&self, _quantities: &Quantities) -> u32 {
        1
    }

    fn cost(&self) -> Resources {
        let mut res = new_res();
        res[Clay.index()] = 2;
        res
    }
}

impl MajorImprovement for Fireplace2 {
    fn anytime_exchanges(&self) -> Vec<ResourceExchange> {
        vec![
            ResourceExchange {
                from: Sheep.index(),
                to: Food.index(),
                num_from: 1,
                num_to: 2,
            },
            ResourceExchange {
                from: Boar.index(),
                to: Food.index(),
                num_from: 1,
                num_to: 2,
            },
            ResourceExchange {
                from: Vegetable.index(),
                to: Food.index(),
                num_from: 1,
                num_to: 2,
            },
            ResourceExchange {
                from: Cattle.index(),
                to: Food.index(),
                num_from: 1,
                num_to: 3,
            },
        ]
    }

    fn baking_exchanges(&self) -> Vec<ResourceExchange> {
        vec![ResourceExchange {
            from: Grain.index(),
            to: Food.index(),
            num_from: 1,
            num_to: 2,
        }]
    }

    fn points(&self, _quantities: &Quantities) -> u32 {
        1
    }

    fn cost(&self) -> Resources {
        let mut res = new_res();
        res[Clay.index()] = 3;
        res
    }
}

impl MajorImprovement for CookingHearth1 {
    fn anytime_exchanges(&self) -> Vec<ResourceExchange> {
        vec![
            ResourceExchange {
                from: Sheep.index(),
                to: Food.index(),
                num_from: 1,
                num_to: 2,
            },
            ResourceExchange {
                from: Boar.index(),
                to: Food.index(),
                num_from: 1,
                num_to: 3,
            },
            ResourceExchange {
                from: Vegetable.index(),
                to: Food.index(),
                num_from: 1,
                num_to: 3,
            },
            ResourceExchange {
                from: Cattle.index(),
                to: Food.index(),
                num_from: 1,
                num_to: 4,
            },
        ]
    }

    fn baking_exchanges(&self) -> Vec<ResourceExchange> {
        vec![ResourceExchange {
            from: Grain.index(),
            to: Food.index(),
            num_from: 1,
            num_to: 3,
        }]
    }

    fn points(&self, _quantities: &Quantities) -> u32 {
        1
    }

    fn cost(&self) -> Resources {
        let mut res = new_res();
        res[Clay.index()] = 4;
        res
    }
}

impl MajorImprovement for CookingHearth2 {
    fn anytime_exchanges(&self) -> Vec<ResourceExchange> {
        vec![
            ResourceExchange {
                from: Sheep.index(),
                to: Food.index(),
                num_from: 1,
                num_to: 2,
            },
            ResourceExchange {
                from: Boar.index(),
                to: Food.index(),
                num_from: 1,
                num_to: 3,
            },
            ResourceExchange {
                from: Vegetable.index(),
                to: Food.index(),
                num_from: 1,
                num_to: 3,
            },
            ResourceExchange {
                from: Cattle.index(),
                to: Food.index(),
                num_from: 1,
                num_to: 4,
            },
        ]
    }

    fn baking_exchanges(&self) -> Vec<ResourceExchange> {
        vec![ResourceExchange {
            from: Grain.index(),
            to: Food.index(),
            num_from: 1,
            num_to: 3,
        }]
    }

    fn points(&self, _quantities: &Quantities) -> u32 {
        1
    }

    fn cost(&self) -> Resources {
        let mut res = new_res();
        res[Clay.index()] = 5;
        res
    }
}

impl MajorImprovement for Well {
    fn points(&self, _quantities: &Quantities) -> u32 {
        4
    }

    fn cost(&self) -> Resources {
        let mut res = new_res();
        res[Wood.index()] = 1;
        res[Stone.index()] = 3;
        res
    }
}

impl MajorImprovement for ClayOven {
    fn baking_exchanges(&self) -> Vec<ResourceExchange> {
        vec![ResourceExchange {
            from: Grain.index(),
            to: Food.index(),
            num_from: 1,
            num_to: 5,
        }]
    }

    fn points(&self, _quantities: &Quantities) -> u32 {
        2
    }

    fn cost(&self) -> Resources {
        let mut res = new_res();
        res[Clay.index()] = 3;
        res[Stone.index()] = 1;
        res
    }
}

impl MajorImprovement for StoneOven {
    fn baking_exchanges(&self) -> Vec<ResourceExchange> {
        vec![
            ResourceExchange {
                from: Grain.index(),
                to: Food.index(),
                num_from: 1,
                num_to: 4,
            },
            ResourceExchange {
                from: Grain.index(),
                to: Food.index(),
                num_from: 2,
                num_to: 8,
            },
        ]
    }

    fn points(&self, _quantities: &Quantities) -> u32 {
        3
    }

    fn cost(&self) -> Resources {
        let mut res = new_res();
        res[Clay.index()] = 1;
        res[Stone.index()] = 3;
        res
    }
}

impl MajorImprovement for Joinery {
    fn harvest_exchanges(&self) -> Vec<ResourceExchange> {
        vec![ResourceExchange {
            from: Wood.index(),
            to: Food.index(),
            num_from: 1,
            num_to: 2,
        }]
    }

    fn points(&self, quantities: &Quantities) -> u32 {
        if quantities[Wood.index()] >= 7 {
            5
        } else if quantities[Wood.index()] >= 5 {
            4
        } else if quantities[Wood.index()] >= 3 {
            3
        } else {
            2
        }
    }

    fn cost(&self) -> Resources {
        let mut res = new_res();
        res[Wood.index()] = 2;
        res[Stone.index()] = 2;
        res
    }
}

impl MajorImprovement for Pottery {
    fn harvest_exchanges(&self) -> Vec<ResourceExchange> {
        vec![ResourceExchange {
            from: Clay.index(),
            to: Food.index(),
            num_from: 1,
            num_to: 2,
        }]
    }

    fn points(&self, quantities: &Quantities) -> u32 {
        if quantities[Clay.index()] >= 7 {
            5
        } else if quantities[Clay.index()] >= 5 {
            4
        } else if quantities[Clay.index()] >= 3 {
            3
        } else {
            2
        }
    }

    fn cost(&self) -> Resources {
        let mut res = new_res();
        res[Clay.index()] = 2;
        res[Stone.index()] = 2;
        res
    }
}

impl MajorImprovement for BasketmakersWorkshop {
    fn harvest_exchanges(&self) -> Vec<ResourceExchange> {
        vec![ResourceExchange {
            from: Reed.index(),
            to: Food.index(),
            num_from: 1,
            num_to: 3,
        }]
    }

    fn points(&self, quantities: &Quantities) -> u32 {
        if quantities[Reed.index()] >= 5 {
            5
        } else if quantities[Reed.index()] >= 4 {
            4
        } else if quantities[Reed.index()] >= 2 {
            3
        } else {
            2
        }
    }

    fn cost(&self) -> Resources {
        let mut res = new_res();
        res[Reed.index()] = 2;
        res[Stone.index()] = 2;
        res
    }
}

pub fn anytime_exchanges(major_idx: usize) -> Vec<ResourceExchange> {
    match major_idx {
        0 => Fireplace1.anytime_exchanges(),
        1 => Fireplace2.anytime_exchanges(),
        2 => CookingHearth1.anytime_exchanges(),
        3 => CookingHearth2.anytime_exchanges(),
        _ => vec![],
    }
}

pub fn baking_exchanges(major_idx: usize) -> Vec<ResourceExchange> {
    match major_idx {
        0 => Fireplace1.baking_exchanges(),
        1 => Fireplace2.baking_exchanges(),
        2 => CookingHearth1.baking_exchanges(),
        3 => CookingHearth2.baking_exchanges(),
        5 => ClayOven.baking_exchanges(),
        6 => StoneOven.baking_exchanges(),
        _ => vec![],
    }
}

pub fn harvest_exchanges(major_idx: usize) -> Vec<ResourceExchange> {
    match major_idx {
        7 => Joinery.harvest_exchanges(),
        8 => Pottery.harvest_exchanges(),
        9 => BasketmakersWorkshop.harvest_exchanges(),
        _ => vec![],
    }
}

pub fn points(major_idx: usize, quantities: &Quantities) -> u32 {
    match major_idx {
        0 => Fireplace1.points(quantities),
        1 => Fireplace2.points(quantities),
        2 => CookingHearth1.points(quantities),
        3 => CookingHearth2.points(quantities),
        4 => Well.points(quantities),
        5 => ClayOven.points(quantities),
        6 => StoneOven.points(quantities),
        7 => Joinery.points(quantities),
        8 => Pottery.points(quantities),
        9 => BasketmakersWorkshop.points(quantities),
        _ => 0,
    }
}

pub fn cost(major_idx: usize) -> Resources {
    match major_idx {
        0 => Fireplace1.cost(),
        1 => Fireplace2.cost(),
        2 => CookingHearth1.cost(),
        3 => CookingHearth2.cost(),
        4 => Well.cost(),
        5 => ClayOven.cost(),
        6 => StoneOven.cost(),
        7 => Joinery.cost(),
        8 => Pottery.cost(),
        9 => BasketmakersWorkshop.cost(),
        _ => new_res(),
    }
}
