use std::collections::{HashMap, VecDeque};

const A: usize = 5;
const B: usize = 3;
const NUM_FARMYARD_SPACES: usize = A * B;
const MAX_FENCES: usize = 15;
const ROOM_INDICES: [usize; 2] = [5, 10];

// 5 pastures actually decrease total capacity while needing more wood. Plus score is (by default, without any bonuses) capped at 4 pastures
// If some card allows additional bonuses for 5 pastures, this flag can be set to true
const ALLOW_FIVE_PASTURES: bool = false;

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

// Get all arrangements of fences for a single pasture
fn get_all_fence_arrangements(occupied_spaces: &[bool]) -> Vec<Vec<usize>> {
    let mut ret = Vec::new();

    for i in 0..NUM_FARMYARD_SPACES {
        let mut q = VecDeque::new();
        if occupied_spaces[i] {
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

                    if occupied_spaces[NEIGHBOR_SPACES[*x][n].unwrap()] {
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
fn wood_required(arrangement: &[usize]) -> usize {
    let mut ret = 4 * arrangement.len();
    for i in arrangement.iter() {
        for n in [0, 1, 2, 3] {
            if NEIGHBOR_SPACES[*i][n].is_none() {
                continue;
            }
            if arrangement.contains(&NEIGHBOR_SPACES[*i][n].unwrap()) {
                ret -= 1;
            }
        }
    }
    ret
}

// Combined a multi pasture with a single pasture
fn combine_pastures(
    ps: &[Vec<usize>],
    p: &[usize],
    ws: usize,
    w: usize,
) -> Option<(Vec<Vec<usize>>, usize)> {
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
fn pasture_config_hash(pasture: &[Vec<usize>]) -> u64 {
    let mut hash = 1;
    let mut pasture_sizes = Vec::new();
    for p in pasture.iter() {
        pasture_sizes.push(p.len());
    }
    pasture_sizes.sort();

    for i in 0..pasture_sizes.len() {
        hash *= PRIMES[i].pow(pasture_sizes[i] as u32);
    }

    hash
}

// Every pasture space can hold 2 animals, with a stable each pasture has double capacity, given a max of 4 stables
fn pasture_max_capacities(pasture: &[Vec<usize>]) -> Vec<usize> {
    let mut max_capacity = Vec::new();
    for (i, p) in pasture.iter().enumerate() {
        let stable_multipler = if i < 4 { 2 } else { 1 };
        max_capacity.push(p.len() * 2 * stable_multipler);
    }
    max_capacity.sort();
    max_capacity
}

// House indices are at 0, 5. Fiels also need to be adjacent. So we prefer pastures to not break the connectivity of farm tiles in the rest of the farmyard
fn breaks_connectivity(pastures: &[Vec<usize>]) -> bool {
    let pasture_indices = pastures.iter().flatten().collect::<Vec<&usize>>();

    let mut visited = [false; NUM_FARMYARD_SPACES];

    for idx in pasture_indices.iter() {
        visited[**idx] = true;
    }

    for idx in ROOM_INDICES.iter() {
        visited[*idx] = true;
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

fn main() {
    let mut occupied_spaces = [
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false,
    ];

    for idx in ROOM_INDICES.iter() {
        occupied_spaces[*idx] = true;
    }

    let fence_arrangements = get_all_fence_arrangements(&occupied_spaces);
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
        if breaks_connectivity(&[p.to_vec()]) {
            continue;
        }
        all_pastures.push((vec![p.clone()], *w));
    }

    // Loop until we have 5 pastures (more than 5 pastures are impossible with 15 fences) if ALLOW_FIVE_PASTURES is true
    let max_pastures = if ALLOW_FIVE_PASTURES { 5 } else { 4 };
    for _ in 1..max_pastures {
        let l = all_pastures.len();
        for i in 0..l {
            for p in single_pastures.iter() {
                let ps = &all_pastures[i];
                let combined = combine_pastures(&ps.0, &p.0, ps.1, p.1);
                if let Some((pastures, wood)) = combined {
                    if all_pastures.contains(&(pastures.clone(), wood)) {
                        continue;
                    }

                    if breaks_connectivity(pastures.as_slice()) {
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

    for (i, all_pastures) in possible_pastures_from_wood.iter().enumerate() {
        println!("All pasture configs for wood {}", i);
        for pastures in all_pastures.iter() {
            let mut flat_p_indices = pastures.iter().flatten().collect::<Vec<&usize>>();
            flat_p_indices.sort();

            let mut future_extensions = 0;
            for future_pastures in possible_pastures_from_wood.iter().skip(i + 1) {
                for future_pasture in future_pastures.iter() {
                    if is_future_extension(future_pasture, pastures) {
                        future_extensions += 1;
                    }
                }
            }

            let max_capacity = pasture_max_capacities(pastures);
            println!(
                "{:?} Max Capacity {:?} Future extensions {}",
                pastures, max_capacity, future_extensions
            );
        }
        println!("-----------------");
    }
}
