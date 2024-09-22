#[derive(Debug, Clone, Hash, PartialEq)]
pub enum Occupation {
    AssistantTiller,
    Childless,
}

impl Occupation {
    pub fn all() -> Vec<Self> {
        vec![Self::AssistantTiller, Self::Childless]
    }
}
