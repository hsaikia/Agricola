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
pub struct CanRenovate;
pub struct CanBuildRoom;
pub struct CanBuildStable;
pub struct BakedOnceWithClayOven;
pub struct BakedOnceWithStoneOven;
pub struct BakedTwiceWithStoneOven;
pub struct UsedJoinery;
pub struct UsedPottery;
pub struct UsedBasketmakersWorkshop;

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

impl Flag for CanRenovate {
    fn index(&self) -> usize {
        8
    }
}

impl Flag for CanBuildRoom {
    fn index(&self) -> usize {
        9
    }
}

impl Flag for CanBuildStable {
    fn index(&self) -> usize {
        10
    }
}

impl Flag for BakedOnceWithClayOven {
    fn index(&self) -> usize {
        11
    }
}

impl Flag for BakedOnceWithStoneOven {
    fn index(&self) -> usize {
        12
    }
}

impl Flag for BakedTwiceWithStoneOven {
    fn index(&self) -> usize {
        13
    }
}

impl Flag for UsedJoinery {
    fn index(&self) -> usize {
        14
    }
}

impl Flag for UsedPottery {
    fn index(&self) -> usize {
        15
    }
}

impl Flag for UsedBasketmakersWorkshop {
    fn index(&self) -> usize {
        16
    }
}

pub const NUM_FLAGS: usize = 17;
