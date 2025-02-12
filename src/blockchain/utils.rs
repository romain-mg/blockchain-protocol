pub use super::block::Transaction;
use sha256::digest;

pub fn hash_transaction(transaction: &Transaction) -> String {
    digest(concat_transaction(transaction))
}

pub fn concat_transaction(transaction: &Transaction) -> String {
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
