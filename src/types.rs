use std::cmp::Ordering;
use bytes::Bytes;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ValueType {
    Deletion = 0,
    Value = 1,
}

impl From<u8> for ValueType {
    fn from(val: u8) -> Self {
        match val {
            0 => ValueType::Deletion,
            1 => ValueType::Value,
            _ => panic!("Invalid ValueType byte: {}", val),
        }
    }
}

/// Internal Key combining user key with MVCC sequence number and operation type.
/// Ordering:
/// 1. User key in lexicographical ascending order.
/// 2. Sequence number in descending order (newer versions appear before older versions).
#[derive(Debug, Clone, Eq)]
pub struct InternalKey {
    pub user_key: Bytes,
    pub seq_num: u64,
    pub value_type: ValueType,
}

impl InternalKey {
    pub fn new(user_key: Bytes, seq_num: u64, value_type: ValueType) -> Self {
        Self {
            user_key,
            seq_num,
            value_type,
        }
    }

    pub fn for_lookup(user_key: Bytes, seq_num: u64) -> Self {
        Self {
            user_key,
            seq_num,
            value_type: ValueType::Value,
        }
    }
}

impl PartialEq for InternalKey {
    fn eq(&self, other: &Self) -> bool {
        self.user_key == other.user_key && self.seq_num == other.seq_num
    }
}

impl Ord for InternalKey {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.user_key.cmp(&other.user_key) {
            Ordering::Equal => other.seq_num.cmp(&self.seq_num), // Descending for newer versions first
            ord => ord,
        }
    }
}

impl PartialOrd for InternalKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyValue {
    pub key: InternalKey,
    pub value: Bytes,
}
