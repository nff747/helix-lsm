//! Key Comparator trait and Bytestring comparator.

use std::cmp::Ordering;

pub trait KeyComparator: Send + Sync {
    fn compare(&self, a: &[u8], b: &[u8]) -> Ordering;
    fn name(&self) -> &'static str;
}

pub struct BytewiseComparator;

impl KeyComparator for BytewiseComparator {
    fn compare(&self, a: &[u8], b: &[u8]) -> Ordering {
        a.cmp(b)
    }

    fn name(&self) -> &'static str {
        "helix.BytewiseComparator"
    }
}
