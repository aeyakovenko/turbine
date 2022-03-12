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
    const HOOD_SIZE: usize = 200;
    let mut fails = 0;
    let mut total: usize = 0;

    for block in 1..10_001 {
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
                for node in &index[0..HOOD_SIZE] {
                    nodes[*node].shreds[shred] = 1;
                }
            }
            //lvl 1
            for x in 1..(NUM_NODES / HOOD_SIZE) {
                let retransmitter = index[x];
                //skip if node was skipped by a bad node
                if nodes[retransmitter].shreds[shred] == 0 {
                    continue;
                }
                for node in &index[x * HOOD_SIZE..(x + 1) * HOOD_SIZE] {
                    if retransmitter < BAD_NODES {
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
        fails += NUM_NODES - recovered;
        total += 1;
        println!("{} {}/{}", recovered, fails, total);
    }
}
