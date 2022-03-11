use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

#[derive(Clone, Copy)]
struct Node {
    shreds: [u8; 64],
}

fn main() {
    const num_nodes: usize = 10_000;
    const bad_nodes: usize = 3_333;
    const num_packets: usize = 64;
    const my_node: usize = 9_999;
    let mut success: usize = 0;
    let mut total: usize = 0;

    for block in 1..10_001 {
        let mut nodes: [Node; num_nodes] = [Node { shreds: [0; 64] }; 10_000];
        for shred in 0..num_packets {
            let mut rng = ChaCha8Rng::seed_from_u64(shred as u64 * block as u64);
            let mut index: Vec<usize> = (0..num_nodes).into_iter().collect();
            index.shuffle(&mut rng);
            //leader is reliable
            //lvl 0
            let retransmitter = index[0];
            // if a bad node, skip retransmitting to lvl 0
            if retransmitter >= bad_nodes {
                for node in &index[0..200] {
                    nodes[*node].shreds[shred] = 1;
                }
            }
            //lvl 1
            for x in 1..50 {
                let retransmitter = index[x];
                //skip if node was skipped by a bad node
                if nodes[retransmitter].shreds[shred] == 0 {
                    continue;
                }
                for node in &index[x * 200..(x + 1) * 200] {
                    if retransmitter < bad_nodes {
                        continue;
                    }
                    nodes[*node].shreds[shred] = 1;
                }
            }
        }
        if nodes[my_node].shreds.into_iter().sum::<u8>() > 33u8 {
            success += 1;
        }
        total += 1;
        println!("{}/{}", success, total);
    }
}
