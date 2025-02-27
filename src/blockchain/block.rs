use super::utils::serialize_transaction;
use k256::ecdsa::VerifyingKey;
use primitive_types::U256;
use sha256::digest;

#[derive(Debug)]
pub struct Header {
    pub nonce: u64,
    pub timestamp: u64,
    pub prev_hash: String,
    pub merkle_root: String,
}

#[derive(Debug)]
pub struct Block {
    pub header: Header,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub public_key_from: VerifyingKey,
    pub public_key_to: VerifyingKey,
    pub amount: U256,
    pub fee: U256,
    pub nonce: u128,
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
            merkle_root: block_merkle_root,
        };

        Self {
            header,
            transactions: transactions.clone(),
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
                value: digest(&serialize_transaction(tx)),
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
