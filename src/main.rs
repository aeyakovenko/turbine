use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

const BATCH_SIZE: usize = 64;
const RECOVER_SIZE: usize = 33;
#[derive(Clone, Copy)]
struct Node {
    shreds: [u8; BATCH_SIZE],
}

fn main() {
    //turbine_recoverable();
    turbine_recoverable_duplicate_blocks();
}

impl Node {
    //0: no shreds in block
    //1: only block 1 shreds
    //2: only block 2 shreds
    //3: mixed shreds detected
    fn check_mixed(&self) -> u8 {
        let mut mixed = 0;
        for s in &self.shreds {
            if mixed == 0 {
                mixed = *s;
            }
            if *s != 0 && mixed != *s {
                mixed = 3;
                break;
            }
        }
        mixed
    }
    fn check_recovered(&self) -> bool {
        self.shreds.into_iter().map(|x| u8::from(x > 0)).sum::<u8>() >= RECOVER_SIZE as u8
    }
}
 
fn turbine_recoverable_duplicate_blocks() {
    const NUM_NODES: usize = 10_000;
    const BAD_NODES: usize = 100;
    const NUM_PACKETS: usize = BATCH_SIZE;
    const L0_SIZE: usize = 200;
    const L1_SIZE: usize = (NUM_NODES - L0_SIZE) / L0_SIZE;
    let mut total: usize = 0;

    for block in 1..100_000_000 {
        let mut nodes: [Node; NUM_NODES] = [Node {
            shreds: [0; BATCH_SIZE],
        }; 10_000];
        let mut rounds = 0;
        let mut recovered = true;
        while recovered {
            recovered = false;
            for shred in 0..NUM_PACKETS {
                let mut rng = ChaCha8Rng::seed_from_u64(shred as u64 * block as u64);
                let mut index: Vec<usize> = (0..NUM_NODES).into_iter().collect();
                index.shuffle(&mut rng);
                //leader is reliable
                //lvl 0
                let retransmitter = index[0];
                for node in &index[0..L0_SIZE] {
                    // if retransmitter is a bad node, retransmit block 1 to odd, block 2 to even
                    if retransmitter < BAD_NODES && *node % 2 == 0 {
                        nodes[*node].shreds[shred] = 2;
                    } else {
                        nodes[*node].shreds[shred] = 1;
                    }
                }

                //lvl 1
                //each l0 node does the same amount of work for l1
                for x in 0..L0_SIZE {
                    let retransmitter = index[x];
                    //skip shred is empty
                    if nodes[retransmitter].shreds[shred] == 0 {
                        continue;
                    }
                    let start = 200 + x * L1_SIZE;
                    for node in &index[start..start + L1_SIZE] {
                        // if retransmitter is a bad node, retransmit block 2 to even
                        if retransmitter < BAD_NODES && *node % 2 == 0 {
                            nodes[*node].shreds[shred] = 2;
                        } else {
                            nodes[*node].shreds[shred] = nodes[retransmitter].shreds[shred];
                        }
                    }
                }
            }
            //recover all shreds
            for (ix, node) in &mut nodes.iter_mut().enumerate() {
                let mixed = node.check_mixed();
                let recover: bool = node.check_recovered();
                //cant recover mixed blocks
                if recover && mixed != 3 {
                    for s in &mut node.shreds {
                        if *s == 0 {
                            *s = mixed;
                            //continue the retransmit if even 1 new shred was recovered
                            recovered = true;
                        }
                    }
                }
            }
            rounds += 1;
        }
        let mut recovered_1 = 0;
        let mut recovered_2 = 0;
        let mut mixed_3 = 0;
        for node in 0..NUM_NODES {
            let mixed = nodes[node].check_mixed();
            let recover: bool = nodes[node].check_recovered();
            if recover && mixed == 1 {
                recovered_1 += 1;
            }
            if recover && mixed == 2 {
                recovered_2 += 1;
            }
            if mixed == 3 {
                mixed_3 += 1;
            }
        }
        total += 1;
        println!(
            "total: {}\nr1: {}\nr2: {}\nmixed: {}\n",
            total, recovered_1, recovered_2, mixed_3
        );
    }
}

fn turbine_recoverable() {
    const NUM_NODES: usize = 10_000;
    const BAD_NODES: usize = 4_000;
    //const BAD_NODES: usize = 4_333;
    const NUM_PACKETS: usize = BATCH_SIZE;
    const L0_SIZE: usize = 200;
    const L1_SIZE: usize = (NUM_NODES - L0_SIZE) / L0_SIZE;
    let mut fails = 0;
    let mut r_33_50 = 0;
    let mut r_33 = 0;
    let mut r_50 = 0;
    let mut vote_fail = 0;
    let mut total: usize = 0;
    let mut max_fail: usize = 0;
    let my_node = 9_999;
    let mut my_node_fail = 0;
    let mut signaled = 0;
    let mut my_node_fails = 0;

    for block in 1..100_000_000 {
        let mut nodes: [Node; NUM_NODES] = [Node {
            shreds: [0; BATCH_SIZE],
        }; 10_000];
        let mut rounds = 0;
        let mut recovered = true;
        while recovered {
            recovered = false;
            for shred in 0..NUM_PACKETS {
                let mut rng = ChaCha8Rng::seed_from_u64(shred as u64 * block as u64);
                let mut index: Vec<usize> = (0..NUM_NODES).into_iter().collect();
                index.shuffle(&mut rng);
                //leader is reliable
                //lvl 0
                let retransmitter = index[0];
                // if a bad node, skip retransmitting to lvl 0
                if retransmitter >= BAD_NODES {
                    for node in &index[0..L0_SIZE] {
                        if *node < BAD_NODES {
                            continue;
                        }
                        nodes[*node].shreds[shred] = 1;
                    }
                }

                //lvl 1
                //each l0 node does the same amount of work for l1
                for x in 0..L0_SIZE {
                    let retransmitter = index[x];
                    //skip shred is empty
                    if nodes[retransmitter].shreds[shred] == 0 {
                        continue;
                    }
                    //skip if bad node
                    if retransmitter < BAD_NODES {
                        continue;
                    }
                    let start = 200 + x * L1_SIZE;
                    for node in &index[start..start + L1_SIZE] {
                        if *node < BAD_NODES {
                            continue;
                        }
                        nodes[*node].shreds[shred] = 1;
                    }
                }
            }
            //recover all shreds
            for (ix, node) in &mut nodes.iter_mut().enumerate() {
                //skip if bad node
                if ix < BAD_NODES {
                    continue;
                }
                let recover: bool = node.shreds.into_iter().sum::<u8>() > RECOVER_SIZE as u8;
                if recover {
                    for s in &mut node.shreds {
                        if *s == 0 {
                            *s = 1;
                            //continue the retransmit if even 1 new shred was recovered
                            recovered = true;
                        }
                    }
                }
            }
            rounds += 1;
        }
        let mut recovered = 0;
        for node in 0..NUM_NODES {
            if nodes[node].shreds.into_iter().sum::<u8>() > RECOVER_SIZE as u8 {
                recovered += 1;
            }
        }
        if recovered <= 3_333 {
            r_33 += 1;
        } else if recovered <= 5_000 && recovered > 3_333 {
            r_33_50 += 1;
        } else if recovered > 5_000 {
            r_50 += 1;
        }
        if recovered <= 6_666 {
            let max = nodes
                .iter()
                .map(|n| n.shreds.into_iter().sum::<u8>())
                .max()
                .unwrap_or(0);
            if max_fail < max.into() {
                signaled = 0;
            }
            max_fail = std::cmp::max(max.into(), max_fail);
            vote_fail += 1;
        }
        if recovered > 6_666 {
            for node in 0..NUM_NODES {
                if nodes[node].shreds.into_iter().sum::<u8>() > max_fail as u8 {
                    signaled += 1;
                }
            }
        }
        //conditinal prob
        if nodes[my_node].shreds.into_iter().sum::<u8>() <= RECOVER_SIZE as u8 {
            my_node_fail += NUM_NODES - recovered;
            my_node_fails += 1;
        }

        fails += NUM_NODES - recovered;
        total += 1;
        println!(
            "rounds: {}\nr_33: {}\nr_33_50: {}\nr_50: {}\nsignaled: {}\nrecovered: {}\ntotal_failed: {}\nmax shred in 2/3 fail: {}\n2/3 vote failure: {}/{}\nconditinal failure rate {}/{}\n",
            rounds, r_33, r_33_50, r_50, signaled, recovered, fails, max_fail, vote_fail, total, my_node_fail, my_node_fails
        );
    }
}

fn turbine() {
    const NUM_NODES: usize = 10_000;
    const BAD_NODES: usize = 3_333;
    const NUM_PACKETS: usize = BATCH_SIZE;
    const L0_SIZE: usize = 200;
    const L1_SIZE: usize = (NUM_NODES - L0_SIZE) / L0_SIZE;
    let mut fails = 0;
    let mut vote_fail = 0;
    let mut total: usize = 0;
    let mut max_fail: usize = 0;
    let my_node = 9_999;
    let mut my_node_fail = 0;
    let mut signaled = 0;
    let mut my_node_fails = 0;

    for block in 1..100_000_000 {
        let mut nodes: [Node; NUM_NODES] = [Node {
            shreds: [0; BATCH_SIZE],
        }; 10_000];
        for shred in 0..NUM_PACKETS {
            let mut rng = ChaCha8Rng::seed_from_u64(shred as u64 * block as u64);
            let mut index: Vec<usize> = (0..NUM_NODES).into_iter().collect();
            index.shuffle(&mut rng);
            //leader is reliable
            //lvl 0
            let retransmitter = index[0];
            // if a bad node, skip retransmitting to lvl 0
            if retransmitter >= BAD_NODES {
                for node in &index[0..L0_SIZE] {
                    if *node < BAD_NODES {
                        continue;
                    }
                    nodes[*node].shreds[shred] = 1;
                }
            }
            //lvl 1
            //each l0 node does the same amount of work for l1
            for x in 0..L0_SIZE {
                let retransmitter = index[x];
                //skip if node was skipped by a bad node
                if nodes[retransmitter].shreds[shred] == 0 {
                    continue;
                }
                //skip if bad node
                if retransmitter < BAD_NODES {
                    continue;
                }
                let start = 200 + x * L1_SIZE;
                for node in &index[start..start + L1_SIZE] {
                    if *node < BAD_NODES {
                        continue;
                    }
                    nodes[*node].shreds[shred] = 1;
                }
            }
        }
        let mut recovered = 0;
        for node in 0..NUM_NODES {
            if nodes[node].shreds.into_iter().sum::<u8>() > RECOVER_SIZE as u8 {
                recovered += 1;
            }
        }
        if recovered <= 6_666 {
            let max = nodes
                .iter()
                .map(|n| n.shreds.into_iter().sum::<u8>())
                .max()
                .unwrap_or(0);
            if max_fail < max.into() {
                signaled = 0;
            }
            max_fail = std::cmp::max(max.into(), max_fail);
            vote_fail += 1;
        }
        if recovered > 6_666 {
            for node in 0..NUM_NODES {
                if nodes[node].shreds.into_iter().sum::<u8>() > max_fail as u8 {
                    signaled += 1;
                }
            }
        }
        //conditinal prob
        if nodes[my_node].shreds.into_iter().sum::<u8>() <= RECOVER_SIZE as u8 {
            my_node_fail += NUM_NODES - recovered;
            my_node_fails += 1;
        }

        fails += NUM_NODES - recovered;
        total += 1;
        println!(
            "signaled: {}\nrecovered: {}\ntotal_failed: {}\nmax shred in 2/3 fail: {}\n2/3 vote failure: {}/{}\nconditinal failure rate {}/{}\n",
            signaled, recovered, fails, max_fail, vote_fail, total, my_node_fail, my_node_fails
        );
    }
}

fn turbine_recoverable_data_only() {
    const NUM_NODES: usize = 10_000;
    const BAD_NODES: usize = 3_333;
    const NUM_PACKETS: usize = BATCH_SIZE;
    const L0_SIZE: usize = 200;
    const L1_SIZE: usize = (NUM_NODES - L0_SIZE) / L0_SIZE;
    let mut fails = 0;
    let mut vote_fail = 0;
    let mut total: usize = 0;
    let mut max_fail: usize = 0;
    let my_node = 9_999;
    let mut my_node_fail = 0;
    let mut signaled = 0;
    let mut my_node_fails = 0;

    for block in 1..100_000_000 {
        let mut nodes: [Node; NUM_NODES] = [Node {
            shreds: [0; BATCH_SIZE],
        }; 10_000];
        let mut rounds = 0;
        let mut recovered = true;
        while recovered {
            recovered = false;
            for shred in 0..NUM_PACKETS {
                let mut rng = ChaCha8Rng::seed_from_u64(shred as u64 * block as u64);
                let mut index: Vec<usize> = (0..NUM_NODES).into_iter().collect();
                index.shuffle(&mut rng);
                //leader is reliable
                //lvl 0
                let retransmitter = index[0];
                // if a bad node, skip retransmitting to lvl 0
                if retransmitter >= BAD_NODES {
                    for node in &index[0..L0_SIZE] {
                        if *node < BAD_NODES {
                            continue;
                        }
                        nodes[*node].shreds[shred] = 1;
                    }
                }

                //lvl 1
                //each l0 node does the same amount of work for l1
                for x in 0..L0_SIZE {
                    let retransmitter = index[x];
                    //skip shred is empty
                    if nodes[retransmitter].shreds[shred] == 0 {
                        continue;
                    }
                    //skip if bad node
                    if retransmitter < BAD_NODES {
                        continue;
                    }
                    let start = 200 + x * L1_SIZE;
                    for node in &index[start..start + L1_SIZE] {
                        if *node < BAD_NODES {
                            continue;
                        }
                        nodes[*node].shreds[shred] = 1;
                    }
                }
            }
            //recover all shreds
            for (ix, node) in &mut nodes.iter_mut().enumerate() {
                //skip if bad node
                if ix < BAD_NODES {
                    continue;
                }
                let recover: bool = node.shreds.into_iter().sum::<u8>() > RECOVER_SIZE as u8;
                if recover {
                    for s in &mut node.shreds[0..RECOVER_SIZE] {
                        if *s == 0 {
                            *s = 1;
                            //continue the retransmit if even 1 new shred was recovered
                            recovered = true;
                        }
                    }
                }
            }
            rounds += 1;
        }
        let mut recovered = 0;
        for node in 0..NUM_NODES {
            if nodes[node].shreds.into_iter().sum::<u8>() > RECOVER_SIZE as u8 {
                recovered += 1;
            }
        }
        if recovered <= 6_666 {
            let max = nodes
                .iter()
                .map(|n| n.shreds.into_iter().sum::<u8>())
                .max()
                .unwrap_or(0);
            if max_fail < max.into() {
                signaled = 0;
            }
            max_fail = std::cmp::max(max.into(), max_fail);
            vote_fail += 1;
        }
        if recovered > 6_666 {
            for node in 0..NUM_NODES {
                if nodes[node].shreds.into_iter().sum::<u8>() > max_fail as u8 {
                    signaled += 1;
                }
            }
        }
        //conditinal prob
        if nodes[my_node].shreds.into_iter().sum::<u8>() <= RECOVER_SIZE as u8 {
            my_node_fail += NUM_NODES - recovered;
            my_node_fails += 1;
        }

        fails += NUM_NODES - recovered;
        total += 1;
        println!(
            "rounds: {}\nsignaled: {}\nrecovered: {}\ntotal_failed: {}\nmax shred in 2/3 fail: {}\n2/3 vote failure: {}/{}\nconditinal failure rate {}/{}\n",
            rounds, signaled, recovered, fails, max_fail, vote_fail, total, my_node_fail, my_node_fails
        );
    }
}
