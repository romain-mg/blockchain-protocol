use super::utils::concat_transaction;
use k256::ecdsa::VerifyingKey;
use sha256::digest;

#[derive(Debug)]
pub struct Header {
    pub nonce: u64,
    pub timestamp: u64,
    pub hash: String,
    pub prev_hash: String,
    pub merkle_root: String,
}

#[derive(Debug)]
pub struct Block {
    pub header: Header,
    pub transactions: MerkleTree,
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub public_key_from: VerifyingKey,
    pub public_key_to: VerifyingKey,
    pub amount: u32,
    pub fee: u32,
}

impl Block {
    pub fn create_block(
        nonce: u64,
        timestamp: u64,
        prev_hash: String,
        transactions: &Vec<Transaction>,
    ) -> Self {
        let mut merkle_tree: MerkleTree = MerkleTree { root: None };
        merkle_tree.build_tree(transactions);

        let mut block_merkle_root = String::from("");

        if let Some(ref root) = merkle_tree.root {
            let root_hash = &root.value;
            block_merkle_root.push_str(root_hash);
        }

        let mut header = Header {
            nonce,
            timestamp,
            hash: String::from(""),
            prev_hash,
            merkle_root: block_merkle_root,
        };

        header.hash = Block::hash_header(&header);

        Self {
            header,
            transactions: merkle_tree,
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

#[derive(Debug)]
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
    pub fn build_tree(&mut self, transactions: &Vec<Transaction>) {
        if transactions.len() == 0 {
            return;
        }
        self.root = self._build_tree_helper(&transactions[0], 0, &transactions);
    }

    fn _build_tree_helper(
        &self,
        transaction: &Transaction,
        index: usize,
        transactions: &Vec<Transaction>,
    ) -> Option<Box<MerkleNode>> {
        let transactions_count: usize = transactions.len();
        let concat_transaction = concat_transaction(transaction);

        let left_child_index;
        let right_child_index;

        if index == 0 {
            left_child_index = 1;
            right_child_index = 2;
        } else {
            left_child_index = index * 2 + 1;
            right_child_index = index * 2 + 2;
        }
        if left_child_index >= transactions_count {
            return Some(Box::new(MerkleNode {
                left: None,
                right: None,
                value: digest(&concat_transaction),
            }));
        } else {
            let left_node: Option<Box<MerkleNode>> = self._build_tree_helper(
                &transactions[left_child_index],
                left_child_index,
                transactions,
            );

            let mut right_node: Option<Box<MerkleNode>> = None;

            if right_child_index < transactions_count {
                right_node = self._build_tree_helper(
                    &transactions[right_child_index],
                    right_child_index,
                    transactions,
                );
            }

            let mut child_hashes: String = String::from("");

            if let Some(ref left) = left_node {
                child_hashes.push_str(&left.value);
                if let Some(ref right) = right_node {
                    child_hashes.push_str(&right.value);
                } else {
                    child_hashes.push_str(&left.value);
                }
            }

            Some(Box::new(MerkleNode {
                left: left_node,
                right: right_node,
                value: digest(child_hashes),
            }))
        }
    }
}
