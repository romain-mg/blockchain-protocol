pub use crate::blockchain::{
    self,
    block::{self, Block, Header, Transaction},
    utils::{convert_public_key_to_bytes, hash_transaction},
    Blockchain,
};
use k256::ecdsa::{signature::Verifier, Signature};
use primitive_types::U256;
use std::time::SystemTime;

pub struct Miner {
    pub mempool: Vec<Transaction>,
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
            blockchain.create_account(public_key.clone());
            account = blockchain.get_account(public_key);
        }
        let unwraped_account = account.unwrap();
        if transaction.nonce < unwraped_account.nonce {
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
            let processed_txn_account = temp_account_state.get_mut(public_key_bytes).unwrap();
            if processed_txn.nonce != processed_txn_account.nonce {
                transactions_copy.remove(i);
            } else {
                i += 1;
                processed_txn_account.nonce += 1;
            }
        }

        let transaction_count = transactions_copy.len();

        let latest_block_hash = if let Some(block) = blockchain.get_latest_block() {
            Block::hash_header(&block.header)
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
}
