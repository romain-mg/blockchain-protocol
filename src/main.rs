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

    let mut sender_account = account::AccountKeys::new();
    let sender_account_public_key = sender_account.get_public_key();
    blockchain.create_account(&sender_account_public_key);

    let receiver_account = account::AccountKeys::new();
    let receiver_account_public_key = receiver_account.get_public_key();
    blockchain.create_account(&receiver_account_public_key);

    blockchain.mint(&sender_account_public_key, U256::from(1));

    let transaction_0: Transaction = Transaction {
        public_key_from: sender_account_public_key,
        public_key_to: receiver_account_public_key,
        amount: U256::from(1),
        fee: U256::from(1),
        nonce: 0,
    };

    let signature_0: Signature = sender_account.sign_transaction(&transaction_0);

    let transaction_1: Transaction = Transaction {
        public_key_from: sender_account_public_key,
        public_key_to: receiver_account_public_key,
        amount: U256::from(1),
        fee: U256::from(1),
        nonce: 1,
    };

    let signature_1: Signature = sender_account.sign_transaction(&transaction_1);

    let transaction_2: Transaction = Transaction {
        public_key_from: sender_account_public_key,
        public_key_to: receiver_account_public_key,
        amount: U256::from(1),
        fee: U256::from(1),
        nonce: 2,
    };

    let signature_2: Signature = sender_account.sign_transaction(&transaction_2);

    miner.add_transaction_to_mempool(transaction_0, &signature_0, &mut blockchain);
    miner.add_transaction_to_mempool(transaction_1, &signature_1, &mut blockchain);
    miner.add_transaction_to_mempool(transaction_2, &signature_2, &mut blockchain);

    miner.compute_next_block(&mut blockchain);

    if let Some(block) = blockchain.get_block(0) {
        let merkle_root: &String = &block.header.merkle_root;
        println!("{}", merkle_root);
        let txns = &block.transactions;
        println!("{txns:?}");
    }
}
