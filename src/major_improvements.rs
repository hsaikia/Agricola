use crate::primitives::{
    can_pay_for_resource, new_res, ConversionTime, Resource, ResourceConversion, Resources,
    MAX_RESOURCE_TO_CONVERT,
};

#[derive(Clone, Debug)]
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

pub const ALL_MAJORS: [MajorImprovement; 10] = [
    MajorImprovement::Fireplace2,
    MajorImprovement::Fireplace3,
    MajorImprovement::CookingHearth4,
    MajorImprovement::CookingHearth5,
    MajorImprovement::Well,
    MajorImprovement::ClayOven,
    MajorImprovement::StoneOven,
    MajorImprovement::Joinery,
    MajorImprovement::Pottery,
    MajorImprovement::BasketmakersWorkshop,
];

impl MajorImprovement {
    pub fn available_majors_to_build(
        majors_owned: &[bool; ALL_MAJORS.len()],
        majors_on_board: &[bool; ALL_MAJORS.len()],
        resources: &Resources,
    ) -> Vec<Vec<usize>> {
        // If one of the FPs are already built
        let fp2_built: bool = majors_owned[MajorImprovement::Fireplace2.index()];
        let fp3_built: bool = majors_owned[MajorImprovement::Fireplace3.index()];

        let mut available = [false; ALL_MAJORS.len()];

        for idx in 0..ALL_MAJORS.len() {
            if !majors_on_board[idx] {
                continue;
            }

            // If FP2 or FP3 is already built
            if fp2_built || fp3_built {
                if idx == MajorImprovement::CookingHearth4.index()
                    || idx == MajorImprovement::CookingHearth5.index()
                {
                    available[idx] = true;
                }

                if idx == MajorImprovement::Fireplace2.index()
                    || idx == MajorImprovement::Fireplace3.index()
                {
                    continue;
                }
            }
            if can_pay_for_resource(&ALL_MAJORS[idx].cost(), resources) {
                available[idx] = true;
            }
        }

        // If CH4 and CH5 are both present remove the expensive one
        // Or if CH4 is already built remove CH5
        if (available[MajorImprovement::CookingHearth4.index()]
            && available[MajorImprovement::CookingHearth5.index()])
            || majors_owned[MajorImprovement::CookingHearth4.index()]
        {
            available[MajorImprovement::CookingHearth5.index()] = false;
        }

        // If FP2 and FP3 are both present remove the expensive one
        if available[MajorImprovement::Fireplace2.index()]
            && available[MajorImprovement::Fireplace3.index()]
        {
            available[MajorImprovement::Fireplace3.index()] = false;
        }

        // Populate indices of all available majors
        let mut available_indices = vec![];

        for (i, e) in available.iter().enumerate() {
            if *e {
                available_indices.push(vec![i]);
            }
        }

        available_indices
    }

    // Vec <Resource to use, Amount to use, Food for each resource, Number of times usable at once>
    pub fn conversions_to_food(&self) -> Option<Vec<ResourceConversion>> {
        match self {
            Self::Fireplace2 | Self::Fireplace3 => Some(vec![
                ResourceConversion::food_conversion(
                    Resource::Sheep,
                    1,
                    2,
                    MAX_RESOURCE_TO_CONVERT,
                    ConversionTime::Any,
                ),
                ResourceConversion::food_conversion(
                    Resource::Pigs,
                    1,
                    2,
                    MAX_RESOURCE_TO_CONVERT,
                    ConversionTime::Any,
                ),
                ResourceConversion::food_conversion(
                    Resource::Vegetable,
                    1,
                    2,
                    MAX_RESOURCE_TO_CONVERT,
                    ConversionTime::Any,
                ),
                ResourceConversion::food_conversion(
                    Resource::Cattle,
                    1,
                    3,
                    MAX_RESOURCE_TO_CONVERT,
                    ConversionTime::Any,
                ),
                ResourceConversion::food_conversion(Resource::Grain, 1, 2, 1, ConversionTime::Bake),
            ]),
            Self::CookingHearth4 | Self::CookingHearth5 => Some(vec![
                ResourceConversion::food_conversion(
                    Resource::Sheep,
                    1,
                    2,
                    MAX_RESOURCE_TO_CONVERT,
                    ConversionTime::Any,
                ),
                ResourceConversion::food_conversion(
                    Resource::Pigs,
                    1,
                    3,
                    MAX_RESOURCE_TO_CONVERT,
                    ConversionTime::Any,
                ),
                ResourceConversion::food_conversion(
                    Resource::Vegetable,
                    1,
                    3,
                    MAX_RESOURCE_TO_CONVERT,
                    ConversionTime::Any,
                ),
                ResourceConversion::food_conversion(
                    Resource::Cattle,
                    1,
                    4,
                    MAX_RESOURCE_TO_CONVERT,
                    ConversionTime::Any,
                ),
                ResourceConversion::food_conversion(Resource::Grain, 1, 3, 1, ConversionTime::Bake),
            ]),
            Self::ClayOven => Some(vec![ResourceConversion::food_conversion(
                Resource::Grain,
                1,
                5,
                1,
                ConversionTime::Bake,
            )]),
            Self::StoneOven => Some(vec![ResourceConversion::food_conversion(
                Resource::Grain,
                1,
                4,
                2,
                ConversionTime::Bake,
            )]),
            Self::Joinery => Some(vec![ResourceConversion::food_conversion(
                Resource::Wood,
                1,
                2,
                1,
                ConversionTime::Harvest,
            )]),
            Self::Pottery => Some(vec![ResourceConversion::food_conversion(
                Resource::Clay,
                1,
                2,
                1,
                ConversionTime::Harvest,
            )]),
            Self::BasketmakersWorkshop => Some(vec![ResourceConversion::food_conversion(
                Resource::Reed,
                1,
                3,
                1,
                ConversionTime::Harvest,
            )]),
            _ => None,
        }
    }

    pub fn points(&self) -> u32 {
        match self {
            Self::Fireplace2 => 1,
            Self::Fireplace3 => 1,
            Self::CookingHearth4 => 1,
            Self::CookingHearth5 => 1,
            Self::Well => 4,
            Self::ClayOven => 2,
            Self::StoneOven => 3,
            Self::Joinery => 2,
            Self::Pottery => 2,
            Self::BasketmakersWorkshop => 2,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Fireplace2 => "Fireplace (2)",
            Self::Fireplace3 => "Fireplace (3)",
            Self::CookingHearth4 => "Cooking Hearth (4)",
            Self::CookingHearth5 => "Cooking Hearth (5)",
            Self::Well => "Well",
            Self::ClayOven => "Clay Oven",
            Self::StoneOven => "Stone Oven",
            Self::Joinery => "Joinery",
            Self::Pottery => "Pottery",
            Self::BasketmakersWorkshop => "Basketmaker's Workshop",
        }
    }

    pub fn index(&self) -> usize {
        match self {
            Self::Fireplace2 => 0,
            Self::Fireplace3 => 1,
            Self::CookingHearth4 => 2,
            Self::CookingHearth5 => 3,
            Self::Well => 4,
            Self::ClayOven => 5,
            Self::StoneOven => 6,
            Self::Joinery => 7,
            Self::Pottery => 8,
            Self::BasketmakersWorkshop => 9,
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
}
