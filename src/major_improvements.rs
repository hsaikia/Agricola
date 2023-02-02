use crate::primitives::Resources;

#[derive(Clone)]
pub enum MajorImprovementType {
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

pub struct MajorImprovement {
    name: String,
    imp_type: MajorImprovementType,
    cost: Resources,
    points: u32,
}

impl MajorImprovement {
    pub fn create_new(
        p_name: &str,
        p_type: MajorImprovementType,
        p_cost: Resources,
        p_points: u32,
    ) -> Self {
        MajorImprovement {
            name: String::from(p_name),
            imp_type: p_type,
            cost: p_cost,
            points: p_points,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn cost(&self) -> &Resources {
        &self.cost
    }

    pub fn major_type(&self) -> MajorImprovementType {
        self.imp_type.clone()
    }

    pub fn points(&self) -> u32 {
        self.points
    }
}
