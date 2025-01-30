pub mod blockchain;
pub mod miner;
use miner::Transaction;

pub use crate::blockchain::Blockchain;
pub use crate::miner::Miner;

fn main() {
    let difficulty: u32 = u32::MAX - u32::MAX / 10;
    let target_duration_between_blocks = 5;
    let blockchain: Blockchain =
        Blockchain::create_blockchain(difficulty, target_duration_between_blocks);
    let mut miner: Miner = Miner { blockchain };

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
}
