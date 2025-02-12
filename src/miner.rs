pub use crate::blockchain::{
    self,
    block::{self, Block, Header, Transaction},
    Blockchain,
};
use primitive_types::U256;
use std::time::SystemTime;

pub struct Miner<'a> {
    pub blockchain: &'a mut Blockchain,
    pub mempool: Vec<Transaction>,
}

impl Miner<'_> {
    pub fn add_tx_to_mempool(&mut self, tx: Transaction) {
        let mut idx: usize = 0;
        for mempool_tx in self.mempool.iter() {
            if mempool_tx.fee < tx.fee {
                idx += 1;
            }
        }
        self.mempool.insert(idx, tx);
    }

    pub fn compute_next_block(&mut self) {
        let max_tx_count_in_block: usize = self.blockchain.max_transactions_per_block;

        let transactions_copy = {
            let transactions_slice = if self.mempool.len() > max_tx_count_in_block {
                &self.mempool[0..max_tx_count_in_block - 1]
            } else {
                &self.mempool[..]
            };
            transactions_slice.to_vec()
        };

        let tx_count = transactions_copy.len();

        let latest_block_hash = if let Some(block) = self.blockchain.get_latest_block() {
            block.header.hash.clone()
        } else {
            String::new()
        };

        let block: Block = self._compute_next_block(transactions_copy, latest_block_hash);
        if self.blockchain.add_block(block) {
            if self.mempool.len() > tx_count {
                self.mempool = self.mempool[tx_count..].to_vec();
            } else {
                self.mempool.clear();
            }
        }
    }

    fn _compute_next_block(
        &mut self,
        transactions: Vec<Transaction>,
        latest_block_hash: String,
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
                if hash <= self.blockchain.difficulty {
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
