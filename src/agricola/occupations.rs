#[derive(Debug, Clone, Hash, PartialEq)]
pub enum Occupation {
    AssistantTiller,
    Childless,
}

impl Occupation {
    pub fn all() -> Vec<Self> {
        vec![Self::AssistantTiller, Self::Childless]
    }

    pub fn display(cards: &Vec<Self>) -> String {
        let mut ret = String::new();
        for card in cards {
            ret = format!("{ret}\n{card:?}");
        }
        ret
    }
}
