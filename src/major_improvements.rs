use crate::primitives::{
    new_res, ConversionTime, Resource, ResourceConversion, Resources, MAX_RESOURCE_TO_CONVERT,
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
