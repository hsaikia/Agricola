use super::{
    card::CARD_NAMES,
    farm::{Farm, FarmyardSpace, L, W},
    fencing::MAX_PASTURES,
    quantity::{
        AdultMembers, Children, Clay, Grain, Quantity, Resources, Stone, Vegetable, Wood,
        NUM_RESOURCES,
    },
    state::State,
};

pub const RESOURCE_EMOJIS: [&str; NUM_RESOURCES] = [
    "\u{1fab5}",
    "\u{1f9f1}",
    "\u{1faa8}",
    "\u{1f372}",
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
    let unharvested_grain_and_veg = state.grain_and_veg_on_fields(player_idx);
    let mut ret = String::from("\n");

    for (i, num_res) in resource.iter().take(NUM_RESOURCES).enumerate() {
        let mut extra = 0;
        if i == Grain.index() {
            extra = unharvested_grain_and_veg.0;
        }

        if i == Vegetable.index() {
            extra = unharvested_grain_and_veg.1;
        }

        if num_res == &0 && extra == 0 {
            continue;
        }

        if extra > 0 {
            ret.push_str(&format!(
                "\n{:2} {} + {:2} {}",
                num_res,
                RESOURCE_EMOJIS[i].repeat(*num_res),
                extra,
                RESOURCE_EMOJIS[i].repeat(extra)
            ));
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

/// # Panics
/// If `room_material_index` is not one of the valid indices
#[must_use]
pub fn print_farm(farm: &Farm, room_material_index: usize) -> String {
    const PASTURE_EMOJIS: [[&str; MAX_PASTURES]; 2] = [
        ["[p1]", "[p2]", "[p3]", "[p4]"],
        ["[P1]", "[P2]", "[P3]", "[P4]"],
    ];
    let mut ret = String::from("\n\n\n");

    let room_emoji = if room_material_index == Wood.index() {
        "[WR]"
    } else if room_material_index == Clay.index() {
        "[CR]"
    } else if room_material_index == Stone.index() {
        "[SR]"
    } else {
        panic!("Invalid room material index");
    };

    for i in 0..W {
        for j in 0..L {
            let idx = i * L + j;
            match farm.farmyard_spaces[idx] {
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
