pub mod account;
pub mod block;
pub mod utils;

use std::{collections::HashMap, str::FromStr};

pub use account::Account;
pub use block::{Block, Header, Transaction};
use k256::ecdsa::VerifyingKey;
use primitive_types::U256;
use uint::FromStrRadixErr;

#[derive(Debug)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub difficulty: U256,
    pub target_duration_between_blocks: u64,
    pub latest_block_timestamp: u64,
    pub max_transactions_per_block: usize,
    pub accounts: HashMap<[u8; 33], U256>,
}

impl Blockchain {
    pub fn create_blockchain(
        difficulty: U256,
        target_duration_between_blocks: u64,
        max_transactions_per_block: usize,
    ) -> Self {
        Self {
            blocks: Vec::new(),
            difficulty,
            target_duration_between_blocks,
            latest_block_timestamp: 0,
            max_transactions_per_block,
            accounts: HashMap::new(),
        }
    }

    pub fn get_block(&self, index: usize) -> Option<&Block> {
        self.blocks.get(index)
    }

    pub fn get_latest_block(&self) -> Option<&Block> {
        if self.blocks.len() == 0 {
            return None;
        }
        let latest_index: usize = self.blocks.len() - 1 as usize;
        self.blocks.get(latest_index)
    }

    pub fn add_block(&mut self, mut block: Block) -> bool {
        let blocks_length: usize = self.blocks.len();
        if blocks_length == 0 {
            self.blocks.push(block);
        } else if self.blocks.len() > 0 {
            let latest_block: &Block = &self.blocks[blocks_length - 1];
            if block.header.prev_hash != latest_block.header.hash {
                return false;
            }
            let block_hash: Result<U256, FromStrRadixErr> =
                U256::from_str_radix(&block.header.hash, 16);

            match block_hash {
                Ok(hash) => {
                    let difficulty_variation = U256::from_str("60000").unwrap();
                    if hash > self.difficulty {
                        return false;
                    }
                    if block.header.timestamp - latest_block.header.timestamp
                        > self.target_duration_between_blocks + 2
                        && self.difficulty < U256::MAX
                    {
                        self.difficulty += difficulty_variation;
                    } else if block.header.timestamp - latest_block.header.timestamp
                        < self.target_duration_between_blocks - 2
                    {
                        self.difficulty -= difficulty_variation;
                    }
                    let hash = Block::hash_header(&block.header);
                    block.header.hash = hash;
                    self.latest_block_timestamp = block.header.timestamp;
                    self.blocks.push(block);
                    return true;
                }
                Err(err) => {
                    print!(
                        "Error: cannot parse block hash {}: encountered {}",
                        block.header.hash, err
                    );
                    return false;
                }
            }
        }
        return false;
    }

    pub fn set_difficulty(&mut self, new_difficulty: U256) {
        self.difficulty = new_difficulty;
    }

    pub fn get_balance(&self, account: &VerifyingKey) -> U256 {
        let encoded_public_key = account.to_encoded_point(true);
        let public_key_bytes = encoded_public_key.as_bytes();
        *self.accounts.get(public_key_bytes).unwrap_or(&U256::zero())
    }

    pub fn create_account(&mut self, public_key: VerifyingKey) -> VerifyingKey {
        let encoded_public_key = public_key.to_encoded_point(true);
        let public_key_bytes: [u8; 33] = encoded_public_key
            .as_bytes()
            .try_into()
            .expect("Public key should be 33 bytes");
        self.accounts.insert(public_key_bytes, U256::zero());
        public_key
    }
}
