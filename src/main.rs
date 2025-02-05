pub mod blockchain;
pub mod miner;
pub use crate::blockchain::Blockchain;
pub use crate::miner::Miner;
use miner::Transaction;
use primitive_types::U256;

fn main() {
    let difficulty_divisor: i32 = 20000;
    let difficulty: U256 = U256::MAX / difficulty_divisor;
    let target_duration_between_blocks = 5;
    let mut blockchain: Blockchain =
        Blockchain::create_blockchain(difficulty, target_duration_between_blocks);
    let mut miner: Miner = Miner {
        blockchain: &mut blockchain,
    };

    let transaction: Transaction = Transaction {
        address_from: String::from("address_from"),
        address_to: String::from("address_to"),
        amount: 1,
    };
    let transactions: Vec<Transaction> = vec![
        transaction.clone(),
        transaction.clone(),
        transaction.clone(),
    ];

    for _n in 0..10 {
        miner.compute_next_block(transactions.clone());
    }

    if let Some(block) = blockchain.get_block(0) {
        let merkle_root: &String = &block.header.merkle_root;
        println!("{}", merkle_root);
        let merkle_tree = &block.transactions;
        println!("{merkle_tree:?}");
        if let Some(root) = &merkle_tree.root {
            let left = &root.left;
            let right = &root.right;
            println!("{left:?}");
            println!("{right:?}");
        }
    }
}
