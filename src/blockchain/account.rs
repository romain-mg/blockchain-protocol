pub use crate::blockchain::{
    self,
    block::{self, Block, Header, Transaction},
    Blockchain,
};
use k256::ecdsa::{signature::SignerMut, Signature, SigningKey, VerifyingKey};
use primitive_types::U256;
use rand::rngs::OsRng;

pub struct Account {
    public_key: VerifyingKey,
    private_key: SigningKey,
}

impl Account {
    pub fn get_balance(&self, blockchain: &Blockchain) -> U256 {
        blockchain.get_balance(&self.public_key)
    }

    pub fn sign_transaction(&mut self, transaction_hash: &[u8]) -> Signature {
        self.private_key.sign(transaction_hash)
    }

    pub fn new(blockchain: &mut Blockchain) -> Self {
        let private_key = SigningKey::random(&mut OsRng);
        let public_key = VerifyingKey::from(&private_key);
        blockchain.create_account(private_key.clone());
        Self {
            private_key,
            public_key,
        }
    }
}
