pub mod account;
pub mod block;
pub mod utils;

use std::{collections::HashMap, str::FromStr};

pub use account::AccountKeys;
use block::MerkleTree;
pub use block::{Block, Header, Transaction};
use k256::ecdsa::VerifyingKey;
use primitive_types::U256;
use uint::FromStrRadixErr;
use utils::convert_public_key_to_bytes;

#[derive(Debug)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub difficulty: U256,
    pub target_duration_between_blocks: u64,
    pub latest_block_timestamp: u64,
    pub max_transactions_per_block: usize,
    pub accounts: HashMap<[u8; 33], AccountState>,
}

#[derive(Debug, Clone)]
pub struct AccountState {
    pub balance: U256,
    pub nonce: u128,
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

    pub fn add_block(&mut self, block: Block) -> bool {
        let block_merkle_root = &block.header.merkle_root;
        let recomputed_merkle_root = &MerkleTree::build_tree(&block.transactions.clone())
            .root
            .unwrap()
            .value;
        if block_merkle_root != recomputed_merkle_root {
            return false;
        }
        let blocks_length: usize = self.blocks.len();
        if blocks_length == 0 {
            self.blocks.push(block);
        } else if self.blocks.len() > 0 {
            let latest_block: &Block = &self.blocks[blocks_length - 1];
            if block.header.prev_hash != Block::hash_header(&latest_block.header) {
                return false;
            }
            let block_hash = Block::hash_header(&block.header);
            let block_hash_u256: Result<U256, FromStrRadixErr> =
                U256::from_str_radix(&block_hash, 16);

            match block_hash_u256 {
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

                    for transaction in block.transactions.iter() {
                        let sender_public_key = &transaction.public_key_from;
                        let sender_account = self
                            .accounts
                            .get_mut(&convert_public_key_to_bytes(sender_public_key));
                        if sender_account.is_none() {
                            return false;
                        }
                        let sender_account_state =
                            sender_account.expect("Sender account does not exist");
                        if transaction.nonce != sender_account_state.nonce {
                            return false;
                        }
                        if sender_account_state.balance < transaction.amount {
                            return false;
                        }
                        sender_account_state.balance -= transaction.amount;
                        sender_account_state.nonce += 1;

                        let receiver_public_key = &transaction.public_key_to;
                        let receiver_account = self
                            .accounts
                            .get_mut(&convert_public_key_to_bytes(receiver_public_key));
                        if receiver_account.is_none() {
                            return false;
                        }
                        let receiver_account_state =
                            receiver_account.expect("Sender account does not exist");
                        receiver_account_state.balance += transaction.amount;
                    }

                    self.latest_block_timestamp = block.header.timestamp;
                    self.blocks.push(block);
                    return true;
                }
                Err(err) => {
                    print!(
                        "Error: cannot parse block hash {}: encountered {}",
                        block_hash, err
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

    pub fn get_account(&self, public_key: &VerifyingKey) -> Option<&AccountState> {
        let encoded_public_key = public_key.to_encoded_point(true);
        let public_key_bytes = encoded_public_key.as_bytes();
        self.accounts.get(public_key_bytes)
    }

    pub fn get_balance(&mut self, public_key: &VerifyingKey) -> U256 {
        let account = self.get_account(public_key);
        if account.is_some() {
            account.unwrap().balance
        } else {
            let new_account = self.create_account(public_key);
            new_account.balance
        }
    }

    pub fn create_account(&mut self, public_key: &VerifyingKey) -> &AccountState {
        let public_key_bytes = convert_public_key_to_bytes(public_key);
        let new_account: AccountState = AccountState {
            balance: U256::zero(),
            nonce: 0,
        };
        self.accounts.insert(public_key_bytes, new_account);
        &self.accounts[&public_key_bytes]
    }

    pub fn mint(&mut self, public_key: &VerifyingKey, amount: U256) {
        let _account = self
            .accounts
            .get_mut(&convert_public_key_to_bytes(public_key));
        _account.expect("Account does not exist").balance += amount;
    }
}
