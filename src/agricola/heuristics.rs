use super::state::State;

pub trait Heuristic {
    fn index(&self) -> usize;
    fn is_valid_state(state: &State) -> bool;
}

pub struct BuildRoomHeuristic;
pub struct GrowFamilyHeuristic;

impl Heuristic for BuildRoomHeuristic {
    fn index(&self) -> usize {
        0
    }

    fn is_valid_state(state: &State) -> bool {
        state.can_build_room()
    }
}

impl Heuristic for GrowFamilyHeuristic {
    fn index(&self) -> usize {
        1
    }

    fn is_valid_state(state: &State) -> bool {
        state.can_grow_family_with_room() || state.can_grow_family_without_room()
    }
}

pub const NUM_HEURISTICS: usize = 2;
