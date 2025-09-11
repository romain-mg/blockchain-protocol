use crate::blockchain::{
    block::{Block, Header, Transaction},
    Blockchain,
};
use crate::mock::mock_miner::Miner;
use k256::ecdsa::Signature;
use primitive_types::U256;
use serde::{Deserialize, Serialize};
use k256::elliptic_curve::sec1::ToEncodedPoint; 


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
        serialized_transaction: Vec<u8>,
        signature: &Signature,
        connected_miner: &mut Miner,
        blockchain: &mut Blockchain,
    ) {
        connected_miner
            .on_transaction_receive(serialized_transaction, signature, blockchain)
            .await;
    }
}
