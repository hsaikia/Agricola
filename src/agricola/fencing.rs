use std::collections::{HashMap, VecDeque};

use super::farm::FarmyardSpace;

const A: usize = 5;
const B: usize = 3;
const NUM_FARMYARD_SPACES: usize = A * B;
const MAX_FENCES: usize = 15;
const ROOM_INDICES: [usize; 2] = [5, 10];

// 5 pastures actually decrease total capacity while needing more wood. Plus score is (by default, without any bonuses) capped at 4 pastures
// If some card allows additional bonuses for 5 pastures, set this to 5 (more than 5 pastures are impossible with 15 fences).
const MAX_PASTURES: usize = 4;

// Farmyard spaces
// 00 01 02 03 04
// 05 06 07 08 09
// 10 11 12 13 14

// Order : NEWS
const NEIGHBOR_SPACES: [[Option<usize>; 4]; NUM_FARMYARD_SPACES] = [
    [None, Some(1), None, Some(5)],
    [None, Some(2), Some(0), Some(6)],
    [None, Some(3), Some(1), Some(7)],
    [None, Some(4), Some(2), Some(8)],
    [None, None, Some(3), Some(9)],
    [Some(0), Some(6), None, Some(10)],
    [Some(1), Some(7), Some(5), Some(11)],
    [Some(2), Some(8), Some(6), Some(12)],
    [Some(3), Some(9), Some(7), Some(13)],
    [Some(4), None, Some(8), Some(14)],
    [Some(5), Some(11), None, None],
    [Some(6), Some(12), Some(10), None],
    [Some(7), Some(13), Some(11), None],
    [Some(8), Some(14), Some(12), None],
    [Some(9), None, Some(13), None],
];

type Pasture = Vec<usize>;

#[derive(Clone, Debug, Hash)]
pub struct PastureConfig {
    pub pastures: Vec<Pasture>,
    pub wood: usize,
    pub hash: u64,
}

// Get all arrangements of fences for a single pasture
// Does not consider existing pastures
fn get_all_fence_arrangements(room_and_field_spaces: &[bool]) -> Vec<Pasture> {
    let mut ret = Vec::new();

    for i in 0..NUM_FARMYARD_SPACES {
        let mut q = VecDeque::new();
        if room_and_field_spaces[i] {
            continue;
        }
        q.push_back(vec![i]);

        while !q.is_empty() {
            let mut current = q.pop_front().unwrap();
            current.sort();
            if ret.contains(&current) {
                continue;
            }
            //println!("{:?}", current);
            for x in current.iter() {
                // Check East and South neighbor for each space
                for n in [1, 3] {
                    if NEIGHBOR_SPACES[*x][n].is_none() {
                        continue;
                    }

                    if room_and_field_spaces[NEIGHBOR_SPACES[*x][n].unwrap()] {
                        continue;
                    }

                    if current.contains(&NEIGHBOR_SPACES[*x][n].unwrap()) {
                        continue;
                    }
                    let mut new_arrangement = current.clone();
                    new_arrangement.push(NEIGHBOR_SPACES[*x][n].unwrap());
                    q.push_back(new_arrangement);
                }
            }

            ret.push(current);
        }
    }

    ret
}

// Wood required for a single pasture arrangement
fn wood_required(pasture: &Pasture) -> usize {
    let mut ret = 4 * pasture.len();
    for i in pasture.iter() {
        for n in [0, 1, 2, 3] {
            if NEIGHBOR_SPACES[*i][n].is_none() {
                continue;
            }
            if pasture.contains(&NEIGHBOR_SPACES[*i][n].unwrap()) {
                ret -= 1;
            }
        }
    }
    ret
}

// Combined a multi pasture with a single pasture
fn combine_pastures(
    ps: &[Pasture],
    p: &Pasture,
    ws: usize,
    w: usize,
) -> Option<(Vec<Pasture>, usize)> {
    let ps_flat = ps.iter().flatten().collect::<Vec<&usize>>();
    // We need to make sure that no space is shared between the two pastures
    let mut shared_space = false;
    for x in p.iter() {
        if ps_flat.contains(&x) {
            shared_space = true;
            break;
        }
    }

    if shared_space {
        return None;
    }

    // We need to find if any neighbor of p1 is in p2
    // The number of neighbors indicates the number of wood discounts
    let mut shared_fence = 0;
    for x in p.iter() {
        for n in [0, 1, 2, 3] {
            if NEIGHBOR_SPACES[*x][n].is_none() {
                continue;
            }
            if ps_flat.contains(&&NEIGHBOR_SPACES[*x][n].unwrap()) {
                shared_fence += 1;
            }
        }
    }

    if shared_fence > 0 {
        let wood = ws + w - shared_fence;
        if wood > MAX_FENCES {
            return None;
        }
        let mut ret = ps.to_vec();
        ret.push(p.to_vec());
        ret.sort();
        return Some((ret, wood));
    }
    None
}

const PRIMES: [u64; 5] = [2, 3, 5, 7, 11];

// Hash a multi-pasture configuration based on the number of spaces in each pasture
fn pasture_config_hash(pastures: &[Pasture]) -> u64 {
    let mut hash = 1;
    let mut pasture_sizes = Vec::new();
    for p in pastures.iter() {
        pasture_sizes.push(p.len());
    }
    pasture_sizes.sort();

    for i in 0..pasture_sizes.len() {
        hash *= PRIMES[i].pow(pasture_sizes[i] as u32);
    }

    hash
}

pub fn pasture_sizes_from_hash(hash: u64) -> Vec<usize> {
    let mut ret = Vec::new();
    let mut h = hash;
    for prime in PRIMES.iter() {
        let mut size = 0;
        if h % prime != 0 {
            continue;
        }
        while h % prime == 0 {
            size += 1;
            h /= prime;
        }
        ret.push(size);
    }
    ret
}

// House indices are at 5, 10. Fiels also need to be adjacent. So we prefer pastures to not break the connectivity of farm tiles in the rest of the farmyard
fn breaks_connectivity(pastures: &[Pasture], room_and_field_spaces: &[bool]) -> bool {
    let pasture_indices = pastures.iter().flatten().collect::<Vec<&usize>>();

    let mut visited = room_and_field_spaces.to_vec();

    for idx in pasture_indices.iter() {
        visited[**idx] = true;
    }

    let mut q = VecDeque::from(ROOM_INDICES.to_vec());

    while !q.is_empty() {
        let current = q.pop_front().unwrap();
        for n in [0, 1, 2, 3] {
            if NEIGHBOR_SPACES[current][n].is_none() {
                continue;
            }

            let neighbor = NEIGHBOR_SPACES[current][n].unwrap();
            if visited[neighbor] {
                continue;
            }

            visited[neighbor] = true;
            q.push_back(neighbor);
        }
    }

    for v in visited.iter() {
        if !v {
            return true;
        }
    }

    false
}

// If pasture p1 is contained entirely in pasture p2
fn contained_in(p1: &[usize], p2: &[usize]) -> bool {
    for x in p1.iter() {
        if !p2.contains(x) {
            return false;
        }
    }
    true
}

// If p1 is a future extension of p2
fn is_future_extension(pastures1: &[Vec<usize>], pastures2: &[Vec<usize>]) -> bool {
    // A pasture config can be extended in the future by adding more wood
    // Pastures can either be created adjacent to the existing pastures or existing pastures can be split into two or more pastures
    // In all those cases, such a (future) pasture config is considered an extension
    let mut p2_indices = pastures2.iter().flatten().collect::<Vec<&usize>>();
    p2_indices.sort();
    let mut p1_indices_fully_contained_in_p2: Vec<&usize> = Vec::new();
    for p1 in pastures1.iter() {
        for p in pastures2.iter() {
            if contained_in(p1, p) {
                p1_indices_fully_contained_in_p2.extend(p1);
            }
        }
    }
    p1_indices_fully_contained_in_p2.sort();
    if p1_indices_fully_contained_in_p2.len() == p2_indices.len() {
        return true;
    }
    false
}

fn all_possible_fence_configs(
    room_and_field_spaces: &[bool],
) -> [Vec<Vec<Vec<usize>>>; MAX_FENCES + 1] {
    let fence_arrangements = get_all_fence_arrangements(room_and_field_spaces);
    let mut pasture_config_to_min_wood_map: HashMap<u64, usize> = std::collections::HashMap::new();

    for arrangement in fence_arrangements.iter() {
        let wood = wood_required(arrangement);
        if wood > MAX_FENCES {
            continue;
        }
        let hash = pasture_config_hash(&[arrangement.clone()]);
        pasture_config_to_min_wood_map
            .entry(hash)
            .and_modify(|e| {
                *e = (*e).min(wood);
            })
            .or_insert(wood);
    }

    let mut single_pastures = Vec::new();
    for arrangement in fence_arrangements {
        let wood = wood_required(&arrangement);

        if wood > MAX_FENCES {
            continue;
        }

        let hash = pasture_config_hash(&[arrangement.clone()]);
        let min_wood = pasture_config_to_min_wood_map.get(&hash).unwrap();

        if wood > *min_wood {
            continue;
        }

        single_pastures.push((arrangement.clone(), wood));
    }

    let mut all_pastures = Vec::new();

    for (p, w) in single_pastures.iter() {
        if breaks_connectivity(&[p.to_vec()], room_and_field_spaces) {
            continue;
        }
        all_pastures.push((vec![p.clone()], *w));
    }

    // Loop until we have MAX_PASTURES pastures
    for _ in 1..MAX_PASTURES {
        let l = all_pastures.len();
        for i in 0..l {
            for p in single_pastures.iter() {
                let ps = &all_pastures[i];
                let combined = combine_pastures(&ps.0, &p.0, ps.1, p.1);
                if let Some((pastures, wood)) = combined {
                    if all_pastures.contains(&(pastures.clone(), wood)) {
                        continue;
                    }

                    if breaks_connectivity(pastures.as_slice(), room_and_field_spaces) {
                        continue;
                    }

                    let hash = pasture_config_hash(&pastures);
                    pasture_config_to_min_wood_map
                        .entry(hash)
                        .and_modify(|e| {
                            *e = (*e).min(wood);
                        })
                        .or_insert(wood);

                    all_pastures.push((pastures, wood));
                }
            }
        }
    }

    // Wood to pasture configuration map
    let mut possible_pastures_from_wood: [Vec<Vec<Vec<usize>>>; MAX_FENCES + 1] =
        Default::default();

    for (ps, wood) in all_pastures.iter() {
        let hash = pasture_config_hash(ps);
        let min_wood = pasture_config_to_min_wood_map.get(&hash).unwrap();

        // Reject a multi-pasture if the wood required to build the exact same config is greater than the min_wood required to build that config
        if wood > min_wood {
            continue;
        }

        possible_pastures_from_wood[*wood].push(ps.clone());
    }
    possible_pastures_from_wood
}

pub fn get_all_pasture_configs(
    farmyard_spaces: &[FarmyardSpace],
    fences_available: usize,
    fences_used: usize,
) -> Vec<PastureConfig> {
    let mut room_and_field_spaces = [
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false,
    ];

    let mut existing_pastures_configs: [Pasture; MAX_PASTURES] = Default::default();

    for (idx, space) in farmyard_spaces.iter().enumerate() {
        match space {
            FarmyardSpace::Field(_) | FarmyardSpace::Room => {
                room_and_field_spaces[idx] = true;
            }
            FarmyardSpace::FencedPasture(_, _, pasture_idx) => {
                existing_pastures_configs[*pasture_idx].push(idx);
            }
            _ => (),
        }
    }

    let possible_pasture_configs_for_wood = all_possible_fence_configs(&room_and_field_spaces);

    let mut ret = Vec::new();

    if existing_pastures_configs.iter().all(|p| p.is_empty()) {
        for (w, all_pastures) in possible_pasture_configs_for_wood.iter().enumerate() {
            if w > fences_available {
                break;
            }
            for pastures in all_pastures.iter() {
                ret.push(PastureConfig {
                    pastures: pastures.clone(),
                    wood: w,
                    hash: pasture_config_hash(pastures),
                });
            }
        }
    } else {
        for (w, all_pastures) in possible_pasture_configs_for_wood.iter().enumerate() {
            for pastures in all_pastures.iter() {
                if is_future_extension(pastures, &existing_pastures_configs) {
                    if w == fences_used {
                        continue;
                    }

                    ret.push(PastureConfig {
                        pastures: pastures.clone(),
                        wood: w - fences_used,
                        hash: pasture_config_hash(pastures),
                    });
                }
            }
        }
    }

    ret
}
