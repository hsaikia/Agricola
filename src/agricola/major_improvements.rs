use super::primitives::*;

pub const TOTAL_MAJOR_IMPROVEMENTS: usize = 10;

#[derive(Clone, Debug, Hash, PartialEq)]
pub enum MajorImprovement {
    Fireplace { cheaper: bool },
    CookingHearth { cheaper: bool },
    Well,
    ClayOven,
    StoneOven,
    Joinery,
    Pottery,
    BasketmakersWorkshop,
}

impl MajorImprovement {
    pub const fn index(&self) -> usize {
        match self {
            Self::Fireplace { cheaper: true } => 0,
            Self::Fireplace { cheaper: false } => 1,
            Self::CookingHearth { cheaper: true } => 2,
            Self::CookingHearth { cheaper: false } => 3,
            Self::Well => 4,
            Self::ClayOven => 5,
            Self::StoneOven => 6,
            Self::Joinery => 7,
            Self::Pottery => 8,
            Self::BasketmakersWorkshop => 9,
        }
    }

    pub fn exchanges(&self) -> Vec<ResourceExchange> {
        match self {
            Self::Fireplace { cheaper: _ } => {
                let ret: Vec<ResourceExchange> = vec![
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
                ];
                ret
            }
            Self::CookingHearth { cheaper: _ } => {
                let ret: Vec<ResourceExchange> = vec![
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
                ];
                ret
            }
            Self::Joinery => {
                vec![ResourceExchange {
                    from: Wood.index(),
                    to: Food.index(),
                    num_from: 1,
                    num_to: 2,
                }]
            }
            Self::Pottery => {
                vec![ResourceExchange {
                    from: Clay.index(),
                    to: Food.index(),
                    num_from: 1,
                    num_to: 2,
                }]
            }
            Self::BasketmakersWorkshop => {
                vec![ResourceExchange {
                    from: Reed.index(),
                    to: Food.index(),
                    num_from: 1,
                    num_to: 3,
                }]
            }
            _ => vec![],
        }
    }

    pub fn points(&self) -> u32 {
        match self {
            Self::Fireplace { cheaper: _ } | Self::CookingHearth { cheaper: _ } => 1,
            Self::ClayOven | Self::Joinery | Self::Pottery | Self::BasketmakersWorkshop => 2,
            Self::StoneOven => 3,
            Self::Well => 4,
        }
    }

    pub fn cost(&self) -> Resources {
        let mut res = new_res();
        match self {
            Self::Fireplace { cheaper } => {
                res[Clay.index()] = if *cheaper { 2 } else { 3 };
            }
            Self::CookingHearth { cheaper } => {
                res[Clay.index()] = if *cheaper { 4 } else { 5 };
            }
            Self::Well => {
                res[Wood.index()] = 1;
                res[Stone.index()] = 3;
            }
            Self::ClayOven => {
                res[Clay.index()] = 3;
                res[Stone.index()] = 1;
            }
            Self::StoneOven => {
                res[Clay.index()] = 1;
                res[Stone.index()] = 3;
            }
            Self::Joinery => {
                res[Wood.index()] = 2;
                res[Stone.index()] = 2;
            }
            Self::Pottery => {
                res[Clay.index()] = 2;
                res[Stone.index()] = 2;
            }
            Self::BasketmakersWorkshop => {
                res[Reed.index()] = 2;
                res[Stone.index()] = 2;
            }
        }
        res
    }

    pub fn display(cards: &Vec<Self>) -> String {
        let mut ret = String::new();
        for card in cards {
            ret = format!("{ret}\n{card:?}");
        }
        ret
    }
}
