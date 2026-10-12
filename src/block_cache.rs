//! LRU in-memory block cache to accelerate repeated SSTable index and data block reads.

use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

pub struct BlockCache {
    capacity: usize,
    entries: Mutex<BlockCacheInner>,
}

struct BlockCacheInner {
    map: HashMap<u64, Vec<u8>>,
    order: VecDeque<u64>,
}

impl BlockCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            entries: Mutex::new(BlockCacheInner {
                map: HashMap::new(),
                order: VecDeque::new(),
            }),
        }
    }

    pub fn get(&self, block_id: u64) -> Option<Vec<u8>> {
        let mut inner = self.entries.lock().unwrap();
        let val = inner.map.get(&block_id)?.clone();
        if let Some(pos) = inner.order.iter().position(|&id| id == block_id) {
            inner.order.remove(pos);
        }
        inner.order.push_back(block_id);
        Some(val)
    }

    pub fn insert(&self, block_id: u64, data: Vec<u8>) {
        let mut inner = self.entries.lock().unwrap();
        if inner.map.contains_key(&block_id) {
            inner.map.insert(block_id, data);
            return;
        }

        while inner.map.len() >= self.capacity && !inner.order.is_empty() {
            let oldest = inner.order.pop_front().unwrap();
            inner.map.remove(&oldest);
        }

        inner.order.push_back(block_id);
        inner.map.insert(block_id, data);
    }
}
