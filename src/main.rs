use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

const BATCH_SIZE: usize = 96;
const RECOVER_SIZE: usize = BATCH_SIZE / 3;
#[derive(Clone, Copy)]
struct Node {
    shreds: [u8; BATCH_SIZE],
}

fn main() {
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
