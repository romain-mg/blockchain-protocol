pub mod account;
pub mod block;
pub mod utils;

use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

pub use account::AccountKeys;
use block::MerkleTree;
pub use block::{Block, Header, Transaction};
use k256::ecdsa::VerifyingKey;
use multimap::MultiMap;
use primitive_types::U256;
use uint::FromStrRadixErr;
use utils::convert_public_key_to_bytes;

#[derive(Debug)]
pub struct Blockchain {
    pub hash_to_block: HashMap<String, Block>,
    pub hash_to_miner: HashMap<String, VerifyingKey>,
    pub block_parent_map: HashMap<String, String>,
    pub parent_block_map: HashMap<String, String>,
    pub hash_to_cumulative_difficulty: HashMap<String, U256>,
    pub cumulative_difficulty_to_hash: MultiMap<U256, String>,
    pub difficulty: U256,
    pub target_duration_between_blocks: u64,
    pub latest_block_timestamp: u64,
    pub max_transactions_per_block: usize,
    pub accounts: HashMap<[u8; 33], AccountState>,
    pub mining_reward: U256,
    pub current_longest_chain_latest_block_hash: String,
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
        let mut hash_to_cumulative_difficulty = HashMap::new();
        let mut cumulative_difficulty_to_hash = MultiMap::new();
        hash_to_cumulative_difficulty.insert(String::from(""), U256::zero());
        cumulative_difficulty_to_hash.insert(U256::zero(), String::from(""));
        Self {
            hash_to_block: HashMap::new(),
            hash_to_miner: HashMap::new(),
            block_parent_map: HashMap::new(),
            parent_block_map: HashMap::new(),
            hash_to_cumulative_difficulty,
            cumulative_difficulty_to_hash,
            difficulty,
            target_duration_between_blocks,
            latest_block_timestamp: 0,
            max_transactions_per_block,
            accounts: HashMap::new(),
            mining_reward: U256::from(1000),
            current_longest_chain_latest_block_hash: String::from(""),
        }
    }

    pub fn get_block(&self, hash: &String) -> Option<&Block> {
        self.hash_to_block.get(hash)
    }

    pub fn add_block(&mut self, mut block: Block, miner_public_key: VerifyingKey) -> bool {
        let block_merkle_root = &block.header.merkle_root;
        let recomputed_merkle_root = &MerkleTree::build_tree(&block.transactions.clone())
            .root
            .expect("Blockchain cannot add block: Merkle root is None")
            .value;
        if block_merkle_root != recomputed_merkle_root {
            return false;
        }

        block.header.difficulty = self.difficulty;
        let block_hash = Block::hash_header(&block.header);
        let block_prev_hash = &block.header.prev_hash;
        if block.header.prev_hash != "" {
            let block_hash_u256: Result<U256, FromStrRadixErr> =
                U256::from_str_radix(&block_hash, 16);
            let parent_block = self.hash_to_block.get(block_prev_hash).unwrap();
            match block_hash_u256 {
                Ok(hash) => {
                    let difficulty_variation = U256::from_str("60000").unwrap();
                    if hash > self.difficulty {
                        return false;
                    }
                    if block.header.timestamp - parent_block.header.timestamp
                        > self.target_duration_between_blocks + 2
                        && self.difficulty < U256::MAX
                    {
                        self.difficulty += difficulty_variation;
                    } else if block.header.timestamp - parent_block.header.timestamp
                        < self.target_duration_between_blocks - 2
                    {
                        self.difficulty -= difficulty_variation;
                    }
                }
                Err(err) => {
                    println!(
                        "Error: cannot parse block hash {}: encountered {}",
                        &block_hash, err
                    );
                    return false;
                }
            }
        }
        self.hash_to_miner
            .insert(block_hash.clone(), miner_public_key.clone());
        self.hash_to_block.insert(block_hash.clone(), block.clone());
        self.block_parent_map
            .insert(block_hash.clone(), block_prev_hash.clone());

        // Apply the longest chain rule
        let current_longest_chain_latest_block_hash =
            self.current_longest_chain_latest_block_hash.clone();
        let total_block_difficulty =
            self.hash_to_cumulative_difficulty[&block_prev_hash.clone()] + block.header.difficulty;
        if block_prev_hash != &current_longest_chain_latest_block_hash {
            let current_longest_chain_latest_block_difficulty = self
                .hash_to_cumulative_difficulty
                .get(&current_longest_chain_latest_block_hash)
                .unwrap();
            if &total_block_difficulty > current_longest_chain_latest_block_difficulty {
                self.reorg_to_new_longest_chain(block_hash.clone());
                if self.parent_block_map.contains_key(&block_prev_hash.clone()) {
                    self.parent_block_map
                        .insert(block_prev_hash.clone(), block_hash.clone());
                }
            }
        } else {
            self.current_longest_chain_latest_block_hash = block_hash.clone();
            self.parent_block_map
                .insert(block_prev_hash.clone(), block_hash.clone());
            self.apply_block_transactions(&block_hash);
        }

        self.hash_to_cumulative_difficulty
            .insert(block_hash.clone(), total_block_difficulty);
        self.cumulative_difficulty_to_hash
            .insert(total_block_difficulty, block_hash.clone());
        return true;
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

    pub fn reorg_to_new_longest_chain(&mut self, block_hash: String) {
        let mut curr_old_chain_block_hash = self.current_longest_chain_latest_block_hash.clone();
        let mut old_chain_block_hashes: HashSet<String> = HashSet::new();
        let mut old_chain_block_hashes_vec = vec![];
        while curr_old_chain_block_hash != String::from("") {
            old_chain_block_hashes.insert(curr_old_chain_block_hash.clone());
            old_chain_block_hashes_vec.push(curr_old_chain_block_hash.clone());
            curr_old_chain_block_hash = self
                .block_parent_map
                .get(&curr_old_chain_block_hash)
                .unwrap()
                .clone();
        }
        let mut curr_new_chain_block_hash = block_hash.clone();
        let mut new_chain_block_hashes: HashSet<String> = HashSet::new();
        let mut new_chain_block_hashes_vec = vec![];
        while curr_new_chain_block_hash != String::from("") {
            new_chain_block_hashes.insert(curr_new_chain_block_hash.clone());
            new_chain_block_hashes_vec.push(curr_new_chain_block_hash.clone());
            curr_new_chain_block_hash = self
                .block_parent_map
                .get(&curr_new_chain_block_hash)
                .unwrap()
                .clone();
        }

        let fork_hash = old_chain_block_hashes
            .intersection(&new_chain_block_hashes)
            .next()
            .expect("No fork hash found");

        let fork_hash_idx_in_old_block_vec = old_chain_block_hashes_vec
            .iter()
            .position(|h| h == fork_hash)
            .unwrap();

        let old_chain_block_hashes_vec_slice =
            &old_chain_block_hashes_vec[..fork_hash_idx_in_old_block_vec + 1];

        for old_chain_block_hash in old_chain_block_hashes_vec_slice.iter() {
            self.revert_block_transactions(&old_chain_block_hash);
        }

        new_chain_block_hashes_vec.reverse();
        let fork_hash_idx_in_new_block_vec = new_chain_block_hashes_vec
            .iter()
            .position(|h| h == fork_hash)
            .unwrap();
        let new_chain_block_hashes_vec_slice =
            &new_chain_block_hashes_vec[fork_hash_idx_in_new_block_vec..];
        for new_chain_block_hash in new_chain_block_hashes_vec_slice.iter() {
            self.apply_block_transactions(&new_chain_block_hash);
        }
    }

    fn apply_block_transactions(&mut self, block_hash: &str) -> bool {
        let block = self
            .hash_to_block
            .get(block_hash)
            .expect("Block does not exist.");
        let miner_public_key = self.hash_to_miner.get(block_hash).unwrap();
        let mut miner_account_balance = self
            .accounts
            .get(&convert_public_key_to_bytes(&miner_public_key))
            .unwrap()
            .balance;
        for transaction in block.transactions.iter() {
            let sender_public_key = &transaction.public_key_from;
            let sender_account = self
                .accounts
                .get_mut(&convert_public_key_to_bytes(sender_public_key));
            if sender_account.is_none() {
                return false;
            }
            let sender_account_state = sender_account.expect("Sender account does not exist");
            if transaction.nonce != sender_account_state.nonce {
                return false;
            }
            if sender_account_state.balance < transaction.amount + transaction.fee {
                return false;
            }
            sender_account_state.balance -= transaction.amount + transaction.fee;
            sender_account_state.nonce += 1;

            let receiver_public_key = &transaction.public_key_to;
            let receiver_account = self
                .accounts
                .get_mut(&convert_public_key_to_bytes(receiver_public_key));
            let receiver_account_state = receiver_account.expect("Receiver account does not exist");
            receiver_account_state.balance += transaction.amount;
            miner_account_balance += transaction.fee;
        }
        let miner_account = self
            .accounts
            .get_mut(&convert_public_key_to_bytes(&miner_public_key))
            .unwrap();
        miner_account.balance = miner_account_balance + self.mining_reward;
        return true;
    }

    fn revert_block_transactions(&mut self, block_hash: &str) -> bool {
        let block = self
            .hash_to_block
            .get(block_hash)
            .expect("Block does not exist.");
        let miner_public_key = self.hash_to_miner.get(block_hash).unwrap();
        let mut miner_account_balance = self
            .accounts
            .get(&convert_public_key_to_bytes(&miner_public_key))
            .unwrap()
            .balance;
        for transaction in block.transactions.iter() {
            let sender_public_key = &transaction.public_key_from;
            let sender_account = self
                .accounts
                .get_mut(&convert_public_key_to_bytes(sender_public_key));
            let sender_account_state = sender_account.expect("Sender account does not exist");
            sender_account_state.balance += transaction.amount + transaction.fee;
            sender_account_state.nonce -= 1;
            let receiver_public_key = &transaction.public_key_to;
            let receiver_account = self
                .accounts
                .get_mut(&convert_public_key_to_bytes(receiver_public_key));
            let receiver_account_state = receiver_account.expect("Receiver account does not exist");
            receiver_account_state.balance -= transaction.amount;
            miner_account_balance -= transaction.fee;
        }
        let miner_account = self
            .accounts
            .get_mut(&convert_public_key_to_bytes(&miner_public_key))
            .unwrap();
        miner_account.balance = miner_account_balance - self.mining_reward;
        return true;
    }
}
