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
    pub address_from: String,
    pub address_to: String,
    pub amount: u32,
}

impl Block {
    pub fn create_block(
        nonce: u64,
        timestamp: u64,
        prev_hash: String,
        transactions: Vec<Transaction>,
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
    left: Option<Box<MerkleNode>>,
    right: Option<Box<MerkleNode>>,
    pub value: String,
    content: String,
}

impl Clone for MerkleNode {
    fn clone(&self) -> Self {
        Self {
            left: self.left.clone(),
            right: self.right.clone(),
            value: self.value.clone(),
            content: self.content.clone(),
        }
    }
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
    root: Option<Box<MerkleNode>>,
}

impl MerkleTree {
    pub fn build_tree(&mut self, transactions: Vec<Transaction>) {
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
        let mut concat_transaction = String::from(&transaction.address_from);
        concat_transaction.push_str(&transaction.address_to);
        concat_transaction.push_str(&transaction.amount.to_string());
        if index * 2 >= transactions_count {
            return Some(Box::new(MerkleNode {
                left: None,
                right: None,
                value: digest(&concat_transaction),
                content: concat_transaction,
            }));
        } else {
            let left_index: usize = index * 2;
            let right_index: usize = index * 2 + 1;
            let mut left_node: Option<Box<MerkleNode>> = None;
            let mut right_node: Option<Box<MerkleNode>> = None;

            if left_index < transactions_count {
                left_node =
                    self._build_tree_helper(&transactions[left_index], left_index, transactions);
            }

            if right_index < transactions_count {
                right_node =
                    self._build_tree_helper(&transactions[right_index], right_index, transactions);
            }

            let mut node_value = String::from("");
            let mut node_content = String::from("");

            if let Some(ref left) = left_node {
                node_value.push_str(&left.value);
                node_content.push_str(&left.content);
            }

            if let Some(ref right) = right_node {
                node_value.push_str(&right.value);
                node_content.push_str(&right.content);
            }

            Some(Box::new(MerkleNode {
                left: left_node,
                right: right_node,
                value: digest(node_value),
                content: node_content,
            }))
        }
    }
}
