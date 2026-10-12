//! Atomic WriteBatch: groups multiple Put and Delete operations atomically.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatchOp {
    Put(Vec<u8>, Vec<u8>),
    Delete(Vec<u8>),
}

pub struct WriteBatch {
    pub operations: Vec<BatchOp>,
}

impl WriteBatch {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    pub fn put(&mut self, key: &[u8], value: &[u8]) {
        self.operations.push(BatchOp::Put(key.to_vec(), value.to_vec()));
    }

    pub fn delete(&mut self, key: &[u8]) {
        self.operations.push(BatchOp::Delete(key.to_vec()));
    }

    pub fn count(&self) -> usize {
        self.operations.len()
    }

    pub fn clear(&mut self) {
        self.operations.clear();
    }
}
