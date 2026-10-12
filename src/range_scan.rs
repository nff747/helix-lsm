//! Range Scan iterator over lower and upper key bounds [start, end).

use crate::merge_iter::KeyValue;

pub struct RangeScanIterator<I> {
    iter: I,
    end_bound: Option<Vec<u8>>,
}

impl<I: Iterator<Item = KeyValue>> RangeScanIterator<I> {
    pub fn new(iter: I, end_bound: Option<Vec<u8>>) -> Self {
        Self { iter, end_bound }
    }
}

impl<I: Iterator<Item = KeyValue>> Iterator for RangeScanIterator<I> {
    type Item = KeyValue;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.iter.next()?;
        if let Some(ref end) = self.end_bound {
            if item.key.as_slice() >= end.as_slice() {
                return None;
            }
        }
        Some(item)
    }
}
