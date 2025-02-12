pub use crate::blockchain::{
    self,
    block::{self, Block, Header, Transaction},
    utils::hash_transaction,
    Blockchain,
};
use k256::ecdsa::{signature::Verifier, Signature};
use primitive_types::U256;
use std::time::SystemTime;

pub struct Miner {
    pub mempool: Vec<Transaction>,
}

impl Miner {
    pub fn add_transaction_to_mempool(&mut self, transaction: Transaction, signature: &Signature) {
        if !(transaction
            .public_key_from
            .verify(hash_transaction(&transaction).as_bytes(), signature)
            .is_ok())
        {
            return;
        }
        let mut idx: usize = 0;
        for mempool_transaction in self.mempool.iter() {
            if mempool_transaction.fee < transaction.fee {
                idx += 1;
            }
        }
        self.mempool.insert(idx, transaction);
    }

    pub fn compute_next_block(&mut self, blockchain: &mut Blockchain) {
        let max_transaction_count_in_block: usize = blockchain.max_transactions_per_block;

        let transactions_copy = {
            let transactions_slice = if self.mempool.len() > max_transaction_count_in_block {
                &self.mempool[0..max_transaction_count_in_block - 1]
            } else {
                &self.mempool[..]
            };
            transactions_slice.to_vec()
        };

        let transaction_count = transactions_copy.len();

        let latest_block_hash = if let Some(block) = blockchain.get_latest_block() {
            block.header.hash.clone()
        } else {
            String::new()
        };

        let block: Block =
            self._compute_next_block(transactions_copy, latest_block_hash, &blockchain);
        if blockchain.add_block(block) {
            if self.mempool.len() > transaction_count {
                self.mempool = self.mempool[transaction_count..].to_vec();
            } else {
                self.mempool.clear();
            }
        }
    }

    fn _compute_next_block(
        &mut self,
        transactions: Vec<Transaction>,
        latest_block_hash: String,
        blockchain: &Blockchain,
    ) -> Block {
        let mut timestamp: u64;
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => timestamp = n.as_secs(),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        }
        let mut nonce = 1;
        let mut block: Block =
            Block::create_block(nonce, timestamp, latest_block_hash.clone(), &transactions);
        loop {
            if let Ok(hash) = block.header.hash.parse::<U256>() {
                if hash <= blockchain.difficulty {
                    break;
                }
            }
            nonce += 1;
            match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                Ok(n) => timestamp = n.as_secs(),
                Err(_) => panic!("SystemTime before UNIX EPOCH!"),
            }
            block = Block::create_block(nonce, timestamp, latest_block_hash.clone(), &transactions);
        }
        block
    }
}
