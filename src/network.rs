use crate::blockchain::{
    block::{Block, Header, Transaction},
    Blockchain,
};
use crate::miner::Miner;
use k256::ecdsa::Signature;
use primitive_types::U256;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq)]
pub struct Network {
    pub miners: Vec<Miner>,
}

impl Network {
    pub fn new() -> Network {
        Network { miners: Vec::new() }
    }

    pub fn add_miner(&mut self, miner: Miner) {
        self.miners.push(miner);
    }

    pub fn send_transaction(
        &mut self,
        transaction: Transaction,
        signature: &Signature,
        mut connected_miner: Miner,
        blockchain: &mut Blockchain,
    ) {
        connected_miner.add_transaction_to_mempool(transaction, signature, blockchain);
    }

    pub fn broadcast_block(&self, sender_miner: &Miner, block: Block, blockchain: &mut Blockchain) {
        if !self.miners.contains(&sender_miner) {
            return;
        }
        self.send_block_to_miners(block, sender_miner, blockchain);
    }

    fn send_block_to_miners(
        &self,
        block: Block,
        sender_miner: &Miner,
        blockchain: &mut Blockchain,
    ) {
        for miner in self.miners.iter() {
            if miner.account_keys.get_public_key() == sender_miner.account_keys.get_public_key() {
                continue;
            }
            println!(
                "Sending block {:?} to miner: {:?}",
                block,
                miner.account_keys.get_public_key()
            );
            miner.add_block_to_blockchain(block.clone(), blockchain);
        }
    }

    pub fn serialize_block(block: Block) -> String {
        let ready_to_serialize_block = ReadyToSerializeBlock::new(block);
        serde_json::to_string(&ready_to_serialize_block).unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadyToSerializeBlock {
    pub header: Header,
    pub transactions: Vec<SerializedTransaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedTransaction {
    pub public_key_from: Box<[u8]>,
    pub public_key_to: Box<[u8]>,
    pub amount: U256,
    pub fee: U256,
    pub nonce: u128,
}

impl ReadyToSerializeBlock {
    pub fn new(block: Block) -> Self {
        let mut serialized_transactions = Vec::new();
        for transaction in block.transactions.iter() {
            serialized_transactions.push(SerializedTransaction {
                public_key_from: transaction
                    .public_key_from
                    .to_encoded_point(true)
                    .to_bytes(),
                public_key_to: transaction.public_key_to.to_encoded_point(true).to_bytes(),
                amount: transaction.amount,
                fee: transaction.fee,
                nonce: transaction.nonce,
            });
        }
        Self {
            header: block.header,
            transactions: serialized_transactions,
        }
    }
}
