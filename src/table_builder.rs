//! TableBuilder accumulates sorted key/value entries into 4KiB data blocks.

use crate::block_index::BlockIndex;

pub struct TableBuilder {
    pub target_block_size: usize,
    pub current_block: Vec<u8>,
    pub current_offset: u64,
    pub index: BlockIndex,
    pub last_key: Vec<u8>,
}

impl TableBuilder {
    pub fn new(target_block_size: usize) -> Self {
        Self {
            target_block_size,
            current_block: Vec::new(),
            current_offset: 0,
            index: BlockIndex::new(),
            last_key: Vec::new(),
        }
    }

    pub fn add(&mut self, key: &[u8], value: &[u8]) {
        self.current_block.extend_from_slice(key);
        self.current_block.extend_from_slice(value);
        self.last_key = key.to_vec();

        if self.current_block.len() >= self.target_block_size {
            let len = self.current_block.len() as u64;
            self.index.add_entry(self.last_key.clone(), self.current_offset, len);
            self.current_offset += len;
            self.current_block.clear();
        }
    }
}
