//! Tombstone Garbage Collection during compaction.
//! When a deletion tombstone reaches the bottom-most level, it can be purged.

use crate::merge_iter::KeyValue;

pub struct TombstoneGc;

impl TombstoneGc {
    pub fn filter_bottom_level(entries: Vec<KeyValue>, is_bottom_level: bool) -> Vec<KeyValue> {
        if !is_bottom_level {
            return entries;
        }
        entries.into_iter().filter(|e| e.value.is_some()).collect()
    }
}
