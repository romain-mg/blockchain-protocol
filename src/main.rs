pub mod account;
pub mod blockchain;
pub mod miner;

use crate::blockchain::Blockchain;
use crate::miner::Miner;

use miner::Transaction;
use primitive_types::U256;

fn main() {
    let difficulty_divisor: i32 = 20000;
    let difficulty: U256 = U256::MAX / difficulty_divisor;
    let target_duration_between_blocks = 5;
    let max_transactions_per_block = 3;
    let mut blockchain: Blockchain = Blockchain::create_blockchain(
        difficulty,
        target_duration_between_blocks,
        max_transactions_per_block,
    );
    let mut miner: Miner = Miner {
        blockchain: &mut blockchain,
        mempool: Vec::new(),
    };

    let transaction: Transaction = Transaction {
        public_key_from: String::from("public_key_from"),
        public_key_to: String::from("public_key_to"),
        amount: 1,
        fee: 1,
    };
    let transactions: Vec<Transaction> = vec![
        transaction.clone(),
        transaction.clone(),
        transaction.clone(),
        transaction.clone(),
        transaction.clone(),
        transaction.clone(),
        transaction.clone(),
        transaction.clone(),
        transaction.clone(),
        transaction.clone(),
        transaction.clone(),
        transaction.clone(),
    ];

    for transaction in transactions {
        miner.add_tx_to_mempool(transaction);
    }

    for _n in 0..3 {
        miner.compute_next_block();
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
