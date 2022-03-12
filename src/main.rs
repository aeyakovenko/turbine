use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

const BATCH_SIZE: usize = 96;
const VOTE_SIZE: usize = 42; // 0.44 * 96
const RECOVER_SIZE: usize = BATCH_SIZE/3;
#[derive(Clone, Copy)]
struct Node {
    shreds: [u8; BATCH_SIZE],
}

fn main() {
    const num_nodes: usize = 10_000;
    const bad_nodes: usize = 3_333;
    const num_packets: usize = BATCH_SIZE;
    const hood_size: usize = 200;
    let mut fails = 0;
    let mut success: usize = 0;
    let mut blocks = 0;
    let mut total: usize = 0;

    for block in 1..10_001 {
        let mut nodes: [Node; num_nodes] = [Node {
            shreds: [0; BATCH_SIZE],
        }; 10_000];
        for shred in 0..num_packets {
            let mut rng = ChaCha8Rng::seed_from_u64(shred as u64 * block as u64);
            let mut index: Vec<usize> = (0..num_nodes).into_iter().collect();
            index.shuffle(&mut rng);
            //leader is reliable
            //lvl 0
            let retransmitter = index[0];
            // if a bad node, skip retransmitting to lvl 0
            if retransmitter >= bad_nodes {
                for node in &index[0..hood_size] {
                    nodes[*node].shreds[shred] = 1;
                }
            }
            //lvl 1
            for x in 1..(num_nodes/hood_size) {
                let retransmitter = index[x];
                //skip if node was skipped by a bad node
                if nodes[retransmitter].shreds[shred] == 0 {
                    continue;
                }
                for node in &index[x * hood_size..(x + 1) * hood_size] {
                    if retransmitter < bad_nodes {
                        continue;
                    }
                    nodes[*node].shreds[shred] = 1;
                }
            }
        }
        let mut voted = 0;
        for node in 0..num_nodes {
            if nodes[node].shreds.into_iter().sum::<u8>() > VOTE_SIZE  as u8 {
                voted += 1;
            }
        }
        let mut recovered = 0;
        for node in 0..num_nodes {
            if nodes[node].shreds.into_iter().sum::<u8>() > RECOVER_SIZE as u8 {
                recovered += 1;
            }
        }

        if voted > 3_3333 && recovered < 6_666 {
            fails += recovered;
        }
        total += 1;
        println!("{} {} {}/{}", voted, recovered, fails, total);
    }
}
