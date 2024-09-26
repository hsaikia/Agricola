pub trait Flag {
    fn index(&self) -> usize;
}

pub struct WoodHouse;
pub struct ClayHouse;
pub struct StoneHouse;
pub struct HasCookingImprovement;
pub struct GotEnoughFood;
pub struct HarvestPaid;
pub struct HasRoomToGrow;
pub struct BeforeRoundStart;

impl Flag for WoodHouse {
    fn index(&self) -> usize {
        0
    }
}

impl Flag for ClayHouse {
    fn index(&self) -> usize {
        1
    }
}

impl Flag for StoneHouse {
    fn index(&self) -> usize {
        2
    }
}

impl Flag for HasCookingImprovement {
    fn index(&self) -> usize {
        3
    }
}

impl Flag for HarvestPaid {
    fn index(&self) -> usize {
        4
    }
}

impl Flag for HasRoomToGrow {
    fn index(&self) -> usize {
        5
    }
}

impl Flag for GotEnoughFood {
    fn index(&self) -> usize {
        6
    }
}

impl Flag for BeforeRoundStart {
    fn index(&self) -> usize {
        7
    }
}

pub const NUM_FLAGS: usize = 8;
