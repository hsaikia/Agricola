use super::flag::{ClayHouse, Flag};
use crate::agricola::{fencing::MAX_PASTURES, flag::WoodHouse};

use super::{
    card::CARD_NAMES,
    farm::{FarmyardSpace, L, W},
    quantity::{AdultMembers, Children, Quantity, Resources, NUM_RESOURCES},
    state::State,
};

pub const RESOURCE_EMOJIS: [&str; NUM_RESOURCES] = [
    "\u{1f372}",
    "\u{1fab5}",
    "\u{1f9f1}",
    "\u{1faa8}",
    "\u{1f344}",
    "\u{1f33e}",
    "🎃",
    "\u{1f411}",
    "\u{1f416}",
    "\u{1f404}",
];

#[must_use]
pub fn format_resources(resource: &Resources) -> String {
    let mut ret = String::new();
    let available = resource.iter().enumerate().filter(|&(_, x)| x > &0);
    for (i, n) in available {
        if !ret.is_empty() {
            ret.push_str(" +");
        }
        ret.push_str(&format!(" {}{}", n, RESOURCE_EMOJIS[i]));
    }
    ret
}

#[must_use]
pub fn print_resources(state: &State, player_idx: usize) -> String {
    let resource = state.player_quantities(player_idx);
    let mut ret = String::from("\n");

    for (i, num_res) in resource.iter().take(NUM_RESOURCES).enumerate() {
        if num_res == &0 {
            continue;
        }
        ret.push_str(&format!(
            "\n{:2} {}",
            num_res,
            RESOURCE_EMOJIS[i].repeat(*num_res)
        ));
    }

    ret.push_str(&format!(
        "\n{:2} 👤",
        state.player_quantities(player_idx)[AdultMembers.index()]
    ));
    if state.player_quantities(player_idx)[Children.index()] > 0 {
        ret.push_str(&format!(
            "\n{:2} 👶",
            state.player_quantities(player_idx)[Children.index()]
        ));
    }

    for (i, card) in state.player_cards(player_idx).iter().enumerate() {
        if *card {
            ret.push_str(&format!("\n{}", CARD_NAMES[i]));
        }
    }
    ret
}

#[must_use]
pub fn print_farm(state: &State, player_idx: usize) -> String {
    const PASTURE_EMOJIS: [[&str; MAX_PASTURES]; 2] = [
        ["[p1]", "[p2]", "[p3]", "[p4]"],
        ["[P1]", "[P2]", "[P3]", "[P4]"],
    ];
    let mut ret = String::from("\n\n\n");

    let room_emoji = if state.player_flags(player_idx)[WoodHouse.index()] {
        "[WR]"
    } else if state.player_flags(player_idx)[ClayHouse.index()] {
        "[CR]"
    } else {
        "[SR]"
    };

    for i in 0..W {
        for j in 0..L {
            let idx = i * L + j;
            match state.player_farm(player_idx).farmyard_spaces[idx] {
                FarmyardSpace::Empty => {
                    ret.push_str("[--]");
                }
                FarmyardSpace::Room => {
                    ret.push_str(room_emoji);
                }
                FarmyardSpace::Field(_) => {
                    ret.push_str("[FF]");
                }
                FarmyardSpace::FencedPasture(stable, pasture_idx) => {
                    if stable {
                        ret.push_str(PASTURE_EMOJIS[1][pasture_idx]);
                    } else {
                        ret.push_str(PASTURE_EMOJIS[0][pasture_idx]);
                    }
                }
                FarmyardSpace::UnfencedStable => {
                    ret.push_str("[us]");
                }
            }
        }
        ret.push('\n');
    }

    ret
}
