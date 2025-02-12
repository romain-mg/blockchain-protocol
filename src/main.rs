pub mod blockchain;
pub mod miner;

use crate::blockchain::{account, Blockchain};
use crate::miner::Miner;
use k256::ecdsa::Signature;
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
        mempool: Vec::new(),
    };

    let mut account = account::Account::new();
    blockchain.create_account(account.get_public_key());

    let transaction: Transaction = Transaction {
        public_key_from: account.get_public_key(),
        public_key_to: account.get_public_key(),
        amount: 1,
        fee: 1,
    };

    let signature: Signature = account.sign_transaction(&transaction);
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
        miner.add_transaction_to_mempool(transaction, &signature);
    }

    for _n in 0..3 {
        miner.compute_next_block(&mut blockchain);
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
