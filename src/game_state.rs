pub trait Action {}

pub trait GameState {
    fn actions(&self) -> Vec<Box<dyn Action>> {
        Vec::new()
    }

    fn apply_action(&mut self, _action: Box<dyn Action>) {}
}
