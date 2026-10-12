//! Streaming K-way sorted iterator merger using BinaryHeap for O(N log K) compaction.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyValue {
    pub key: Vec<u8>,
    pub value: Option<Vec<u8>>,
}

#[allow(dead_code)]
struct HeapNode<I> {
    item: KeyValue,
    iter_idx: usize,
    iterator: I,
}

impl<I> PartialEq for HeapNode<I> {
    fn eq(&self, other: &Self) -> bool {
        self.item.key == other.item.key
    }
}

impl<I> Eq for HeapNode<I> {}

impl<I> PartialOrd for HeapNode<I> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<I> Ord for HeapNode<I> {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse for min-heap
        other.item.key.cmp(&self.item.key)
    }
}

pub struct MergeIterator<I> {
    heap: BinaryHeap<HeapNode<I>>,
}

impl<I: Iterator<Item = KeyValue>> MergeIterator<I> {
    pub fn new(iterators: Vec<I>) -> Self {
        let mut heap = BinaryHeap::new();
        for (iter_idx, mut iter) in iterators.into_iter().enumerate() {
            if let Some(item) = iter.next() {
                heap.push(HeapNode {
                    item,
                    iter_idx,
                    iterator: iter,
                });
            }
        }
        Self { heap }
    }
}

impl<I: Iterator<Item = KeyValue>> Iterator for MergeIterator<I> {
    type Item = KeyValue;

    fn next(&mut self) -> Option<Self::Item> {
        let mut top = self.heap.pop()?;
        let item = top.item.clone();

        if let Some(next_item) = top.iterator.next() {
            top.item = next_item;
            self.heap.push(top);
        }

        Some(item)
    }
}
