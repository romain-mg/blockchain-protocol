pub use super::block::Transaction;
use k256::{PublicKey, ecdsa::VerifyingKey};
use sha256::digest;
use k256::elliptic_curve::sec1::ToEncodedPoint; 

pub fn hash_transaction(transaction: &Transaction) -> String {
    digest(convert_transaction_to_string(transaction))
}

pub fn convert_transaction_to_string(transaction: &Transaction) -> String {
    let public_key_from_string = transaction
        .public_key_from
        .to_encoded_point(true)
        .to_string();
    let public_key_to_string = transaction.public_key_to.to_encoded_point(true).to_string();
    public_key_from_string
        + &public_key_to_string
        + &transaction.amount.to_string()
        + &transaction.fee.to_string()
}

pub fn convert_public_key_to_bytes(public_key: &PublicKey) -> Vec<u8> {
    let encoded_public_key = public_key.to_encoded_point(true);
    encoded_public_key
        .as_bytes()
        .try_into()
        .expect("Public key should be 33 bytes")
}
