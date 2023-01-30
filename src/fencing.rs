use std::collections::HashSet;

const L: u32 = 3;
const W: u32 = 5;

fn to_idx(i: u32, j: u32) -> u32 {
    (W + 1) * i + j
}

fn from_idx(idx: u32) -> (u32, u32) {
    (idx / (W + 1), idx % (W + 1))
}

fn display_fences(fences: &HashSet<(u32, u32)>) {
    const D: u32 = (2 * L + 1) * (2 * W + 1);
    let mut arr: [&str; D as usize] = ["+"; D as usize];

    for i in 0..D {
        let j: u32 = i / (2 * W + 1);
        if j % 2 == 1 || i % 2 == 1 {
            arr[i as usize] = " ";
        }
    }

    for (x, y) in fences {
        let idx = x + (x / (W + 1)) * W + y + (y / (W + 1)) * W;
        if x + 1 < *y {
            arr[idx as usize] = "|";
        } else {
            arr[idx as usize] = "-";
        }
    }

    for i in 0..D {
        if i % (2 * W + 1) == 0 {
            println!();
        }

        print!(" {}", arr[i as usize]);
    }
    println!();
}

fn encircle(x1: u32, y1: u32, x2: u32, y2: u32) -> HashSet<(u32, u32)> {
    let mut ret: HashSet<(u32, u32)> = HashSet::new();

    let mut x = x1;
    let mut y = y1;

    while x < x2 {
        ret.insert((to_idx(x, y), to_idx(x + 1, y)));
        x += 1;
    }
    while y < y2 {
        ret.insert((to_idx(x, y), to_idx(x, y + 1)));
        y += 1;
    }

    let mut x = x1;
    let mut y = y1;

    while y < y2 {
        ret.insert((to_idx(x, y), to_idx(x, y + 1)));
        y += 1;
    }
    while x < x2 {
        ret.insert((to_idx(x, y), to_idx(x + 1, y)));
        x += 1;
    }
    ret
}

// Input is the flattened 4x6 index
// num is the number of fences
// Output is a vector of all possible fence arrangements
// First element contains the fence indices (X -> Y where Y > X) and second is the number of fences used in each pasture

type Fence = HashSet<(u32, u32)>;
type PastureSizes = Vec<u32>;

fn generate(idx: u32, num: u32) -> Vec<(Fence, PastureSizes)> {
    let (x, y) = from_idx(idx);
    let mut tot_ret: Vec<(Fence, PastureSizes)> = Vec::new();
    if num == 0 {
        return tot_ret;
    }
    //println!("Calling generate from ({},{}) with {} fences", x, y, num);
    for i in (x + 1)..=L {
        for j in (y + 1)..=W {
            let fences = 2 * (i - x + j - y);
            if fences <= num {
                let res = encircle(x, y, i, j);
                let pasture_sizes: PastureSizes = vec![(i - x) * (j - y)];

                // This logic has a small bug, since not all remaining fences may be accounted for
                let sub_res1 = generate(to_idx(i, y), num - fences + j - y);
                let sub_res2 = generate(to_idx(x, j), num - fences + i - x);

                if sub_res1.is_empty() && sub_res2.is_empty() {
                    tot_ret.push((res, pasture_sizes));
                    continue;
                }

                for (r, n) in sub_res1 {
                    let mut res_tmp = res.clone();
                    let mut pasture_fences_tmp = pasture_sizes.clone();
                    pasture_fences_tmp.extend(n);
                    res_tmp.extend(r);

                    if !res_tmp.is_empty() && res_tmp.len() <= num as usize {
                        tot_ret.push((res_tmp, pasture_fences_tmp));
                    }
                }

                for (r, n) in sub_res2 {
                    let mut res_tmp = res.clone();
                    let mut pasture_fences_tmp = pasture_sizes.clone();
                    pasture_fences_tmp.extend(n);
                    res_tmp.extend(r);

                    if !res_tmp.is_empty() && res_tmp.len() <= num as usize {
                        tot_ret.push((res_tmp, pasture_fences_tmp));
                    }
                }
            }
        }
    }
    tot_ret
}

pub fn test_fencing(num_fences : u32) {
    let res = generate(0, num_fences);
    let mut i = 1;
    for (r, n) in &res {
        // Reject fence arrangements that cannot accomodate all 3 types of animals
        // Also ensure max points for stables
        if n.len() < 4 {
            continue;
        }
        let total_capacity_with_stables : u32 = 4 * n.iter().sum::<u32>(); 

        // If total capacity is less than 21 (8 sheep + 7 pigs + 6 cows) reject
        if total_capacity_with_stables < 21 {
            continue;
        }

        println!(
            "{}. Using {} Fences with Pasture sizes {:?}",
            i,
            r.len(),
            n
        );
        display_fences(r);
        println!();
        i += 1;
    }
}
