use std::fmt::Debug;
use std::hash::Hash;

pub trait Resource: Debug {
    fn index(&self) -> usize;
}

#[derive(Clone, Debug, Hash)]
pub struct Food;

#[derive(Clone, Debug, Hash)]
pub struct Wood;

#[derive(Clone, Debug, Hash)]
pub struct Clay;

#[derive(Clone, Debug, Hash)]
pub struct Stone;

#[derive(Clone, Debug, Hash)]
pub struct Reed;

#[derive(Clone, Debug, Hash)]
pub struct Grain;

#[derive(Clone, Debug, Hash)]
pub struct Vegetable;

#[derive(Clone, Debug, Hash)]
pub struct Sheep;

#[derive(Clone, Debug, Hash)]
pub struct Boar;

#[derive(Clone, Debug, Hash)]
pub struct Cattle;

pub const NUM_RESOURCES: usize = 10;

impl Resource for Food {
    fn index(&self) -> usize {
        0
    }
}

impl Resource for Wood {
    fn index(&self) -> usize {
        1
    }
}

impl Resource for Clay {
    fn index(&self) -> usize {
        2
    }
}

impl Resource for Stone {
    fn index(&self) -> usize {
        3
    }
}

impl Resource for Reed {
    fn index(&self) -> usize {
        4
    }
}

impl Resource for Grain {
    fn index(&self) -> usize {
        5
    }
}

impl Resource for Vegetable {
    fn index(&self) -> usize {
        6
    }
}

impl Resource for Sheep {
    fn index(&self) -> usize {
        7
    }
}

impl Resource for Boar {
    fn index(&self) -> usize {
        8
    }
}

impl Resource for Cattle {
    fn index(&self) -> usize {
        9
    }
}

pub type Resources = [usize; NUM_RESOURCES];

pub fn new_res() -> Resources {
    [0; NUM_RESOURCES]
}

pub fn can_pay_for_resource(cost: &Resources, store: &Resources) -> bool {
    for it in cost.iter().zip(store.iter()) {
        let (a, b) = it;
        if a > b {
            return false;
        }
    }
    true
}

pub fn pay_for_resource(cost: &Resources, store: &mut Resources) {
    assert!(can_pay_for_resource(cost, store));
    for it in cost.iter().zip(store.iter_mut()) {
        let (a, b) = it;
        *b -= a;
    }
}

pub fn take_resource(res: &Resources, store: &mut Resources) {
    for it in res.iter().zip(store.iter_mut()) {
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
