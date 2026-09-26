//! In-memory MemTable backed by a BTreeMap with byte-level memory tracking.

use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct MemTable {
    map: BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    byte_size: usize,
}

impl MemTable {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
            byte_size: 0,
        }
    }

    pub fn put(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.byte_size += key.len() + value.len();
        if let Some(old) = self.map.insert(key, Some(value)) {
            if let Some(old_val) = old {
                self.byte_size = self.byte_size.saturating_sub(old_val.len());
            }
        }
    }

    pub fn delete(&mut self, key: Vec<u8>) {
        self.byte_size += key.len();
        if let Some(old) = self.map.insert(key, None) {
            if let Some(old_val) = old {
                self.byte_size = self.byte_size.saturating_sub(old_val.len());
            }
        }
    }

    pub fn get(&self, key: &[u8]) -> Option<Option<&[u8]>> {
        self.map.get(key).map(|v| v.as_deref())
    }

    pub fn byte_size(&self) -> usize {
        self.byte_size
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn into_sorted_vec(self) -> Vec<(Vec<u8>, Option<Vec<u8>>)> {
        self.map.into_iter().collect()
    }

    pub fn clear(&mut self) {
        self.map.clear();
        self.byte_size = 0;
    }
}
