use std::collections::BTreeMap;

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
}

fn main() {
    println!("Hello, world!");
}
