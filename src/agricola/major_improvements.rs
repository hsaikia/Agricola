use super::primitives::{can_pay_for_resource, new_res, Resource, ResourceExchange, Resources};

#[derive(Clone, Debug, Hash, PartialEq)]
pub enum MajorImprovement {
    Fireplace(bool),
    CookingHearth(bool),
    Well,
    ClayOven,
    StoneOven,
    Joinery,
    Pottery,
    BasketmakersWorkshop,
}

impl MajorImprovement {
    pub fn can_build_major(
        majors_owned: &[Self],
        majors_on_board: &[Self],
        resources: &Resources,
    ) -> bool {
        for card in majors_on_board {
            match card {
                Self::Fireplace(cheaper) => {
                    if !majors_owned.contains(&Self::Fireplace(!cheaper))
                        && can_pay_for_resource(&card.cost(), resources)
                    {
                        return true;
                    }
                }
                Self::CookingHearth(cheaper) => {
                    if (majors_owned.contains(&Self::Fireplace(*cheaper))
                        || majors_owned.contains(&Self::Fireplace(!cheaper))
                        || can_pay_for_resource(&card.cost(), resources))
                        && !majors_owned.contains(&Self::CookingHearth(!cheaper))
                    {
                        return true;
                    }
                }
                _ => {
                    if can_pay_for_resource(&card.cost(), resources) {
                        return true;
                    }
                }
            }
        }

        false
    }

    pub fn available_majors_to_build(
        majors_owned: &[Self],
        majors_on_board: &[Self],
        resources: &Resources,
    ) -> Vec<Self> {
        let mut available: Vec<Self> = vec![];

        for card in majors_on_board {
            match card {
                Self::Fireplace(cheaper) => {
                    if !majors_owned.contains(&Self::Fireplace(!cheaper))
                        && can_pay_for_resource(&card.cost(), resources)
                    {
                        available.push(card.clone());
                    }
                }
                Self::CookingHearth(cheaper) => {
                    if (majors_owned.contains(&Self::Fireplace(*cheaper))
                        || majors_owned.contains(&Self::Fireplace(!cheaper))
                        || can_pay_for_resource(&card.cost(), resources))
                        && !majors_owned.contains(&Self::CookingHearth(!cheaper))
                    {
                        available.push(card.clone());
                    }
                }
                _ => {
                    if can_pay_for_resource(&card.cost(), resources) {
                        available.push(card.clone());
                    }
                }
            }
        }
        available
    }

    pub fn exchanges(&self) -> Vec<ResourceExchange> {
        match self {
            Self::Fireplace(_) => {
                let ret: Vec<ResourceExchange> = vec![
                    ResourceExchange {
                        from: Resource::Sheep,
                        to: Resource::Food,
                        num_from: 1,
                        num_to: 2,
                    },
                    ResourceExchange {
                        from: Resource::Pigs,
                        to: Resource::Food,
                        num_from: 1,
                        num_to: 2,
                    },
                    ResourceExchange {
                        from: Resource::Vegetable,
                        to: Resource::Food,
                        num_from: 1,
                        num_to: 2,
                    },
                    ResourceExchange {
                        from: Resource::Cattle,
                        to: Resource::Food,
                        num_from: 1,
                        num_to: 3,
                    },
                ];
                ret
            }
            Self::CookingHearth(_) => {
                let ret: Vec<ResourceExchange> = vec![
                    ResourceExchange {
                        from: Resource::Sheep,
                        to: Resource::Food,
                        num_from: 1,
                        num_to: 2,
                    },
                    ResourceExchange {
                        from: Resource::Pigs,
                        to: Resource::Food,
                        num_from: 1,
                        num_to: 3,
                    },
                    ResourceExchange {
                        from: Resource::Vegetable,
                        to: Resource::Food,
                        num_from: 1,
                        num_to: 3,
                    },
                    ResourceExchange {
                        from: Resource::Cattle,
                        to: Resource::Food,
                        num_from: 1,
                        num_to: 4,
                    },
                ];
                ret
            }
            Self::Joinery => {
                vec![ResourceExchange {
                    from: Resource::Wood,
                    to: Resource::Food,
                    num_from: 1,
                    num_to: 2,
                }]
            }
            Self::Pottery => {
                vec![ResourceExchange {
                    from: Resource::Clay,
                    to: Resource::Food,
                    num_from: 1,
                    num_to: 2,
                }]
            }
            Self::BasketmakersWorkshop => {
                vec![ResourceExchange {
                    from: Resource::Reed,
                    to: Resource::Food,
                    num_from: 1,
                    num_to: 3,
                }]
            }
            _ => vec![],
        }
    }

    pub fn points(&self) -> u32 {
        match self {
            Self::Fireplace(_) | Self::CookingHearth(_) => 1,
            Self::ClayOven | Self::Joinery | Self::Pottery | Self::BasketmakersWorkshop => 2,
            Self::StoneOven => 3,
            Self::Well => 4,
        }
    }

    pub fn cost(&self) -> Resources {
        match self {
            Self::Fireplace(cheaper) => {
                let mut res = new_res();
                res[Resource::Clay] = if *cheaper { 2 } else { 3 };
                res
            }
            Self::CookingHearth(cheaper) => {
                let mut res = new_res();
                res[Resource::Clay] = if *cheaper { 4 } else { 5 };
                res
            }
            Self::Well => {
                let mut res = new_res();
                res[Resource::Wood] = 1;
                res[Resource::Stone] = 3;
                res
            }
            Self::ClayOven => {
                let mut res = new_res();
                res[Resource::Clay] = 3;
                res[Resource::Stone] = 1;
                res
            }
            Self::StoneOven => {
                let mut res = new_res();
                res[Resource::Clay] = 1;
                res[Resource::Stone] = 3;
                res
            }
            Self::Joinery => {
                let mut res = new_res();
                res[Resource::Wood] = 2;
                res[Resource::Stone] = 2;
                res
            }
            Self::Pottery => {
                let mut res = new_res();
                res[Resource::Clay] = 2;
                res[Resource::Stone] = 2;
                res
            }
            Self::BasketmakersWorkshop => {
                let mut res = new_res();
                res[Resource::Reed] = 2;
                res[Resource::Stone] = 2;
                res
            }
        }
    }

    pub fn display(cards: &Vec<Self>) -> String {
        let mut ret = String::new();
        for card in cards {
            ret = format!("{ret}\n{card:?}");
        }
        ret
    }
}
