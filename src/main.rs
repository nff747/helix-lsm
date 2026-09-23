use std::collections::BTreeMap;

#[derive(Debug)]
pub enum Error {
    NotFound,
}

pub struct WalStub {
}

pub struct MemTable {
    data: BTreeMap<String, String>,
}

impl MemTable {
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let mut memtable = MemTable::new();
        memtable.insert("key1".to_string(), "value1".to_string());
        assert_eq!(memtable.len(), 1);
    }
}

fn main() {
    println!("Hello, world!");
}
