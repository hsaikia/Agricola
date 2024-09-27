pub trait Quantity {
    fn index(&self) -> usize;
}

pub struct Food;
pub struct Wood;
pub struct Clay;
pub struct Stone;
pub struct Reed;
pub struct Grain;
pub struct Vegetable;
pub struct Sheep;
pub struct Boar;
pub struct Cattle;
pub struct AdultMembers;
pub struct Children;
pub struct MembersPlacedThisRound;
pub struct Rooms;
pub struct Fields;
pub struct Pastures;
pub struct PastureSpaces;
pub struct UnfencedStables;
pub struct FencedStables;
pub struct BeggingTokens;

impl Quantity for Food {
    fn index(&self) -> usize {
        0
    }
}

impl Quantity for Wood {
    fn index(&self) -> usize {
        1
    }
}

impl Quantity for Clay {
    fn index(&self) -> usize {
        2
    }
}

impl Quantity for Stone {
    fn index(&self) -> usize {
        3
    }
}

impl Quantity for Reed {
    fn index(&self) -> usize {
        4
    }
}

impl Quantity for Grain {
    fn index(&self) -> usize {
        5
    }
}

impl Quantity for Vegetable {
    fn index(&self) -> usize {
        6
    }
}

impl Quantity for Sheep {
    fn index(&self) -> usize {
        7
    }
}

impl Quantity for Boar {
    fn index(&self) -> usize {
        8
    }
}

impl Quantity for Cattle {
    fn index(&self) -> usize {
        9
    }
}

impl Quantity for AdultMembers {
    fn index(&self) -> usize {
        10
    }
}

impl Quantity for Children {
    fn index(&self) -> usize {
        11
    }
}

impl Quantity for MembersPlacedThisRound {
    fn index(&self) -> usize {
        12
    }
}

impl Quantity for Rooms {
    fn index(&self) -> usize {
        13
    }
}

impl Quantity for Fields {
    fn index(&self) -> usize {
        14
    }
}

impl Quantity for Pastures {
    fn index(&self) -> usize {
        15
    }
}

impl Quantity for PastureSpaces {
    fn index(&self) -> usize {
        16
    }
}

impl Quantity for UnfencedStables {
    fn index(&self) -> usize {
        17
    }
}

impl Quantity for FencedStables {
    fn index(&self) -> usize {
        18
    }
}

impl Quantity for BeggingTokens {
    fn index(&self) -> usize {
        19
    }
}

pub const NUM_QUANTITIES: usize = 20;
pub const NUM_RESOURCES: usize = 10;

pub type Resources = [usize; NUM_RESOURCES];
pub type Quantities = [usize; NUM_QUANTITIES];

pub trait QuantitiesImpl {
    fn zero_resources(&mut self);
    fn get_resources(&self) -> Resources;
}

impl QuantitiesImpl for Quantities {
    fn zero_resources(&mut self) {
        for res in self.iter_mut().take(NUM_RESOURCES) {
            *res = 0;
        }
    }

    fn get_resources(&self) -> Resources {
        let mut resources = [0; NUM_RESOURCES];
        resources[..NUM_RESOURCES].copy_from_slice(&self[..NUM_RESOURCES]);
        resources
    }
}

pub fn new_res() -> Resources {
    [0; NUM_RESOURCES]
}

pub fn can_pay_for_resource(cost: &Resources, store: &Quantities) -> bool {
    for it in cost.iter().zip(store.iter()).take(NUM_RESOURCES) {
        let (a, b) = it;
        if a > b {
            return false;
        }
    }
    true
}

pub fn pay_for_resource(cost: &Resources, store: &mut Quantities) {
    assert!(can_pay_for_resource(cost, store));
    for it in cost.iter().zip(store.iter_mut()).take(NUM_RESOURCES) {
        let (a, b) = it;
        *b -= a;
    }
}

pub fn take_resource(res: &Resources, store: &mut Quantities) {
    for it in res.iter().zip(store.iter_mut()).take(NUM_RESOURCES) {
        let (a, b) = it;
        *b += a;
    }
}

#[derive(Debug, Clone, Hash)]
pub struct ResourceExchange {
    pub from: usize,
    pub to: usize,
    pub num_from: usize,
    pub num_to: usize,
}
