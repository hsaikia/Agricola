use crate::agricola::fencing::MAX_PASTURES;
use crate::agricola::player::Player;

use super::{farm::{FarmyardSpace, House, L, W}, quantity::*, state::State};

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

pub fn format_resources(res: &Resources) -> String {
    let mut ret = String::new();
    let available = res.iter().enumerate().filter(|&(_, x)| x > &0);
    for (i, n) in available {
        if !ret.is_empty() {
            ret = format!("{} +", ret);
        }
        ret = format!("{} {}{}", ret, n, RESOURCE_EMOJIS[i]);
    }
    ret
}

pub fn display_resources(state: &State, player_idx : usize) -> String {
    let player = &state.players[player_idx];
    let res = &player.resources;
    let mut ret = String::from("\n\n");

    for (i, num_res) in res.iter().enumerate() {
        if num_res == &0 {
            continue;
        }
        ret.push_str(&format!(
            "\n{:2} {:?}",
            num_res,
            RESOURCE_EMOJIS[i].repeat(*num_res)
        ));
    }

    ret.push_str(&format!("\n{:2} 👤", state.player_quantities(player_idx)[AdultMembers.index()]));
    if state.player_quantities(player_idx)[Children.index()] > 0 {
        ret.push_str(&format!("\n{:2} 👶", state.player_quantities(player_idx)[Children.index()]));
    }

    for occ in &player.occupations {
        ret = format!("{ret}\n{occ:?}");
    }
    ret
}

pub fn display_farm(player: &Player) -> String {
    // TODO : Fix these!
    let mut ret = String::from("\n\n\n");

    let room_emoji = match player.house {
        House::Clay => "[CR]",
        House::Stone => "[SR]",
        House::Wood => "[WR]",
    };

    const PASTURE_EMOJIS: [[&str; MAX_PASTURES]; 2] = [
        ["[p1]", "[p2]", "[p3]", "[p4]"],
        ["[P1]", "[P2]", "[P3]", "[P4]"],
    ];

    for i in 0..W {
        for j in 0..L {
            let idx = i * L + j;
            match player.farm.farmyard_spaces[idx] {
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
