pub use crate::blockchain::{
    self,
    block::{self, Block, Header, Transaction},
};
use crate::Blockchain;
use primitive_types::U256;
use std::time::SystemTime;

pub struct Miner<'a> {
    pub blockchain: &'a mut Blockchain,
}

impl Miner<'_> {
    pub fn compute_next_block(&mut self, transactions: Vec<Transaction>) {
        let mut latest_block_hash: String = String::new();

        let blockchain_latest_block: Option<&Block> = self.blockchain.get_latest_block();
        match blockchain_latest_block {
            None => None,
            Some(blockchain_latest_block) => Some({
                latest_block_hash.push_str(&blockchain_latest_block.header.hash);
            }),
        };
        let block: Block = self._compute_next_block(transactions, latest_block_hash);
        self.blockchain.add_block(block);
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
        let mut block: Block = Block::create_block(
            nonce,
            timestamp,
            latest_block_hash.clone(),
            transactions.clone(),
        );
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
            block = Block::create_block(
                nonce,
                timestamp,
                latest_block_hash.clone(),
                transactions.clone(),
            );
        }
        block
    }
}
