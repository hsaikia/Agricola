#[derive(Debug, Clone, Hash, PartialEq)]
pub enum Occupation {
    AssistantTiller,
    Childless,
}

impl Occupation {
    pub fn all() -> Vec<Self> {
        vec![Self::AssistantTiller, Self::Childless]
    }

    pub fn display(occs: &Vec<Self>) {
        for occ in occs {
            print!("[{occ:?}]");
        }
    }
}
