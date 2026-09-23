use std::collections::BTreeMap;

#[derive(Debug)]
pub enum Error {
    NotFound,
}

pub struct WalStub {
}

impl WalStub {
    pub fn new() -> Self {
        Self {}
    }

    pub fn append(&mut self, _key: &str, _value: &str) {
        // Mock append logic
    }
}

pub struct MemTable {
    data: BTreeMap<String, String>,
    wal: WalStub,
}

impl MemTable {
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
            wal: WalStub::new(),
        }
    }

    pub fn insert(&mut self, key: String, value: String) {
        self.wal.append(&key, &value);
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

    #[test]
    fn test_get() {
        let mut memtable = MemTable::new();
        memtable.insert("key1".to_string(), "value1".to_string());
        assert_eq!(memtable.get("key1"), Some("value1".to_string()));
        assert_eq!(memtable.get("key2"), None);
    }
}

fn main() {
    let mut memtable = MemTable::new();
    memtable.insert("hello".to_string(), "world".to_string());
    println!("Value for 'hello': {:?}", memtable.get("hello"));
}
