use super::utils::convert_transaction_to_string;
use k256::{PublicKey};
use primitive_types::U256;
use serde::{Deserialize, Serialize};
use sha256::digest;
use serde_json;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    pub nonce: u64,
    pub timestamp: u64,
    pub prev_hash: String,
    pub difficulty: U256,
    pub merkle_root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: Header,
    pub transactions: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub public_key_from: PublicKey,
    pub public_key_to: PublicKey,
    pub amount: U256,
    pub fee: U256,
    pub nonce: u128,
}

impl Transaction {
    pub fn serialize(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("Transaction to be serialized")
    }

    pub fn deseralize(serialized_tx: &[u8]) -> Self {
        serde_json::from_slice(serialized_tx).expect("Transaction to be deserialized")
    }
}

impl Block {
    pub fn create_block(
        nonce: u64,
        timestamp: u64,
        prev_hash: String,
        transactions: &Vec<Transaction>,
    ) -> Self {
        let merkle_tree = MerkleTree::build_tree(transactions);

        let mut block_merkle_root = String::from("");

        if let Some(ref root) = merkle_tree.root {
            let root_hash = &root.value;
            block_merkle_root.push_str(root_hash);
        }

        let header = Header {
            nonce,
            timestamp,
            prev_hash,
            difficulty: U256::zero(),
            merkle_root: block_merkle_root,
        };

        let mut serialized_transactions: Vec<Vec<u8>>  = vec![];
        for transaction in transactions.iter() {
            serialized_transactions.push(transaction.serialize());
        }

        Self {
            header,
            transactions: serialized_transactions
        }
    }

    pub fn hash_header(header: &Header) -> String {
        let mut hash_string = String::from("");

        hash_string.push_str(&header.nonce.to_string());
        hash_string.push_str(&header.timestamp.to_string());
        hash_string.push_str(&header.prev_hash);
        hash_string.push_str(&header.merkle_root);

        digest(hash_string)
    }

    pub fn get_deseralized_transactions(&self) -> Vec<Transaction> {
        let mut deserialized_transactions: Vec<Transaction>  = vec![];
        for transaction in self.transactions.iter() {
        deserialized_transactions.push(Transaction::deseralize(transaction));
    }
        deserialized_transactions
    }
}

#[derive(Debug, Clone)]
pub struct MerkleNode {
    pub left: Option<Box<MerkleNode>>,
    pub right: Option<Box<MerkleNode>>,
    pub value: String,
}

impl MerkleNode {
    pub fn hash(val: String) -> String {
        digest(val)
    }

    pub fn get_value(&self) -> &str {
        &self.value
    }
}

#[derive(Debug)]
pub struct MerkleTree {
    pub root: Option<Box<MerkleNode>>,
}

impl MerkleTree {
    pub fn build_tree(transactions: &Vec<Transaction>) -> Self {
        if transactions.is_empty() {
            return Self { root: None };
        }
        let mut nodes: Vec<MerkleNode> = transactions
            .iter()
            .map(|tx| MerkleNode {
                left: None,
                right: None,
                value: digest(&convert_transaction_to_string(tx)),
            })
            .collect();

        while nodes.len() > 1 {
            let mut next_level: Vec<MerkleNode> = Vec::new();
            let mut i = 0;
            while i < nodes.len() {
                let left = nodes[i].clone();
                let right = if i + 1 < nodes.len() {
                    nodes[i + 1].clone()
                } else {
                    left.clone()
                };
                let combined_hash = digest(&(left.value.clone() + &right.value));
                let parent = MerkleNode {
                    left: Some(Box::new(left)),
                    right: Some(Box::new(right)),
                    value: combined_hash,
                };
                next_level.push(parent);
                i += 2;
            }
            nodes = next_level;
        }

        Self {
            root: Some(Box::new(nodes[0].clone())),
        }
    }

    pub fn get_root(&self) -> Option<String> {
        if let Some(ref root) = self.root {
            Some(root.value.clone())
        } else {
            None
        }
    }
}
