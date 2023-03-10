use std::cmp;
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
pub type Resources = [u32; NUM_RESOURCES];

pub fn new_res() -> Resources {
    [0; NUM_RESOURCES]
}

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
    let available = res.iter().enumerate().filter(|&(_, x)| x > &0);
    for (i, n) in available {
        print!("[{}]", RESOURCE_NAMES[i].repeat(*n as usize));
    }
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
pub enum ConversionTime {
    Any,
    Harvest,
    Bake,
}

#[derive(Debug, Clone, Hash)]
pub struct ResourceConversion {
    from: Resource,
    to: Resource,
    from_amt: u32,
    to_amt: u32,
    times: u32,
    used: u32,
    conv_time: ConversionTime,
}

impl ResourceConversion {
    pub fn food_conversion(
        p_from: Resource,
        p_from_amt: u32,
        p_to_amt: u32,
        p_times: u32,
        p_conv_time: ConversionTime,
    ) -> Self {
        ResourceConversion {
            from: p_from,
            to: Resource::Food,
            from_amt: p_from_amt,
            to_amt: p_to_amt,
            times: p_times,
            used: 0,
            conv_time: p_conv_time,
        }
    }

    pub fn default_conversions() -> Vec<Self> {
        vec![
            Self::food_conversion(Resource::Grain, 1, 1, u32::MAX, ConversionTime::Any),
            Self::food_conversion(Resource::Vegetable, 1, 1, u32::MAX, ConversionTime::Any),
        ]
    }

    pub fn default_grain_conversions() -> Vec<Self> {
        vec![Self::food_conversion(
            Resource::Grain,
            1,
            1,
            u32::MAX,
            ConversionTime::Any,
        )]
    }

    pub fn conversion_options(
        &self,
        res: &Resources,
        conv_time: &ConversionTime,
    ) -> Option<(Resource, u32, u32)> {
        if self.can_convert(res, conv_time) {
            return Some((
                self.from.clone(),
                self.to_amt,
                cmp::min(self.times, res[self.from.clone()]),
            ));
        }
        None
    }

    pub fn can_convert(&self, res: &Resources, conv_time: &ConversionTime) -> bool {
        match conv_time {
            ConversionTime::Harvest | ConversionTime::Any => {
                if let ConversionTime::Bake = self.conv_time {
                    return false;
                }
            }
            ConversionTime::Bake => match self.conv_time {
                ConversionTime::Harvest | ConversionTime::Any => return false,
                ConversionTime::Bake => (),
            },
        }
        self.used < self.times && res[self.from.clone()] >= self.from_amt
    }

    pub fn convert_all(&mut self, res: &mut Resources, conv_time: &ConversionTime) {
        while self.can_convert(res, conv_time) {
            self.convert_once(res, conv_time);
        }
    }

    pub fn convert_once(&mut self, res: &mut Resources, conv_time: &ConversionTime) {
        if self.can_convert(res, conv_time) {
            res[self.from.clone()] -= self.from_amt;
            res[self.to.clone()] += self.to_amt;
            self.used += 1;
        }
    }

    pub fn reset(&mut self) {
        self.used = 0;
    }
}
