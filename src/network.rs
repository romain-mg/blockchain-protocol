use crate::blockchain::{
    block::{Block, Transaction},
    Blockchain,
};
use crate::miner::Miner;
use k256::ecdsa::Signature;

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
            miner.add_block_to_blockchain(block.clone(), blockchain);
        }
    }
}
