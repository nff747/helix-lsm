use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use bytes::Bytes;
use crossbeam_skiplist::SkipMap;

use crate::types::{InternalKey, KeyValue, ValueType};

/// High-throughput, lock-free MemTable implementation backed by a concurrent SkipList.
/// Reads and writes never acquire global locks, enabling linear scale with CPU cores.
pub struct MemTable {
    map: Arc<SkipMap<InternalKey, Bytes>>,
    size_bytes: AtomicUsize,
    id: usize,
}

impl MemTable {
    pub fn new(id: usize) -> Self {
        Self {
            map: Arc::new(SkipMap::new()),
            size_bytes: AtomicUsize::new(0),
            id,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    /// Insert or update a key with MVCC sequence number in lock-free fashion.
    pub fn put(&self, key: Bytes, value: Bytes, seq_num: u64) {
        let entry_size = key.len() + value.len() + std::mem::size_of::<InternalKey>();
        let internal_key = InternalKey::new(key, seq_num, ValueType::Value);
        self.map.insert(internal_key, value);
        self.size_bytes.fetch_add(entry_size, Ordering::Relaxed);
    }

    /// Append a tombstone marker for key deletion
    pub fn delete(&self, key: Bytes, seq_num: u64) {
        let entry_size = key.len() + std::mem::size_of::<InternalKey>();
        let internal_key = InternalKey::new(key, seq_num, ValueType::Deletion);
        self.map.insert(internal_key, Bytes::new());
        self.size_bytes.fetch_add(entry_size, Ordering::Relaxed);
    }

    /// Lock-free lookup respecting snapshot isolation sequence number
    pub fn get(&self, user_key: &[u8], snapshot_seq: u64) -> Option<Option<Bytes>> {
        let probe = InternalKey::for_lookup(Bytes::copy_from_slice(user_key), snapshot_seq);

        // Scan from probe entry
        for entry in self.map.range(probe..) {
            let k = entry.key();
            if k.user_key.as_ref() != user_key {
                break;
            }

            if k.seq_num <= snapshot_seq {
                return match k.value_type {
                    ValueType::Value => Some(Some(entry.value().clone())),
                    ValueType::Deletion => Some(None), // Tombstone hit: explicitly deleted
                };
            }
        }

        None // Not found in this memtable
    }

    /// Return estimated memory usage in bytes
    pub fn approximate_size(&self) -> usize {
        self.size_bytes.load(Ordering::Relaxed)
    }

    /// Drain all key-values in sorted order for flushing to an SSTable
    pub fn iter_all(&self) -> Vec<KeyValue> {
        self.map
            .iter()
            .map(|entry| KeyValue {
                key: entry.key().clone(),
                value: entry.value().clone(),
            })
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}
