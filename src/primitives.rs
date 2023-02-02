use std::ops::{Index, IndexMut};

pub enum Resource {
    Food,
    Wood,
    Clay,
    Stone,
    Reed,
    Grain,
    Vegetable,
    Sheep,
    Pigs,
    Cattle,
}

pub const NUM_RESOURCES: usize = 10;
const RESOURCE_NAMES: [&str; NUM_RESOURCES] = [
    "Fd", "Wd", "Cl", "St", "Rd", "Gr", "Vg", "Sheep", "Pig", "Cow",
];
pub type Resources = [u32; NUM_RESOURCES];

impl Index<Resource> for Resources {
    type Output = u32;
    fn index(&self, res: Resource) -> &Self::Output {
        &self[res as usize]
    }
}

impl IndexMut<Resource> for Resources {
    fn index_mut(&mut self, res: Resource) -> &mut u32 {
        &mut self[res as usize]
    }
}

pub fn print_resources(res: &Resources) {
    for i in 0..NUM_RESOURCES {
        if res[i] > 0 {
            print!("[{} {}]", res[i], RESOURCE_NAMES[i]);
        }
    }
}

pub fn can_pay_for_resource(cost: &Resources, store: &Resources) -> bool {
    for (i, e) in cost.iter().enumerate().take(NUM_RESOURCES) {
        if e > &store[i] {
            return false;
        }
    }
    true
}

pub fn pay_for_resource(cost: &Resources, store: &mut Resources) {
    assert!(can_pay_for_resource(cost, store));
    for (i, e) in cost.iter().enumerate().take(NUM_RESOURCES) {
        store[i] -= e;
    }
}
