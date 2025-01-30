pub mod block;
pub use block::{Block, Header, Transaction};

#[derive(Debug)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub difficulty: u32,
    pub target_duration_between_blocks: u64,
    latest_block_timestamp: u64,
}

impl Blockchain {
    pub fn create_blockchain(difficulty: u32, target_duration_between_blocks: u64) -> Self {
        Self {
            blocks: Vec::new(),
            difficulty,
            target_duration_between_blocks,
            latest_block_timestamp: 0,
        }
    }

    pub fn get_block(&self, index: usize) -> Option<&Block> {
        self.blocks.get(index)
    }

    pub fn get_latest_block(&self) -> Option<&Block> {
        if self.blocks.len() == 0 {
            return None;
        }
        let latest_index: usize = self.blocks.len() - 1 as usize;
        self.blocks.get(latest_index)
    }

    pub fn add_block(&mut self, mut block: Block) {
        let blocks_length: usize = self.blocks.len();
        if blocks_length == 0 {
            self.blocks.push(block);
        } else if self.blocks.len() > 0 {
            let latest_block: &Block = &self.blocks[blocks_length - 1];
            if block.header.prev_hash != latest_block.header.hash {
                return;
            }
            let block_hash: u32 = block.header.hash.parse::<u32>().unwrap();
            if block_hash > self.difficulty {
                return;
            }
            if block.header.timestamp - latest_block.header.timestamp
                > self.target_duration_between_blocks + 1000
            {
                self.target_duration_between_blocks -= 60000;
            } else if block.header.timestamp - latest_block.header.timestamp
                < self.target_duration_between_blocks - 1000
            {
                self.target_duration_between_blocks += 60000;
            }
            let hash = Block::hash_header(&block.header);
            block.header.hash = hash;
            self.latest_block_timestamp = block.header.timestamp;
            self.blocks.push(block);
        }
    }

    pub fn set_difficulty(&mut self, new_difficulty: u32) {
        self.difficulty = new_difficulty;
    }
}
