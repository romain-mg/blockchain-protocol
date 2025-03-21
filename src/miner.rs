pub use crate::blockchain::{
    self,
    account::AccountKeys,
    block::{self, Block, Header, MerkleTree, Transaction},
    utils::{convert_public_key_to_bytes, hash_transaction},
    Blockchain,
};
use crate::network::Network;
use k256::ecdsa::{signature::Verifier, Signature};
use primitive_types::U256;
use std::time::SystemTime;
use uint::FromStrRadixErr;

#[derive(Clone, PartialEq)]
pub struct Miner {
    pub mempool: Vec<Transaction>,
    pub account_keys: AccountKeys,
    pub network: Network,
}

impl Miner {
    pub fn add_transaction_to_mempool(
        &mut self,
        transaction: Transaction,
        signature: &Signature,
        blockchain: &mut Blockchain,
    ) {
        if !(transaction
            .public_key_from
            .verify(hash_transaction(&transaction).as_bytes(), signature)
            .is_ok())
        {
            return;
        }

        let public_key = &transaction.public_key_from;
        let mut account = blockchain.get_account(public_key);
        if account.is_none() {
            blockchain.create_account(&public_key);
            account = blockchain.get_account(public_key);
        }
        let unwraped_account = account.expect("Account not existing");
        if transaction.nonce < unwraped_account.nonce {
            return;
        }
        if unwraped_account.balance < transaction.amount + transaction.fee {
            return;
        }

        let mut idx: usize = 0;
        for mempool_transaction in self.mempool.iter() {
            if mempool_transaction.fee > transaction.fee {
                idx += 1;
            }
        }
        self.mempool.insert(idx, transaction);
    }

    pub fn compute_next_block(
        &mut self,
        blockchain: &mut Blockchain,
        parent_block_hash: String,
    ) -> Option<String> {
        let max_transaction_count_in_block: usize = blockchain.max_transactions_per_block;

        let mut transactions_copy = {
            let transactions_slice = if self.mempool.len() > max_transaction_count_in_block {
                &self.mempool[0..max_transaction_count_in_block - 1]
            } else {
                &self.mempool[..]
            };
            transactions_slice.to_vec()
        };
        transactions_copy.sort_by(|a, b| a.nonce.cmp(&b.nonce));
        let mut temp_account_state = blockchain.accounts.clone();

        let mut i = 0;
        while i < transactions_copy.len() {
            let processed_txn = &transactions_copy[i];
            let public_key_bytes = &convert_public_key_to_bytes(&processed_txn.public_key_from);
            let processed_txn_sender = temp_account_state.get_mut(public_key_bytes).unwrap();
            if processed_txn.nonce != processed_txn_sender.nonce
                || processed_txn_sender.balance < processed_txn.amount + processed_txn.fee
            {
                transactions_copy.remove(i);
            } else {
                i += 1;
                processed_txn_sender.nonce += 1;
                processed_txn_sender.balance -= processed_txn.amount + processed_txn.fee;
            }
        }

        let transaction_count = transactions_copy.len();

        let block: Block =
            self._compute_next_block(transactions_copy, parent_block_hash.clone(), &blockchain);
        if blockchain.add_block(block.clone(), self.account_keys.get_public_key()) {
            if self.mempool.len() > transaction_count {
                self.mempool = self.mempool[transaction_count..].to_vec();
            } else {
                self.mempool.clear();
            }
            self.network
                .broadcast_block(self, block.clone(), blockchain);
            return Some(Block::hash_header(&block.header));
        }
        return None;
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
        let nonce = 1;
        let mut block: Block =
            Block::create_block(nonce, timestamp, latest_block_hash.clone(), &transactions);
        loop {
            if let Ok(hash) = Block::hash_header(&block.header).parse::<U256>() {
                if hash <= blockchain.difficulty {
                    break;
                }
            }
            block.header.nonce += 1;
            match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                Ok(n) => timestamp = n.as_secs(),
                Err(_) => panic!("SystemTime before UNIX EPOCH!"),
            }
            block.header.timestamp = timestamp;
        }
        block
    }

    pub fn new(blockchain: &mut Blockchain, network: Network) -> Self {
        let miner = Miner {
            mempool: Vec::new(),
            account_keys: AccountKeys::new(),
            network,
        };
        blockchain.create_account(&miner.account_keys.get_public_key());
        miner
    }

    pub fn add_block_to_blockchain(&self, block: Block, blockchain: &mut Blockchain) {
        if !self.validate_block(block.clone(), &blockchain) {
            return;
        }
        blockchain.add_block(block, self.account_keys.get_public_key());
    }

    fn validate_block(&self, block: Block, blockchain: &Blockchain) -> bool {
        let block_merkle_root = &block.header.merkle_root;
        let recomputed_merkle_root = &MerkleTree::build_tree(&block.transactions.clone())
            .root
            .expect("Merkle root is None")
            .value;
        if block_merkle_root != recomputed_merkle_root {
            return false;
        }

        if block.header.prev_hash == "" {
            return true;
        }

        let block_hash = Block::hash_header(&block.header);
        let block_hash_u256: Result<U256, FromStrRadixErr> = U256::from_str_radix(&block_hash, 16);
        match block_hash_u256 {
            Ok(hash) => {
                if hash > blockchain.difficulty {
                    return false;
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
        return true;
    }
}
