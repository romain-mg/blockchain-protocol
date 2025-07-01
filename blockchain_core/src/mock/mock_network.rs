use crate::blockchain::{
    block::{Block, Header, Transaction},
    Blockchain,
};
use crate::mock::mock_miner::Miner;
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

    pub async fn send_transaction(
        &mut self,
        transaction: Transaction,
        signature: &Signature,
        connected_miner: &mut Miner,
        blockchain: &mut Blockchain,
    ) {
        connected_miner
            .on_transaction_receive(transaction, signature, blockchain)
            .await;
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
