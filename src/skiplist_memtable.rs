//! Concurrent Skiplist MemTable for ordered in-memory buffer.

use std::collections::BTreeMap;
use std::sync::RwLock;

pub struct SkiplistMemtable {
    map: RwLock<BTreeMap<Vec<u8>, Option<Vec<u8>>>>,
    approximate_bytes: RwLock<usize>,
}

impl SkiplistMemtable {
    pub fn new() -> Self {
        Self {
            map: RwLock::new(BTreeMap::new()),
            approximate_bytes: RwLock::new(0),
        }
    }

    pub fn put(&self, key: &[u8], value: &[u8]) {
        let mut map = self.map.write().unwrap();
        let mut bytes = self.approximate_bytes.write().unwrap();
        *bytes += key.len() + value.len();
        map.insert(key.to_vec(), Some(value.to_vec()));
    }

    pub fn get(&self, key: &[u8]) -> Option<Option<Vec<u8>>> {
        let map = self.map.read().unwrap();
        map.get(key).cloned()
    }

    pub fn size_bytes(&self) -> usize {
        *self.approximate_bytes.read().unwrap()
    }
}
