use crate::primitives::{can_pay_for_resource, new_res, Resource, ResourceExchange, Resources};

#[derive(Clone, Debug, Hash, PartialEq)]
pub enum MajorImprovement {
    Fireplace2,
    Fireplace3,
    CookingHearth4,
    CookingHearth5,
    Well,
    ClayOven,
    StoneOven,
    Joinery,
    Pottery,
    BasketmakersWorkshop,
}

impl MajorImprovement {
    pub fn can_build_major(
        majors_owned: &Vec<Self>,
        majors_on_board: &Vec<Self>,
        resources: &Resources,
    ) -> bool {
        let fp2_built: bool = majors_owned.contains(&Self::Fireplace2);
        let fp3_built: bool = majors_owned.contains(&Self::Fireplace3);
        let ch4_built: bool = majors_owned.contains(&Self::CookingHearth4);
        let ch5_built: bool = majors_owned.contains(&Self::CookingHearth5);

        for card in majors_on_board {
            match card {
                Self::Fireplace2 => {
                    if !fp3_built && can_pay_for_resource(&card.cost(), resources) {
                        return true;
                    }
                }
                Self::Fireplace3 => {
                    if !fp2_built && can_pay_for_resource(&card.cost(), resources) {
                        return true;
                    }
                }
                Self::CookingHearth4 => {
                    if (fp2_built || fp3_built || can_pay_for_resource(&card.cost(), resources))
                        && !ch5_built
                    {
                        return true;
                    }
                }
                Self::CookingHearth5 => {
                    if (fp2_built || fp3_built || can_pay_for_resource(&card.cost(), resources))
                        && !ch4_built
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
        majors_owned: &Vec<Self>,
        majors_on_board: &Vec<Self>,
        resources: &Resources,
    ) -> Vec<Self> {
        let fp2_built: bool = majors_owned.contains(&Self::Fireplace2);
        let fp3_built: bool = majors_owned.contains(&Self::Fireplace3);
        let ch4_built: bool = majors_owned.contains(&Self::CookingHearth4);
        let ch5_built: bool = majors_owned.contains(&Self::CookingHearth5);

        let mut available: Vec<Self> = vec![];

        for card in majors_on_board {
            match card {
                Self::Fireplace2 => {
                    if !fp3_built && can_pay_for_resource(&card.cost(), resources) {
                        available.push(card.clone());
                    }
                }
                Self::Fireplace3 => {
                    if !fp2_built && can_pay_for_resource(&card.cost(), resources) {
                        available.push(card.clone());
                    }
                }
                Self::CookingHearth4 => {
                    if (fp2_built || fp3_built || can_pay_for_resource(&card.cost(), resources))
                        && !ch5_built
                    {
                        available.push(card.clone());
                    }
                }
                Self::CookingHearth5 => {
                    if (fp2_built || fp3_built || can_pay_for_resource(&card.cost(), resources))
                        && !ch4_built
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

    pub fn exchanges(&self, used: &Vec<Self>) -> Option<Vec<ResourceExchange>> {
        match self {
            Self::Fireplace2 | Self::Fireplace3 => {
                let mut ret: Vec<ResourceExchange> = vec![];
                ret.push(ResourceExchange {
                    from: Resource::Sheep,
                    to: Resource::Food,
                    num_from: 1,
                    num_to: 2,
                });
                ret.push(ResourceExchange {
                    from: Resource::Pigs,
                    to: Resource::Food,
                    num_from: 1,
                    num_to: 2,
                });
                ret.push(ResourceExchange {
                    from: Resource::Vegetable,
                    to: Resource::Food,
                    num_from: 1,
                    num_to: 2,
                });
                ret.push(ResourceExchange {
                    from: Resource::Cattle,
                    to: Resource::Food,
                    num_from: 1,
                    num_to: 3,
                });
                Some(ret)
            }
            Self::CookingHearth4 | Self::CookingHearth5 => {
                let mut ret: Vec<ResourceExchange> = vec![];
                ret.push(ResourceExchange {
                    from: Resource::Sheep,
                    to: Resource::Food,
                    num_from: 1,
                    num_to: 2,
                });
                ret.push(ResourceExchange {
                    from: Resource::Pigs,
                    to: Resource::Food,
                    num_from: 1,
                    num_to: 3,
                });
                ret.push(ResourceExchange {
                    from: Resource::Vegetable,
                    to: Resource::Food,
                    num_from: 1,
                    num_to: 3,
                });
                ret.push(ResourceExchange {
                    from: Resource::Cattle,
                    to: Resource::Food,
                    num_from: 1,
                    num_to: 4,
                });
                Some(ret)
            }
            Self::Joinery => {
                let mut ret: Vec<ResourceExchange> = vec![];
                if !used.contains(&self) {
                    ret.push(ResourceExchange {
                        from: Resource::Wood,
                        to: Resource::Food,
                        num_from: 1,
                        num_to: 2,
                    });
                }
                Some(ret)
            }
            Self::Pottery => {
                let mut ret: Vec<ResourceExchange> = vec![];
                if !used.contains(&self) {
                    ret.push(ResourceExchange {
                        from: Resource::Clay,
                        to: Resource::Food,
                        num_from: 1,
                        num_to: 2,
                    });
                }
                Some(ret)
            }
            Self::BasketmakersWorkshop => {
                let mut ret: Vec<ResourceExchange> = vec![];
                if !used.contains(&self) {
                    ret.push(ResourceExchange {
                        from: Resource::Reed,
                        to: Resource::Food,
                        num_from: 1,
                        num_to: 3,
                    });
                }
                Some(ret)
            }
            _ => None,
        }
    }

    pub fn points(&self) -> u32 {
        match self {
            Self::Fireplace2 | Self::Fireplace3 | Self::CookingHearth4 | Self::CookingHearth5 => 1,
            Self::ClayOven | Self::Joinery | Self::Pottery | Self::BasketmakersWorkshop => 2,
            Self::StoneOven => 3,
            Self::Well => 4,
        }
    }

    pub fn cost(&self) -> Resources {
        match self {
            Self::Fireplace2 => {
                let mut res = new_res();
                res[Resource::Clay] = 2;
                res
            }
            Self::Fireplace3 => {
                let mut res = new_res();
                res[Resource::Clay] = 3;
                res
            }
            Self::CookingHearth4 => {
                let mut res = new_res();
                res[Resource::Clay] = 4;
                res
            }
            Self::CookingHearth5 => {
                let mut res = new_res();
                res[Resource::Clay] = 5;
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

    pub fn display(majors: &Vec<Self>) {
        for major in majors {
            print!("[{:?}]", major);
        }
    }
}
