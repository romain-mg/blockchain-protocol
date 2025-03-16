pub use crate::blockchain::{
    self,
    block::{self, Block, Header, Transaction},
    utils::hash_transaction,
    Blockchain,
};
use k256::ecdsa::{signature::SignerMut, Signature, SigningKey, VerifyingKey};
use primitive_types::U256;
use rand::rngs::OsRng;

#[derive(Clone)]
pub struct AccountKeys {
    pub public_key: VerifyingKey,
    private_key: SigningKey,
}

impl AccountKeys {
    pub fn get_balance(&self, blockchain: &mut Blockchain) -> U256 {
        blockchain.get_balance(&self.public_key)
    }

    pub fn sign_transaction(&mut self, transaction: &Transaction) -> Signature {
        let transaction_hash = hash_transaction(transaction);
        self.private_key.sign(transaction_hash.as_bytes())
    }

    pub fn new() -> Self {
        let private_key = SigningKey::random(&mut OsRng);
        let public_key = VerifyingKey::from(&private_key);
        Self {
            private_key,
            public_key,
        }
    }

    pub fn get_public_key(&self) -> VerifyingKey {
        self.public_key
    }
}
