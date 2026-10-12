//! Sparse Block Index for SSTable key search.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockHandle {
    pub offset: u64,
    pub length: u64,
}

#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub key: Vec<u8>,
    pub handle: BlockHandle,
}

pub struct BlockIndex {
    pub entries: Vec<IndexEntry>,
}

impl BlockIndex {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn add_entry(&mut self, key: Vec<u8>, offset: u64, length: u64) {
        self.entries.push(IndexEntry {
            key,
            handle: BlockHandle { offset, length },
        });
    }

    /// Binary search for candidate block that could contain search key
    pub fn find_block(&self, target_key: &[u8]) -> Option<&BlockHandle> {
        if self.entries.is_empty() {
            return None;
        }
        let mut low = 0;
        let mut high = self.entries.len();

        while low < high {
            let mid = (low + high) / 2;
            if self.entries[mid].key.as_slice() < target_key {
                low = mid + 1;
            } else {
                high = mid;
            }
        }

        if low < self.entries.len() {
            Some(&self.entries[low].handle)
        } else {
            // Might be in the last block
            Some(&self.entries.last().unwrap().handle)
        }
    }
}
