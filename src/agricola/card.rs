trait Card {
    fn index(&self) -> usize;
}

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

pub const NUM_CARDS: usize = 12;
