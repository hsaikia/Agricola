use std::ops::{Index, IndexMut};

pub const MAX_RESOURCE_TO_CONVERT: u32 = 1000; // Large enough number to simulate infinity

#[derive(Clone, Debug)]
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
const RESOURCE_NAMES: [&str; NUM_RESOURCES] = [
    "Fd", "Wd", "Cl", "St", "Rd", "Gr", "Vg", "Sheep", "Pig", "Cow",
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
    for i in 0..NUM_RESOURCES {
        if res[i] > 0 {
            print!("[{} {}]", res[i], RESOURCE_NAMES[i]);
        }
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

#[derive(Debug)]
pub enum ConversionTime {
    Any,
    Harvest,
    Bake,
}

#[derive(Debug)]
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
            Self::food_conversion(
                Resource::Grain,
                1,
                1,
                MAX_RESOURCE_TO_CONVERT,
                ConversionTime::Any,
            ),
            Self::food_conversion(
                Resource::Vegetable,
                1,
                1,
                MAX_RESOURCE_TO_CONVERT,
                ConversionTime::Any,
            ),
        ]
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
                _ => (),
            },
        }
        self.used < self.times && res[self.from.clone()] >= self.from_amt
    }

    pub fn convert_all(&mut self, res: &mut Resources, conv_time: &ConversionTime) {
        while self.can_convert(res, conv_time) {
            self.convert_once(res, conv_time)
        }
    }

    pub fn convert_once(&mut self, res: &mut Resources, conv_time: &ConversionTime) {
        if self.can_convert(res, conv_time) {
            print!("\nConverting a {:?} to {} Food.", &self.from, self.to_amt);
            res[self.from.clone()] -= self.from_amt;
            res[self.to.clone()] += self.to_amt;
            self.used += 1;
        }
    }

    pub fn reset(&mut self) {
        self.used = 0;
    }
}
