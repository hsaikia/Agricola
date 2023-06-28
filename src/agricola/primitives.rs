use std::ops::{Index, IndexMut};

#[derive(Clone, Debug, Hash)]
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

const NUM_RESOURCES: usize = 10;

pub const RESOURCE_NAMES: [&str; NUM_RESOURCES] = [
    "FOOD",
    "WOOD",
    "CLAY",
    "STONE",
    "REED",
    "GRAIN",
    "VEGETABLE",
    "SHEEP",
    "BOAR",
    "CATTLE",
];

pub const RESOURCE_EMOJIS: [&str; NUM_RESOURCES] = [
    "\u{1f372}",
    "\u{1fab5}",
    "\u{1f9f1}",
    "\u{1faa8}",
    "\u{1f344}",
    "\u{1f33e}",
    "\u{1f966}",
    "\u{1f411}",
    "\u{1f416}",
    "\u{1f404}",
];

pub type Resources = [usize; NUM_RESOURCES];

pub fn new_res() -> Resources {
    [0; NUM_RESOURCES]
}

impl Index<Resource> for Resources {
    type Output = usize;
    fn index(&self, res: Resource) -> &Self::Output {
        &self[res as usize]
    }
}

impl IndexMut<Resource> for Resources {
    fn index_mut(&mut self, res: Resource) -> &mut usize {
        &mut self[res as usize]
    }
}

pub fn print_resources(res: &Resources) {
    let available = res.iter().enumerate().filter(|&(_, x)| x > &0);
    for (i, n) in available {
        print!("[{}]", RESOURCE_EMOJIS[i].repeat(*n));
        //print!("[{} {}]", n, RESOURCE_NAMES[i]);
    }
}

pub fn format_resources(res: &Resources) -> String {
    let mut ret: String = String::new();
    let available = res.iter().enumerate().filter(|&(_, x)| x > &0);
    for (i, n) in available {
        if !ret.is_empty() {
            ret = format!("{} + ", ret);
        }
        ret = format!("{} {}{}", ret, n, RESOURCE_NAMES[i]);
    }
    ret
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
    pub from: Resource,
    pub to: Resource,
    pub num_from: usize,
    pub num_to: usize,
}
