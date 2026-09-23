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
}

fn main() {
    println!("Hello, world!");
}
