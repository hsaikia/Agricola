use std::collections::{HashMap, VecDeque};

const A: usize = 5;
const B: usize = 3;
const NUM_FARMYARD_SPACES: usize = A * B;
const MAX_FENCES: usize = 15;
const ROOM_INDICES: [usize; 2] = [5, 10];

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

    // Loop until we have 5 pastures (more than 5 pastures are impossible with 15 fences)
    for _ in 1..5 {
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

        // Reject a multi-pasture if the whood required to build the exact same config is greater than the min_wood required to build that config
        if wood > min_wood {
            continue;
        }

        possible_pastures_from_wood[*wood].push(ps.clone());
    }

    for (i, all_pastures) in possible_pastures_from_wood.iter().enumerate() {
        println!("All pasture configs for wood {}", i);
        for pastures in all_pastures.iter() {
            let mut future_extensions = 0;
            for future_pastures in possible_pastures_from_wood.iter().skip(i + 1) {
                for future_ps in future_pastures.iter() {
                    let mut num_contains = 0;
                    for p in pastures.iter() {
                        if future_ps.contains(p) {
                            // future_extensions += 1;
                            num_contains += 1;
                        }
                    }
                    if num_contains == pastures.len() {
                        future_extensions += 1;
                        //println!("\tFuture extension: {:?}", future_ps);
                    }
                }
            }

            println!("{:?} Future extensions {}", pastures, future_extensions);
        }
        println!("-----------------");
    }
}
